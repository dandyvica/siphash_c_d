use core::marker::PhantomData;

use crate::{iter::MessageChunk, residue::Residue, siphashkey::SipHashKey, state::State, SipError};

/// Defines a 64-bit hash calculation.
pub struct Hash64;

/// Defines a 128-bit hash calculation.
pub struct Hash128;

//#[derive(Debug, Default)]
/// The generic `siphash_c_d` structure which is keeping the internal state of the algorithm.
pub struct SipHash<const C: u8, const D: u8, T> {
    // internal state
    pub(crate) state: State<C, D>,

    // the residue is the block keeping the data when using the write() hash function
    pub(crate) residue: Residue,

    // need this because no T is passed
    output: PhantomData<T>,
}

impl<const C: u8, const D: u8> SipHash<C, D, Hash64> {
    /// Calculate the `siphash_c_d` 64-bit value of the message `msg` using the key `key`.
    ///
    /// If the length of the key is less than 16 bytes, returns an error (`SipError::KeyTooShort`).
    pub fn with_key<K>(key: K, msg: &[u8]) -> Result<u64, SipError>
    where
        K: TryInto<SipHashKey, Error = SipError>,
    {
        let mut siphash = SipHash::<C, D, Hash64>::new(key)?;
        siphash.compression(msg);
        Ok(siphash.state.finalization(2, 0xFF))
    }
}

impl<const C: u8, const D: u8> SipHash<C, D, Hash128> {
    /// Calculate the `siphash_c_d` 128-bit value of the message `msg` using the key `key`.
    ///
    /// If the length of the key is less than 16 bytes, returns an error (`SipError::KeyTooShort`).
    pub fn with_key<K>(key: K, msg: &[u8]) -> Result<u128, SipError>
    where
        K: TryInto<SipHashKey, Error = SipError>,
    {
        let mut siphash = SipHash::<C, D, Hash128>::new(key)?;

        // additional step for 128
        siphash.state.hash128_additional();

        siphash.compression(msg);

        let u0 = siphash.state.finalization(2, 0xEE) as u128;

        // additional step for 128
        let u1 = siphash.state.finalization(1, 0xDD) as u128;

        Ok(u1 << 64_u128 | u0)
    }
}

impl<const C: u8, const D: u8, T> SipHash<C, D, T> {
    /// Assign the key for the `siphash_c_d` calculation.
    ///
    /// If the length of the key is less than 16 bytes, returns an error (`SipError::KeyTooShort`).
    pub fn new<K>(key: K) -> Result<Self, SipError>
    where
        K: TryInto<SipHashKey, Error = SipError>,
    {
        let k = key.try_into()?;

        Ok(Self {
            state: State::new(k.0, k.1),
            residue: Residue::default(),
            output: PhantomData,
        })
    }

    // as described in the paper
    fn compression(&mut self, msg: &[u8]) {
        // use the custom iterator to iterate through m_i blocks
        // from 0 to w-1
        let wrapped_msg = MessageChunk(msg);

        for m_i in &wrapped_msg {
            self.state.compress_chunk(m_i);
        }
    }
}

/// The `siphash_2_4` 64-bit hash calculation.
pub type SipHash24 = SipHash<2, 4, Hash64>;

