//! The [`Iban`] structure for the IBAN variant of MOD 97-10.
//!
//! The International Bank Account Number (IBAN) is one of the most famous adopters of ISO/IEC 7064,
//! whereas it employs MOD 97-10 in a distinctive configuration: the check digits are placed within
//! the string rather than appended at its end, the alphanumeric input is converted to a numeric
//! stream prior to validation, and some check digit pairs otherwise permitted under ISO/IEC 7064
//! are rejected as invalid under this variant. The [`Iban`] structure provides support for
//! computing and verifying this variant.

use core::{error, fmt};

use crate::accumulator::{Accumulator as _, Mod97_10};
use crate::charset::{Encoder as _, Numeric};

/// The IBAN variant of MOD 97-10.
pub const IBAN: Iban = Iban {};

/// The check character system interface for the IBAN variant of MOD 97-10.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Iban {}

impl Iban {
    /// Computes the check digits for the country code and basic bank account number (BBAN).
    ///
    /// # Errors
    ///
    /// Returns an [`IbanError`] if the arguments are invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::iban::IBAN;
    ///
    /// assert_eq!(IBAN.compute("GB", "BUKB20201555555555")?, ['3', '3']);
    /// # Ok::<_, iso_7064::iban::IbanError>(())
    /// ```
    pub fn compute(&self, country: &str, bank_account: &str) -> Result<[char; 2], IbanError> {
        self.compute_from_chars(country.chars(), bank_account.chars())
    }

    /// Computes the check digits from iterators generating the country code and basic bank account
    /// number (BBAN).
    ///
    /// # Errors
    ///
    /// Returns an [`IbanError`] if the arguments generate invalid strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::iban::IBAN;
    ///
    /// let ctry_iter = "NL".chars();
    /// let bban_iter = "ABNA0123456789".chars();
    /// assert_eq!(IBAN.compute_from_chars(ctry_iter, bban_iter)?, ['0', '2']);
    /// # Ok::<_, iso_7064::iban::IbanError>(())
    /// ```
    pub fn compute_from_chars(
        &self,
        country: impl IntoIterator<Item = char>,
        bank_account: impl IntoIterator<Item = char>,
    ) -> Result<[char; 2], IbanError> {
        let mut ctry_iter = country.into_iter();
        let ctry = take_two(&mut ctry_iter).ok_or(IbanError {
            kind: IbanErrorKind::MalformedCountry,
        })?;
        if ctry_iter.next().is_some() {
            return Err(IbanError {
                kind: IbanErrorKind::MalformedCountry,
            });
        }
        self.compute_inner(ctry, bank_account)
    }

    /// Verifies whether the check digits in the string `s` are valid.
    ///
    /// This method verifies only the check digits and does not validate other IBAN properties, such
    /// as country code existence or country-specific lengths and formats.
    ///
    /// # Errors
    ///
    /// Returns an [`IbanError`] if the argument is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::iban::IBAN;
    ///
    /// assert!(IBAN.verify("GB33BUKB20201555555555")?);
    /// # Ok::<_, iso_7064::iban::IbanError>(())
    /// ```
    pub fn verify(&self, s: &str) -> Result<bool, IbanError> {
        self.verify_from_chars(s.chars())
    }

    /// Verifies whether the check digits in the iterator of characters are valid.
    ///
    /// This method verifies only the check digits and does not validate other IBAN properties, such
    /// as country code existence or country-specific lengths and formats.
    ///
    /// # Errors
    ///
    /// Returns an [`IbanError`] if the argument generates an invalid string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::iban::IBAN;
    ///
    /// let iter = "NL02ABNA0123456789".chars();
    /// assert!(IBAN.verify_from_chars(iter)?);
    /// # Ok::<_, iso_7064::iban::IbanError>(())
    /// ```
    pub fn verify_from_chars(
        &self,
        chars: impl IntoIterator<Item = char>,
    ) -> Result<bool, IbanError> {
        let mut iter = chars.into_iter();
        let ctry = take_two(&mut iter).ok_or(IbanError {
            kind: IbanErrorKind::TooShort,
        })?;
        let cc = take_two(&mut iter).ok_or(IbanError {
            kind: IbanErrorKind::TooShort,
        })?;
        Ok(cc == self.compute_inner(ctry, iter)?)
    }

    fn compute_inner(
        &self,
        country: impl IntoIterator<Item = char>,
        bank_account: impl IntoIterator<Item = char>,
    ) -> Result<[char; 2], IbanError> {
        let mut acc = Mod97_10::default();

        let mut pos = 4;
        for c in bank_account {
            let value = c.to_digit(36).ok_or(IbanError {
                kind: IbanErrorKind::NonAlphanumeric(c),
            })?;
            if value < 10 {
                acc.accumulate(value);
            } else {
                acc.accumulate(value / 10);
                acc.accumulate(value % 10);
            }
            pos += 1;
        }
        if pos > 34 {
            return Err(IbanError {
                kind: IbanErrorKind::TooLong,
            });
        }

        pos = 0;
        for c in country {
            let value = c.to_digit(36).ok_or(IbanError {
                kind: IbanErrorKind::NonAlphanumeric(c),
            })?;
            if value < 10 {
                return Err(IbanError {
                    kind: IbanErrorKind::MalformedCountry,
                });
            } else {
                acc.accumulate(value / 10);
                acc.accumulate(value % 10);
            }
            pos += 1;
        }
        debug_assert_eq!(pos, 2);

        Ok(acc.compute().map(|a| Numeric.encode(a).unwrap()))
    }
}

