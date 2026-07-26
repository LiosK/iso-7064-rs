//! The [`System`] structure providing a check character system interface.
//!
//! ```rust
//! use iso_7064::{accumulator, charset, system::System};
//!
//! // Build a custom MOD 11,10 with case-insensitive alphabetic character set.
//! let my_mod11_10 =
//!     System::<1, accumulator::Mod11_10, _, _>::with_charset(charset::Alphabetic, |c: char| {
//!         c.to_digit(36)?.checked_sub(10)
//!     });
//!
//! assert_eq!(my_mod11_10.compute("AhJe")?, ['F']);
//! assert!(my_mod11_10.verify("aHjEf")?);
//! assert!(my_mod11_10.verify("AhJe5").is_err());
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

use core::{error, fmt, marker};

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::accumulator::{self, AccumulateResult, Accumulator};
use crate::charset::{self, Decoder, Encoder};

/// A generic facade structure combining [`Accumulator`] and character set into a check character
/// system interface.
#[derive(Debug, Default)]
pub struct System<const N_CC: usize, Acc, Enc, Dec> {
    _acc: marker::PhantomData<Acc>,
    encoder: Enc,
    decoder: Dec,
}

impl<const N_CC: usize, Acc, Enc, Dec> System<N_CC, Acc, Enc, Dec>
where
    Acc: Accumulator<Computed = [u32; N_CC]> + Default,
    Enc: Encoder,
    Dec: Decoder,
{
    /// Computes the check characters for the string `s` and appends them.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD11_2;
    ///
    /// let mut buf = String::from("327");
    /// MOD11_2.protect(&mut buf)?;
    /// assert_eq!(buf, "327X");
    /// # Ok::<_, iso_7064::system::ComputeError<_>>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn protect(&self, s: &mut alloc::string::String) -> Result<(), ComputeError<char>> {
        self.compute(s).map(|cc| s.extend(cc))
    }

    /// Computes the check characters for the string `s` and appends them, ignoring any invalid
    /// characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD11_2;
    ///
    /// let mut buf = String::from("3.2.7.");
    /// MOD11_2.protect_lax(&mut buf);
    /// assert_eq!(buf, "3.2.7.X");
    /// ```
    #[cfg(feature = "alloc")]
    pub fn protect_lax(&self, s: &mut alloc::string::String) {
        s.extend(self.compute_lax(s));
    }

    /// Computes the check characters for the string `s`.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD37_2;
    ///
    /// assert_eq!(MOD37_2.compute("5S7U")?, ['G']);
    /// # Ok::<_, iso_7064::system::ComputeError<_>>(())
    /// ```
    pub fn compute(&self, s: &str) -> Result<[char; N_CC], ComputeError<char>> {
        self.compute_from_chars(s.chars())
    }

    /// Computes the check characters for the string `s`, ignoring any invalid characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD37_2;
    ///
    /// assert_eq!(MOD37_2.compute_lax("5S=7U"), ['G']);
    /// ```
    pub fn compute_lax(&self, s: &str) -> [char; N_CC] {
        let vs = self.lax_from_iter(s.chars()).compute();
        vs.map(|v| self.force_encode(v))
    }

    /// Computes the check characters from an iterator of characters.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD11_10;
    ///
    /// let iter = ['2', '0', '6', '5', '1'];
    /// assert_eq!(MOD11_10.compute_from_chars(iter)?, ['8']);
    /// # Ok::<_, iso_7064::system::ComputeError<_>>(())
    /// ```
    pub fn compute_from_chars(
        &self,
        chars: impl IntoIterator<Item = char>,
    ) -> Result<[char; N_CC], ComputeError<char>> {
        self.compute_from_iter(chars)
            .map(|vs| vs.map(|v| self.force_encode(v)))
    }

    fn force_encode(&self, v: u32) -> char {
        const ERR: &str = "invalid charset implementation";
        self.encoder.encode(v).expect(ERR)
    }
}

