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

// the internal state keeps all intermediate values for the algorithm
#[derive(Copy, Clone)]
pub(crate) struct State<const C: u8, const D: u8> {
    v: [u64; 4],
}

impl<const C: u8, const D: u8> State<C, D> {
    // this is described in §2.1
    pub fn new(k0: u64, k1: u64) -> Self {
        let v = [
            k0 ^ 0x736f6d6570736575_u64,
            k1 ^ 0x646f72616e646f6d_u64,
            k0 ^ 0x6c7967656e657261_u64,
            k1 ^ 0x7465646279746573_u64,
        ];

        Self { v }
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

    // compression algorithm for a message m_i
    pub fn compress_chunk(&mut self, m_i: u64) {
        // The mi’s are iteratively processed by doing
        self.v[3] ^= m_i;

        // then C iteration of SipRound
        (0..C).for_each(|_| self.sip_round());

        // followed by
        self.v[0] ^= m_i;
    }

    // finalization step
    pub(crate) fn finalization(&mut self, i: usize, u: u64) -> u64 {
        // After all the message words have been processed, SipHash-c-d xors the constant u to the state
        //i is the index for which the constant u is xored
        // i = 2, u = 0xFF pour SipHash64
        // i = 1, u = 0xEE pour SipHash128
        self.v[i] ^= u;

        // then does D iterations of SipRound
        (0..D).for_each(|_| self.sip_round());

        // returns the 64-bit value
        self.v[0] ^ self.v[1] ^ self.v[2] ^ self.v[3]
    }

    // this step is just for the Hash128 algo
    pub(crate) fn hash128_additional(&mut self) {
        self.v[1] ^= 0xEE;
    }
}
