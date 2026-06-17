import requests
import json
import os
import re
import sys
from pathlib import Path
from datetime import datetime

OPENROUTER_API_KEY = os.environ["OPENROUTER_API_KEY"]
MODEL = "nex-agi/nex-n2-pro:free"
PROJECTS_DIR = Path("projects")

PROMPT = """Generate a random self-contained Rust project that explores one of these areas:

- Mathematical algorithms (number theory, combinatorics, algebra, geometry)
- Unusual numeral systems (balanced ternary, bijective base-k, p-adic, factorial number system...)
- Cryptography or hashing (implement from scratch, not using libraries)
- Bit manipulation and low-level tricks
- Data structures implemented from scratch (tries, skip lists, finger trees, union-find...)
- Automata, grammars, or formal languages
- Procedural generation (fractals, L-systems, cellular automata, noise...)
- Physics or simulation (nbody, wave equation, diffusion...)
- Compression algorithms
- Anything that shows off interesting Rust: unsafe, const generics, trait magic, zero-cost abstractions

Avoid: web servers, CRUD apps, file managers, todo lists, simple games.
Prefer: things that produce interesting output when run, demonstrate a non-obvious concept, or explore edges of the language.

Respond ONLY in this exact XML format, nothing else before or after:

<project>
<name>short-kebab-case-name</name>
<description>one sentence what it does</description>
<cargo_toml>
[package]
name = "..."
version = "0.1.0"
edition = "2021"
...rest of Cargo.toml...
</cargo_toml>
<files>
<file>
<path>src/main.rs</path>
<content>
...full file content...
</content>
</file>
</files>
</project>

Rules:
- Stable Rust only
- External crates allowed if genuinely needed
- Fully functional, produces real output when run
- No placeholders, no TODO comments, no skeleton code
- Single binary project (no workspaces)"""


def call_api(prompt: str) -> str:
    response = requests.post(
        url="https://openrouter.ai/api/v1/chat/completions",
        headers={
            "Authorization": f"Bearer {OPENROUTER_API_KEY}",
            "Content-Type": "application/json",
            "X-Title": "rust-zoo",
        },
        data=json.dumps({
            "model": MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "reasoning": {"enabled": True},
            "max_tokens": 8000,
            "temperature": 1.0,
        }),
        timeout=300,
    )
    print("DEBUG status:", response.status_code)
    print("DEBUG body:", response.text[:1000])
    response.raise_for_status()
    return response.json()["choices"][0]["message"]["content"]


def extract_tag(text: str, tag: str) -> str | None:
    m = re.search(rf"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
    return m.group(1).strip() if m else None


def parse_files(files_block: str) -> list[tuple[str, str]]:
    result = []
    for m in re.finditer(r"<file>\s*<path>(.*?)</path>\s*<content>(.*?)</content>\s*</file>", files_block, re.DOTALL):
        path = m.group(1).strip()
        content = m.group(2).strip()
        result.append((path, content))
    return result


def write_project(name: str, cargo_toml: str, files: list[tuple[str, str]]) -> Path:
    project_dir = PROJECTS_DIR / name
    project_dir.mkdir(parents=True, exist_ok=True)

    (project_dir / "Cargo.toml").write_text(cargo_toml)
    for path, content in files:
        full_path = project_dir / path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(content)

    return project_dir


def main():
    print(f"[{datetime.now().isoformat()}] Calling API...")
    raw = call_api(PROMPT)

    print("--- Raw response (first 500 chars) ---")
    print(raw[:500])
    print("---")

    name        = extract_tag(raw, "name")
    description = extract_tag(raw, "description")
    cargo_toml  = extract_tag(raw, "cargo_toml")
    files_block = extract_tag(raw, "files")

    if not all([name, cargo_toml, files_block]):
        print("ERROR: Failed to parse response. Saving raw output to debug.txt")
        Path("debug.txt").write_text(raw)
        sys.exit(1)

    files = parse_files(files_block)
    if not files:
        print("ERROR: No files found in response")
        sys.exit(1)

    # Sanitize name
    name = re.sub(r"[^a-z0-9\-]", "-", name.lower()).strip("-")
    # Avoid collisions with timestamp suffix
    if (PROJECTS_DIR / name).exists():
        name = f"{name}-{datetime.now().strftime('%H%M%S')}"

    project_dir = write_project(name, cargo_toml, files)
    print(f"✅ Written: {project_dir}")
    print(f"   Description: {description}")
    print(f"   Files: {[f[0] for f in files]}")

    # Write a small metadata file for later analysis
    meta = {
        "name": name,
        "description": description,
        "generated_at": datetime.now().isoformat(),
        "model": MODEL,
        "file_count": len(files),
    }
    (project_dir / "meta.json").write_text(json.dumps(meta, indent=2))

    # Output name for the GitHub Actions step to use in commit message
    print(f"::set-output name=project_name::{name}")


if __name__ == "__main__":
    main()
