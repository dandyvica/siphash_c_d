This crate provides a pure `no_std` and `safe` Rust implementation of the `siphash_c_d` algorithm as originally described in:
<https://cr.yp.to/siphash/siphash-20120918.pdf>.

The paper only describes the algorithm for an output value of 64 bits. The algorithm for a 128-bit output
value is very similar but only described in: <https://github.com/veorq/SipHash>

*SipHash* was invented by Jean-Philippe Aumasson and Daniel J. Bernstein.

Using the `Hash64` or `Hash128` keyword, you can get the `u64` or `u128` bits hash value.

The algorithm is made generic for the *c* and *d* `u8`integers. For the most common use, 2 types aliases are defined:

* `SipHash24` for `siphash_2_4` (64-bit hash value)
* `SipHash48` for `siphash_4_8` (64-bit hash value)

It has been tested on a *bigendian* platform using qemu on an emulated MIPS Malta platform.

[![Rust](https://github.com/dandyvica/siphash_c_d/actions/workflows/rust.yml/badge.svg)](https://github.com/dandyvica/siphash_c_d/actions/workflows/rust.yml)

# Usage

This crate is [on crates.io](https://crates.io/crates/siphash_c_d) and can be
used by adding `siphash_c_d` to your dependencies in your project's `Cargo.toml`.
```toml
[dependencies]
siphash_c_d = "0.1.0"
```

The key can be:

* a 2-tuple of `u64` integers
* a slice `&[u8]` (of a least 16 bytes, otherwise it panics)
* an array `[u8;16]`
* a `u128` integer

## Example 1: the key is made of a 2-tuple of `u64` integers

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

let k0 = 0x0706050403020100_u64;
let k1 = 0x0f0e0d0c0b0a0908_u64;
let hash = SipHash24::new((k0, k1), &msg).unwrap();

assert_eq!(hash, 0xa129ca6149be45e5);
```

## The key is made of a slice of `u8` integers

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let key = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F".as_bytes();
let msg: Vec<_> = (0..=14_u8).collect();

let hash = SipHash24::new(key, &msg).unwrap();
assert_eq!(hash, 0xa129ca6149be45e5);
```

## The key is made of a array of 16 `u8` integers

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let key = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15_u8];
let msg: Vec<_> = (0..=14_u8).collect();

let hash = SipHash24::new(&key, &msg).unwrap();
assert_eq!(hash, 0xa129ca6149be45e5);
```

## The key is a `u128` integer

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let key: u128 = 0x0706050403020100_0f0e0d0c0b0a0908;
let msg: Vec<_> = (0..=14_u8).collect();

let hash = SipHash24::new(key, &msg).unwrap();
assert_eq!(hash, 0xa129ca6149be45e5);
```

If you feel adventurous, you can try higher values of `c` and `d`:

```rust
use siphash_c_d::{Hash128, SipHash};

let msg = "The quick brown fox jumps over the lazy dog".as_bytes();
let key = "0123456789ABCDEF0".as_bytes();

let higher_hash = SipHash::<32, 64, Hash128>::new(key, &msg).unwrap();
```

If the key length is < 16 bytes, an error is returned:

```rust
use siphash_c_d::SipHash24;

let msg = "The quick brown fox jumps over the lazy dog".as_bytes();
let key = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E".as_bytes();

let hash = SipHash24::new(key, &msg);
assert!(hash.is_err());
```

