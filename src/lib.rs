//! A Rust implementation of ISO/IEC 7064:2003 Check character systems
//!
//! ```rust
//! use iso_7064::{MOD11_2, MOD1271_36};
//!
//! assert_eq!(MOD11_2.compute("079")?, ['X']);
//! assert_eq!(MOD11_2.compute_lax("{0-7-9}"), ['X']);
//!
//! assert!(MOD11_2.verify("079X")?);
//! assert!(MOD11_2.verify_lax("{0-7-9}[X]"));
//! assert!(MOD11_2.verify_from_values([0, 7, 9, 10])?);
//! assert!(!MOD11_2.verify_from_chars("0790".chars())?);
//!
//! # #[cfg(feature = "alloc")]
//! # {
//! let mut buf = String::from("ISO 79");
//! MOD1271_36.protect_lax(&mut buf);
//! assert_eq!(buf, "ISO 793W");
//! # }
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod accumulator;
pub mod charset;
pub mod system;

/// The ISO/IEC 7064, MOD 11-2 pure system with a single check character.
pub const MOD11_2: system::Mod11_2 =
    system::Mod11_2::with_charset(charset::NumericX, charset::NumericX);

/// The ISO/IEC 7064, MOD 37-2 pure system with a single check character.
pub const MOD37_2: system::Mod37_2 =
    system::Mod37_2::with_charset(charset::AlphanumericAst, charset::AlphanumericAst);

/// The ISO/IEC 7064, MOD 97-10 pure system with two check characters.
pub const MOD97_10: system::Mod97_10 =
    system::Mod97_10::with_charset(charset::Numeric, charset::Numeric);

/// The ISO/IEC 7064, MOD 661-26 pure system with two check characters.
pub const MOD661_26: system::Mod661_26 =
    system::Mod661_26::with_charset(charset::Alphabetic, charset::Alphabetic);

/// The ISO/IEC 7064, MOD 1271-36 pure system with two check characters.
pub const MOD1271_36: system::Mod1271_36 =
    system::Mod1271_36::with_charset(charset::Alphanumeric, charset::Alphanumeric);

/// The ISO/IEC 7064, MOD 11,10 hybrid system.
pub const MOD11_10: system::Mod11_10 =
    system::Mod11_10::with_charset(charset::Numeric, charset::Numeric);

/// The ISO/IEC 7064, MOD 27,26 hybrid system.
pub const MOD27_26: system::Mod27_26 =
    system::Mod27_26::with_charset(charset::Alphabetic, charset::Alphabetic);

/// The ISO/IEC 7064, MOD 37,36 hybrid system.
pub const MOD37_36: system::Mod37_36 =
    system::Mod37_36::with_charset(charset::Alphanumeric, charset::Alphanumeric);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_mod_11_2() {
        assert_eq!(MOD11_2.compute("0794").unwrap(), ['0']);
        assert_eq!(MOD11_2.compute_lax("0794"), ['0']);
        assert_eq!(MOD11_2.compute_lax("{0-7-9-4}"), ['0']);

        assert!(MOD11_2.verify("07940").unwrap());
        assert!(MOD11_2.verify_lax("07940"));
        assert!(MOD11_2.verify_lax("{0-7-9-4}[0]"));

        assert_eq!(MOD11_2.compute("079").unwrap(), ['X']);
        assert_eq!(MOD11_2.compute_lax("079"), ['X']);
        assert_eq!(MOD11_2.compute_lax("{0-7-9}"), ['X']);

        assert!(MOD11_2.verify("079X").unwrap());
        assert!(MOD11_2.verify_lax("079X"));
        assert!(MOD11_2.verify_lax("{0-7-9}[X]"));
    }

    #[test]
    fn examples_mod_97_10() {
        assert_eq!(MOD97_10.compute("794").unwrap(), ['4', '4']);
        assert_eq!(MOD97_10.compute_lax("794"), ['4', '4']);
        assert_eq!(MOD97_10.compute_lax("{7-9-4}"), ['4', '4']);

        assert!(MOD97_10.verify("79444").unwrap());
        assert!(MOD97_10.verify_lax("79444"));
        assert!(MOD97_10.verify_lax("{7-9-4}[4, 4]"));
    }

    #[test]
    fn examples_mod_1271_36() {
        assert_eq!(MOD1271_36.compute("ISO79").unwrap(), ['3', 'W']);
        assert_eq!(MOD1271_36.compute_lax("ISO79"), ['3', 'W']);
        assert_eq!(MOD1271_36.compute_lax("ISO 79"), ['3', 'W']);
        assert_eq!(MOD1271_36.compute_lax("{I-S-O 7-9}"), ['3', 'W']);

        assert!(MOD1271_36.verify("ISO793W").unwrap());
        assert!(MOD1271_36.verify_lax("ISO793W"));
        assert!(MOD1271_36.verify_lax("ISO 793W"));
        assert!(MOD1271_36.verify_lax("{I-S-O 7-9}[3, W]"));
    }

    #[test]
    fn examples_mod_11_10() {
        assert_eq!(MOD11_10.compute("0794").unwrap(), ['5']);
        assert_eq!(MOD11_10.compute_lax("0794"), ['5']);
        assert_eq!(MOD11_10.compute_lax("{0-7-9-4}"), ['5']);

        assert!(MOD11_10.verify("07945").unwrap());
        assert!(MOD11_10.verify_lax("07945"));
        assert!(MOD11_10.verify_lax("{0-7-9-4}[5]"));
    }
}
