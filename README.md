# A Rust implementation of ISO/IEC 7064:2003 Check character systems

[![Crates.io](https://img.shields.io/crates/v/iso-7064)](https://crates.io/crates/iso-7064)
[![Docs.rs](https://img.shields.io/docsrs/iso-7064)](https://docs.rs/iso-7064)
[![License](https://img.shields.io/crates/l/iso-7064)](https://github.com/LiosK/iso-7064-rs/blob/main/LICENSE)

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

## License

Licensed under the Apache License, Version 2.0.