#[inline(always)]
fn take_two<T>(it: &mut impl Iterator<Item = T>) -> Option<[T; 2]> {
    Some([it.next()?, it.next()?])
}

/// An error returned when IBAN check digit computation or verification fails.
#[derive(Debug)]
pub struct IbanError {
    kind: IbanErrorKind,
}

impl fmt::Display for IbanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl error::Error for IbanError {}

/// The specific kind of an IBAN error.
#[derive(Debug)]
enum IbanErrorKind {
    TooLong,
    TooShort,
    MalformedCountry,
    NonAlphanumeric(char),
}

impl fmt::Display for IbanErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong => write!(f, "length exceeding 34 characters"),
            Self::TooShort => write!(f, "too short to have country code and check chars"),
            Self::MalformedCountry => write!(f, "country code not two letters"),
            Self::NonAlphanumeric(c) => write!(f, "non-alphanumeric char found: '{}'", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid() {
        let examples = [
            "AL35202111090000000001234567",
            "AD1400080001001234567890",
            "AT483200000012345864",
            "AZ77VTBA00000000001234567890",
            "BH02CITI00001077181611",
            "BY86AKBB10100000002966000000",
            "BE71096123456769",
            "BA393385804800211234",
            "BR1500000000000010932840814P2",
            "BG19STSA93000123456789",
            "BI1320001100010000123456789",
            "CR23015108410026012345",
            "HR1723600001101234565",
            "CY21002001950000357001234567",
            "CZ5508000000001234567899",
            "DK9520000123456789",
            "DJ2110002010010409943020008",
            "DO22ACAU00000000000123456789",
            "EG800002000156789012345180002",
            "SV43ACAT00000000000000123123",
            "EE471000001020145685",
            "FK12SC987654321098",
            "FO0664600004330406",
            "FI1410093000123458",
            "FR7630006000011234567890189",
            "GE60NB0000000123456789",
            "DE75512108001245126199",
            "GI56XAPO000001234567890",
            "GR9608100010000001234567890",
            "GL9264710002171039",
            "GT20AGRO00000000001234567890",
            "HN54PISA00000000000000123124",
            "HU93116000060000000012345676",
            "IS750001121234563108962099",
            "IQ20CBIQ861800101010500",
            "IE06BOFI90008412345671",
            "IL170108000000012612345",
            "IT60X0542811101000000123456",
            "JO71CBJO0000000000001234567890",
            "KZ244350000012344567",
            "XK051212012345678906",
            "KW81CBKU0000000000001234560101",
            "LV97HABA0012345678910",
            "LB92000700000000123123456123",
            "LY38021001000000123456789",
            "LI7408806123456789012",
            "LT601010012345678901",
            "LU120010001234567891",
            "MT31MALT01100000000000000000123",
            "MR1300020001010000123456753",
            "MU43BOMM0101123456789101000MUR",
            "MD21EX000000000001234567",
            "MC5810096180790123456789085",
            "MN580050099123456789",
            "ME25510000012345678920",
            "NL02ABNA0123456789",
            "NI79BAMC00000000000003123123",
            "MK07200002785123453",
            "NO8330001234567",
            "PK36SCBL0000001123456702",
            "PS92PALS000000000400123456702",
            "PL10105000997603123456789123",
            "PT50003310311234567890197",
            "QA54QNBA000000000000693123456",
            "RO66BACX0000001234567890",
            "RU0204452560040702810412345678901",
            "LC14BOSL123456789012345678901234",
            "SM76P0854009812123456789123",
            "ST23000200000289355710148",
            "SA4420000001234567891234",
            "RS35105008123123123173",
            "SC74MCBL01031234567890123456USD",
            "SK8975000000000012345671",
            "SI56192001234567892",
            "SO061000001123123456789",
            "ES7921000813610123456789",
            "VA59001123000012345678",
            "SD8811123456789012",
            "OM040280000012345678901",
            "SE7280000810340009783242",
            "CH5604835012345678009",
            "TL380010012345678910106",
            "TN5904018104004942712345",
            "TR190006200009112345678901",
            "UA903052992990004149123456789",
            "AE460090000000123456789",
            "GB33BUKB20201555555555",
            "VG07ABVI0000000123456789",
            "YE20ALMF0000000000001234560101",
            "GB94BARC10201530093459",
        ];

        for iban in examples {
            assert!(IBAN.verify(iban).unwrap(), "{}", iban);
        }
    }

    #[test]
    fn reject_invalid() {
        let examples = [
            "GB94BARC20201530093459",
            "GB2LABBY09012857201707",
            "GB01BARC20714583608387",
            "GB00HLFX11016111455365",
        ];

        for iban in examples {
            assert!(!IBAN.verify(iban).unwrap(), "{}", iban);
        }
    }
}
