//! [`Accumulator`]s for check character computation and verification.
//!
//! ```rust
//! use iso_7064::accumulator::{Accumulator as _, Mod11_2};
//!
//! let mut acc = Mod11_2::default();
//! acc.accumulate(0);
//! acc.accumulate(7);
//! acc.accumulate(9);
//! assert_eq!(acc.compute(), [10], "check char is X (10)");
//!
//! acc.accumulate(10);
//! assert!(acc.verify(), "079X is valid string");
//! ```

/// A trait for accumulating values to compute or verify check characters.
pub trait Accumulator<const N_CC: usize> {
    /// Accumulates a numerical value, returning the result of the step as an [`AccumulateResult`].
    ///
    /// The accumulated state is updated only when `Processed` is returned. `NotInCharset` usually
    /// indicates an error, and the input value is discarded.
    ///
    /// A supplementary check character (e.g., `'X'` or `'*'`) does not update the state when
    /// `SupplFound` is returned; instead, it is stored and taken into account on the next call to
    /// `verify()`. The stored value is cleared once a subsequent `accumulate()` call returns
    /// `Processed` or `SupplFound`. `SupplFound` is considered an error unless it appears at the
    /// end of the input during verification.
    fn accumulate(&mut self, value: u32) -> AccumulateResult;

    /// Computes the final check characters from the accumulated state.
    fn compute(&self) -> [u32; N_CC];

    /// Verifies whether the accumulated state satisfies the check character condition.
    fn verify(&self) -> bool;
}

/// The result of an accumulation step.
///
/// See also [`Accumulator::accumulate`] for the semantics of the variants.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AccumulateResult {
    /// The value was successfully processed, updating the accumulated state.
    Processed,

    /// The value is a supplementary check character (e.g., `'X'` or `'*'`).
    SupplFound,

    /// The value is not in the character set.
    NotInCharset,
}

/// A generic accumulator for the pure system with a single check character.
#[derive(Debug, Clone)]
pub struct PureSingle<const MODULUS: u32, const RADIX: u32, const CHARSET_SIZE: u32> {
    carry: u32,
    suppl: Option<u32>,
}

impl<const MODULUS: u32, const RADIX: u32, const CHARSET_SIZE: u32>
    PureSingle<MODULUS, RADIX, CHARSET_SIZE>
{
    /// Creates a new accumulator.
    const fn new() -> Self {
        assert!(MODULUS >= CHARSET_SIZE);
        assert!(MODULUS > 0 && RADIX > 0 && CHARSET_SIZE > 0);
        assert!(MODULUS < u32::MAX);
        Self {
            carry: 0,
            suppl: None,
        }
    }

    #[inline]
    const fn accumulate_const(&mut self, value: u32) -> AccumulateResult {
        if value >= MODULUS {
            AccumulateResult::NotInCharset
        } else if value >= CHARSET_SIZE {
            self.suppl = Some(value);
            AccumulateResult::SupplFound
        } else {
            self.suppl = None;
            self.carry = Self::step(self.carry, value);
            AccumulateResult::Processed
        }
    }

    const fn compute_const(&self) -> [u32; 1] {
        let carry = Self::step(self.carry, 0);
        let v = MODULUS + 1 - carry % MODULUS;
        [spec_rem(v, MODULUS)]
    }

    const fn verify_const(&self) -> bool {
        let carry = match self.suppl {
            None => self.carry,
            Some(value) => Self::step(self.carry, value),
        };
        carry % MODULUS == 1
    }

    #[inline(always)]
    const fn step(carry: u32, value: u32) -> u32 {
        if carry > (u32::MAX - MODULUS) / RADIX {
            // observed that LLVM better optimizes div-by-const with u64
            let wide = carry as u64 * RADIX as u64 + value as u64;
            (wide % MODULUS as u64) as u32
        } else {
            carry * RADIX + value
        }
    }
}

/// A generic accumulator for the pure system with two check characters.
#[derive(Debug, Clone)]
pub struct PureDouble<const MODULUS: u32, const CHARSET_SIZE: u32> {
    carry: u32,
}

impl<const MODULUS: u32, const CHARSET_SIZE: u32> PureDouble<MODULUS, CHARSET_SIZE> {
    /// Creates a new accumulator.
    const fn new() -> Self {
        assert!(MODULUS <= CHARSET_SIZE * CHARSET_SIZE);
        assert!(MODULUS > 0 && CHARSET_SIZE > 0);
        assert!(MODULUS < u32::MAX);
        Self { carry: 0 }
    }

