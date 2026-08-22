//! Check character algorithms other than ISO/IEC 7064.
//!
//! This module provides implementations for other widely used check digit algorithms:
//!
//! - [`LUHN`]: The Luhn algorithm (also known as the MOD 10 algorithm), widely used in credit card
//!   numbers, IMEI numbers, and National Provider Identifiers.
//! - [`GS1`]: The standard check digit algorithm for GS1 data structures (including GTIN-8, GTIN-12
//!   / UPC, GTIN-13 / EAN-13, GTIN-14, and SSCC).
//! - [`IBAN`]: The International Bank Account Number (IBAN), which employs MOD 97-10 in a
//!   distinctive configuration.
//!
//! ```rust
//! use iso_7064::extra::{GS1, IBAN, LUHN};
//!
//! assert_eq!(LUHN.compute("1789372997")?, ['4']);
//! assert!(LUHN.verify("17893729974")?);
//!
//! assert_eq!(GS1.compute("9761234500001")?, ['8']);
//! assert!(GS1.verify("97612345000018")?);
//!
//! assert_eq!(IBAN.compute("GB", "NWBK60161331926819")?, ['2', '9']);
//! assert!(IBAN.verify("GB29NWBK60161331926819")?);
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

mod gs1;
mod iban;
mod luhn;

pub use gs1::{GS1, Gs1Acc, Gs1Sys};
pub use iban::{IBAN, IbanError, IbanSys};
pub use luhn::{LUHN, LuhnAcc, LuhnSys};

#[cfg(test)]
fn random_luhn_gs1_inner<Acc>(naive_fn: impl Fn(&[u32]) -> u32)
where
    Acc: crate::accumulator::Accumulator<Computed = [u32; 1]> + Default + Clone,
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
