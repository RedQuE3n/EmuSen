//! The IPU's input (EE User's Manual §8.3, IPU_BP): IPU_in_FIFO's eight qwords behind an internal buffer of up to two,
//! read as one bit stream from the bit pointer BP in the first qword of the buffer. A qword entering the IPU fills the
//! internal buffer first (Sapphire_Disputes.md D-302), so FP counts the qwords before the FIFO and IFC the FIFO's. The
//! stream's bits are taken in memory order, each byte's highest bit first, as an MPEG-2 bit stream is written
//! (H.262 §6.2, "bslbf"). Sapphire_Native.md §34.4.

use std::collections::VecDeque;

/// IPU_in_FIFO's depth, and the internal buffer's (IPU_BP.FP's largest value).
pub const FIFO_QWORDS: usize = 8;
pub const INTERNAL_QWORDS: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    /// The qwords in the IPU, the internal buffer's first, each with its bytes reversed so the stream's first bit is
    /// bit 127.
    q: VecDeque<u128>,
    bp: u32,
}

impl Default for Input {
    fn default() -> Self {
        Input { q: VecDeque::with_capacity(FIFO_QWORDS + INTERNAL_QWORDS), bp: 0 }
    }
}

impl Input {
    /// BCLR (§8.4): every qword dropped, the next to enter read from bit `bp`.
    pub fn clear(&mut self, bp: u32) {
        self.q.clear();
        self.bp = bp & 0x7F;
    }

    pub fn bp(&self) -> u32 {
        self.bp
    }

    /// IPU_BP.FP: the qwords in the internal buffer.
    pub fn fp(&self) -> usize {
        self.q.len().min(INTERNAL_QWORDS)
    }

    /// IPU_BP.IFC and IPU_CTRL.IFC: the qwords in IPU_in_FIFO.
    pub fn ifc(&self) -> usize {
        self.q.len() - self.fp()
    }

    /// Qwords IPU_in_FIFO can take now.
    pub fn room(&self) -> usize {
        FIFO_QWORDS - self.ifc()
    }

    /// A qword written to IPU_in_FIFO; refused when it is full.
    pub fn push(&mut self, q: u128) -> bool {
        let ok = self.room() > 0;
        if ok {
            self.q.push_back(q.swap_bytes());
        }
        ok
    }

    /// The bits in the IPU from BP on.
    pub fn available(&self) -> u32 {
        (self.q.len() as u32 * 128).saturating_sub(self.bp)
    }

    /// The 32 bits `offset` bits after BP, the first highest; bits past the last qword read 0.
    pub fn bits_at(&self, offset: u32) -> u32 {
        let p = self.bp + offset;
        let (i, s) = ((p / 128) as usize, p % 128);
        let a = self.q.get(i).copied().unwrap_or(0);
        let w = if s == 0 { a } else { a << s | self.q.get(i + 1).copied().unwrap_or(0) >> (128 - s) };
        (w >> 96) as u32
    }

    /// The next 32 bits.
    pub fn peek32(&self) -> u32 {
        self.bits_at(0)
    }

    /// The next `n` bits (1 to 32) as a number.
    pub fn peek(&self, n: u32) -> u32 {
        self.peek32() >> (32 - n)
    }

    /// Takes `n` bits; a qword wholly read leaves the IPU and the FIFO's next enters the internal buffer.
    pub fn skip(&mut self, n: u32) {
        self.bp += n;
        while self.bp >= 128 && !self.q.is_empty() {
            self.q.pop_front();
            self.bp -= 128;
        }
    }

    /// Takes the next `n` bits (0 to 32).
    pub fn take(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let v = self.peek(n);
        self.skip(n);
        v
    }

    /// The zero bits from BP up to the first one, counted no further than `limit` or the bits in the IPU.
    pub fn zeros(&self, limit: u32) -> u32 {
        let limit = limit.min(self.available());
        let mut z = 0;
        while z < limit {
            let lz = self.bits_at(z).leading_zeros();
            if lz < 32 {
                return (z + lz).min(limit);
            }
            z += 32;
        }
        limit
    }

    /// Whether BP lies on a byte of the stream (qwords lie on 16-byte boundaries of memory).
    pub fn aligned(&self) -> bool {
        self.bp % 8 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_cross_qwords_in_memory_order_highest_bit_first() {
        let mut i = Input::default();
        i.push(u128::from_le_bytes(core::array::from_fn(|k| k as u8)));
        i.push(u128::from_le_bytes(core::array::from_fn(|k| 0x10 + k as u8)));
        assert_eq!(i.peek32(), 0x0001_0203);
        i.skip(124);
        assert_eq!(i.peek32(), 0xF101_1121, "the last nibble of the first qword, then the second's bytes");
        i.skip(4);
        assert_eq!((i.fp(), i.ifc(), i.bp()), (1, 0, 0));
        assert_eq!(i.peek(8), 0x10);
    }

    #[test]
    fn the_internal_buffer_fills_before_the_fifo() {
        let mut i = Input::default();
        for n in 0..10 {
            assert!(i.push(n));
        }
        assert!(!i.push(10), "eight in the FIFO and two before it");
        assert_eq!((i.fp(), i.ifc(), i.room()), (2, 8, 0));
        i.skip(128);
        assert_eq!((i.fp(), i.ifc(), i.room()), (2, 7, 1));
        assert_eq!(i.zeros(1000), 7, "the qword 1's first byte");
    }
}
