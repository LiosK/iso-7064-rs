# A Rust implementation of ISO/IEC 7064:2003 Check character systems

```rust
use iso_7064::{MOD11_2, MOD1271_36};

assert_eq!(MOD11_2.compute("079")?, ['X']);
assert_eq!(MOD11_2.compute_lax("{0-7-9}"), ['X']);

assert!(MOD11_2.verify("079X")?);
assert!(MOD11_2.verify_lax("{0-7-9}[X]"));
assert!(MOD11_2.verify_from_values([0, 7, 9, 10])?);
assert!(!MOD11_2.verify_from_chars("0790".chars())?);

# #[cfg(feature = "alloc")]
# {
let mut buf = String::from("ISO 79");
MOD1271_36.protect_lax(&mut buf);
assert_eq!(buf, "ISO 793W");
# }
# Ok::<_, Box<dyn core::error::Error>>(())
```
