use std::fmt::Write;

fn main() {
    let samples = [
        (10u64, 13u64, 8u32),
        ...
    ];

    println!("Hensel lifting for p-adic square roots");
    println!("For each sample, the program solves x^2 = a modulo p^n.");
    println!("Digits are printed least-significant first: x = d0 + d1*p + d2*p^2 + ...");
    println!();

    for (a, p, n) in samples {
        println!("x^2 = {} in the {}-adic integers, requested precision n = {}", a, p, n);
        let legendre = legendre_symbol(a % p, p);
        println!("  Legendre symbol ({}|{}) = {}", a % p, p, legendre);

        match p_adic_square_roots(a, p, n) {
            Some((modulus, roots)) => {
                for root in roots {
                    let digits = p_adic_digits(root, p, n as usize);
                    println!(
                        "  root {} has {}^2 = {} (mod {}) and digits [{}]",
                        root,
                        root,
                        mul_mod(root, root, modulus),
                        modulus,
                        digits_to_text(&amp;digits)
                    );
                }
            }
            None => println!("  no unit square root exists for this sample"),
        }
        println!();
    }
}
...