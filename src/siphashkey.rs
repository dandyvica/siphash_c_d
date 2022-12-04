use std::convert::TryFrom;

use crate::{iter::slice_to_u64, SipError};

#[derive(Debug)]
pub struct SipHashKey(pub(crate) u64, pub(crate) u64);

impl TryFrom<&[u8]> for SipHashKey {
    type Error = SipError;

    fn try_from(key: &[u8]) -> Result<Self, Self::Error> {
        if key.len() < 16 {
            println!("============> s={}", key.len());
            Err(SipError::KeyTooShort(key.len()))
        } else {
            Ok(SipHashKey(
                slice_to_u64(&key[0..8]),
                slice_to_u64(&key[8..16]),
            ))
        }
    }
}

impl TryFrom<u128> for SipHashKey {
    type Error = SipError;

    fn try_from(key: u128) -> Result<Self, Self::Error> {
        let k0 = (key >> 64) as u64;
        let k1 = ((key << 64) >> 64) as u64;
        Ok(SipHashKey(k0, k1))
    }
}

impl TryFrom<&[u8; 16]> for SipHashKey {
    type Error = SipError;

    fn try_from(key: &[u8; 16]) -> Result<Self, Self::Error> {
        Ok(SipHashKey(
            slice_to_u64(&key[0..8]),
            slice_to_u64(&key[8..]),
        ))
    }
}

impl TryFrom<&Vec<u8>> for SipHashKey {
    type Error = SipError;

    fn try_from(key: &Vec<u8>) -> Result<Self, Self::Error> {
        SipHashKey::try_from(key.as_slice())
    }
}

impl TryFrom<(u64, u64)> for SipHashKey {
    type Error = SipError;

    fn try_from(key: (u64, u64)) -> Result<Self, Self::Error> {
        Ok(SipHashKey(key.0, key.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // taken from Appendix A
    fn test_from_tuple() {
        let s = SipHashKey::try_from((0x0706050403020100, 0x0f0e0d0c0b0a0908)).unwrap();
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_vec() {
        let key: Vec<_> = (0..=15_u8).collect();

        let s = SipHashKey::try_from(&key).unwrap();
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_slice() {
        let key = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F".as_bytes();
        let s = SipHashKey::try_from(key).unwrap();
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_wrong_slice() {
        // key is not 16 bytes => Err
        let key = "\x00\x01\x02".as_bytes();
        let s = SipHashKey::try_from(key);
        assert!(s.is_err());
        let err = s.unwrap_err();
        assert!(matches!(err, SipError::KeyTooShort(x) if x == 3));
    }

    #[test]
    // taken from Appendix A
    fn test_from_wrong_vec() {
        // key is not 16 bytes => Err
        let key = vec![0_u8, 1, 2];
        let s = SipHashKey::try_from(&key);
        assert!(s.is_err());
        let err = s.unwrap_err();
        assert!(matches!(err, SipError::KeyTooShort(x) if x == 3));
    }

    #[test]
    // taken from Appendix A
    fn test_from_array() {
        let v: Vec<_> = (0..=15_u8).collect();
        let key: [u8; 16] = v.try_into().unwrap();

        let s = SipHashKey::try_from(&key).unwrap();
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }

    #[test]
    // taken from Appendix A
    fn test_from_u128() {
        let key: u128 = 0x0706050403020100_0f0e0d0c0b0a0908;

        let s = SipHashKey::try_from(key).unwrap();
        assert_eq!(s.0, 0x0706050403020100);
        assert_eq!(s.1, 0x0f0e0d0c0b0a0908);
    }
}
