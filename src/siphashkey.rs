use std::convert::From;

use crate::iter::slice_to_u64;

pub struct SipHashKey(pub(crate) u64, pub(crate) u64);

impl From<&[u8]> for SipHashKey {
    fn from(key: &[u8]) -> Self {
        if key.len() < 16 {
            panic!("key must be 128 bits (16 bytes)");
        }
        SipHashKey(slice_to_u64(&key[0..8]), slice_to_u64(&key[8..16]))
    }
}

impl From<u128> for SipHashKey {
    fn from(key: u128) -> Self {
        let k0 = (key >> 64) as u64;
        let k1 = ((key << 64) >> 64) as u64;
        SipHashKey(k0, k1)
    }
}

impl From<&[u8; 16]> for SipHashKey {
    fn from(key: &[u8; 16]) -> Self {
        SipHashKey(slice_to_u64(&key[0..8]), slice_to_u64(&key[8..]))
    }
}

impl From<&Vec<u8>> for SipHashKey {
    fn from(key: &Vec<u8>) -> Self {
        SipHashKey::from(key.as_slice())
    }
}

impl From<(u64, u64)> for SipHashKey {
    fn from(key: (u64, u64)) -> Self {
        SipHashKey(key.0, key.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // taken from Appendix A
    fn test_from_tuple() {
        let s = SipHashKey::from((0x0706050403020100, 0x0f0e0d0c0b0a0908));
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_vec() {
        let key: Vec<_> = (0..=15_u8).collect();

        let s = SipHashKey::from(&key);
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_slice() {
        let key = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F".as_bytes();

        let s = SipHashKey::from(key);
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_array() {
        let v: Vec<_> = (0..=15_u8).collect();
        let key: [u8; 16] = v.try_into().unwrap();

        let s = SipHashKey::from(&key);
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_u128() {
        let key: u128 = 0x0706050403020100_0f0e0d0c0b0a0908;

        let s = SipHashKey::from(key);
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    #[should_panic]
    // taken from Appendix A
    fn test_from_bad_slice() {
        let key = "\x00\x01".as_bytes();
        let _ = SipHashKey::from(key);
    }

    #[test]
    #[should_panic]
    // taken from Appendix A
    fn test_from_bad_vec() {
        let key: Vec<_> = (0..4_u8).collect();
        let _ = SipHashKey::from(&key);
    }
}
