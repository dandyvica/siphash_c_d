use core::hash::Hasher;

use crate::{
    iter::slice_to_u64,
    residue::Residue,
    siphash::{Hash64, SipHash},
};

impl<const C: u8, const D: u8> Hasher for SipHash<C, D, Hash64> {
    fn write(&mut self, bytes: &[u8]) {
        //as this fn could be called recursively, this is a safeguard
        if bytes.len() == 0 {
            return;
        }

        // keep the total length updated
        self.residue.total_length += bytes.len();

        // it's all depending on the bytes length
        let iter = bytes.iter();

        // try to fill the residue
        let added = self.residue.push(iter);

        if self.residue.is_full() {
            let m_i = slice_to_u64(&self.residue.data);
            self.residue = Residue::default();
            self.state.compress_chunk(m_i);

            // now read exact 8 bytes
            let mut iter_chunk = bytes[added..].chunks_exact(8);
            while let Some(block_i) = iter_chunk.next() {
                // convert block to little endian u64
                let m_i = slice_to_u64(block_i);
                self.state.compress_chunk(m_i);
            }

            // for the remaning bytes (should be less thant 8 bytes), the process is the same
            // so call it recursively
            self.write(iter_chunk.remainder());
        }
    }

    fn finish(&self) -> u64 {
        // as self is not passed as mutable, need to copy the state to finalize the algorithm
        let mut state = self.state;

        // manage the residue which is the last block
        let mut last_block = self.residue.data;
        last_block[7] = (self.residue.total_length % 256) as u8;
        let m_i = slice_to_u64(&last_block);
        state.compress_chunk(m_i);

        // finalization for the 64-bit version of the algorithm
        state.finalization(2, 0xFF)
    }
}

#[cfg(test)]
mod tests {
    use crate::SipHash24;

    use super::*;

    #[test]
    fn test_msglength_6() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5];
        let mut siphash_2_4 = SipHash24::new((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();

        siphash_2_4.write(&msg);

        assert_eq!(siphash_2_4.residue.length, 6);
        assert_eq!(&siphash_2_4.residue.data, &[0_u8, 1, 2, 3, 4, 5, 0, 0]);

        assert_eq!(siphash_2_4.finish(), 0xcbc9466e58fee3ce);
    }

    #[test]
    fn test_msglength_9() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
        let mut siphash_2_4 = SipHash24::new((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();

        siphash_2_4.write(&msg);

        assert_eq!(siphash_2_4.residue.length, 1);
        assert_eq!(&siphash_2_4.residue.data, &[8_u8, 0, 0, 0, 0, 0, 0, 0]);

        assert_eq!(siphash_2_4.finish(), 0xecad45d97caa54fd);
    }

    #[test]
    fn test_msglength_16() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut siphash_2_4 = SipHash24::new((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();

        siphash_2_4.write(&msg);

        assert_eq!(siphash_2_4.residue.length, 0);
        assert_eq!(&siphash_2_4.residue.data, &[0_u8, 0, 0, 0, 0, 0, 0, 0]);

        assert_eq!(siphash_2_4.finish(), 0xe0dbe59a346ec38f);
    }

    #[test]
    fn test_msglength_17() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut siphash_2_4 = SipHash24::new((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();

        siphash_2_4.write(&msg);

        assert_eq!(siphash_2_4.residue.length, 1);
        assert_eq!(&siphash_2_4.residue.data, &[16_u8, 0, 0, 0, 0, 0, 0, 0]);

        assert_eq!(siphash_2_4.finish(), 0x21465b9896b2b9a0);
    }

    #[test]
    fn test_write_u64() {
        let mut siphash_2_4 = SipHash24::new((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();

        #[cfg(target_endian = "little")]
        {
            siphash_2_4.write_u64(0x0706050403020100);
            assert_eq!(siphash_2_4.finish(), 0xdd7a02a58bb1f0ab);
        }

        #[cfg(target_endian = "big")]
        {
            siphash_2_4.write_u64(0x0001020304050607);
            assert_eq!(siphash_2_4.finish(), 0xdd7a02a58bb1f0ab);
        }
    }
}
