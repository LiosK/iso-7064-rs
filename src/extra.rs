//! Types for other supported algorithms than ISO/IEC 7064.

use crate::accumulator::{AccumulateResult, Accumulator};
use crate::{charset::Numeric, spec_rem, system::System};

/// The Luhn algorithm.
pub const LUHN: System<1, LuhnAcc, Numeric, Numeric> = System::with_charset(Numeric, Numeric);

/// The standard check digit algorithm for GS1 data structures (including GTIN).
pub const GTIN: System<1, GtinAcc, Numeric, Numeric> = System::with_charset(Numeric, Numeric);

/// An accumulator for the Luhn algorithm.
#[derive(Debug, Clone, Default)]
pub struct LuhnAcc(u32, u32);

impl Accumulator for LuhnAcc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= 10 {
            AccumulateResult::NotInCharset
        } else {
            let Self(mut a, b) = *self;
            if a > u32::MAX - 10 * 2 {
                a = cold_rem::<10>(a);
            }
            self.0 = b + value;
            self.1 = a + if value < 5 { value * 2 } else { value * 2 - 9 };
            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        [spec_rem(10 - self.1 % 10, 10)]
    }

    fn verify(&self) -> bool {
        self.0 % 10 == 0
    }
}

/// An accumulator for the check digit algorithm for GS1 data structures.
#[derive(Debug, Clone, Default)]
pub struct GtinAcc(u32, u32);

impl Accumulator for GtinAcc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= 10 {
            AccumulateResult::NotInCharset
        } else {
            let Self(mut a, b) = *self;
            if a > u32::MAX - 10 {
                a = cold_rem::<10>(a);
            }
            *self = Self(b, a + value);
            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        [spec_rem(10 - gtin_sum(self.0, self.1), 10)]
    }

    fn verify(&self) -> bool {
        gtin_sum(self.1, self.0) == 0
    }
}

const fn gtin_sum(w1: u32, w3: u32) -> u32 {
    ((w1 as u64 + w3 as u64 * 3) % 10) as u32
}

#[cold]
#[inline(always)]
const fn cold_rem<const MODULUS: u32>(carry: u32) -> u32 {
    carry % MODULUS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accumulate_values(acc: &mut impl Accumulator, values: impl IntoIterator<Item = u32>) {
        for value in values {
            assert_eq!(acc.accumulate(value), AccumulateResult::Processed);
        }
    }

    #[test]
    fn examples_luhn() {
        let mut acc = LuhnAcc::default();
        accumulate_values(&mut acc, [4, 8, 7, 2, 1, 4, 8]);
        assert_eq!(acc.compute(), [4]);
        accumulate_values(&mut acc, [4]);
        assert!(acc.verify());
    }

    #[test]
    fn examples_gtin() {
        let mut acc = GtinAcc::default();
        accumulate_values(
            &mut acc,
            [3, 7, 6, 1, 0, 4, 2, 5, 0, 0, 2, 1, 2, 3, 4, 5, 6],
        );
        assert_eq!(acc.compute(), [9]);
        accumulate_values(&mut acc, [9]);
        assert!(acc.verify());
    }

    #[test]
    fn boundaries_luhn() {
        let mut acc = LuhnAcc::default();

        accumulate_values(&mut acc, 0..10);

        let carry = (acc.0, acc.1);
        for value in 10..2048 {
            assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
            assert_eq!((acc.0, acc.1), carry);
        }
    }

    #[test]
    fn boundaries_gtin() {
        let mut acc = GtinAcc::default();

        accumulate_values(&mut acc, 0..10);

        let carry = (acc.0, acc.1);
        for value in 10..2048 {
            assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
            assert_eq!((acc.0, acc.1), carry);
        }
    }

    #[test]
    fn random_luhn() {
        fn naive_luhn(values: &[u32]) -> u32 {
            const SUB: [u32; 10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { SUB[v as usize] } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        use rand::{RngExt as _, rngs::SmallRng};
        let mut rng: SmallRng = rand::make_rng();
        for _ in 0..8 {
            let mut acc = LuhnAcc::default();
            let mut values = [0; 1024];
            for i in 0..values.len() {
                values[i] = rng.random_range(0..10);
                acc.accumulate(values[i]);

                assert_eq!(acc.verify(), naive_luhn(&values[..i]) == values[i]);

                let cc = acc.compute();
                assert_eq!(cc, [naive_luhn(&values[..=i])]);

                let mut clone = acc.clone();
                for value in cc {
                    clone.accumulate(value);
                }
                assert!(clone.verify())
            }
        }
    }

    #[test]
    fn random_gtin() {
        fn naive_gtin(values: &[u32]) -> u32 {
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { v * 3 } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        use rand::{RngExt as _, rngs::SmallRng};
        let mut rng: SmallRng = rand::make_rng();
        for _ in 0..8 {
            let mut acc = GtinAcc::default();
            let mut values = [0; 1024];
            for i in 0..values.len() {
                values[i] = rng.random_range(0..10);
                acc.accumulate(values[i]);

                assert_eq!(acc.verify(), naive_gtin(&values[..i]) == values[i]);

                let cc = acc.compute();
                assert_eq!(cc, [naive_gtin(&values[..=i])]);

                let mut clone = acc.clone();
                for value in cc {
                    clone.accumulate(value);
                }
                assert!(clone.verify())
            }
        }
    }
}
