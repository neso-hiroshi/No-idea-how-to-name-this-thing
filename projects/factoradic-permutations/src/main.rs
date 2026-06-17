use std::fmt;
use std::iter::FusedIterator;
use rand::Rng;

/// Compile-time factorial computation using const generics
const fn factorial<const N: usize>() -> usize {
    let mut result = 1;
    let mut i = 2;
    while i <= N {
        result *= i;
        i += 1;
    }
    result
}

/// Factorial number system (factoradic) representation.
/// A number N is represented as: d_n * n! + d_{n-1} * (n-1)! + ... + d_1 * 1! + d_0 * 0!
/// where 0 <= d_i <= i
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Factoradic<const N: usize> {
    digits: [u8; N],
}

impl<const N: usize> Factoradic<N> {
    /// Create from raw digits (least significant first: d_0, d_1, ..., d_{N-1})
    pub const fn from_digits(digits: [u8; N]) -> Self {
        Self { digits }
    }

    /// Convert a natural number to factoradic representation
    pub fn from_u64(mut n: u64) -> Self {
        let mut digits = [0u8; N];
        for i in 1..=N {
            digits[i - 1] = (n % i as u64) as u8;
            n /= i as u64;
        }
        Self { digits }
    }

    /// Convert factoradic back to natural number
    pub fn to_u64(&self) -> u64 {
        let mut result = 0u64;
        let mut fact = 1u64;
        for i in 1..=N {
            result += self.digits[i - 1] as u64 * fact;
            fact *= i as u64;
        }
        result
    }

    /// Get digit at position i (0-indexed, least significant)
    pub fn digit(&self, i: usize) -> u8 {
        self.digits[i]
    }

    /// Normalize digits to ensure 0 <= d_i <= i
    pub fn normalize(&mut self) {
        let mut carry = 0u64;
        for i in 1..=N {
            let val = self.digits[i - 1] as u64 + carry;
            self.digits[i - 1] = (val % i as u64) as u8;
            carry = val / i as u64;
        }
    }

    /// Iterator over digits (least significant first)
    pub fn digits(&self) -> impl Iterator<Item = u8> + '_ {
        self.digits.iter().copied()
    }

    /// Check if this is a valid factoradic (all digits in range)
    pub fn is_valid(&self) -> bool {
        self.digits.iter().enumerate().all(|(i, &d)| d <= i as u8)
    }
}

impl<const N: usize> fmt::Display for Factoradic<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display most significant first
        write!(f, "(")?;
        for i in (1..=N).rev() {
            write!(f, "{}", self.digits[i - 1])?;
            if i > 1 { write!(f, ",")?; }
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// A permutation of N elements (0..N)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Permutation<const N: usize> {
    elements: [u8; N],
}

impl<const N: usize> Permutation<N> {
    /// Identity permutation
    pub const fn identity() -> Self {
        let mut elements = [0u8; N];
        let mut i = 0;
        while i < N {
            elements[i] = i as u8;
            i += 1;
        }
        Self { elements }
    }

    /// Create from array
    pub const fn from_array(elements: [u8; N]) -> Self {
        Self { elements }
    }

    /// Get element at index
    pub fn get(&self, i: usize) -> u8 {
        self.elements[i]
    }

    /// Apply permutation to a slice (permute in place)
    pub fn apply<T: Copy>(&self, data: &mut [T; N]) {
        let mut result = [data[0]; N];
        for i in 0..N {
            result[i] = data[self.elements[i] as usize];
        }
        *data = result;
    }

    /// Inverse permutation
    pub fn inverse(&self) -> Self {
        let mut inv = [0u8; N];
        for i in 0..N {
            inv[self.elements[i] as usize] = i as u8;
        }
        Self { elements: inv }
    }

    /// Compose with another permutation (self ∘ other)
    pub fn compose(&self, other: &Self) -> Self {
        let mut result = [0u8; N];
        for i in 0..N {
            result[i] = self.elements[other.elements[i] as usize];
        }
        Self { elements: result }
    }