    #[inline]
    const fn accumulate_const(&mut self, value: u32) -> AccumulateResult {
        if value >= CHARSET_SIZE {
            AccumulateResult::NotInCharset
        } else {
            self.carry = Self::step(self.carry, value);
            AccumulateResult::Processed
        }
    }

    const fn compute_const(&self) -> [u32; 2] {
        let radix = CHARSET_SIZE;
        let carry = Self::step(Self::step(self.carry, 0), 0);
        let v = MODULUS + 1 - carry % MODULUS;
        [v / radix, v % radix]
    }

    const fn verify_const(&self) -> bool {
        self.carry % MODULUS == 1
    }

    #[inline(always)]
    const fn step(carry: u32, value: u32) -> u32 {
        let radix = CHARSET_SIZE;
        if carry > (u32::MAX - MODULUS) / radix {
            // observed that LLVM better optimizes div-by-const with u64
            let wide = carry as u64 * radix as u64 + value as u64;
            (wide % MODULUS as u64) as u32
        } else {
            carry * radix + value
        }
    }
}

/// A generic accumulator for the hybrid system.
#[derive(Debug, Clone)]
pub struct Hybrid<const CHARSET_SIZE: u32> {
    carry: u32,
}

impl<const CHARSET_SIZE: u32> Hybrid<CHARSET_SIZE> {
    /// Creates a new accumulator.
    const fn new() -> Self {
        assert!(CHARSET_SIZE % 2 == 0, "`CHARSET_SIZE` must be even");
        assert!(CHARSET_SIZE > 0);
        assert!(u32::MAX / 2 > CHARSET_SIZE);
        Self { carry: 0 }
    }

    #[inline]
    const fn accumulate_const(&mut self, value: u32) -> AccumulateResult {
        let modulus = CHARSET_SIZE;
        if value >= CHARSET_SIZE {
            AccumulateResult::NotInCharset
        } else {
            self.carry = match self.carry {
                0 => modulus,                      // first loop only
                c => spec_rem(c * 2, modulus + 1), // non-zero even mod odd != 0
            };
            self.carry = non_zero_spec_rem(self.carry + value, modulus);
            debug_assert!(self.carry > 0);
            AccumulateResult::Processed
        }
    }

    const fn compute_const(&self) -> [u32; 1] {
        let modulus = CHARSET_SIZE;
        let carry = spec_rem(self.carry * 2, modulus + 1);
        [spec_rem(modulus + 1 - carry, modulus)]
    }

    const fn verify_const(&self) -> bool {
        self.carry == 1
    }
}

#[inline(always)]
const fn spec_rem(lhs: u32, rhs: u32) -> u32 {
    debug_assert!(lhs < rhs * 2);
    if lhs < rhs { lhs } else { lhs - rhs }
}

#[inline(always)]
const fn non_zero_spec_rem(lhs: u32, rhs: u32) -> u32 {
    debug_assert!(0 < lhs && lhs < rhs * 2);
    if lhs <= rhs { lhs } else { lhs - rhs }
}

macro_rules! impl_accumulator_traits {
    ($n_cc:expr, $ty:ident < $($param:ident),+ >) => {
        impl<$(const $param: u32),+> Default for $ty<$($param),+> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<$(const $param: u32),+> Accumulator<$n_cc> for $ty<$($param),+> {
            #[inline]
            fn accumulate(&mut self, value: u32) -> AccumulateResult {
                self.accumulate_const(value)
            }

            fn compute(&self) -> [u32; $n_cc] {
                self.compute_const()
            }

            fn verify(&self) -> bool {
                self.verify_const()
            }
        }
    };
}

impl_accumulator_traits!(1, PureSingle<MODULUS, RADIX, CHARSET_SIZE>);
impl_accumulator_traits!(2, PureDouble<MODULUS, CHARSET_SIZE>);
impl_accumulator_traits!(1, Hybrid<CHARSET_SIZE>);

/// An accumulator for ISO/IEC 7064, MOD 11-2.
pub type Mod11_2 = PureSingle<11, 2, 10>;

/// An accumulator for ISO/IEC 7064, MOD 37-2.
pub type Mod37_2 = PureSingle<37, 2, 36>;

/// An accumulator for ISO/IEC 7064, MOD 97-10.
pub type Mod97_10 = PureDouble<97, 10>;

/// An accumulator for ISO/IEC 7064, MOD 661-26.
pub type Mod661_26 = PureDouble<661, 26>;