impl<const N_CC: usize, Acc, Enc, Dec> System<N_CC, Acc, Enc, Dec>
where
    Acc: Accumulator + Default,
    Dec: Decoder,
{
    /// Verifies whether the check characters in the string `s` are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any character is not in the character set, or if a
    /// supplementary check character (e.g., `X` or `*`) is found before the end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD661_26;
    ///
    /// assert!(MOD661_26.verify("MVEISJV")?);
    /// # Ok::<_, iso_7064::system::VerifyError<_>>(())
    /// ```
    pub fn verify(&self, s: &str) -> Result<bool, VerifyError<char>> {
        self.verify_from_chars(s.chars())
    }

    /// Verifies whether the check characters in the string `s` are valid, ignoring any invalid
    /// characters.
    ///
    /// For this purpose, supplementary check characters (e.g., `X` or `*`) found before the end are
    /// regarded as invalid and ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD661_26;
    ///
    /// assert!(MOD661_26.verify_lax("MV-EIS:JV"));
    /// ```
    pub fn verify_lax(&self, s: &str) -> bool {
        self.lax_from_iter(s.chars()).verify()
    }

    /// Verifies whether the check characters in the iterator of characters are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any character is not in the character set, or if a
    /// supplementary check character (e.g., `X` or `*`) is found before the end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD97_10;
    ///
    /// let iter = "3793".chars().chain(['6', '8']);
    /// assert!(MOD97_10.verify_from_chars(iter)?);
    /// # Ok::<_, iso_7064::system::VerifyError<_>>(())
    /// ```
    pub fn verify_from_chars(
        &self,
        chars: impl IntoIterator<Item = char>,
    ) -> Result<bool, VerifyError<char>> {
        self.verify_from_iter(chars)
    }
}

impl<const N_CC: usize, Acc, Enc, Dec> System<N_CC, Acc, Enc, Dec>
where
    Acc: Accumulator + Default,
{
    /// Computes the check character values from an iterator of numerical values.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any value is not in the character set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD37_36;
    ///
    /// let iter = [21, 14, 5, 34];
    /// assert_eq!(MOD37_36.compute_from_values(iter)?, [17]);
    /// # Ok::<_, iso_7064::system::ComputeError<_>>(())
    /// ```
    pub fn compute_from_values(
        &self,
        values: impl IntoIterator<Item = u32>,
    ) -> Result<Acc::Computed, ComputeError<u32>> {
        self.compute_from_iter(values)
    }

    /// Verifies whether the check character values in the iterator of numerical values are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any value is not in the character set, or if a supplementary
    /// check character value is found before the end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use iso_7064::MOD27_26;
    ///
    /// let iter = [2, 13, 17, 11].into_iter().chain([15]);
    /// assert!(MOD27_26.verify_from_values(iter)?);
    /// # Ok::<_, iso_7064::system::VerifyError<_>>(())
    /// ```
    pub fn verify_from_values(
        &self,
        values: impl IntoIterator<Item = u32>,
    ) -> Result<bool, VerifyError<u32>> {
        self.verify_from_iter(values)
    }

    fn compute_from_iter<T: Accumulatable<Dec> + Copy>(
        &self,
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Acc::Computed, ComputeError<T>> {
        let mut acc = Acc::default();
        for (pos, val) in iter.into_iter().enumerate() {
            let AccumulateResult::Processed = val.accumulate_to(&mut acc, &self.decoder) else {
                return Err(ComputeError { val, pos });
            };
        }
        Ok(acc.compute())
    }

    fn verify_from_iter<T: Accumulatable<Dec> + Copy>(
        &self,
        iter: impl IntoIterator<Item = T>,
    ) -> Result<bool, VerifyError<T>> {
        let mut acc = Acc::default();
        let mut it = iter.into_iter().enumerate();
        while let Some((pos, val)) = it.next() {
            match val.accumulate_to(&mut acc, &self.decoder) {
                AccumulateResult::Processed => (),
                AccumulateResult::SupplFound => match it.next() {
                    None => break,
                    Some(_) => {
                        let kind = VerifyErrorKind::UnexpectedSuppl;
                        return Err(VerifyError { val, pos, kind });
                    }
                },
                AccumulateResult::NotInCharset => {
                    let kind = VerifyErrorKind::NotInCharset;
                    return Err(VerifyError { val, pos, kind });
                }
            }
        }
        Ok(acc.verify())
    }

    fn lax_from_iter<T: Accumulatable<Dec>>(&self, iter: impl IntoIterator<Item = T>) -> Acc {
        let mut acc = Acc::default();
        for val in iter {
            let _ = val.accumulate_to(&mut acc, &self.decoder);
        }
        acc
    }

    /// Creates an instance with [`Encoder`] and [`Decoder`].
    pub const fn with_charset(encoder: Enc, decoder: Dec) -> Self {
        Self {
            _acc: marker::PhantomData,
            encoder,
            decoder,
        }
    }
}

trait Accumulatable<Dec> {
    fn accumulate_to(self, acc: &mut impl Accumulator, decoder: &Dec) -> AccumulateResult;
}

impl<Dec> Accumulatable<Dec> for u32 {
    #[inline]
    fn accumulate_to(self, acc: &mut impl Accumulator, _decoder: &Dec) -> AccumulateResult {
        acc.accumulate(self)
    }
}

impl<Dec: Decoder> Accumulatable<Dec> for char {
    #[inline]
    fn accumulate_to(self, acc: &mut impl Accumulator, decoder: &Dec) -> AccumulateResult {
        match decoder.decode(self) {
            Some(value) => acc.accumulate(value),
            None => AccumulateResult::NotInCharset,
        }
    }
}

/// The ISO/IEC 7064, MOD 11-2 pure system with a single check character.
pub type Mod11_2 = System<1, accumulator::Mod11_2, charset::NumericX, charset::NumericX>;

/// The ISO/IEC 7064, MOD 37-2 pure system with a single check character.
pub type Mod37_2 =
    System<1, accumulator::Mod37_2, charset::AlphanumericAst, charset::AlphanumericAst>;

/// The ISO/IEC 7064, MOD 97-10 pure system with two check characters.
pub type Mod97_10 = System<2, accumulator::Mod97_10, charset::Numeric, charset::Numeric>;

/// The ISO/IEC 7064, MOD 661-26 pure system with two check characters.
pub type Mod661_26 = System<2, accumulator::Mod661_26, charset::Alphabetic, charset::Alphabetic>;

/// The ISO/IEC 7064, MOD 1271-36 pure system with two check characters.
pub type Mod1271_36 =
    System<2, accumulator::Mod1271_36, charset::Alphanumeric, charset::Alphanumeric>;

/// The ISO/IEC 7064, MOD 11,10 hybrid system.
pub type Mod11_10 = System<1, accumulator::Mod11_10, charset::Numeric, charset::Numeric>;

/// The ISO/IEC 7064, MOD 27,26 hybrid system.
pub type Mod27_26 = System<1, accumulator::Mod27_26, charset::Alphabetic, charset::Alphabetic>;

/// The ISO/IEC 7064, MOD 37,36 hybrid system.
pub type Mod37_36 = System<1, accumulator::Mod37_36, charset::Alphanumeric, charset::Alphanumeric>;

/// An error returned when check character computation fails.
#[derive(Debug)]
pub struct ComputeError<T> {
    val: T,
    pos: usize,
}

impl<T: fmt::Debug> fmt::Display for ComputeError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = VerifyErrorKind::NotInCharset;
        write!(f, "{}: {:?} at {}", kind, self.val, self.pos)
    }
}

