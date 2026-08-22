//! Character set types for mapping between characters and their numerical values.

/// A trait for encoding a numerical value into its corresponding character.
pub trait Encoder {
    /// Encodes a numerical value into its corresponding character.
    ///
    /// Returns `None` if the value is not representable in this character set.
    fn encode(&self, v: u32) -> Option<char>;
}

/// A trait for decoding a character into its corresponding numerical values.
pub trait Decoder {
    /// The decoded numerical values.
    ///
    /// This type should generally implement `IntoIterator<Item = u32>`.
    type Decoded;

    /// Decodes a character into its corresponding numerical values.
    ///
    /// Returns `None` if the character is not part of this character set.
    fn decode(&self, c: char) -> Option<Self::Decoded>;
}

/// A character set type representing numeric characters (`'0'`-`'9'`).
#[derive(Debug, Default, Clone)]
pub struct Numeric;

impl Encoder for Numeric {
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_ascii(v, b'0')),
            _ => None,
        }
    }
}

impl Decoder for Numeric {
    type Decoded = [u32; 1];

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        match c {
            '0'..='9' => Some([decode_ascii(c, b'0')]),
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
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_ascii(v, b'0')),
            10 => Some('X'),
            _ => None,
        }
    }
}

impl Decoder for NumericX {
    type Decoded = [u32; 1];

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        match c {
            '0'..='9' => Some([decode_ascii(c, b'0')]),
            'X' => Some([10]),
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
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..26 => Some(encode_ascii(v, b'A')),
            _ => None,
        }
    }
}

impl Decoder for Alphabetic {
    type Decoded = [u32; 1];

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        match c {
            'A'..='Z' => Some([decode_ascii(c, b'A')]),
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
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_ascii(v, b'0')),
            10..36 => Some(encode_ascii(v, b'A' - 10)),
            _ => None,
        }
    }
}

impl Decoder for Alphanumeric {
    type Decoded = [u32; 1];

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        match c {
            '0'..='9' => Some([decode_ascii(c, b'0')]),
            'A'..='Z' => Some([decode_ascii(c, b'A' - 10)]),
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
    fn encode(&self, v: u32) -> Option<char> {
        match v {
            ..10 => Some(encode_ascii(v, b'0')),
            10..36 => Some(encode_ascii(v, b'A' - 10)),
            36 => Some('*'),
            _ => None,
        }
    }
}

impl Decoder for AlphanumericAst {
    type Decoded = [u32; 1];

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        match c {
            '0'..='9' => Some([decode_ascii(c, b'0')]),
            'A'..='Z' => Some([decode_ascii(c, b'A' - 10)]),
            '*' => Some([36]),
            _ => None,
        }
    }
}

impl<F: Fn(u32) -> Option<char>> Encoder for F {
    #[inline]
    fn encode(&self, v: u32) -> Option<char> {
        self(v)
    }
}

impl<F: Fn(char) -> Option<D>, D> Decoder for F {
    type Decoded = D;

    #[inline]
    fn decode(&self, c: char) -> Option<Self::Decoded> {
        self(c)
    }
}

#[inline(always)]
fn encode_ascii(v: u32, zero_value: u8) -> char {
    char::from(v as u8 + zero_value)
}

#[inline(always)]
fn decode_ascii(c: char, zero_value: u8) -> u32 {
    u32::from(c).wrapping_sub(u32::from(zero_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    type DynDecoder = dyn Decoder<Decoded = [u32; 1]>;
    const CHARSETS: &[(&dyn Encoder, &DynDecoder, &str)] = &[
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
                assert_eq!(dec.decode(c).unwrap(), [n as u32]);
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