/// The `siphash_4_8` 64-bit hash calculation.
pub type SipHash48 = SipHash<4, 8, Hash64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // taken from Appendix A
    fn test_using_tuple() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert_eq!(msg.len(), 15);

        let siphash_2_4 =
            SipHash24::with_key((0x0706050403020100, 0x0f0e0d0c0b0a0908), &msg).unwrap();
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from Appendix A
    fn test_using_vec() {
        let key: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert_eq!(msg.len(), 15);

        let siphash_2_4 = SipHash24::with_key(key, &msg).unwrap();
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from Appendix A
    fn test_using_u128() {
        let key: u128 = 0x0706050403020100_0f0e0d0c0b0a0908;
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert_eq!(msg.len(), 15);

        let siphash_2_4 = SipHash24::with_key(key, &msg).unwrap();
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from Appendix A
    fn test_using_slice() {
        let key = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F".as_bytes();
        let msg = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E".as_bytes();
        assert_eq!(msg.len(), 15);

        let siphash_2_4 = SipHash24::with_key(key, msg).unwrap();
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from https://github.com/google/guava/blob/master/guava-tests/test/com/google/common/hash/SipHashFunctionTest.java
    fn test_sample2() {
        let key: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        const EXPECTED: [u64; 64] = [
            0x726fdb47dd0e0e31,
            0x74f839c593dc67fd,
            0x0d6c8009d9a94f5a,
            0x85676696d7fb7e2d,
            0xcf2794e0277187b7,
            0x18765564cd99a68d,
            0xcbc9466e58fee3ce,
            0xab0200f58b01d137,
            0x93f5f5799a932462,
            0x9e0082df0ba9e4b0,
            0x7a5dbbc594ddb9f3,
            0xf4b32f46226bada7,
            0x751e8fbc860ee5fb,
            0x14ea5627c0843d90,
            0xf723ca908e7af2ee,
            0xa129ca6149be45e5,
            0x3f2acc7f57c29bdb,
            0x699ae9f52cbe4794,
            0x4bc1b3f0968dd39c,
            0xbb6dc91da77961bd,
            0xbed65cf21aa2ee98,
            0xd0f2cbb02e3b67c7,
            0x93536795e3a33e88,
            0xa80c038ccd5ccec8,
            0xb8ad50c6f649af94,
            0xbce192de8a85b8ea,
            0x17d835b85bbb15f3,
            0x2f2e6163076bcfad,
            0xde4daaaca71dc9a5,
            0xa6a2506687956571,
            0xad87a3535c49ef28,
            0x32d892fad841c342,
            0x7127512f72f27cce,
            0xa7f32346f95978e3,
            0x12e0b01abb051238,
            0x15e034d40fa197ae,
            0x314dffbe0815a3b4,
            0x027990f029623981,
            0xcadcd4e59ef40c4d,
            0x9abfd8766a33735c,
            0x0e3ea96b5304a7d0,
            0xad0c42d6fc585992,
            0x187306c89bc215a9,
            0xd4a60abcf3792b95,
            0xf935451de4f21df2,
            0xa9538f0419755787,
            0xdb9acddff56ca510,
            0xd06c98cd5c0975eb,
            0xe612a3cb9ecba951,
            0xc766e62cfcadaf96,
            0xee64435a9752fe72,
            0xa192d576b245165a,
            0x0a8787bf8ecb74b2,
            0x81b3e73d20b49b6f,
            0x7fa8220ba3b2ecea,
            0x245731c13ca42499,
            0xb78dbfaf3a8d83bd,
            0xea1ad565322a1a0b,
            0x60e61c23a3795013,
            0x6606d7e446282b93,
            0x6ca4ecb15c5f91e1,
            0x9f626da15c9625f3,
            0xe51b38608ef25f57,
            0x958a324ceb064572,
        ];

        // no Vec in no_std
        let mut msg = [0u8; 64];

        for i in 0..EXPECTED.len() {
            (0..i).for_each(|k| msg[k] = k as u8);
            assert_eq!(SipHash24::with_key(key, &msg[0..i]).unwrap(), EXPECTED[i]);
        }
    }

    // tests taken from https://github.com/veorq/SipHash
    #[test]
    fn test_siphash128() {
        let key: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        const EXPECTED: [[u8; 16]; 64] = [
            [
                0xa3, 0x81, 0x7f, 0x04, 0xba, 0x25, 0xa8, 0xe6, 0x6d, 0xf6, 0x72, 0x14, 0xc7, 0x55,
                0x02, 0x93,
            ],
            [
                0xda, 0x87, 0xc1, 0xd8, 0x6b, 0x99, 0xaf, 0x44, 0x34, 0x76, 0x59, 0x11, 0x9b, 0x22,
                0xfc, 0x45,
            ],
            [
                0x81, 0x77, 0x22, 0x8d, 0xa4, 0xa4, 0x5d, 0xc7, 0xfc, 0xa3, 0x8b, 0xde, 0xf6, 0x0a,
                0xff, 0xe4,
            ],
            [
                0x9c, 0x70, 0xb6, 0x0c, 0x52, 0x67, 0xa9, 0x4e, 0x5f, 0x33, 0xb6, 0xb0, 0x29, 0x85,
                0xed, 0x51,
            ],
            [
                0xf8, 0x81, 0x64, 0xc1, 0x2d, 0x9c, 0x8f, 0xaf, 0x7d, 0x0f, 0x6e, 0x7c, 0x7b, 0xcd,
                0x55, 0x79,
            ],
            [
                0x13, 0x68, 0x87, 0x59, 0x80, 0x77, 0x6f, 0x88, 0x54, 0x52, 0x7a, 0x07, 0x69, 0x0e,
                0x96, 0x27,
            ],
            [
                0x14, 0xee, 0xca, 0x33, 0x8b, 0x20, 0x86, 0x13, 0x48, 0x5e, 0xa0, 0x30, 0x8f, 0xd7,
                0xa1, 0x5e,
            ],
            [
                0xa1, 0xf1, 0xeb, 0xbe, 0xd8, 0xdb, 0xc1, 0x53, 0xc0, 0xb8, 0x4a, 0xa6, 0x1f, 0xf0,
                0x82, 0x39,
            ],
            [
                0x3b, 0x62, 0xa9, 0xba, 0x62, 0x58, 0xf5, 0x61, 0x0f, 0x83, 0xe2, 0x64, 0xf3, 0x14,
                0x97, 0xb4,
            ],
            [
                0x26, 0x44, 0x99, 0x06, 0x0a, 0xd9, 0xba, 0xab, 0xc4, 0x7f, 0x8b, 0x02, 0xbb, 0x6d,
                0x71, 0xed,
            ],
            [
                0x00, 0x11, 0x0d, 0xc3, 0x78, 0x14, 0x69, 0x56, 0xc9, 0x54, 0x47, 0xd3, 0xf3, 0xd0,
                0xfb, 0xba,
            ],
            [
                0x01, 0x51, 0xc5, 0x68, 0x38, 0x6b, 0x66, 0x77, 0xa2, 0xb4, 0xdc, 0x6f, 0x81, 0xe5,
                0xdc, 0x18,
            ],
            [
                0xd6, 0x26, 0xb2, 0x66, 0x90, 0x5e, 0xf3, 0x58, 0x82, 0x63, 0x4d, 0xf6, 0x85, 0x32,
                0xc1, 0x25,
            ],
            [
                0x98, 0x69, 0xe2, 0x47, 0xe9, 0xc0, 0x8b, 0x10, 0xd0, 0x29, 0x93, 0x4f, 0xc4, 0xb9,
                0x52, 0xf7,
            ],
            [
                0x31, 0xfc, 0xef, 0xac, 0x66, 0xd7, 0xde, 0x9c, 0x7e, 0xc7, 0x48, 0x5f, 0xe4, 0x49,
                0x49, 0x02,
            ],
            [
                0x54, 0x93, 0xe9, 0x99, 0x33, 0xb0, 0xa8, 0x11, 0x7e, 0x08, 0xec, 0x0f, 0x97, 0xcf,
                0xc3, 0xd9,
            ],
            [
                0x6e, 0xe2, 0xa4, 0xca, 0x67, 0xb0, 0x54, 0xbb, 0xfd, 0x33, 0x15, 0xbf, 0x85, 0x23,
                0x05, 0x77,
            ],
            [
                0x47, 0x3d, 0x06, 0xe8, 0x73, 0x8d, 0xb8, 0x98, 0x54, 0xc0, 0x66, 0xc4, 0x7a, 0xe4,
                0x77, 0x40,
            ],
            [
                0xa4, 0x26, 0xe5, 0xe4, 0x23, 0xbf, 0x48, 0x85, 0x29, 0x4d, 0xa4, 0x81, 0xfe, 0xae,
                0xf7, 0x23,
            ],
            [
                0x78, 0x01, 0x77, 0x31, 0xcf, 0x65, 0xfa, 0xb0, 0x74, 0xd5, 0x20, 0x89, 0x52, 0x51,
                0x2e, 0xb1,
            ],
            [
                0x9e, 0x25, 0xfc, 0x83, 0x3f, 0x22, 0x90, 0x73, 0x3e, 0x93, 0x44, 0xa5, 0xe8, 0x38,
                0x39, 0xeb,
            ],
            [
                0x56, 0x8e, 0x49, 0x5a, 0xbe, 0x52, 0x5a, 0x21, 0x8a, 0x22, 0x14, 0xcd, 0x3e, 0x07,
                0x1d, 0x12,
            ],
            [
                0x4a, 0x29, 0xb5, 0x45, 0x52, 0xd1, 0x6b, 0x9a, 0x46, 0x9c, 0x10, 0x52, 0x8e, 0xff,
                0x0a, 0xae,
            ],
            [
                0xc9, 0xd1, 0x84, 0xdd, 0xd5, 0xa9, 0xf5, 0xe0, 0xcf, 0x8c, 0xe2, 0x9a, 0x9a, 0xbf,
                0x69, 0x1c,
            ],
            [
                0x2d, 0xb4, 0x79, 0xae, 0x78, 0xbd, 0x50, 0xd8, 0x88, 0x2a, 0x8a, 0x17, 0x8a, 0x61,
                0x32, 0xad,
            ],
            [
                0x8e, 0xce, 0x5f, 0x04, 0x2d, 0x5e, 0x44, 0x7b, 0x50, 0x51, 0xb9, 0xea, 0xcb, 0x8d,
                0x8f, 0x6f,
            ],
            [
                0x9c, 0x0b, 0x53, 0xb4, 0xb3, 0xc3, 0x07, 0xe8, 0x7e, 0xae, 0xe0, 0x86, 0x78, 0x14,
                0x1f, 0x66,
            ],
            [
                0xab, 0xf2, 0x48, 0xaf, 0x69, 0xa6, 0xea, 0xe4, 0xbf, 0xd3, 0xeb, 0x2f, 0x12, 0x9e,
                0xeb, 0x94,
            ],
            [
                0x06, 0x64, 0xda, 0x16, 0x68, 0x57, 0x4b, 0x88, 0xb9, 0x35, 0xf3, 0x02, 0x73, 0x58,
                0xae, 0xf4,
            ],
            [
                0xaa, 0x4b, 0x9d, 0xc4, 0xbf, 0x33, 0x7d, 0xe9, 0x0c, 0xd4, 0xfd, 0x3c, 0x46, 0x7c,
                0x6a, 0xb7,
            ],
            [
                0xea, 0x5c, 0x7f, 0x47, 0x1f, 0xaf, 0x6b, 0xde, 0x2b, 0x1a, 0xd7, 0xd4, 0x68, 0x6d,
                0x22, 0x87,
            ],
            [
                0x29, 0x39, 0xb0, 0x18, 0x32, 0x23, 0xfa, 0xfc, 0x17, 0x23, 0xde, 0x4f, 0x52, 0xc4,
                0x3d, 0x35,
            ],
            [
                0x7c, 0x39, 0x56, 0xca, 0x5e, 0xea, 0xfc, 0x3e, 0x36, 0x3e, 0x9d, 0x55, 0x65, 0x46,
                0xeb, 0x68,
            ],
            [
                0x77, 0xc6, 0x07, 0x71, 0x46, 0xf0, 0x1c, 0x32, 0xb6, 0xb6, 0x9d, 0x5f, 0x4e, 0xa9,
                0xff, 0xcf,
            ],
            [
                0x37, 0xa6, 0x98, 0x6c, 0xb8, 0x84, 0x7e, 0xdf, 0x09, 0x25, 0xf0, 0xf1, 0x30, 0x9b,
                0x54, 0xde,
            ],
            [
                0xa7, 0x05, 0xf0, 0xe6, 0x9d, 0xa9, 0xa8, 0xf9, 0x07, 0x24, 0x1a, 0x2e, 0x92, 0x3c,
                0x8c, 0xc8,
            ],
            [
                0x3d, 0xc4, 0x7d, 0x1f, 0x29, 0xc4, 0x48, 0x46, 0x1e, 0x9e, 0x76, 0xed, 0x90, 0x4f,
                0x67, 0x11,
            ],
            [
                0x0d, 0x62, 0xbf, 0x01, 0xe6, 0xfc, 0x0e, 0x1a, 0x0d, 0x3c, 0x47, 0x51, 0xc5, 0xd3,
                0x69, 0x2b,
            ],
            [
                0x8c, 0x03, 0x46, 0x8b, 0xca, 0x7c, 0x66, 0x9e, 0xe4, 0xfd, 0x5e, 0x08, 0x4b, 0xbe,
                0xe7, 0xb5,
            ],
            [
                0x52, 0x8a, 0x5b, 0xb9, 0x3b, 0xaf, 0x2c, 0x9c, 0x44, 0x73, 0xcc, 0xe5, 0xd0, 0xd2,
                0x2b, 0xd9,
            ],
            [
                0xdf, 0x6a, 0x30, 0x1e, 0x95, 0xc9, 0x5d, 0xad, 0x97, 0xae, 0x0c, 0xc8, 0xc6, 0x91,
                0x3b, 0xd8,
            ],
            [
                0x80, 0x11, 0x89, 0x90, 0x2c, 0x85, 0x7f, 0x39, 0xe7, 0x35, 0x91, 0x28, 0x5e, 0x70,
                0xb6, 0xdb,
            ],
            [
                0xe6, 0x17, 0x34, 0x6a, 0xc9, 0xc2, 0x31, 0xbb, 0x36, 0x50, 0xae, 0x34, 0xcc, 0xca,
                0x0c, 0x5b,
            ],
            [
                0x27, 0xd9, 0x34, 0x37, 0xef, 0xb7, 0x21, 0xaa, 0x40, 0x18, 0x21, 0xdc, 0xec, 0x5a,
                0xdf, 0x89,
            ],
            [
                0x89, 0x23, 0x7d, 0x9d, 0xed, 0x9c, 0x5e, 0x78, 0xd8, 0xb1, 0xc9, 0xb1, 0x66, 0xcc,
                0x73, 0x42,
            ],
            [
                0x4a, 0x6d, 0x80, 0x91, 0xbf, 0x5e, 0x7d, 0x65, 0x11, 0x89, 0xfa, 0x94, 0xa2, 0x50,
                0xb1, 0x4c,
            ],
            [
                0x0e, 0x33, 0xf9, 0x60, 0x55, 0xe7, 0xae, 0x89, 0x3f, 0xfc, 0x0e, 0x3d, 0xcf, 0x49,
                0x29, 0x02,
            ],
            [
                0xe6, 0x1c, 0x43, 0x2b, 0x72, 0x0b, 0x19, 0xd1, 0x8e, 0xc8, 0xd8, 0x4b, 0xdc, 0x63,
                0x15, 0x1b,
            ],
            [
                0xf7, 0xe5, 0xae, 0xf5, 0x49, 0xf7, 0x82, 0xcf, 0x37, 0x90, 0x55, 0xa6, 0x08, 0x26,
                0x9b, 0x16,
            ],
            [
                0x43, 0x8d, 0x03, 0x0f, 0xd0, 0xb7, 0xa5, 0x4f, 0xa8, 0x37, 0xf2, 0xad, 0x20, 0x1a,
                0x64, 0x03,
            ],
            [
                0xa5, 0x90, 0xd3, 0xee, 0x4f, 0xbf, 0x04, 0xe3, 0x24, 0x7e, 0x0d, 0x27, 0xf2, 0x86,
                0x42, 0x3f,
            ],
            [
                0x5f, 0xe2, 0xc1, 0xa1, 0x72, 0xfe, 0x93, 0xc4, 0xb1, 0x5c, 0xd3, 0x7c, 0xae, 0xf9,
                0xf5, 0x38,
            ],
            [
                0x2c, 0x97, 0x32, 0x5c, 0xbd, 0x06, 0xb3, 0x6e, 0xb2, 0x13, 0x3d, 0xd0, 0x8b, 0x3a,
                0x01, 0x7c,
            ],
            [
                0x92, 0xc8, 0x14, 0x22, 0x7a, 0x6b, 0xca, 0x94, 0x9f, 0xf0, 0x65, 0x9f, 0x00, 0x2a,
                0xd3, 0x9e,
            ],
            [
                0xdc, 0xe8, 0x50, 0x11, 0x0b, 0xd8, 0x32, 0x8c, 0xfb, 0xd5, 0x08, 0x41, 0xd6, 0x91,
                0x1d, 0x87,
            ],
            [
                0x67, 0xf1, 0x49, 0x84, 0xc7, 0xda, 0x79, 0x12, 0x48, 0xe3, 0x2b, 0xb5, 0x92, 0x25,
                0x83, 0xda,
            ],
            [
                0x19, 0x38, 0xf2, 0xcf, 0x72, 0xd5, 0x4e, 0xe9, 0x7e, 0x94, 0x16, 0x6f, 0xa9, 0x1d,
                0x2a, 0x36,
            ],
            [
                0x74, 0x48, 0x1e, 0x96, 0x46, 0xed, 0x49, 0xfe, 0x0f, 0x62, 0x24, 0x30, 0x16, 0x04,
                0x69, 0x8e,
            ],
            [
                0x57, 0xfc, 0xa5, 0xde, 0x98, 0xa9, 0xd6, 0xd8, 0x00, 0x64, 0x38, 0xd0, 0x58, 0x3d,
                0x8a, 0x1d,
            ],
            [
                0x9f, 0xec, 0xde, 0x1c, 0xef, 0xdc, 0x1c, 0xbe, 0xd4, 0x76, 0x36, 0x74, 0xd9, 0x57,
                0x53, 0x59,
            ],
            [
                0xe3, 0x04, 0x0c, 0x00, 0xeb, 0x28, 0xf1, 0x53, 0x66, 0xca, 0x73, 0xcb, 0xd8, 0x72,
                0xe7, 0x40,
            ],
            [
                0x76, 0x97, 0x00, 0x9a, 0x6a, 0x83, 0x1d, 0xfe, 0xcc, 0xa9, 0x1c, 0x59, 0x93, 0x67,
                0x0f, 0x7a,
            ],
            [
                0x58, 0x53, 0x54, 0x23, 0x21, 0xf5, 0x67, 0xa0, 0x05, 0xd5, 0x47, 0xa4, 0xf0, 0x47,
                0x59, 0xbd,
            ],
            [
                0x51, 0x50, 0xd1, 0x77, 0x2f, 0x50, 0x83, 0x4a, 0x50, 0x3e, 0x06, 0x9a, 0x97, 0x3f,
                0xbd, 0x7c,
            ],
        ];

        // no Vec in no_std
        let mut msg = [0u8; 64];

        for i in 0..EXPECTED.len() {
            (0..i).for_each(|k| msg[k] = k as u8);
            let h = SipHash::<2, 4, Hash128>::with_key(key, &msg[0..i]).unwrap();

            assert_eq!(h.to_le_bytes(), EXPECTED[i]);
        }
    }

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SipHash24>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SipHash24>();
    }
}
