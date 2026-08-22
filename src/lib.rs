//! # A Rust implementation of ISO/IEC 7064:2003 Check character systems
//!
//! This crate provides `no_std`-compatible implementations of the check character
//! (digit) systems specified by [ISO/IEC 7064:2003].
//!
//! [ISO/IEC 7064:2003]: https://www.iso.org/standard/31531.html
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
//!
//! This crate supports all the eight check character systems as shown below
//! specified by the standard:
//!
//! | System                    | Type   | Input string            | Check character(s)                    |
//! | ------------------------- | ------ | ----------------------- | ------------------------------------- |
//! | ISO/IEC 7064, MOD 11-2    | Pure   | Numeric (`0-9`)         | 1 digit or `'X'` (`0-9X`)             |
//! | ISO/IEC 7064, MOD 37-2    | Pure   | Alphanumeric (`0-9A-Z`) | 1 digit, letter, or `'*'` (`0-9A-Z*`) |
//! | ISO/IEC 7064, MOD 97-10   | Pure   | Numeric (`0-9`)         | 2 digits (`0-9`)                      |
//! | ISO/IEC 7064, MOD 661-26  | Pure   | Alphabetic (`A-Z`)      | 2 letters (`A-Z`)                     |
//! | ISO/IEC 7064, MOD 1271-36 | Pure   | Alphanumeric (`0-9A-Z`) | 2 digits or letters (`0-9A-Z`)        |
//! | ISO/IEC 7064, MOD 11,10   | Hybrid | Numeric (`0-9`)         | 1 digit (`0-9`)                       |
//! | ISO/IEC 7064, MOD 27,26   | Hybrid | Alphabetic (`A-Z`)      | 1 letter (`A-Z`)                      |
//! | ISO/IEC 7064, MOD 37,36   | Hybrid | Alphanumeric (`0-9A-Z`) | 1 digit or letter (`0-9A-Z`)          |
//!
//! This library also provides support for the variant of MOD 97-10 used in the
//! International Bank Account Number (IBAN).
//!
//! ## Crate features
//!
//! - `alloc` (enabled by default) enables APIs operating over `String`.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod accumulator;
pub mod charset;
pub mod extra;
pub mod system;

use system::System;

/// The ISO/IEC 7064, MOD 11-2 pure system with a single check character.
///
/// See the [`System`] type for provided methods.
pub const MOD11_2: system::Mod11_2 = System::with_charset(charset::NumericX, charset::NumericX);

/// The ISO/IEC 7064, MOD 37-2 pure system with a single check character.
///
/// See the [`System`] type for provided methods.
pub const MOD37_2: system::Mod37_2 =
    System::with_charset(charset::AlphanumericAst, charset::AlphanumericAst);

/// The ISO/IEC 7064, MOD 97-10 pure system with two check characters.
///
/// See the [`System`] type for provided methods.
pub const MOD97_10: system::Mod97_10 = System::with_charset(charset::Numeric, charset::Numeric);

/// The ISO/IEC 7064, MOD 661-26 pure system with two check characters.
///
/// See the [`System`] type for provided methods.
pub const MOD661_26: system::Mod661_26 =
    System::with_charset(charset::Alphabetic, charset::Alphabetic);

/// The ISO/IEC 7064, MOD 1271-36 pure system with two check characters.
///
/// See the [`System`] type for provided methods.
pub const MOD1271_36: system::Mod1271_36 =
    System::with_charset(charset::Alphanumeric, charset::Alphanumeric);

/// The ISO/IEC 7064, MOD 11,10 hybrid system.
///
/// See the [`System`] type for provided methods.
pub const MOD11_10: system::Mod11_10 = System::with_charset(charset::Numeric, charset::Numeric);

/// The ISO/IEC 7064, MOD 27,26 hybrid system.
///
/// See the [`System`] type for provided methods.
pub const MOD27_26: system::Mod27_26 =
    System::with_charset(charset::Alphabetic, charset::Alphabetic);

