// A custom iterator to iterator through message blocks fomr 0 to w-1 (paper notation)
// It handles the last message block and returns a u64 value for each item
use core::{iter::Iterator, slice::ChunksExact};

pub(crate) struct MessageChunk<'a>(pub(crate) &'a [u8]);

#[derive(Debug)]
pub(crate) struct IterHelper<'a> {
    last: bool,
    length: usize,
    iter: ChunksExact<'a, u8>,
}

impl<'a> IntoIterator for &'a MessageChunk<'a> {
    type Item = u64;
    type IntoIter = IterHelper<'a>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterHelper {
            last: false,
            length: self.0.len(),
            iter: self.0.chunks_exact(8),
        }
    }
}

impl<'a> Iterator for IterHelper<'a> {
    type Item = u64;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        if self.last {
            None
        } else if let Some(m_i) = self.iter.next() {
            Some(slice_to_u64(m_i))
        } else {
            let mut last_m = [0u8; 8];
            last_m[7] = (self.length % 256) as u8;

            for b in self.iter.remainder().iter().enumerate() {
                last_m[b.0] = *b.1;
            }

            self.last = true;
            Some(slice_to_u64(&last_m))
        }
    }
}

// internal helper
#[inline]
pub(crate) fn slice_to_u64(s: &[u8]) -> u64 {
    debug_assert!(s.len() == 8);
    u64::from_le_bytes(s.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // taken from Appendix A: 2 chunks
    fn test_iterator_1() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert_eq!(msg.len(), 15);

        // first block
        let chunks = MessageChunk(&msg);
        let mut iter = chunks.into_iter();

        let m1 = iter.next().unwrap();
        assert_eq!(m1, 0x0706050403020100);

        let m2 = iter.next().unwrap();
        assert_eq!(m2, 0x0f0e0d0c0b0a0908);

        assert!(iter.next().is_none());
    }

    #[test]
    // 3 chunks
    fn test_iterator_2() {
        let msg: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        assert_eq!(msg.len(), 16);

        // first block
        let chunks = MessageChunk(&msg);
        let mut iter = chunks.into_iter();

        let m1 = iter.next().unwrap();
        assert_eq!(m1, 0x0706050403020100);

        let m2 = iter.next().unwrap();
        assert_eq!(m2, 0x0f0e0d0c0b0a0908);

        let m3 = iter.next().unwrap();
        assert_eq!(m3, 0x1000000000000000);

        assert!(iter.next().is_none());
    }

    #[test]
    // 1 chunk (example given page 4)
    fn test_iterator_3() {
        let msg: &[u8] = &[0xAF];
        assert_eq!(msg.len(), 1);

        // first block
        let chunks = MessageChunk(&msg);
        let mut iter = chunks.into_iter();

        let m1 = iter.next().unwrap();
        assert_eq!(m1, 0x01000000000000AF);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_empty_msg() {
        let msg: &[u8] = &[];

        // first block
        let chunks = MessageChunk(&msg);
        let mut iter = chunks.into_iter();

        let m1 = iter.next().unwrap();
        assert_eq!(m1, 0);

        assert!(iter.next().is_none());
    }

    #[test]
    // 1 chunk (example given page 4)
    fn test_slice_to_u64() {
        assert_eq!(slice_to_u64(&[0, 1, 2, 3, 4, 5, 6, 7]), 0x0706050403020100);
    }
}
