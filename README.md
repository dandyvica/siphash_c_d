![Build](https://github.com/dandyvica/siphash_c_d/actions/workflows/rust.yml/badge.svg)

This crate provides a pure Rust implemntation of the `siphash_c_d` algorithm as originally described in:
<https://cr.yp.to/siphash/siphash-20120918.pdf>. 

SipHash was invented by Jean-Philippe Aumasson and Daniel J. Bernstein.
The algorithm is maed generic for the *c* and *d* `u8`integers. For the most common use, 2 types aliases are defined:

* `SipHash24` for `siphash_2_4`
* `SipHash48` for `siphash_4_8`

# Usage

This crate is [on crates.io](https://crates.io/crates/siphash_c_d) and can be
used by adding `siphash_c_d` to your dependencies in your project's `Cargo.toml`.
```toml
[dependencies]
siphash_c_d = "1"
```

# Example: the key is made of 2 `u64` integers

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let msg: Vec<_> = (0..=14_u8).collect();
//let msg = "The quick brown fox jumps over the lazy dog".as_bytes();

// use it with the 128-bit key broken down into 2 little endian 64-bit integers:
let k0 = 0x0706050403020100_u64;
let k1 = 0x0f0e0d0c0b0a0908_u64;
let hash = SipHash24::new(k0, k1, &msg);

assert_eq!(hash, 0xa129ca6149be45e5);
```

# Example: the key is a slice of `[u8]`

```rust
use siphash_c_d::SipHash24;

// message to be hashed
let msg = "The quick brown fox jumps over the lazy dog".as_bytes();
let key = "0123456789ABCDEF0".as_bytes();

let hash = SipHash24::from_slice(&key, &msg);

assert_eq!(hash, 0x19cce661759c2a06);
```

If you feel adventurous, you can try high values of `c` and `d`:

```rust
use siphash_c_d::SipHash; 

let msg = "The quick brown fox jumps over the lazy dog".as_bytes();
let key = "0123456789ABCDEF0".as_bytes();
let higher_hash = SipHash::<32, 64>::from_slice(&key, &msg);
```