    /// Convert to Lehmer code (factoradic representation)
    pub fn to_lehmer(&self) -> Factoradic<N> {
        let mut digits = [0u8; N];
        let mut used = [false; N];
        
        for i in 0..N {
            let val = self.elements[i];
            let mut count = 0;
            for j in 0..val as usize {
                if !used[j] {
                    count += 1;
                }
            }
            digits[N - 1 - i] = count as u8;
            used[val as usize] = true;
        }
        Factoradic::from_digits(digits)
    }

    /// Lexicographic rank (0-indexed)
    pub fn rank(&self) -> u64 {
        self.to_lehmer().to_u64()
    }

    /// Iterator over elements
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.elements.iter().copied()
    }
}

impl<const N: usize> fmt::Display for Permutation<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..N {
            write!(f, "{}", self.elements[i])?;
            if i + 1 < N { write!(f, " ")?; }
        }
        write!(f, "]")?;
        Ok(())
    }
}

/// Unrank: convert a rank (0..N!) to the corresponding permutation
pub fn unrank<const N: usize>(rank: u64) -> Permutation<N> {
    let factoradic = Factoradic::<N>::from_u64(rank);
    lehmer_to_permutation(&factoradic)
}

/// Rank: convert a permutation to its lexicographic rank
pub fn rank<const N: usize>(perm: &Permutation<N>) -> u64 {
    perm.rank()
}

/// Convert Lehmer code (factoradic) to permutation
fn lehmer_to_permutation<const N: usize>(lehmer: &Factoradic<N>) -> Permutation<N> {
    let mut available: Vec<u8> = (0..N as u8).collect();
    let mut elements = [0u8; N];
    
    for i in 0..N {
        let idx = lehmer.digit(N - 1 - i) as usize;
        elements[i] = available.remove(idx);
    }
    
    Permutation::from_array(elements)
}

/// Iterator over all permutations in lexicographic order
pub struct PermutationIter<const N: usize> {
    current: Option<Permutation<N>>,
    count: u64,
    total: u64,
}

impl<const N: usize> PermutationIter<N> {
    pub fn new() -> Self {
        let total = factorial::<N>() as u64;
        Self {
            current: Some(Permutation::identity()),
            count: 0,
            total,
        }
    }
}

impl<const N: usize> Iterator for PermutationIter<N> {
    type Item = Permutation<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.total {
            return None;
        }
        
        let current = self.current.take()?;
        
        // Generate next permutation using std algorithm
        if self.count + 1 < self.total {
            let mut next_elements = current.elements;
            // Next lexicographic permutation
            let mut i = N;
            while i > 1 && next_elements[i - 2] >= next_elements[i - 1] {
                i -= 1;
            }
            if i > 0 {
                let mut j = N;
                while next_elements[j - 1] <= next_elements[i - 2] {
                    j -= 1;
                }
                next_elements.swap(i - 2, j - 1);
                next_elements[i - 1..].reverse();
                self.current = Some(Permutation::from_array(next_elements));
            }
        }
        
        self.count += 1;
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.total - self.count) as usize;
        (remaining, Some(remaining))
    }
}

impl<const N: usize> ExactSizeIterator for PermutationIter<N> {}
impl<const N: usize> FusedIterator for PermutationIter<N> {}

/// Random permutation generator using Fisher-Yates
pub fn random_permutation<const N: usize>(rng: &mut impl Rng) -> Permutation<N> {
    let mut elements: [u8; N] = [0; N];
    for i in 0..N {
        elements[i] = i as u8;
    }
    for i in (1..N).rev() {
        let j = rng.gen_range(0..=i);
        elements.swap(i, j);
    }
    Permutation::from_array(elements)
}

/// Demonstrate cycle decomposition of a permutation
pub fn cycle_decomposition<const N: usize>(perm: &Permutation<N>) -> Vec<Vec<u8>> {
    let mut visited = [false; N];
    let mut cycles = Vec::new();
    
    for i in 0..N {
        if !visited[i] {
            let mut cycle = Vec::new();
            let mut j = i;
            while !visited[j] {
                visited[j] = true;
                cycle.push(j as u8);
                j = perm.elements[j] as usize;
            }
            if cycle.len() > 1 {
                cycles.push(cycle);
            }
        }
    }
    cycles
}