/// The ISO/IEC 7064, MOD 37,36 hybrid system.
///
/// See the [`System`] type for provided methods.
pub const MOD37_36: system::Mod37_36 =
    System::with_charset(charset::Alphanumeric, charset::Alphanumeric);

#[inline(always)]
const fn spec_rem(lhs: u32, rhs: u32) -> u32 {
    debug_assert!(lhs < rhs * 2);
    if lhs < rhs { lhs } else { lhs - rhs }
}

#[cfg(test)]
fn prepared_inner<const N_CC: usize, Acc, Enc, Dec>(
    sys: System<N_CC, Acc, Enc, Dec>,
    valid: &[&str],
    invalid: &[&str],
) where
    Acc: accumulator::Accumulator<Computed = [u32; N_CC]> + Default,
    Enc: charset::Encoder,
    Dec: charset::Decoder,
{
    for &s in valid {
        assert!(sys.verify(s).unwrap());
        assert!(sys.verify_lax(s));
        assert!(sys.verify_from_chars(s.chars()).unwrap());

        let (u, cc) = s.split_at(s.len() - N_CC);
        assert!(cc.chars().eq(sys.compute(u).unwrap()));
        assert!(cc.chars().eq(sys.compute_lax(u)));
        assert!(cc.chars().eq(sys.compute_from_chars(u.chars()).unwrap()));
    }

    for &s in invalid {
        assert!(!sys.verify(s).unwrap());
        assert!(!sys.verify_lax(s));
        assert!(!sys.verify_from_chars(s.chars()).unwrap());

        let (u, cc) = s.split_at(s.len() - N_CC);
        assert!(!cc.chars().eq(sys.compute(u).unwrap()));
        assert!(!cc.chars().eq(sys.compute_lax(u)));
        assert!(!cc.chars().eq(sys.compute_from_chars(u.chars()).unwrap()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_mod11_2() {
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
    fn examples_mod97_10() {
        assert_eq!(MOD97_10.compute("794").unwrap(), ['4', '4']);
        assert_eq!(MOD97_10.compute_lax("794"), ['4', '4']);
        assert_eq!(MOD97_10.compute_lax("{7-9-4}"), ['4', '4']);

        assert!(MOD97_10.verify("79444").unwrap());
        assert!(MOD97_10.verify_lax("79444"));
        assert!(MOD97_10.verify_lax("{7-9-4}[4, 4]"));
    }

    #[test]
    fn examples_mod1271_36() {
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
    fn examples_mod11_10() {
        assert_eq!(MOD11_10.compute("0794").unwrap(), ['5']);
        assert_eq!(MOD11_10.compute_lax("0794"), ['5']);
        assert_eq!(MOD11_10.compute_lax("{0-7-9-4}"), ['5']);

        assert!(MOD11_10.verify("07945").unwrap());
        assert!(MOD11_10.verify_lax("07945"));
        assert!(MOD11_10.verify_lax("{0-7-9-4}[5]"));
    }

    #[test]
    fn prepared_mod11_2() {
        let valid = &[
            "5236022646915810116849",
            "845064805931988",
            "85304841090903666744",
            "337688112X",
            "92004002654",
            "21030484337645X",
            "41818941714",
            "30465694",
            "91184666177593810018",
            "17541621184078843",
            "43494101",
            "11195647730159049192304",
        ];

        let invalid = &[
            "1687459121",
            "31888679450464",
            "9188124336537439",
            "42079822636786",
            "771154512981690148767X",
            "66246345130",
            "671757580705",
            "2387040463142667",
            "44369839572905",
            "527981147467713",
            "81511264248755711569723",
            "91229388880062347394",
        ];

        prepared_inner(MOD11_2, valid, invalid);
    }

    #[test]
    fn prepared_mod37_2() {
        let valid = &[
            "PX4Z7MWY2YJ0LUGL29CUO",
            "3B99E8TMZLCX77R7YIEX",
            "TVBAUKJOL34ZZ4",
            "L52HZG049GKPMXOQ9",
            "E4EV3G1AEY1ISEAEQY",
            "9C9AU99B19GBB7KD2U3V*",
            "5AJQE3G82BEUD7Q41",
            "5EV36MX8ZFI4",
            "7G511LDGP53RN3QVDNB",
            "V4RL7V4E7OMW",
            "WV4XMAFC",
            "JH5LGP0C",
        ];

        let invalid = &[
            "KQATAYZTVO1LGYC7J",
            "0G8F99YCK*",
            "WN76K4IM16V",
            "U80YBMPJF3L71BDXWM9G7Q",
            "A36C7IR7FX9DA0FXU",
            "P64AFP7S7W61I8Y2OZ*",
            "XLME2YI2LB95QEMW9HV",
            "8FNA95QY0NON8O5G8SZD7B8",
            "H862IMPF6C5QT",
            "16826SC2CUM",
            "6LMBNSX15JL0",
            "1RH1UQOSY5OQSJU28MUP0U*",
        ];

        prepared_inner(MOD37_2, valid, invalid);
    }

    #[test]
    fn prepared_mod97_10() {
        let valid = &[
            "4331246445963138455002",
            "2911233259",
            "4764789672",
            "43554844",
            "05917435345034747416569",
            "9366261837206291",
            "262338921851557782765",
            "922783905474546408679",
            "93719954279014181",
            "23516238132346714043",
            "40563587767431",
            "7752266839610",
        ];

        let invalid = &[
            "09827197304756",
            "4684723585",
            "5490749315043",
            "32956005",
            "42985358018829090107143",
            "2583231639143482",
            "10057126",
            "81005043673147533591",
            "600389186253702",
            "844911938825",
            "4611232777673301",
            "653007253600637706",
        ];

        prepared_inner(MOD97_10, valid, invalid);
    }

    #[test]
    fn prepared_mod661_26() {
        let valid = &[
            "FSBULOLEILVNJ",
            "SICFPGUUV",
            "IRMQSMWHFDSXDXQPO",
            "TIPHZXSSMPUZARRFTE",
            "VNMVULKG",
            "BQZPCJYDUCF",
            "NCYUUZBBTAUNUL",
            "BWJVHGTKIC",
            "TFFJYFKEEGBWPEADDEA",
            "SCYMCUKLL",
            "PRBUNOAUIP",
            "JJPSKEDMBKJZ",
        ];

        let invalid = &[
            "QLMLVMME",
            "JCCPQKYDCYCMGGKUDO",
            "HZXWMOWXPAMLVGB",
            "LQPRRTHTB",
            "ZHFQHVETSNZLKQHGRWB",
            "EOOKYXQARTHWUTSJJMOZT",
            "ZWUWDYEPZNTSLQI",
            "DKYEHWOZTHTGMBECYPV",
            "BPJPJMZGASHRSNLZTWWRLZ",
            "PIKGQNCQLEAYMTUVQULQQVC",
            "SIIOQOBFIJMECNCMQBV",
            "HGXBYJGF",
        ];

        prepared_inner(MOD661_26, valid, invalid);
    }

    #[test]
    fn prepared_mod1271_36() {
        let valid = &[
            "7DJO6YKPZVNMZUCMY",
            "2RAG0WGS40GUTZ0PZ",
            "ZALU8MTOAARFAI",
            "28ZGRKLKQ0I5NEXYRUYLCH4",
            "YZY2BIE08YJ",
            "M2K96UOWU8CUI9GL0",
            "X7SPAGS8GTQULHV66CF2",
            "IYOMJKMY2AVRQUC2Y8YDG",
            "UY1QB886O1OI51YO",
            "IAASX6SF4O7",
            "XLBNR9KOETXV6SYTPJ1W",
            "HHYOH4Y8L6W",
        ];

        let invalid = &[
            "EOMS5TJROR7EAD823QS8ZC",
            "HR0LI5NSSULBLI",
            "HQB7BQP4CMKPSTGAV5VJA",
            "39QYCR54",
            "GQWTGSA4ED",
            "96M4PEEYTONCB1N99O",
            "3MAL1WQFYJXQ3N",
            "9ZT8POIU26",
            "C7OG52Y3MAI3ZFJ0BM87Y0",
            "LS3Y9NFXCUUPSM06RVW06FQ",
            "5XSPZWCPYHTEJK3A4I8O72Q",
            "3M1ZN4WOJ8",
        ];

        prepared_inner(MOD1271_36, valid, invalid);
    }

    #[test]
    fn prepared_mod11_10() {
        let valid = &[
            "72120953571944888637",
            "630956234645",
            "19090053234",
            "6089258076",
            "14436084789",
            "439932244435432",
            "35786443",
            "8171133511285059261",
            "13612873",
            "76819644",
            "6584712648951475",
            "18273960618480754278247",
        ];

        let invalid = &[
            "623644773245095632456",
            "954353828",
            "10097633924659319735",
            "3439643114562420",
            "324267175",
            "569965205768369754942",
            "26813819575426447116",
            "320201360",
            "996439490488298937425",
            "2109782058833871",
            "76377504913003",
            "543551782",
        ];

        prepared_inner(MOD11_10, valid, invalid);
    }

    #[test]
    fn prepared_mod27_26() {
        let valid = &[
            "DNWCSEWBUSMXUHMNVEC",
            "FKNJRXARTPHMFSP",
            "JLWVIPKDZKMRENSERY",
            "ZRFJGLJRQLVOHJCCWKF",
            "GHCIBFCYI",
            "DMMWTSESBKAVQYFA",
            "ZDMTSOWIXITFONEV",
            "WXZNFPLKFAOPTSVMZBYXIXU",
            "LXMBTJOXBJPL",
            "JBVNWCIZRKRWPUGNFT",
            "LHRXBWIVW",
            "YZBROGVCIBDNNOFAB",
        ];

        let invalid = &[
            "GFBZWRJEKNLGEIJIOWFEA",
            "WQCJRWJVWIKIIKUBPNMEYD",
            "PGWQIBTKRUUHDITSINFWWK",
            "OROZGAHCJKJAKTV",
            "NJHTTTHVEAHGSXFCHEW",
            "PVKKNFOIRIECA",
            "SESQRHGOFIQMWKNHGK",
            "MRPLXNZN",
            "YSLZRNMDHWPT",
            "ESXAAHYPRHQBDQ",
            "UCGXCPIZWCRJCIKIMSJVZK",
            "QYFMMQPXIMLOAP",
        ];

        prepared_inner(MOD27_26, valid, invalid);
    }

    #[test]
    fn prepared_mod37_36() {
        let valid = &[
            "6GBVUSQIFXCNTK41WO",
            "D1VOWY4VV42I2",
            "FQJA83FUO2Q2AOZ4NST0G2X",
            "74EO6Z4EJL5IMNAQJPOVEYI",
            "MSG9S4YXQ",
            "SROAMRVX991KV",
            "QH53EF8CSMHIZ69TS1HK",
            "EN5EAXIRLW1DHR42YOR",
            "UPFRXRSFEWR",
            "A97XG2LJ4JLU9LLAJEMN",
            "T4KGMICAHB08ZTPR",
            "QEVA9ODUCO",
        ];

        let invalid = &[
            "AP9A8MNRTZC59GLSXR",
            "XX59GK1HUKTFZP5B",
            "R8WYEAKDVJ4FMH",
            "PR9LJ8DJE0L",
            "5B8JO87TBSMMR1XN",
            "KS27KRVNHG4NMQU3ON5",
            "HEN88FTBS0GZBFMFXO",
            "MV9VR3QH",
            "ZWSFEZ8XFQ1XPGS18AWORFW",
            "Y51MR7862PK",
            "DWEG7LA6FMJ2CJGXMM",
            "FU8SP3V1JOTICTZ",
        ];

        prepared_inner(MOD37_36, valid, invalid);
    }
}
