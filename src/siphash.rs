use crate::iter::MessageChunk;

// automate the AXR network computations
macro_rules! oper {
    (add, $v:expr, $fst:literal, $snd:literal) => {
        $v[$fst] = $v[$fst].wrapping_add($v[$snd]);
    };

    (shiftl, $v:expr, $i:literal, $pos:literal) => {
        $v[$i] = $v[$i].rotate_left($pos);
    };

    (xor, $v:expr, $fst:literal, $snd:literal) => {
        $v[$fst] ^= $v[$snd];
    };
}

#[derive(Debug, Default)]
/// The generic implementation of `siphash_c_d`.
pub struct SipHash<const C: u8, const D: u8> {
    // key is broken down into k0 and k1 which are little endian 64 bits integers
    k0: u64,
    k1: u64,

    // internal state
    v: [u64; 4],
}

impl<const C: u8, const D: u8> SipHash<C, D> {
    /// Calculate the `siphash_c_d` value of the message.
    pub fn new(k0: u64, k1: u64, msg: &[u8]) -> u64 {
        // algorithm explicitly stated that k0 and k1 are little endian integers
        let mut siphash: SipHash<C, D>;
        #[cfg(target_endian = "big")]
        {
            siphash = SipHash::<C, D>::initialization(k0.to_le(), k1.to_le());
        }
        #[cfg(target_endian = "little")]
        {
            siphash = SipHash::<C, D>::initialization(k0, k1);
        }

        //let mut siphash = SipHash::<C,D>::initialization(k0, k1);
        siphash.compression(msg);
        siphash.finalization()
    }

    /// Calculate the `siphash_c_d` value of the message `msg` using the key `key`.
    /// 
    /// # Panics
    /// Panics if the length of the key is less than 16 bytes.
    pub fn from_slice(key: &[u8], msg: &[u8]) -> u64 {
        if key.len() < 16_usize {
            panic!("key must be 128 bits (16 bytes)");
        }

        // we can safely use try_into() because we're sure slice is 8 bytes long
        let k0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
        let k1 = u64::from_le_bytes(key[8..16].try_into().unwrap());

        SipHash::<C, D>::new(k0, k1, msg)
    }

    // core function of the algorithm
    fn sip_round(&mut self) {
        oper!(add, self.v, 0, 1);
        oper!(add, self.v, 2, 3);
        oper!(shiftl, self.v, 1, 13);
        oper!(shiftl, self.v, 3, 16);
        oper!(xor, self.v, 1, 0);
        oper!(xor, self.v, 3, 2);

        oper!(shiftl, self.v, 0, 32);

        oper!(add, self.v, 2, 1);
        oper!(add, self.v, 0, 3);
        oper!(shiftl, self.v, 1, 17);
        oper!(shiftl, self.v, 3, 21);
        oper!(xor, self.v, 1, 2);
        oper!(xor, self.v, 3, 0);

        oper!(shiftl, self.v, 2, 32);
    }

    // this is described in §2.1
    fn initialization(k0: u64, k1: u64) -> Self {
        let v = [
            k0 ^ 0x736f6d6570736575_u64,
            k1 ^ 0x646f72616e646f6d_u64,
            k0 ^ 0x6c7967656e657261_u64,
            k1 ^ 0x7465646279746573_u64,
        ];

        Self { k0, k1, v }
    }

    // compression algorithm for a message m_i
    fn compress_chunk(&mut self, m_i: u64) {
        self.v[3] ^= m_i;

        // then C iteration of SipRound
        (0..C).for_each(|_| self.sip_round());

        // followed by
        self.v[0] ^= m_i;

        //println!("init v={:x?}", self.v);
    }

    //
    fn compression(&mut self, msg: &[u8]) {
        // use the custom iterator to iterate through m_i blocks
        // from 0 to w-1
        let wrapped_msg = MessageChunk(msg);

        for m_i in &wrapped_msg {
            self.compress_chunk(m_i);
        }
    }

    // Finalization step
    fn finalization(&mut self) -> u64 {
        // After all the message words have been processed, SipHash-c-d xors the constant ff to the state
        self.v[2] ^= 0xFF;

        // then does d iterations of SipRound
        (0..D).for_each(|_| self.sip_round());

        // returns the 64-bit value
        self.v[0] ^ self.v[1] ^ self.v[2] ^ self.v[3]
    }
}

/// The `siphash_2_4` hash calculation.
pub type SipHash24 = SipHash<2, 4>;

/// The `siphash_4_8` hash calculation.
pub type SipHash48 = SipHash<4, 8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // taken from Appendix A
    fn test_siphash_new() {
        let msg: Vec<_> = (0..=14_u8).collect();
        assert_eq!(msg.len(), 15);

        let siphash_2_4 = SipHash24::new(0x0706050403020100, 0x0f0e0d0c0b0a0908, &msg);
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from Appendix A
    fn test_siphash_from() {
        let key: Vec<_> = (0..=15_u8).collect();
        let msg: Vec<_> = (0..=14_u8).collect();
        assert_eq!(msg.len(), 15);

        let siphash_2_4 = SipHash24::from_slice(&key, &msg);
        assert_eq!(siphash_2_4, 0xa129ca6149be45e5);
    }

    #[test]
    // taken from https://github.com/google/guava/blob/master/guava-tests/test/com/google/common/hash/SipHashFunctionTest.java
    fn test_sample2() {
        let key: Vec<_> = (0..=15).collect();

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

        for i in 0..EXPECTED.len() {
            let msg: Vec<u8> = (0..i as u8).collect();
            assert_eq!(SipHash24::from_slice(&key, &msg), EXPECTED[i]);
        }
    }
}