/// Permutation parity (sign): +1 for even, -1 for odd
pub fn permutation_sign<const N: usize>(perm: &Permutation<N>) -> i8 {
    let cycles = cycle_decomposition(perm);
    let transpositions: usize = cycles.iter().map(|c| c.len() - 1).sum();
    if transpositions % 2 == 0 { 1 } else { -1 }
}

/// Steinhaus-Johnson-Trotter iterator (adjacent transpositions)
pub struct SJTIter<const N: usize> {
    perm: Permutation<N>,
    directions: [i8; N], // -1 = left, +1 = right
    first: bool,
}

impl<const N: usize> SJTIter<N> {
    pub fn new() -> Self {
        Self {
            perm: Permutation::identity(),
            directions: [-1; N],
            first: true,
        }
    }
}

impl<const N: usize> Iterator for SJTIter<N> {
    type Item = Permutation<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            return Some(self.perm.clone());
        }

        // Find largest mobile element
        let mut mobile_idx = None;
        let mut mobile_val = 0;
        
        for i in 0..N {
            let val = self.perm.elements[i];
            let dir = self.directions[val as usize];
            let neighbor_idx = i as i32 + dir as i32;
            
            if neighbor_idx >= 0 && neighbor_idx < N as i32 {
                let neighbor_val = self.perm.elements[neighbor_idx as usize];
                if val > neighbor_val {
                    if val > mobile_val {
                        mobile_val = val;
                        mobile_idx = Some(i);
                    }
                }
            }
        }

        let idx = mobile_idx?;
        let val = self.perm.elements[idx];
        let dir = self.directions[val as usize];
        let neighbor_idx = (idx as i32 + dir as i32) as usize;
        
        // Swap
        self.perm.elements.swap(idx, neighbor_idx);
        
        // Reverse direction of all elements larger than mobile
        for i in 0..N {
            if self.perm.elements[i] > val {
                self.directions[self.perm.elements[i] as usize] *= -1;
            }
        }

        Some(self.perm.clone())
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       FACTORIAL NUMBER SYSTEM & PERMUTATION Bijection          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // 1. Factoradic basics
    println!("┌─ 1. FACTORADIC REPRESENTATION ────────────────────────────────┐");
    println!("│ Number → Factoradic (digits: d_i × i!)                        │");
    println!("├────────────────────────────────────────────────────────────────┤");
    for n in [0, 1, 5, 23, 47, 119, 719, 5039] {
        let f = Factoradic::<7>::from_u64(n);
        let back = f.to_u64();
        println!("│ {:>5} → {} = {:>5} {}", n, f, back, if n == back { "✓" } else { "✗" });
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 2. Bijection demonstration
    println!("┌─ 2. RANK ↔ UNRANK Bijection (N=5, 5! = 120 permutations) ─────┐");
    println!("│ Rank → Permutation → Rank (should match)                       │");
    println!("├────────────────────────────────────────────────────────────────┤");
    let test_ranks = [0, 1, 2, 5, 23, 59, 119];
    for r in test_ranks {
        let p = unrank::<5>(r);
        let r2 = rank(&p);
        println!("│ {:>3} → {} → {:>3} {}", r, p, r2, if r == r2 { "✓" } else { "✗" });
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 3. All permutations of 4 elements
    println!("┌─ 3. ALL PERMUTATIONS OF 4 ELEMENTS (Lexicographic Order) ─────┐");
    let mut count = 0;
    for p in PermutationIter::<4>::new() {
        let f = p.to_lehmer();
        let r = p.rank();
        let sign = permutation_sign(&p);
        let sign_str = if sign == 1 { "+" } else { "-" };
        println!("│ {:>2}: {}  lehmer={}  sign={}", r, p, f, sign_str);
        count += 1;
        if count >= 24 { break; }
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 4. Steinhaus-Johnson-Trotter (adjacent swaps)
    println!("┌─ 4. STEINHAUS-JOHNSON-TROTTER (N=4, Adjacent Transpositions) ─┐");
    let mut sjt_count = 0;
    for p in SJTIter::<4>::new() {
        let r = rank(&p);
        println!("│ {:>2}: {}", r, p);
        sjt_count += 1;
        if sjt_count >= 24 { break; }
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 5. Factoradic arithmetic
    println!("┌─ 5. FACTORADIC ARITHMETIC ────────────────────────────────────┐");
    let a = Factoradic::<6>::from_u64(123);
    let b = Factoradic::<6>::from_u64(456);
    let sum_val = a.to_u64() + b.to_u64();
    let mut sum_f = Factoradic::<6>::from_u64(sum_val);
    println!("│ {} + {} = {}", a, b, sum_f);
    println!("│   ({} + {} = {})", a.to_u64(), b.to_u64(), sum_val);
    
    // Manual digit-wise addition with normalization
    let mut manual = Factoradic::<6>::from_digits([
        a.digit(0) + b.digit(0),
        a.digit(1) + b.digit(1),
        a.digit(2) + b.digit(2),
        a.digit(3) + b.digit(3),
        a.digit(4) + b.digit(4),
        a.digit(5) + b.digit(5),
    ]);
    manual.normalize();
    println!("│ Manual digit-add + normalize: {} = {}", manual, manual.to_u64());
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 6. Permutation operations
    println!("┌─ 6. PERMUTATION OPERATIONS ───────────────────────────────────┐");
    let p1 = unrank::<6>(247);
    let p2 = unrank::<6>(512);
    println!("│ p1 = {} (rank {})", p1, p1.rank());
    println!("│ p2 = {} (rank {})", p2, p2.rank());
    println!("│ p1⁻¹ = {}", p1.inverse());
    println!("│ p1 ∘ p2 = {}", p1.compose(&p2));
    println!("│ p2 ∘ p1 = {}", p2.compose(&p1));
    
    // Apply to data
    let mut data = ['A', 'B', 'C', 'D', 'E', 'F'];
    p1.apply(&mut data);
    println!("│ Applying p1 to [A B C D E F]: [{}]", data.iter().collect::<String>());
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 7. Cycle decomposition
    println!("┌─ 7. CYCLE DECOMPOSITION ──────────────────────────────────────┐");
    let p = unrank::<7>(3456);
    println!("│ Permutation: {}", p);
    let cycles = cycle_decomposition(&p);
    println!("│ Cycles: {:?}", cycles);
    println!("│ Sign: {} (parity: {})", 
        permutation_sign(&p),
        if permutation_sign(&p) == 1 { "even" } else { "odd" });
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 8. Large N demonstration (N=10, 10! = 3,628,800)
    println!("┌─ 8. LARGE N DEMONSTRATION (N=10, 10! = 3,628,800) ───────────┐");
    let big_rank = 1_234_567;
    let big_perm = unrank::<10>(big_rank);
    println!("│ Rank {} → {}", big_rank, big_perm);
    println!("│ Rank back: {} {}", rank(&big_perm), if rank(&big_perm) == big_rank { "✓" } else { "✗" });
    println!("│ Lehmer code: {}", big_perm.to_lehmer());
    println!("│ Cycle decomposition: {:?}", cycle_decomposition(&big_perm));
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 9. Random permutations
    println!("┌─ 9. RANDOM PERMUTATIONS (N=8) ────────────────────────────────┐");
    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let rp = random_permutation::<8>(&mut rng);
        println!("│ {} (rank {})", rp, rp.rank());
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // 10. Mathematical properties
    println!("┌─ 10. MATHEMATICAL PROPERTIES ─────────────────────────────────┐");
    println!("│ Factorials (compile-time):");
    for n in 1..=12 {
        println!("│   {}! = {}", n, factorial::<{n}>());
    }
    println!("│");
    println!("│ Sum of factoradic digits for 0..N!-1:");
    for n in 1..=6 {
        let total: u64 = (0..factorial::<{n}>() as u64)
            .map(|i| Factoradic::<{n}>::from_u64(i).digits().map(|d| d as u64).sum::<u64>())
            .sum();
        let avg = total as f64 / factorial::<{n}>() as f64;
        println!("│   N={}: total={}, avg={:.2}", n, total, avg);
    }
    println!("└────────────────────────────────────────────────────────────────┘\n");

    println!("✨ Factoradic-permutations exploration complete!");
}