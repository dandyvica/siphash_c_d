use core::{iter::Iterator, slice::Iter};

#[derive(Debug, Default)]
pub(crate) struct Residue {
    pub(crate) length: usize,
    pub(crate) data: [u8; 8],
    pub(crate) total_length: usize,
}

impl Residue {
    pub fn push_byte(&mut self, x: u8) {
        debug_assert!(self.length < 8);

        self.data[self.length] = x;
        self.length += 1;
    }

    pub fn push(&mut self, mut iter: Iter<u8>) -> usize {
        let mut i = 0usize;
        while self.length < 8 {
            let c = iter.next();
            if c.is_none() {
                break;
            };
            self.push_byte(*c.unwrap());
            i += 1;
        }
        i
    }

    pub fn is_full(&self) -> bool {
        self.length == 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_byte() {
        let mut r = Residue::default();
        r.push_byte(0xFF);

        assert_eq!(r.length, 1);
        assert_eq!(&r.data, &[0xFF, 0, 0, 0, 0, 0, 0, 0]);

        r.push_byte(0xFE);
        assert_eq!(r.length, 2);
        assert_eq!(&r.data, &[0xFF, 0xFE, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_push() {
        let mut r = Residue::default();
        let msg = &[10_u8, 11, 12];

        r.push(msg.iter());
        assert_eq!(r.length, 3);
        assert_eq!(&r.data, &[10, 11, 12, 0, 0, 0, 0, 0]);

        r.push(msg.iter());
        assert_eq!(r.length, 6);
        assert_eq!(&r.data, &[10, 11, 12, 10, 11, 12, 0, 0]);

        r.push(msg.iter());
        assert_eq!(r.length, 8);
        assert_eq!(&r.data, &[10, 11, 12, 10, 11, 12, 10, 11]);
    }
}
