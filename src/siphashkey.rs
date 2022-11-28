use std::convert::From;

use crate::iter::slice_to_u64;

pub struct SipHashKey(pub(crate) u64, pub(crate) u64);

impl From<&[u8]> for SipHashKey {
    fn from(msg: &[u8]) -> Self {
        SipHashKey(slice_to_u64(&msg[0..8]), slice_to_u64(&msg[8..]))
    }
}

impl From<u128> for SipHashKey {
    fn from(msg: u128) -> Self {
        let k0 = ((msg << 64) >> 64) as u64;
        let k1 = (msg >> 64) as u64;
        SipHashKey(k0, k1)
    }
}

impl From<&[u8; 16]> for SipHashKey {
    fn from(msg: &[u8; 16]) -> Self {
        SipHashKey(slice_to_u64(&msg[0..8]), slice_to_u64(&msg[8..]))
    }
}

impl From<&Vec<u8>> for SipHashKey {
    fn from(msg: &Vec<u8>) -> Self {
        SipHashKey::from(msg.as_slice())
        //Key(slice_to_u64(&msg[0..8]), slice_to_u64(&msg[8..]))
    }
}

impl From<(u64, u64)> for SipHashKey {
    fn from(msg: (u64, u64)) -> Self {
        SipHashKey(msg.0, msg.1)
    }
}
