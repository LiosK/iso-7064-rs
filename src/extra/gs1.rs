use crate::accumulator::{AccumulateResult, Accumulator};
use crate::{charset::Numeric, cold_rem, spec_rem, system::System};

/// The standard check digit algorithm for GS1 data structures (including GTIN).
///
/// See the [`System`] type for provided methods.
pub const GS1: Gs1Sys = System::with_charset(Numeric, Numeric);

/// The check character system interface for the standard check digit algorithm for GS1 data
/// structures (including GTIN).
pub type Gs1Sys = System<1, Gs1Acc, Numeric, Numeric>;

/// An accumulator for the check digit algorithm for GS1 data structures (including GTIN).
#[derive(Debug, Clone, Default)]
pub struct Gs1Acc {
    carry: (u32, u32),
    has_processed: bool,
}

impl Accumulator for Gs1Acc {
    type Computed = [u32; 1];

    #[inline]
    fn accumulate(&mut self, value: u32) -> AccumulateResult {
        if value >= 10 {
            AccumulateResult::NotInCharset
        } else {
            let (mut a, b) = self.carry;
            if a > u32::MAX - 10 {
                a = cold_rem(a, 10);
            }
            self.carry = (b, a + value);
            self.has_processed = true;
            AccumulateResult::Processed
        }
    }

    fn compute(&self) -> Self::Computed {
        [spec_rem(10 - gs1_sum(self.carry.0, self.carry.1), 10)]
    }

    fn verify(&self) -> bool {
        self.has_processed && gs1_sum(self.carry.1, self.carry.0) == 0
    }
}

const fn gs1_sum(w1: u32, w3: u32) -> u32 {
    ((w1 as u64 + w3 as u64 * 3) % 10) as u32
}

#[cfg(test)]
use super::random_luhn_gs1_inner;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn examples() {
        let mut acc = Gs1Acc::default();
        test_util::accumulate_values(
            &mut acc,
            [3, 7, 6, 1, 0, 4, 2, 5, 0, 0, 2, 1, 2, 3, 4, 5, 6],
        );
        assert_eq!(acc.compute(), [9]);
        test_util::accumulate_values(&mut acc, [9]);
        assert!(acc.verify());
    }

    #[test]
    fn boundaries() {
        let mut acc = Gs1Acc::default();

        test_util::accumulate_values(&mut acc, 0..10);

        let carry = acc.carry;
        for value in 10..2048 {
            assert_eq!(acc.accumulate(value), AccumulateResult::NotInCharset);
            assert_eq!(acc.carry, carry);
        }
    }

    #[test]
    fn random() {
        fn naive_fn(values: &[u32]) -> u32 {
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { v * 3 } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        random_luhn_gs1_inner::<Gs1Acc>(naive_fn);
    }

    #[test]
    fn prepared() {
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

        test_util::prepared_inner(GS1, valid, invalid);
    }
}