impl<T: fmt::Debug> error::Error for ComputeError<T> {}

/// An error returned when check character verification fails.
#[derive(Debug)]
pub struct VerifyError<T> {
    val: T,
    pos: usize,
    kind: VerifyErrorKind,
}

impl<T: fmt::Debug> fmt::Display for VerifyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?} at {}", self.kind, self.val, self.pos)
    }
}

impl<T: fmt::Debug> error::Error for VerifyError<T> {}

/// The specific kind of a verification error.
#[derive(Debug)]
enum VerifyErrorKind {
    /// A character in the input was not found in the character set.
    NotInCharset,

    /// A supplementary check character was found before the end of the input.
    UnexpectedSuppl,
}

impl fmt::Display for VerifyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInCharset => f.write_str("char not in charset"),
            Self::UnexpectedSuppl => f.write_str("suppl check char not at end"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn do_not_mutate_on_error() {
        let mod1271_36 = Mod1271_36::default();
        let mut buf = alloc::string::String::from("0 1 2 3");
        assert!(mod1271_36.protect(&mut buf).is_err());
        assert_eq!(buf, "0 1 2 3");
    }

    #[test]
    fn reject_unexpected_suppl() {
        let mod11_2 = Mod11_2::default();
        assert!(mod11_2.compute("012X34").is_err());
        assert!(mod11_2.compute("01234X").is_err());
        assert!(mod11_2.verify("012X34").is_err());
        assert!(mod11_2.verify("01234X").is_ok());
    }

    #[test]
    fn ignore_unexpected_suppl() {
        let mod11_2 = Mod11_2::default();
        assert_eq!(mod11_2.compute_lax("32X37"), ['X']);
        assert_eq!(mod11_2.compute_lax("3237X"), ['X']);
        assert!(mod11_2.verify_lax("32X37X"));
        assert!(mod11_2.verify_lax("3237XX"));
        assert!(mod11_2.verify_lax("3237X"));
    }
}
