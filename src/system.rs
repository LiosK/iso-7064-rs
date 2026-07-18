//! The [`System`] structure providing a check character system interface.
//!
//! ```rust
//! use iso_7064::{accumulator, charset, system::System};
//!
//! // Build a custom MOD 11,10 with case-insensitive alphabetic character set.
//! let my_mod11_10 = System::<1, accumulator::Mod11_10, _, _>::with_charset(
//!     charset::Alphabetic,
//!     charset::decoder_from_fn(|c| c.to_digit(36)?.checked_sub(10)),
//! );
//!
//! assert_eq!(my_mod11_10.compute("AhJe")?, ['F']);
//! assert!(my_mod11_10.verify("aHjEf")?);
//! assert!(my_mod11_10.verify("AhJe5").is_err());
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

use core::{error, fmt, marker};

#[cfg(feature = "alloc")]
extern crate alloc;

use super::accumulator::{self, AccumulateResult, Accumulator};
use super::charset::{self, Decoder, Encoder};

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
    Acc: Accumulator<N_CC> + Default,
    Enc: Encoder,
    Dec: Decoder,
{
    /// Creates an instance with [`Encoder`] and [`Decoder`].
    pub const fn with_charset(encoder: Enc, decoder: Dec) -> Self {
        Self {
            _acc: marker::PhantomData,
            encoder,
            decoder,
        }
    }

    /// Computes the check characters for the string `s` and appends them.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    #[cfg(feature = "alloc")]
    pub fn protect(&self, s: &mut alloc::string::String) -> Result<(), ComputeError<char>> {
        s.extend(self.compute(s)?);
        Ok(())
    }

    /// Computes the check characters for the string `s` and appends them, ignoring any invalid
    /// characters.
    #[cfg(feature = "alloc")]
    pub fn protect_lax(&self, s: &mut alloc::string::String) {
        s.extend(self.compute_lax(s));
    }

    /// Computes the check characters for the string `s`.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    pub fn compute(&self, s: &str) -> Result<[char; N_CC], ComputeError<char>> {
        self.compute_from_chars(s.chars())
    }

    /// Computes the check characters for the string `s`, ignoring any invalid characters.
    pub fn compute_lax(&self, s: &str) -> [char; N_CC] {
        let mut acc = Acc::default();
        for c in s.chars() {
            let _ = self.accumulate_char(&mut acc, c);
        }
        self.compute_char(&mut acc)
    }

    /// Computes the check characters from an iterator of characters.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any character is not in the character set.
    pub fn compute_from_chars(
        &self,
        chars: impl IntoIterator<Item = char>,
    ) -> Result<[char; N_CC], ComputeError<char>> {
        let mut acc = Acc::default();
        for (pos, c) in chars.into_iter().enumerate() {
            match self.accumulate_char(&mut acc, c) {
                AccumulateResult::Processed => (),
                _ => return Err(ComputeError { val: c, pos }),
            }
        }
        Ok(self.compute_char(&mut acc))
    }

    /// Computes the check character values from an iterator of numerical values.
    ///
    /// # Errors
    ///
    /// Returns a [`ComputeError`] if any value is not in the character set.
    pub fn compute_from_values(
        &self,
        values: impl IntoIterator<Item = u32>,
    ) -> Result<[u32; N_CC], ComputeError<u32>> {
        let mut acc = Acc::default();
        for (pos, a) in values.into_iter().enumerate() {
            match acc.accumulate(a) {
                AccumulateResult::Processed => (),
                _ => return Err(ComputeError { val: a, pos }),
            }
        }
        Ok(acc.compute())
    }

    /// Verifies whether the check characters in the string `s` are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any character is not in the character set, or if a
    /// supplementary check character (e.g., `X` or `*`) is found before the end.
    pub fn verify(&self, s: &str) -> Result<bool, VerifyError<char>> {
        self.verify_from_chars(s.chars())
    }

    /// Verifies whether the check characters in the string `s` are valid, ignoring any invalid
    /// characters.
    ///
    /// For this purpose, supplementary check characters (e.g., `X` or `*`) found before the end are
    /// regarded as invalid and ignored.
    pub fn verify_lax(&self, s: &str) -> bool {
        let mut acc = Acc::default();
        for c in s.chars() {
            let _ = self.accumulate_char(&mut acc, c);
        }
        acc.verify()
    }

    /// Verifies whether the check characters in the iterator of characters are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any character is not in the character set, or if a
    /// supplementary check character (e.g., `X` or `*`) is found before the end.
    pub fn verify_from_chars(
        &self,
        chars: impl IntoIterator<Item = char>,
    ) -> Result<bool, VerifyError<char>> {
        let mut acc = Acc::default();
        let mut it = chars.into_iter().enumerate();
        while let Some((pos, c)) = it.next() {
            match self.accumulate_char(&mut acc, c) {
                AccumulateResult::Processed => (),
                AccumulateResult::SupplFound => match it.next() {
                    None => break,
                    Some(_) => {
                        let kind = VerifyErrorKind::UnexpectedSuppl;
                        return Err(VerifyError { val: c, pos, kind });
                    }
                },
                AccumulateResult::NotInCharset => {
                    let kind = VerifyErrorKind::NotInCharset;
                    return Err(VerifyError { val: c, pos, kind });
                }
            }
        }
        Ok(acc.verify())
    }

    /// Verifies whether the check character values in the iterator of numerical values are valid.
    ///
    /// # Errors
    ///
    /// Returns a [`VerifyError`] if any value is not in the character set, or if a supplementary
    /// check character value is found before the end.
    pub fn verify_from_values(
        &self,
        values: impl IntoIterator<Item = u32>,
    ) -> Result<bool, VerifyError<u32>> {
        let mut acc = Acc::default();
        let mut it = values.into_iter().enumerate();
        while let Some((pos, a)) = it.next() {
            match acc.accumulate(a) {
                AccumulateResult::Processed => (),
                AccumulateResult::SupplFound => match it.next() {
                    None => break,
                    Some(_) => {
                        let kind = VerifyErrorKind::UnexpectedSuppl;
                        return Err(VerifyError { val: a, pos, kind });
                    }
                },
                AccumulateResult::NotInCharset => {
                    let kind = VerifyErrorKind::NotInCharset;
                    return Err(VerifyError { val: a, pos, kind });
                }
            }
        }
        Ok(acc.verify())
    }

    #[inline]
    fn accumulate_char(&self, acc: &mut Acc, c: char) -> AccumulateResult {
        match self.decoder.decode(c) {
            Some(a) => acc.accumulate(a),
            None => AccumulateResult::NotInCharset,
        }
    }

    fn compute_char(&self, acc: &mut Acc) -> [char; N_CC] {
        const ERR: &str = "invalid charset implementation";
        acc.compute().map(|a| self.encoder.encode(a).expect(ERR))
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
