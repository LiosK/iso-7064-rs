use crate::accumulator::{AccumulateResult, Accumulator};
use crate::{charset::Numeric, cold_rem, spec_rem, system::System};

/// The Luhn algorithm.
///
/// See the [`System`] type for provided methods.
pub const LUHN: LuhnSys = System::with_charset(Numeric, Numeric);

/// The check character system interface for the Luhn algorithm.
pub type LuhnSys = System<1, LuhnAcc, Numeric, Numeric>;

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
                a = cold_rem(a, 10);
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

#[cfg(test)]
use super::random_luhn_gs1_inner;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn examples() {
        let mut acc = LuhnAcc::default();
        test_util::accumulate_values(&mut acc, [4, 8, 7, 2, 1, 4, 8]);
        assert_eq!(acc.compute(), [4]);
        test_util::accumulate_values(&mut acc, [4]);
        assert!(acc.verify());
    }

    #[test]
    fn boundaries() {
        let mut acc = LuhnAcc::default();

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
            const SUB: [u32; 10] = [0, 2, 4, 6, 8, 1, 3, 5, 7, 9];
            let mut carry = 0;
            for (i, &v) in values.iter().rev().enumerate() {
                carry += if i % 2 == 0 { SUB[v as usize] } else { v };
                carry %= 10;
            }
            (10 - carry) % 10
        }

        random_luhn_gs1_inner::<LuhnAcc>(naive_fn);
    }

    #[test]
    fn prepared() {
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

        test_util::prepared_inner(LUHN, valid, invalid);
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
