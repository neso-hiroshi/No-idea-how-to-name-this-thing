fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1_u64;
    base %= modulus;
    while exp &gt; 0 {
        if exp &amp; 1 == 1 {
            result = result.wrapping_mul(base) % modulus;
        }
        base = base.wrapping_mul(base) % modulus;
        exp &gt;&gt;= 1;
    }
    result
}

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    let (mut old_r, mut r) = (a, b);
    let (mut old_s, mut s) = (1_i64, 0_i64);
    let (mut old_t, mut t) = (0_i64, 1_i64);

    while r != 0 {
        let q = old_r / r;
        let nr = old_r - q * r;
        old_r = r;
        r = nr;

        let ns = old_s - q * s;
        old_s = s;
        s = ns;

        let nt = old_t - q * t;
        old_t = t;
        t = nt;
    }

    (old_r, old_s, old_t)
}

fn inv_mod(a: u64, modulus: u64) -> u64 {
    let (g, x, _) = extended_gcd(a as i64, modulus as i64);
    assert_eq!(g, 1, "value must be invertible modulo {modulus}");
    let r = x % modulus as i64;
    if r &lt; 0 {
        (r + modulus as i64) as u64
    } else {
        r as u64
    }
}

fn legendre_symbol(a: u64, p: u64) -> i64 {
    let s = mod_pow(a, (p - 1) / 2, p);
    if s == p - 1 {
        -1
    } else {
        s as i64
    }
}

fn tonelli_shanks(n: u64, p: u64) -&gt; Option&lt;u64&gt; {
    if n == 0 {
        return Some(0);
    }
    if p == 2 {
        return Some(n % 2);
    }
    if legendre_symbol(n, p) != 1 {
        return None;
    }

    if p % 4 == 3 {
        return Some(mod_pow(n, (p + 1) / 4, p));
    }

    let mut q = p - 1;
    let mut s = 0;
    while q % 2 == 0 {
        q /= 2;
        s += 1;
    }

    let mut z = 2;
    while legendre_symbol(z, p) != -1 {
        z += 1;
    }

    let mut m = s;
    let mut c = mod_pow(z, q, p);
    let mut t = mod_pow(n, q, p);
    let mut r = mod_pow(n, (q + 1) / 2, p);

    loop {
        if t == 1 {
            return Some(r);
        }

        let mut i = 1;
        let mut tt = t.wrapping_mul(t) % p;
        while tt != 1 {
            tt = tt.wrapping_mul(tt) % p;
            i += 1;
            if i == m {
                return None;
            }
        }

        let mut b = c;
        for _ in 0..(m - i - 1) {
            b = b.wrapping_mul(b) % p;
        }

        m = i;
        c = b.wrapping_mul(b) % p;
        t = t.wrapping_mul(c) % p;
        r = r.wrapping_mul(b) % p;
    }
}

fn digits_of(value: u64, base: u64, width: usize) -&gt; Vec&lt;u64&gt; {
    let mut digits = vec![0; width];
    let mut v = value;
    for digit in digits.iter_mut() {
        *digit = v % base;
        v /= base;
    }
    digits
}

fn hensel_lift(value: u64, p: u64, depth: usize) -&gt; Vec&lt;u64&gt; {
    let mut residues = Vec::with_capacity(depth);
    let mut r = tonelli_shanks(value, p).expect("prime must admit a square root modulo p");
    let mut modulus = p;

    for _ in 0..depth {
        residues.push(r);

        let next_modulus = modulus.saturating_mul(p);
        let a = value % next_modulus;
        let diff = (a + next_modulus - (r.wrapping_mul(r) % next_modulus)) % next_modulus;
        let numerator = diff / modulus;
        let denom = (2 * r) % p;
        let t = (numerator * inv_mod(denom, p)) % p;
        r = (r + t * modulus) % next_modulus;
        modulus = next_modulus;
    }

    residues
}

fn print_table(residues: &amp;[u64], p: u64, depth: usize) {
    let width = (depth - 1).max(1);
    println!("p-adic square roots of -1 modulo {p}^{width}");
    println!("digits are least significant first, one new digit per line");
    println!();

    for (level, r) in residues.iter().enumerate() {
        let k = level + 1;
        let digits = digits_of(*r, p, k);
        let digit_text = digits
            .iter()
            .rev()
            .map(|d| format!("{d}"))
            .collect::&lt;Vec&lt;_&gt;&gt;()
            .join(" ");
        println!("{k:2}: residue {r:16}  digits [{digit_text}]");
    }
}

fn print_pair_comparison(a: u64, b: u64, p: u64, depth: usize) {
    println!();
    println!("the two roots are negatives modulo p^k");
    for k in 1..=depth {
        let modulus = p.pow(k as u32);
        let ak = a % modulus;
        let bk = b % modulus;
        let sum = (ak + bk) % modulus;
        println!("k={k:2}: root + other = {sum} mod {p}^{k}");
    }
}

fn main() {
    let p = 5;
    let depth = 18;
    let target = p - 1;

    println!("finding r such that r^2 == -1 in the {p}-adic integers");
    println!("first root modulo p: {:?}", tonelli_shanks(target, p));
    println!();

    let root = hensel_lift(target, p, depth);
    let other = hensel_lift(target, p, depth)
        .into_iter()
        .enumerate()
        .map(|(i, r)| (p.pow((i + 1) as u32) - r) % p.pow((i + 1) as u32))
        .collect::&lt;Vec&lt;_&gt;&gt;();

    print_table(&amp;root, p, depth);
    print_pair_comparison(*root.last().unwrap(), *other.last().unwrap(), p, depth);
}