/// An accumulator for ISO/IEC 7064, MOD 1271-36.
pub type Mod1271_36 = PureDouble<1271, 36>;

/// An accumulator for ISO/IEC 7064, MOD 11,10.
pub type Mod11_10 = Hybrid<10>;

/// An accumulator for ISO/IEC 7064, MOD 27,26.
pub type Mod27_26 = Hybrid<26>;

/// An accumulator for ISO/IEC 7064, MOD 37,36.
pub type Mod37_36 = Hybrid<36>;

#[cfg(test)]
mod tests {
    use super::*;

    fn accumulate_values<const N_CC: usize>(
        acc: &mut impl Accumulator<N_CC>,
        values: impl IntoIterator<Item = u32>,
    ) {
        for value in values {
            assert_eq!(acc.accumulate(value), AccumulateResult::Processed);
        }
    }

    #[test]
    fn examples_mod_11_2() {
        let mut acc = Mod11_2::default();
        accumulate_values(&mut acc, [0, 7, 9, 4]);
        assert_eq!(acc.compute(), [0]);
        accumulate_values(&mut acc, [0]);
        assert!(acc.verify());

        let mut acc = Mod11_2::default();
        accumulate_values(&mut acc, [0, 7, 9]);
        assert_eq!(acc.compute(), [10]);
        assert_eq!(acc.accumulate(10), AccumulateResult::SupplFound);
        assert!(acc.verify());
    }

    #[test]
    fn examples_mod_97_10() {
        let mut acc = Mod97_10::default();
        accumulate_values(&mut acc, [7, 9, 4]);
        assert_eq!(acc.compute(), [4, 4]);
        accumulate_values(&mut acc, [4, 4]);
        assert!(acc.verify());
    }

    #[test]
    fn examples_mod_1271_36() {
        let mut acc = Mod1271_36::default();
        accumulate_values(&mut acc, [18, 28, 24, 7, 9]);
        assert_eq!(acc.compute(), [3, 32]);
        accumulate_values(&mut acc, [3, 32]);
        assert!(acc.verify());
    }

    #[test]
    fn examples_mod_11_10() {
        let mut acc = Mod11_10::default();
        accumulate_values(&mut acc, [0, 7, 9, 4]);
        assert_eq!(acc.compute(), [5]);
        accumulate_values(&mut acc, [5]);
        assert!(acc.verify());
    }

    #[test]
    fn boundaries_pure_single() {
        fn run<const MODULUS: u32, const RADIX: u32, const CHARSET_SIZE: u32>(
            mut acc: PureSingle<MODULUS, RADIX, CHARSET_SIZE>,
        ) {
            assert_eq!(acc.suppl, None);
            for value in 0..CHARSET_SIZE {
                assert_eq!(acc.accumulate(value), AccumulateResult::Processed);
                assert_eq!(acc.suppl, None);
            }
            let carry = acc.carry;
            for value in CHARSET_SIZE..MODULUS {
                assert_eq!(acc.accumulate(value), AccumulateResult::SupplFound);
                assert_eq!(acc.carry, carry);
                assert_eq!(acc.suppl, Some(value));
            }
            let suppl = acc.suppl;
            for value in MODULUS..2048 {
                assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
                assert_eq!(acc.carry, carry);
                assert_eq!(acc.suppl, suppl);
            }
            acc.accumulate(0);
            assert_eq!(acc.suppl, None);
        }

        run(Mod11_2::default());
        run(Mod37_2::default());
        run(PureSingle::<29, 2, 29>::default());
        run(PureSingle::<57, 2, 32>::default());
    }

    #[test]
    fn boundaries_pure_double() {
        fn run<const MODULUS: u32, const CHARSET_SIZE: u32>(
            mut acc: PureDouble<MODULUS, CHARSET_SIZE>,
        ) {
            accumulate_values(&mut acc, 0..CHARSET_SIZE);

            let carry = acc.carry;
            for value in CHARSET_SIZE..2048 {
                assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
                assert_eq!(acc.carry, carry);
            }
        }

        run(Mod97_10::default());
        run(Mod661_26::default());
        run(Mod1271_36::default());
        run(PureDouble::<239, 16>::default());
        run(PureDouble::<1289, 36>::default());
    }

    #[test]
    fn boundaries_hybrid() {
        fn run<const CHARSET_SIZE: u32>(mut acc: Hybrid<CHARSET_SIZE>) {
            accumulate_values(&mut acc, 0..CHARSET_SIZE);

            let carry = acc.carry;
            for value in CHARSET_SIZE..2048 {
                assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
                assert_eq!(acc.carry, carry);
            }
        }

        run(Mod11_10::default());
        run(Mod27_26::default());
        run(Mod37_36::default());
        run(Hybrid::<16>::default());
        run(Hybrid::<64>::default());
    }

