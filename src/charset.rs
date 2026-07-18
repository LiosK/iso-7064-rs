//! Character set types for mapping between characters and their numerical values.

/// A trait for encoding a numerical value into its corresponding character.
pub trait Encoder {
    /// Encodes a numerical value into its corresponding character.
    ///
    /// Returns `None` if the value is not representable in this character set.
    fn encode(&self, v: u32) -> Option<char>;
}

/// A trait for decoding a character into its corresponding numerical value.
pub trait Decoder {
    /// Decodes a character into its corresponding numerical value.
    ///
    /// Returns `None` if the character is not part of this character set.
    fn decode(&self, c: char) -> Option<u32>;
}

/// A character set type representing numeric characters (`'0'`-`'9'`).
#[derive(Debug, Default, Clone)]
pub struct Numeric;

impl Encoder for Numeric {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_numeric(v)),
            _ => None,
        }
    }
}

impl Decoder for Numeric {
    #[inline]
    fn decode(&self, c: char) -> Option<u32> {
        match c {
            '0'..='9' => Some(decode_numeric(c)),
            _ => None,
        }
    }
}

/// A character set type representing numeric characters (`'0'`-`'9'`) and the supplementary check
/// character `'X'`.
///
/// Note that the conversion by this type is case-sensitive.
#[derive(Debug, Default, Clone)]
pub struct NumericX;

impl Encoder for NumericX {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_numeric(v)),
            10 => Some('X'),
            _ => None,
        }
    }
}

impl Decoder for NumericX {
    #[inline]
    fn decode(&self, c: char) -> Option<u32> {
        match c {
            '0'..='9' => Some(decode_numeric(c)),
            'X' => Some(10),
            _ => None,
        }
    }
}

/// A character set type representing uppercase alphabetic characters (`'A'`-`'Z'`).
///
/// Note that the conversion by this type is case-sensitive.
#[derive(Debug, Default, Clone)]
pub struct Alphabetic;

impl Encoder for Alphabetic {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..26 => Some(encode_ascii(v, b'A')),
            _ => None,
        }
    }
}

impl Decoder for Alphabetic {
    #[inline]
    fn decode(&self, c: char) -> Option<u32> {
        match c {
            'A'..='Z' => Some(decode_ascii(c, b'A')),
            _ => None,
        }
    }
}

/// A character set type representing uppercase alphanumeric characters (`'0'`-`'9'`, `'A'`-`'Z'`).
///
/// Note that the conversion by this type is case-sensitive.
#[derive(Debug, Default, Clone)]
pub struct Alphanumeric;

impl Encoder for Alphanumeric {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_numeric(v)),
            10..36 => Some(encode_ascii(v, b'A' - 10)),
            _ => None,
        }
    }
}

impl Decoder for Alphanumeric {
    #[inline]
    fn decode(&self, c: char) -> Option<u32> {
        match c {
            '0'..='9' => Some(decode_numeric(c)),
            'A'..='Z' => Some(decode_ascii(c, b'A' - 10)),
            _ => None,
        }
    }
}

/// A character set type representing uppercase alphanumeric characters (`'0'`-`'9'`, `'A'`-`'Z'`)
/// and the supplementary check character `'*'`.
///
/// Note that the conversion by this type is case-sensitive.
#[derive(Debug, Default, Clone)]
pub struct AlphanumericAst;

impl Encoder for AlphanumericAst {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_numeric(v)),
            10..36 => Some(encode_ascii(v, b'A' - 10)),
            36 => Some('*'),
            _ => None,
        }
    }
}

impl Decoder for AlphanumericAst {
    #[inline]
    fn decode(&self, c: char) -> Option<u32> {
        match c {
            '0'..='9' => Some(decode_numeric(c)),
            'A'..='Z' => Some(decode_ascii(c, b'A' - 10)),
            '*' => Some(36),
            _ => None,
        }
    }
}

/// Creates an [`Encoder`] with the provided closure as its `encode()` method.
pub fn encoder_from_fn(f: impl Fn(u32) -> Option<char>) -> impl Encoder {
    struct FromFn<F>(F);

    impl<F: Fn(u32) -> Option<char>> Encoder for FromFn<F> {
        #[inline]
        fn encode(&self, v: u32) -> Option<char> {
            self.0(v)
        }
    }

    FromFn(f)
}

/// Creates a [`Decoder`] with the provided closure as its `decode()` method.
pub fn decoder_from_fn(f: impl Fn(char) -> Option<u32>) -> impl Decoder {
    struct FromFn<F>(F);

    impl<F: Fn(char) -> Option<u32>> Decoder for FromFn<F> {
        #[inline]
        fn decode(&self, c: char) -> Option<u32> {
            self.0(c)
        }
    }

    FromFn(f)
}

#[inline(always)]
fn encode_numeric(v: u32) -> char {
    char::from(v as u8 | 0x30)
}

#[inline(always)]
fn decode_numeric(c: char) -> u32 {
    u32::from(c) & 0x0F
}

#[inline(always)]
fn encode_ascii(v: u32, zero_value: u8) -> char {
    char::from(v as u8 + zero_value)
}

#[inline(always)]
fn decode_ascii(c: char, zero_value: u8) -> u32 {
    u32::from(c) - u32::from(zero_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHARSETS: &[(&dyn Encoder, &dyn Decoder, &str)] = &[
        (&Numeric, &Numeric, "0123456789"),
        (&NumericX, &NumericX, "0123456789X"),
        (&Alphabetic, &Alphabetic, "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        (
            &Alphanumeric,
            &Alphanumeric,
            "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ),
        (
            &AlphanumericAst,
            &AlphanumericAst,
            "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ*",
        ),
    ];

    #[test]
    fn encode_decode_ok() {
        for (enc, dec, charset) in CHARSETS {
            for (n, c) in charset.chars().enumerate() {
                assert_eq!(enc.encode(n as u32).unwrap(), c);
                assert_eq!(dec.decode(c).unwrap(), n as u32);
            }
        }
    }

    #[test]
    fn encode_err() {
        for (enc, _, charset) in CHARSETS {
            for n in (charset.chars().count()..).take(1024) {
                assert!(enc.encode(n as u32).is_none());
            }
        }
    }

    #[test]
    fn decode_err() {
        for (_, dec, charset) in CHARSETS {
            for c in (char::MIN..).take(1024) {
                if !charset.contains(c) {
                    assert!(dec.decode(c).is_none());
                }
            }
        }
    }
}
