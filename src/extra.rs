//! Types for other supported algorithms than ISO/IEC 7064.

use crate::accumulator::{AccumulateResult, Accumulator};
use crate::{charset::Numeric, spec_rem, system::System};

/// The Luhn algorithm.
pub const LUHN: System<1, LuhnAcc, Numeric, Numeric> = System::with_charset(Numeric, Numeric);

const LUHN_MODULUS: u32 = 10;

/// An accumulator for the Luhn algorithm.
#[derive(Debug, Clone, Default)]
pub struct LuhnAcc(u32, u32);

impl Accumulator for LuhnAcc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= LUHN_MODULUS {
            AccumulateResult::NotInCharset
        } else {
            let Self(mut a, b) = *self;
            if a > u32::MAX - LUHN_MODULUS * 2 {
                a = cold_rem::<LUHN_MODULUS>(a);
            }

            self.0 = b + value;
            self.1 = a + if value < LUHN_MODULUS / 2 {
                value * 2
            } else {
                value * 2 - (LUHN_MODULUS - 1)
            };

            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        let carry = self.1 % LUHN_MODULUS;
        [spec_rem(LUHN_MODULUS - carry, LUHN_MODULUS)]
    }

    fn verify(&self) -> bool {
        self.0 % LUHN_MODULUS == 0
    }
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
    fn boundaries_luhn() {
        let mut acc = LuhnAcc::default();

        accumulate_values(&mut acc, 0..LUHN_MODULUS);

        let carry = (acc.0, acc.1);
        for value in LUHN_MODULUS..2048 {
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
                values[i] = rng.random_range(0..LUHN_MODULUS);
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
}