    /// Accepts alternative check character pairs of the pure double systems.
    #[test]
    fn pure_double_ambiguity() {
        fn run<Acc: Accumulator<2> + Default>(s: &[u32], cc: &[u32], cc_alt: &[u32]) {
            let mut acc = Acc::default();
            accumulate_values(&mut acc, s.iter().copied());
            assert_eq!(&acc.compute(), cc);

            accumulate_values(&mut acc, cc.iter().copied());
            assert!(acc.verify());

            let mut alt = Acc::default();
            accumulate_values(&mut alt, s.iter().copied());
            accumulate_values(&mut alt, cc_alt.iter().copied());
            assert!(alt.verify());
        }

        run::<Mod97_10>(&[9, 7], &[9, 8], &[0, 1]);
        run::<Mod661_26>(&[25, 11], &[25, 12], &[0, 1]);
        run::<Mod1271_36>(&[35, 11], &[35, 12], &[0, 1]);
    }

    /// Tests accumulators using random numbers and naive implementations of the recursive methods
    /// described in the standard.
    fn random_inner<const N_CC: usize, Acc>(
        modulus: u32,
        charset_size: u32,
        initial_p: u32,
        next_s: impl Fn(u32, u32) -> u32,
        next_p: impl Fn(u32) -> u32,
    ) where
        Acc: Accumulator<N_CC> + Default + Clone,
    {
        use rand::{RngExt as _, rngs::SmallRng};
        let mut rng: SmallRng = rand::make_rng();
        for _ in 0..8 {
            let mut acc = Acc::default();
            let mut p = initial_p;
            assert!(!acc.verify());
            for _ in 0..1024 {
                let value = rng.random_range(0..charset_size);
                acc.accumulate(value);
                let s = next_s(p, value);
                p = next_p(s);

                assert_eq!(acc.verify(), s % modulus == 1);

                let cc = acc.compute();
                let cc_expected: &[u32] = match N_CC {
                    1 => &[(modulus + 1 - p) % modulus],
                    2 => {
                        let v = modulus + 1 - next_p(next_s(p, 0));
                        &[v / charset_size, v % charset_size]
                    }
                    _ => unimplemented!(),
                };
                assert_eq!(&cc, cc_expected);

                let mut clone = acc.clone();
                for value in cc {
                    clone.accumulate(value);
                }
                assert!(clone.verify())
            }
        }
    }

    fn random_inner_pure<const N_CC: usize, Acc>(modulus: u32, radix: u32, charset_size: u32)
    where
        Acc: Accumulator<N_CC> + Default + Clone,
    {
        let next_s = |p, a| p + a;
        let next_p = move |s| s * radix % modulus;
        random_inner::<N_CC, Acc>(modulus, charset_size, 0, next_s, next_p);
    }

    fn random_inner_hybrid<Acc>(m: u32)
    where
        Acc: Accumulator<1> + Default + Clone,
    {
        let next_s = |p, a| p + a;
        let next_p = move |s| match s % m {
            0 => m * 2 % (m + 1),
            r => r * 2 % (m + 1),
        };
        random_inner::<1, Acc>(m, m, m, next_s, next_p);
    }

    #[test]
    fn random_mod11_2() {
        random_inner_pure::<1, Mod11_2>(11, 2, 10);
    }

    #[test]
    fn random_mod37_2() {
        random_inner_pure::<1, Mod37_2>(37, 2, 36);
    }

    #[test]
    fn random_mod29_2() {
        random_inner_pure::<1, PureSingle<29, 2, 29>>(29, 2, 29);
    }

    #[test]
    fn random_mod97_10() {
        random_inner_pure::<2, Mod97_10>(97, 10, 10);
    }

    #[test]
    fn random_mod661_26() {
        random_inner_pure::<2, Mod661_26>(661, 26, 26);
    }

    #[test]
    fn random_mod1271_36() {
        random_inner_pure::<2, Mod1271_36>(1271, 36, 36);
    }

    #[test]
    fn random_mod11_10() {
        random_inner_hybrid::<Mod11_10>(10);
    }

    #[test]
    fn random_mod27_26() {
        random_inner_hybrid::<Mod27_26>(26);
    }

    #[test]
    fn random_mod37_36() {
        random_inner_hybrid::<Mod37_36>(36);
    }
}
