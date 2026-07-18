# A Rust implementation of ISO/IEC 7064:2003 Check character systems

[![Crates.io](https://img.shields.io/crates/v/iso-7064)](https://crates.io/crates/iso-7064)
[![Docs.rs](https://img.shields.io/docsrs/iso-7064)](https://docs.rs/iso-7064)
[![License](https://img.shields.io/crates/l/iso-7064)](https://github.com/LiosK/iso-7064-rs/blob/main/LICENSE)

This crate provides `no_std`-compatible implementations of the check character
(digit) systems specified by [ISO/IEC 7064:2003].

[ISO/IEC 7064:2003]: https://www.iso.org/standard/31531.html

```rust
use iso_7064::{MOD11_2, MOD1271_36};

assert_eq!(MOD11_2.compute("079")?, ['X']);
assert_eq!(MOD11_2.compute_lax("{0-7-9}"), ['X']);

assert!(MOD11_2.verify("079X")?);
assert!(MOD11_2.verify_lax("{0-7-9}[X]"));
assert!(MOD11_2.verify_from_values([0, 7, 9, 10])?);
assert!(!MOD11_2.verify_from_chars("0790".chars())?);

let mut buf = String::from("ISO 79");
MOD1271_36.protect_lax(&mut buf);
assert_eq!(buf, "ISO 793W");
```

This crate supports all the eight check character systems as shown below
specified by the standard:

| System                    | Type   | Input string            | Check character(s)                    |
| ------------------------- | ------ | ----------------------- | ------------------------------------- |
| ISO/IEC 7064, MOD 11-2    | Pure   | Numeric (`0-9`)         | 1 digit or `'X'` (`0-9X`)             |
| ISO/IEC 7064, MOD 37-2    | Pure   | Alphanumeric (`0-9A-Z`) | 1 digit, letter, or `'*'` (`0-9A-Z*`) |
| ISO/IEC 7064, MOD 97-10   | Pure   | Numeric (`0-9`)         | 2 digits (`0-9`)                      |
| ISO/IEC 7064, MOD 661-26  | Pure   | Alphabetic (`A-Z`)      | 2 letters (`A-Z`)                     |
| ISO/IEC 7064, MOD 1271-36 | Pure   | Alphanumeric (`0-9A-Z`) | 2 digits or letters (`0-9A-Z`)        |
| ISO/IEC 7064, MOD 11,10   | Hybrid | Numeric (`0-9`)         | 1 digit (`0-9`)                       |
| ISO/IEC 7064, MOD 27,26   | Hybrid | Alphabetic (`A-Z`)      | 1 letter (`A-Z`)                      |
| ISO/IEC 7064, MOD 37,36   | Hybrid | Alphanumeric (`0-9A-Z`) | 1 digit or letter (`0-9A-Z`)          |

## Crate features

- `alloc` (enabled by default) enables APIs operating over `String`.

## License

Licensed under the Apache License, Version 2.0.
