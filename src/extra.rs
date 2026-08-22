//! Check character algorithms other than ISO/IEC 7064.
//!
//! This module provides [`System`] instances and [`Accumulator`] implementations for other widely
//! used check digit algorithms:
//!
//! - [`LUHN`]: The Luhn algorithm (also known as the MOD 10 algorithm), widely used in credit card
//!   numbers, IMEI numbers, and National Provider Identifiers.
//! - [`GTIN`]: The standard check digit algorithm for GS1 data structures (including GTIN-8,
//!   GTIN-12 / UPC, GTIN-13 / EAN-13, GTIN-14, and SSCC).
//!
//! ```rust
//! use iso_7064::extra::{GTIN, LUHN};
//!
//! assert_eq!(LUHN.compute("1789372997")?, ['4']);
//! assert!(LUHN.verify("17893729974")?);
//!
//! assert_eq!(GTIN.compute("9761234500001")?, ['8']);
//! assert!(GTIN.verify("97612345000018")?);
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

use crate::accumulator::{AccumulateResult, Accumulator};
use crate::{charset::Numeric, spec_rem, system::System};

/// The Luhn algorithm.
///
/// See the [`System`] type for provided methods.
pub const LUHN: Luhn = System::with_charset(Numeric, Numeric);

/// The standard check digit algorithm for GS1 data structures (including GTIN).
///
/// See the [`System`] type for provided methods.
pub const GTIN: Gtin = System::with_charset(Numeric, Numeric);

/// The Luhn algorithm.
pub type Luhn = System<1, LuhnAcc, Numeric, Numeric>;

/// The standard check digit algorithm for GS1 data structures (including GTIN).
pub type Gtin = System<1, GtinAcc, Numeric, Numeric>;

/// An accumulator for the Luhn algorithm.
#[derive(Debug, Clone, Default)]
pub struct LuhnAcc {
    carry: (u32, u32),
    has_processed: bool,
}

impl Accumulator for LuhnAcc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= 10 {
            AccumulateResult::NotInCharset
        } else {
            let (mut a, b) = self.carry;
            if a > u32::MAX - 10 * 2 {
                a = cold_rem::<10>(a);
            }
            self.carry.0 = b + value;
            self.carry.1 = a + if value < 5 { value * 2 } else { value * 2 - 9 };
            self.has_processed = true;
            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        [spec_rem(10 - self.carry.1 % 10, 10)]
    }

    fn verify(&self) -> bool {
        self.has_processed && self.carry.0 % 10 == 0
    }
}

/// An accumulator for the check digit algorithm for GS1 data structures (including GTIN).
#[derive(Debug, Clone, Default)]
pub struct GtinAcc {
    carry: (u32, u32),
    has_processed: bool,
}

impl Accumulator for GtinAcc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= 10 {
            AccumulateResult::NotInCharset
        } else {
            let (mut a, b) = self.carry;
            if a > u32::MAX - 10 {
                a = cold_rem::<10>(a);
            }
            self.carry = (b, a + value);
            self.has_processed = true;
            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        [spec_rem(10 - gtin_sum(self.carry.0, self.carry.1), 10)]
    }

    fn verify(&self) -> bool {
        self.has_processed && gtin_sum(self.carry.1, self.carry.0) == 0
    }
}

const fn gtin_sum(w1: u32, w3: u32) -> u32 {
    ((w1 as u64 + w3 as u64 * 3) % 10) as u32
}

#[cold]
const fn cold_rem<const MODULUS: u32>(carry: u32) -> u32 {
    carry % MODULUS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepared_inner;

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

        let carry = acc.carry;
        for value in 10..2048 {
            assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
            assert_eq!(acc.carry, carry);
        }
    }

    #[test]
    fn boundaries_gtin() {
        let mut acc = GtinAcc::default();

        accumulate_values(&mut acc, 0..10);

        let carry = acc.carry;
        for value in 10..2048 {
            assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
            assert_eq!(acc.carry, carry);
        }
    }

    fn random_inner<Acc>(naive_fn: impl Fn(&[u32]) -> u32)
    where
        Acc: Accumulator<Computed = [u32; 1]> + Default + Clone,
    {
        use core::array;
        use rand::{RngExt as _, rngs::SmallRng};
        let mut rng: SmallRng = rand::make_rng();
        for _ in 0..8 {
            let values: [_; 1024] = array::from_fn(|_| rng.random_range(0..10));
            let mut acc = Acc::default();
            assert!(!acc.verify());
            for i in 0..values.len() {
                acc.accumulate(values[i]);

                assert_eq!(acc.verify(), naive_fn(&values[..i]) == values[i]);

                let cc = acc.compute();
                assert_eq!(cc, [naive_fn(&values[..=i])]);

                let mut clone = acc.clone();
                for value in cc {
                    clone.accumulate(value);
                }
                assert!(clone.verify())
            }
        }
    }

    #[test]
    fn random_luhn() {
        fn naive_fn(values: &[u32]) -> u32 {
            const SUB: [u32; 10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { SUB[v as usize] } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        random_inner::<LuhnAcc>(naive_fn);
    }

    #[test]
    fn random_gtin() {
        fn naive_fn(values: &[u32]) -> u32 {
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { v * 3 } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        random_inner::<GtinAcc>(naive_fn);
    }

    #[test]
    fn prepared_luhn() {
        let valid = &[
            "546623412242598542363",
            "429869407",
            "697482027002074929657",
            "24427809",
            "40945102099",
            "7765910180673709242",
            "0334754482118806638",
            "353159977932780875001",
            "24221502320077397",
            "55281270707218379",
            "42861648824",
            "60932589014642580",
        ];

        let invalid = &[
            "743638155913",
            "89472241187133",
            "741010972016880174648",
            "6044520638804741920",
            "51212175412516688233",
            "76245165717037759",
            "6366487495713270993924",
            "23742667215749",
            "2986089939865",
            "44932344897270",
            "16693589271944940",
            "2222675562",
        ];

        prepared_inner(LUHN, valid, invalid);
    }

    #[test]
    fn prepared_gtin() {
        let valid = &[
            "9201696531",
            "49950385619451233889",
            "36147121450673678056505",
            "89051525687652679018",
            "177945840022586907",
            "040966595",
            "4616160305150",
            "32573277760920769",
            "125119223031",
            "53748599212771",
            "858821862656667",
            "64718330",
        ];

        let invalid = &[
            "1320267506819606688870",
            "3244530282",
            "9808980103049501991713",
            "10830077677580",
            "72636966152958653",
            "097892315743878",
            "83641903781222688830",
            "49941940257492255012045",
            "17618635417724453339",
            "1645108330301674",
            "9645092905763",
            "61928210334114471550661",
        ];

        prepared_inner(GTIN, valid, invalid);
    }

    #[test]
    fn decode_isin() {
        let sys: System<1, LuhnAcc, _, _> = System::with_charset(Numeric, |c: char| {
            c.to_digit(36).map(|v| match v {
                ..=9 => [v, 0].into_iter().take(1),
                10.. => [v / 10, v % 10].into_iter().take(2),
            })
        });

        let isin = "US0378331005";
        assert!(sys.verify(isin).unwrap());

        let mut chars = isin.chars();
        let cc = chars.next_back().unwrap();
        assert_eq!(sys.compute(chars.as_str()).unwrap(), [cc]);
    }
}
