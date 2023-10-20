use std::cmp::Ord;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitXor, Not};

use crate::utils::*;

pub const BITMAP_BITS: usize = 2048;
pub const BITMAP_BITS_BITS: usize = 11;

const CHECK_UNUSED_BITS_TABLE: [(u64, u32); 6] = [
    (
        0b0101010101010101010101010101010101010101010101010101010101010101,
        1,
    ),
    (
        0b0011001100110011001100110011001100110011001100110011001100110011,
        2,
    ),
    (
        0b0000111100001111000011110000111100001111000011110000111100001111,
        4,
    ),
    (
        0b0000000011111111000000001111111100000000111111110000000011111111,
        8,
    ),
    (
        0b0000000000000000111111111111111100000000000000001111111111111111,
        16,
    ),
    (
        0b0000000000000000000000000000000011111111111111111111111111111111,
        32,
    ),
];

#[inline]
fn check_unused_bit_u64(index_bit: u32, value: u64) -> bool {
    let index_bit_us = index_bit as usize;
    let (bitmask, shift) = CHECK_UNUSED_BITS_TABLE[index_bit_us];
    (value & bitmask) == ((value >> shift) & bitmask)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SmallVec<T, const N: usize> {
    data: [T; N],
    len: u8,
}

impl<T, const N: usize> SmallVec<T, N>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Self
    where
        T: Default + Clone + Copy,
    {
        let mut out = Self {
            data: [T::default(); N],
            len: 0,
        };
        for (i, t) in iter.into_iter().enumerate() {
            out.data[i] = t;
            out.len += 1;
        }
        out
    }

    pub fn from_slice(t: &[T]) -> SmallVec<T, N>
    where
        T: Default + Clone + Copy,
    {
        let mut out = SmallVec {
            data: [T::default(); N],
            len: u8::try_from(t.len()).unwrap(),
        };
        out.data[0..t.len()].copy_from_slice(t);
        out
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data[0..self.len as usize]
    }

    #[inline]
    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data[0..self.len as usize]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn insert(&mut self, e: T) {
        let p = match self.data().binary_search(&e) {
            Ok(p) => p,
            Err(p) => p,
        };
        let old_len = self.len as usize;
        self.len += 1;
        self.data_mut().copy_within(p..old_len, p + 1);
        self.data_mut()[p] = e;
    }

    #[inline]
    pub fn remove(&mut self, e: T) {
        match self.data().binary_search(&e) {
            Ok(p) => {
                let old_len = self.len as usize;
                self.data_mut().copy_within(p + 1..old_len, p);
                self.data_mut()[old_len - 1] = T::default();
                self.len -= 1;
            }
            Err(_) => {}
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SmartAllValues<T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug> {
    Unknown,
    Bitmap(Box<SmartBitmap<T>>),
}

impl<T> SmartAllValues<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    fn value_eq(&self, other: &Self) -> bool {
        match self {
            SmartAllValues::Unknown => false,
            SmartAllValues::Bitmap(a) => match other {
                SmartAllValues::Unknown => false,
                SmartAllValues::Bitmap(b) => **a == **b,
            },
        }
    }
}

impl<T> Not for SmartAllValues<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = SmartAllValues<T>;
    fn not(self) -> Self::Output {
        match self {
            SmartAllValues::Unknown => SmartAllValues::Unknown,
            SmartAllValues::Bitmap(b) => SmartAllValues::Bitmap(Box::new(!*b)),
        }
    }
}

impl<T> BitAnd for SmartAllValues<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = SmartAllValues<T>;
    fn bitand(self, rhs: Self) -> Self::Output {
        match self {
            SmartAllValues::Unknown => SmartAllValues::Unknown,
            SmartAllValues::Bitmap(a) => match rhs {
                SmartAllValues::Unknown => SmartAllValues::Unknown,
                SmartAllValues::Bitmap(b) => {
                    if let Some(r) = *a & *b {
                        SmartAllValues::Bitmap(Box::new(r))
                    } else {
                        SmartAllValues::Unknown
                    }
                }
            },
        }
    }
}

impl<T> BitXor for SmartAllValues<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = SmartAllValues<T>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match self {
            SmartAllValues::Unknown => SmartAllValues::Unknown,
            SmartAllValues::Bitmap(a) => match rhs {
                SmartAllValues::Unknown => SmartAllValues::Unknown,
                SmartAllValues::Bitmap(b) => {
                    if let Some(r) = *a ^ *b {
                        SmartAllValues::Bitmap(Box::new(r))
                    } else {
                        SmartAllValues::Unknown
                    }
                }
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SmartBitmap<T> {
    // all inputs must be ordered.
    pub inputs: SmallVec<T, BITMAP_BITS_BITS>,
    bitmap: [u64; BITMAP_BITS >> 6],
}

impl<T> SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    pub fn from_bool(value: bool) -> Self {
        let mut out = Self {
            inputs: SmallVec::new(),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        out.bitmap[0] = u64::from(value);
        out
    }

    pub fn from_input(input: T) -> Self {
        let mut out = Self {
            inputs: SmallVec::new(),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        out.inputs.insert(input);
        out.bitmap[0] = 0b10;
        out
    }

    #[inline]
    pub fn bitmap_bitlen(&self) -> usize {
        1 << self.inputs.len()
    }

    #[inline]
    pub fn bitmap_u64len(&self) -> usize {
        let bl = self.bitmap_bitlen();
        (bl + 63) >> 6
    }

    #[inline]
    pub fn bitmap(&self) -> &[u64] {
        &self.bitmap[0..self.bitmap_u64len()]
    }

    #[inline]
    fn bitmap_mut(&mut self) -> &mut [u64] {
        let l = self.bitmap_u64len();
        &mut self.bitmap[0..l]
    }

    fn if_unused_input(&self, i: u32) -> bool {
        let bitlen = self.bitmap_bitlen();
        let bitmap = self.bitmap();
        if i < 6 {
            // check in 64-bit word
            if bitmap.iter().all(|x| check_unused_bit_u64(i, *x)) {
                return true;
            }
        } else if i == 6 {
            // check between two 64-bit word (if same)
            let mut ok = true;
            for j in 0..(bitlen >> 7) {
                if bitmap[j << 1] != bitmap[(j << 1) + 1] {
                    ok = false;
                }
            }
            if ok {
                return true;
            }
        } else {
            // check between many 64-bit words (if same)
            let mut ok = true;
            let shift = i - 6;
            let inc_pos = 1 << shift;
            for j in 0..(bitlen >> (shift + 7)) {
                for k in j << (shift + 1)..(2 * j + 1) << shift {
                    if bitmap[k] != bitmap[k + inc_pos] {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                return true;
            }
        }
        false
    }

    fn check_unused_input(&self, start: u32) -> Option<u32> {
        let input_len = self.inputs.len() as u32;
        let mut found_input = None;
        // try find unused input
        for i in start..input_len {
            if self.if_unused_input(i) {
                found_input = Some(i);
                break;
            }
        }
        found_input
    }

    fn remove_input(&mut self, found_input: u32) {
        // if some unused input - reorganize
        let bitlen = self.bitmap_bitlen();
        let bitmap = self.bitmap_mut();
        if found_input < 6 {
            let elem_mask = (1 << (1 << found_input)) - 1;
            for i in 0..(bitlen >> (found_input + 1)) {
                let si = i << (found_input + 1);
                let elem = (bitmap[si >> 6] >> (si & 63)) & elem_mask;
                let di = i << found_input;
                let bdi = di >> 6;
                bitmap[bdi] = (bitmap[bdi] & !(elem_mask << (di & 63))) | (elem << (di & 63));
            }
        } else if found_input == 6 {
            for i in 0..(bitlen >> 7) {
                bitmap[i] = bitmap[i << 1];
            }
        } else {
            let shift = found_input - 6;
            for i in 0..(bitlen >> (shift + 7)) {
                let k = i << shift;
                for j in 0..1 << shift {
                    bitmap[k + j] = bitmap[(k << 1) + j];
                }
            }
        }
        // zeroing rest of bitmap
        if bitlen < 128 {
            bitmap[0] = bitmap[0] & ((1u64 << (bitlen >> 1)) - 1);
        } else {
            for i in (bitlen >> 7)..(bitlen >> 6) {
                bitmap[i] = 0;
            }
        }
        self.inputs.remove(self.inputs.data()[found_input as usize]);
    }

    fn check_all_unused_inputs(&mut self) -> u64 {
        let mut input_mask = 0u64;
        for i in 0..self.inputs.len() as u32 {
            if self.if_unused_input(i) {
                input_mask |= 1 << i;
            }
        }
        input_mask
    }

    fn remove_unused_inputs(&mut self) {
        let mut start = 0;
        loop {
            let input_len = self.inputs.len() as u32;
            if start >= input_len {
                break;
            }
            let found_input = self.check_unused_input(start);
            if let Some(found_input) = found_input {
                self.remove_input(found_input);
                start = found_input;
            } else {
                break;
            }
        }
    }

    fn apply_new_inputs(
        &self,
        self_input_num: usize,
        bit_start: usize,
        b_inputs: &[T],
    ) -> Result<Self, usize> {
        // create merged list of inputs
        let mut merged_inputs = merge_sorted_by_key(
            self.inputs.data()[0..self_input_num]
                .iter()
                .enumerate()
                .map(|(i, x)| (i, (*x, true))),
            b_inputs.iter().enumerate().map(|(i, x)| (i, (*x, false))),
            // sort for dedup to skip rhs inputs
            |(_, (x, s))| (*x, !s),
        );
        merged_inputs.dedup_by_key(|(_, (x, _))| *x);
        if merged_inputs.len() > BITMAP_BITS_BITS {
            return Err(merged_inputs.len());
        } else if merged_inputs.len() == self_input_num && bit_start == 0 {
            let mut out = SmartBitmap {
                inputs: SmallVec::from_slice(&self.inputs.data()[0..self_input_num]),
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            if out.inputs.len() < 6 {
                out.bitmap[0] = self.bitmap[0] & ((1 << out.bitmap_bitlen()) - 1);
            } else {
                let u64len = out.bitmap_u64len();
                out.bitmap[0..u64len].copy_from_slice(&self.bitmap[0..u64len]);
            }
            return Ok(out);
        } else if self_input_num == 0 {
            let mut out = SmartBitmap {
                inputs: SmallVec::from_slice(&b_inputs),
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            let value = if ((self.bitmap[bit_start >> 6] >> (bit_start & 63)) & 1) != 0 {
                u64::MAX
            } else {
                0
            };
            if out.inputs.len() < 6 {
                out.bitmap[0] = value & ((1 << out.bitmap_bitlen()) - 1);
            } else {
                let u64len = out.bitmap_u64len();
                out.bitmap[0..u64len].fill(value);
            }
            return Ok(out);
        }
        let mut out_bitmap = [0u64; BITMAP_BITS >> 6];
        for i in 0..1 << merged_inputs.len() {
            let si =
                merged_inputs
                    .iter()
                    .enumerate()
                    .fold(0, |si, (sbit, (dbit, (_, selfbmap)))| {
                        if *selfbmap {
                            si | (((i >> sbit) & 1) << dbit)
                        } else {
                            si
                        }
                    })
                    + bit_start;
            out_bitmap[i >> 6] |= ((self.bitmap[si >> 6] >> (si & 63)) & 1) << (i & 63);
        }
        Ok(Self {
            inputs: SmallVec::from_iter(merged_inputs.into_iter().map(|(_, (x, _))| x)),
            bitmap: out_bitmap,
        })
    }

    fn split(self) -> (Self, Self) {
        let inputs_len = self.inputs.len();
        if inputs_len <= 6 {
            let mut out1 = SmartBitmap {
                inputs: SmallVec::from_slice(&self.inputs.data()[0..inputs_len - 1]),
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            let mut out2 = out1;
            let half = 1 << (inputs_len - 1);
            let mask = (1u64 << half) - 1;
            out1.bitmap[0] = self.bitmap[0] & mask;
            out2.bitmap[0] = (self.bitmap[0] >> half) & mask;
            (out1, out2)
        } else {
            let mut out1 = SmartBitmap {
                inputs: SmallVec::from_slice(&self.inputs.data()[0..inputs_len - 1]),
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            let mut out2 = out1;
            let half = 1 << (inputs_len - 6 - 1);
            out1.bitmap[0..half].copy_from_slice(&self.bitmap[0..half]);
            out2.bitmap[0..half].copy_from_slice(&self.bitmap[half..(half << 1)]);
            (out1, out2)
        }
    }

    fn join(self, rhs: Self, last_input: T) -> Self {
        assert_eq!(self.inputs, rhs.inputs);
        assert!(self.inputs.len() < BITMAP_BITS_BITS);
        assert!(*self.inputs.data().last().unwrap() < last_input);
        let inputs_len = self.inputs.len();
        if inputs_len <= 5 {
            let mut out = SmartBitmap {
                inputs: self.inputs,
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            out.inputs.insert(last_input);
            let half = 1 << inputs_len;
            let mask = (1u64 << half) - 1;
            out.bitmap[0] = (self.bitmap[0] & mask) | ((rhs.bitmap[0] & mask) << half);
            out
        } else {
            let mut out = SmartBitmap {
                inputs: self.inputs,
                bitmap: [0u64; BITMAP_BITS >> 6],
            };
            out.inputs.insert(last_input);
            let half = 1 << (inputs_len - 6);
            out.bitmap[0..half].copy_from_slice(&self.bitmap[0..half]);
            out.bitmap[half..(half << 1)].copy_from_slice(&rhs.bitmap[0..half]);
            out
        }
    }

    fn make_op(self, rhs: Self, op: impl Fn(&mut [u64], &[u64], &[u64])) -> Option<Self> {
        // println!("MakeOp");
        match self.apply_new_inputs(self.inputs.len() as usize, 0, rhs.inputs.data()) {
            Ok(ext_self) => {
                // if we can do it in single smart bitmap - not too many inputs
                let ext_rhs = rhs
                    .apply_new_inputs(rhs.inputs.len() as usize, 0, self.inputs.data())
                    .unwrap();
                let mut out = SmartBitmap {
                    inputs: ext_self.inputs,
                    bitmap: [0; BITMAP_BITS >> 6],
                };
                op(out.bitmap_mut(), ext_self.bitmap(), ext_rhs.bitmap());
                out.remove_unused_inputs();
                Some(out)
            }
            Err(merged_inputs_len) => {
                if merged_inputs_len <= BITMAP_BITS_BITS + 1 {
                    let self_input_last = *self.inputs.data().last().unwrap();
                    let rhs_input_last = *rhs.inputs.data().last().unwrap();
                    let (mut out0, mut out1) = if self_input_last != rhs_input_last {
                        // split higher
                        let ((a_bmap0, a_bmap1), b_bmap, reversed) =
                            if self_input_last > rhs_input_last {
                                (self.split(), rhs, false)
                            } else {
                                (rhs.split(), self, true)
                            };
                        let a0input_len = a_bmap0.inputs.len();
                        let ext_a_bmap0 = a_bmap0
                            .apply_new_inputs(a0input_len, 0, b_bmap.inputs.data())
                            .unwrap();
                        let ext_a_bmap1 = a_bmap1
                            .apply_new_inputs(a0input_len, 0, b_bmap.inputs.data())
                            .unwrap();
                        let ext_b_bmap = b_bmap
                            .apply_new_inputs(
                                b_bmap.inputs.len(),
                                0,
                                &a_bmap0.inputs.data()[0..a0input_len],
                            )
                            .unwrap();
                        let mut out0 = SmartBitmap {
                            inputs: ext_a_bmap0.inputs,
                            bitmap: [0; BITMAP_BITS >> 6],
                        };
                        let mut out1 = out0;
                        if reversed {
                            op(out0.bitmap_mut(), ext_b_bmap.bitmap(), ext_a_bmap0.bitmap());
                            op(out1.bitmap_mut(), ext_b_bmap.bitmap(), ext_a_bmap1.bitmap());
                        } else {
                            op(out0.bitmap_mut(), ext_a_bmap0.bitmap(), ext_b_bmap.bitmap());
                            op(out1.bitmap_mut(), ext_a_bmap1.bitmap(), ext_b_bmap.bitmap());
                        }
                        (out0, out1)
                    } else {
                        // println!("If highest are same");
                        let (a_bmap0, a_bmap1) = self.split();
                        let (b_bmap0, b_bmap1) = rhs.split();
                        let a0input_len = a_bmap0.inputs.len();
                        let b0input_len = b_bmap0.inputs.len();
                        let ext_a_bmap0 = a_bmap0
                            .apply_new_inputs(a0input_len, 0, b_bmap0.inputs.data())
                            .unwrap();
                        let ext_a_bmap1 = a_bmap1
                            .apply_new_inputs(a0input_len, 0, b_bmap0.inputs.data())
                            .unwrap();
                        let ext_b_bmap0 = b_bmap0
                            .apply_new_inputs(b0input_len, 0, a_bmap0.inputs.data())
                            .unwrap();
                        let ext_b_bmap1 = b_bmap1
                            .apply_new_inputs(b0input_len, 0, a_bmap0.inputs.data())
                            .unwrap();
                        let mut out0 = SmartBitmap {
                            inputs: ext_a_bmap0.inputs,
                            bitmap: [0; BITMAP_BITS >> 6],
                        };
                        let mut out1 = out0;
                        op(
                            out0.bitmap_mut(),
                            ext_a_bmap0.bitmap(),
                            ext_b_bmap0.bitmap(),
                        );
                        op(
                            out1.bitmap_mut(),
                            ext_a_bmap1.bitmap(),
                            ext_b_bmap1.bitmap(),
                        );
                        (out0, out1)
                    };

                    if out0 != out1 {
                        let unused_inputs =
                            out0.check_all_unused_inputs() & out1.check_all_unused_inputs();
                        if unused_inputs != 0 {
                            let input_to_remove = 63 - unused_inputs.leading_zeros();
                            out0.remove_input(input_to_remove);
                            out1.remove_input(input_to_remove);
                            // join
                            let mut out =
                                out0.join(out1, std::cmp::max(self_input_last, rhs_input_last));
                            out.remove_unused_inputs();
                            Some(out)
                        } else {
                            None
                        }
                    } else {
                        // same copy
                        out0.remove_unused_inputs();
                        Some(out0)
                    }
                } else {
                    None
                }
            }
        }
    }
}

impl<T> Not for SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = SmartBitmap<T>;
    fn not(self) -> Self::Output {
        let mut out = self;
        for x in out.bitmap_mut() {
            *x = !*x;
        }
        if out.inputs.len() < 6 {
            out.bitmap_mut()[0] &= (1 << (1 << out.inputs.len())) - 1;
        }
        out
    }
}

impl<T> BitAnd for SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = Option<SmartBitmap<T>>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.make_op(rhs, |d, a, b| {
            for i in 0..a.len() {
                d[i] = a[i] & b[i];
            }
        })
    }
}

impl<T> BitXor for SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq + Debug,
{
    type Output = Option<SmartBitmap<T>>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.make_op(rhs, |d, a, b| {
            for i in 0..a.len() {
                d[i] = a[i] ^ b[i];
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_vec() {
        let mut svec = SmallVec::<_, 10>::new();
        let data = [15, 1, 3, 58, 5, 18, 11, 53, 21];
        for (i, d) in data.iter().enumerate() {
            svec.insert(*d);
            let mut sorted = data[0..i + 1].to_vec();
            sorted.sort();
            assert_eq!(&sorted, svec.data());
        }
        for (i, d) in data.iter().enumerate() {
            svec.remove(*d);
            let mut sorted = data[i + 1..].to_vec();
            sorted.sort();
            assert_eq!(&sorted, svec.data());
            svec.remove(*d + 1);
            assert_eq!(&sorted, svec.data());
        }
    }

    #[test]
    fn test_check_unused_bit_u64() {
        for (bit, value, exp) in [
            (
                0,
                0b1100110000000000000000000000000000000000000000110000001100111111,
                true,
            ),
            (
                0,
                0b1100110000000000000000000000000000000000000100110000001100111111,
                false,
            ),
            (
                1,
                0b0101000010100000111100001010000000001010000001010000010111110101,
                true,
            ),
            (
                1,
                0b0101000010100000111100001010000000011010000001010000010111110101,
                false,
            ),
            (2, 0xaabbee0011335588, true),
            (2, 0xaabbee2011335588, false),
            (3, 0xabab3a3a7d7de1e1, true),
            (3, 0xabab3a3a7d7de2e1, false),
            (4, 0x0bc60bc64baf4baf, true),
            (4, 0x0bc61bc64baf4baf, false),
            (5, 0x0123456701234567, true),
            (5, 0x0123416701234567, false),
        ] {
            assert_eq!(exp, check_unused_bit_u64(bit, value));
        }
    }

    fn smart_bitmap_from_data<T>(inputs: &[T], bitmap: &[u64]) -> SmartBitmap<T>
    where
        T: Default + Clone + Copy + PartialEq + Eq + Ord,
    {
        let mut bmap = SmartBitmap {
            inputs: SmallVec::from_slice(inputs),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        bmap.bitmap[0..bitmap.len()].copy_from_slice(bitmap);
        bmap
    }

    #[test]
    fn test_smart_bitmap_remove_unused_inputs() {
        let mut bmap = smart_bitmap_from_data(&[3], &[0b11]);
        let exp_bmap = smart_bitmap_from_data(&[], &[0b1]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(&[3], &[0b00]);
        let exp_bmap = smart_bitmap_from_data(&[], &[0b0]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0xbcda2135]);
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0xa50faf05]);
        let exp_bmap = smart_bitmap_from_data(&[3, 6, 9, 11], &[0x93b1]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0xaaff5500]);
        let exp_bmap = smart_bitmap_from_data(&[3, 9, 11], &[0b10110100]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 12, 14, 15],
            &[
                0xbb8811aa006633dd,
                0x00ccdd0055bb2211,
                0x7700331144ddffee,
                0x113300cc55330022,
            ],
        );
        let exp_bmap = smart_bitmap_from_data(
            &[3, 4, 9, 11, 12, 14, 15],
            &[0x0cd05b21b81a063d, 0x130c530270314dfe],
        );
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 12, 14, 15],
            &[
                0xbb8811aa006633dd,
                0x00ccdd0075bb2211,
                0x7700331144ddffee,
                0x113300cc55330022,
            ],
        );
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 12, 14, 15],
            &[
                0xbb8811aa006633dd,
                0xbb8811aa006633dd,
                0x00ccdd0055bb2211,
                0x00ccdd0055bb2211,
            ],
        );
        let exp_bmap = smart_bitmap_from_data(&[3, 4, 9, 11, 12, 15], &[0x0cd05b21b81a063d]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0xaaaa5555]);
        let exp_bmap = smart_bitmap_from_data(&[3, 11], &[0b1001]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19],
            &[0x1095bca065a3, 0x5b0a04421cce2],
        );
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap =
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 15, 19], &[0x1095bca065a3, 0x1095bca065a3]);
        let exp_bmap = smart_bitmap_from_data(&[3, 4, 6, 9, 11, 15], &[0x1095bca065a3]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap =
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 15, 19], &[0x1095bca065a3, 0x1095bca165a3]);
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19, 22],
            &[
                0x1095bca065a3,
                0x1195bca065a3,
                0x1095bca065a3,
                0x1195bca065a3,
            ],
        );
        let exp_bmap =
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 15, 19], &[0x1095bca065a3, 0x1195bca065a3]);
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19, 22],
            &[
                0x1095bca065a3,
                0x1195bca065a3,
                0x1095bca065a3,
                0x1195bca067a3, // not match
            ],
        );
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19, 22, 23, 25],
            &[
                0x1095bca065a3,
                0x1195bca065a3,
                0x1095bca065a3,
                0x1195bca065a3,
                0x2295bca121a3,
                0x16989402967211a,
                0x2295bca121a3,
                0x16989402967211a,
                0x1095bca065a7,
                0x1195bca065a7,
                0x1095bca065a7,
                0x1195bca065a7,
                0x2295bca121a7,
                0x169894029672117,
                0x2295bca121a7,
                0x169894029672117,
            ],
        );
        let exp_bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19, 23, 25],
            &[
                0x1095bca065a3,
                0x1195bca065a3,
                0x2295bca121a3,
                0x16989402967211a,
                0x1095bca065a7,
                0x1195bca065a7,
                0x2295bca121a7,
                0x169894029672117,
            ],
        );
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);

        let mut bmap = smart_bitmap_from_data(
            &[3, 4, 6, 9, 11, 15, 19, 22, 23, 25],
            &[
                0x1095bca065a3,
                0x1195bca065a3,
                0x1095bca065a3,
                0x1195bca065a3,
                0x2295bca221a3, // not match
                0x16989402967211a,
                0x2295bca121a3,
                0x16989402967211a,
                0x1095bca065a7,
                0x1195bca065a7,
                0x1095bca065a7,
                0x1195bca065a7,
                0x2295bca121a7,
                0x169894029672117,
                0x2295bca121a7,
                0x169894029672117,
            ],
        );
        let exp_bmap = bmap.clone();
        bmap.remove_unused_inputs();
        assert_eq!(exp_bmap, bmap);
    }

    #[test]
    fn test_smart_bitmap_apply_new_inputs() {
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[0, 1, 3, 4, 5, 6, 9, 11, 12],
                &[
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00,
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00
                ]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[0, 1, 5, 12]
            )
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[0, 1, 3, 4, 5, 6, 9, 11, 12],
                &[
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00,
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00
                ]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135cdaa]).apply_new_inputs(
                5,
                16,
                &[0, 1, 5, 12]
            )
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[0, 1, 3, 4, 5, 6, 9, 11, 12],
                &[
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00,
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00
                ]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0x112233bcda2135]).apply_new_inputs(
                5,
                0,
                &[0, 1, 5, 12]
            )
        );

        // input duplicates in rhs
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[0, 1, 3, 4, 5, 6, 9, 11, 12],
                &[
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00,
                    0x00ff00ff0f0f0f0f,
                    0x00f000f0000f000f,
                    0xff0fff0ff0f0f0f0,
                    0xf0fff0ffff00ff00
                ]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0x111bcda2135]).apply_new_inputs(
                5,
                0,
                &[0, 1, 3, 4, 5, 12]
            )
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135])),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135]).apply_new_inputs(
                6,
                0,
                &[3, 4, 6, 9, 11, 14]
            )
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0xbcda2135])),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[3, 4, 6, 9, 11]
            )
        );

        // no changes and bitstart
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9, 11], &[0x11bcda21])),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0x11bcda2135]).apply_new_inputs(
                5,
                8,
                &[3, 4, 6, 9, 11]
            )
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[0x1111bcda2135, 0x4859fffaaa]
            )),
            smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15, 16],
                &[0x1111bcda2135, 0x4859fffaaa]
            )
            .apply_new_inputs(7, 0, &[3, 4, 6, 9, 11, 14, 15])
        );

        // no changes and bitstart
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[0xfaaa00001111bcda, 0xfadd0000004859ff]
            )),
            smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15, 16],
                &[0x1111bcda2135, 0x4859fffaaa, 0xfadd]
            )
            .apply_new_inputs(7, 16, &[3, 4, 6, 9, 11, 14, 15])
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[0x1111bcda2135, 0x4859fffaaa]
            )),
            smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15, 16],
                &[0x1111bcda2135, 0x4859fffaaa]
            )
            .apply_new_inputs(7, 0, &[])
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[0x1111bcda2135, 0x4859fffaaa]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14, 15], &[0x1111bcda2135, 0x4859fffaaa])
                .apply_new_inputs(7, 0, &[])
        );

        // no changes
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[0x1111bcda2135, 0x4859fffaaa]
            )),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14, 15], &[0x1111bcda2135, 0x4859fffaaa])
                .apply_new_inputs(7, 0, &[3, 4, 6, 9, 11, 14, 15])
        );

        // no changes (empties)
        assert_eq!(
            Ok(smart_bitmap_from_data(&[], &[1])),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0x111bcda2135]).apply_new_inputs(
                0,
                0,
                &[]
            )
        );

        // no changes (empties) and bit_start
        assert_eq!(
            Ok(smart_bitmap_from_data(&[], &[0])),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0x111bcda2135]).apply_new_inputs(
                0,
                3,
                &[]
            )
        );

        // all zeros
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14, 15], &[0, 0])),
            smart_bitmap_from_data(&[], &[]).apply_new_inputs(0, 0, &[3, 4, 6, 9, 11, 14, 15])
        );

        // all ones
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 9, 11, 14, 15],
                &[u64::MAX, u64::MAX]
            )),
            smart_bitmap_from_data(&[], &[1]).apply_new_inputs(0, 0, &[3, 4, 6, 9, 11, 14, 15])
        );

        // all ones
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9], &[0xffff])),
            smart_bitmap_from_data(&[], &[1]).apply_new_inputs(0, 0, &[3, 4, 6, 9])
        );

        // all zeros
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9], &[0])),
            smart_bitmap_from_data(&[1, 2, 3, 4, 5, 6, 7], &[0, 8]).apply_new_inputs(
                0,
                0,
                &[3, 4, 6, 9]
            )
        );

        // all zeros
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9], &[0])),
            smart_bitmap_from_data(&[1, 2, 3, 4, 5, 6, 7], &[0, 0xff00]).apply_new_inputs(
                0,
                67,
                &[3, 4, 6, 9]
            )
        );

        // all ones
        assert_eq!(
            Ok(smart_bitmap_from_data(&[3, 4, 6, 9], &[0xffff])),
            smart_bitmap_from_data(&[1, 2, 3, 4, 5, 6, 7], &[0, 8]).apply_new_inputs(
                0,
                67,
                &[3, 4, 6, 9]
            )
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 7, 8, 9, 14, 15, 16, 17],
                &[
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0
                ]
            )),
            smart_bitmap_from_data(&[3, 6, 9, 14, 17], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[4, 7, 8, 15, 16]
            )
        );

        // input duplicates in rhs
        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 7, 8, 9, 14, 15, 16, 17],
                &[
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0x0f0f0f0f55555555,
                    0x0a0a0a0a05050505,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0,
                    0xf5f5f5f5aaaaaaaa,
                    0xafafafaff0f0f0f0
                ]
            )),
            smart_bitmap_from_data(&[3, 6, 9, 14, 17], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[3, 4, 6, 7, 8, 9, 14, 15, 16, 17]
            )
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 6, 7, 9, 14, 17, 20, 22, 25, 26],
                &[
                    0xbbccddaa22113355,
                    0xbbccddaa22113355,
                    0xee33aa00cc119955,
                    0xee33aa00cc119955,
                    0x8866110044ccaa11,
                    0x8866110044ccaa11,
                    0xbb55dd00ccaa0099,
                    0xbb55dd00ccaa0099,
                    0xbbccddaa22113355,
                    0xbbccddaa22113355,
                    0xee33aa00cc119955,
                    0xee33aa00cc119955,
                    0x8866110044ccaa11,
                    0x8866110044ccaa11,
                    0xbb55dd00ccaa0099,
                    0xbb55dd00ccaa0099
                ]
            )),
            smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25, 27],
                &[
                    0xe3a0c195bcda2135,
                    0xb5d0ca0986104ca1,
                    0xeeeeeabbbbbb3433,
                    0xa0a0444a03bbcc1
                ]
            )
            .apply_new_inputs(7, 0, &[7, 20, 26])
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25],
                &[0xe3a0c195bcda2135, 0xb5d0ca0986104ca1]
            )),
            smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25, 27],
                &[
                    0xe3a0c195bcda2135,
                    0xb5d0ca0986104ca1,
                    0xeeeeeabbbbbb3433,
                    0xa0a0444a03bbcc1
                ]
            )
            .apply_new_inputs(7, 0, &[3, 6, 9, 14, 17, 22, 25])
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 7, 9],
                &[0b01011010010110101010111110101111]
            )),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b]).apply_new_inputs(3, 0, &[4, 7])
        );

        assert_eq!(
            Ok(smart_bitmap_from_data(
                &[3, 4, 6, 7, 9],
                &[0b00000101000001011111101011111010]
            )),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b]).apply_new_inputs(3, 8, &[4, 7])
        );

        assert_eq!(
            Err(12),
            smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25, 27],
                &[
                    0xe3a0c195bcda2135,
                    0xb5d0ca0986104ca1,
                    0xeeeeeabbbbbb3433,
                    0xa0a0444a03bbcc1
                ]
            )
            .apply_new_inputs(8, 0, &[7, 20, 26, 29])
        );
    }

    #[test]
    fn test_smart_bitmap_not() {
        assert_eq!(
            smart_bitmap_from_data(&[3, 6, 9, 11], &[(!0x1e6b) & 0xffff]),
            !smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b])
        );
        assert_eq!(
            smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25, 27],
                &[
                    !0xe3a0c195bcda2135,
                    !0xb5d0ca0986104ca1,
                    !0xeeeeeabbbbbb3433,
                    !0xa0a0444a03bbcc1
                ]
            ),
            !smart_bitmap_from_data(
                &[3, 6, 9, 14, 17, 22, 25, 27],
                &[
                    0xe3a0c195bcda2135,
                    0xb5d0ca0986104ca1,
                    0xeeeeeabbbbbb3433,
                    0xa0a0444a03bbcc1
                ]
            )
        );
    }

    #[test]
    fn test_smart_bitmap_bitand() {
        assert_eq!(
            Some(smart_bitmap_from_data::<usize>(&[], &[0])),
            smart_bitmap_from_data::<usize>(&[], &[0]) & smart_bitmap_from_data::<usize>(&[], &[1])
        );
        assert_eq!(
            Some(smart_bitmap_from_data::<usize>(&[], &[1])),
            smart_bitmap_from_data::<usize>(&[], &[1]) & smart_bitmap_from_data::<usize>(&[], &[1])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1c68])),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b])
                & smart_bitmap_from_data(&[3, 6, 9, 11], &[0x3dec])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[3, 6, 9], &[0x08])),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b])
                & smart_bitmap_from_data(&[3, 6, 9, 11], &[0x491c])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(
                &[1, 3, 4, 7, 10, 13, 14, 16],
                &[
                    0xe300c0009c100020,
                    0x1010420184104021,
                    0xeacc001199414211,
                    0x0a000510a03b20c0
                ]
            )),
            smart_bitmap_from_data(
                &[1, 3, 4, 7, 10, 13, 14, 16],
                &[
                    0xe3a0c195bcda2135,
                    0xb5d0ca0986104ca1,
                    0xebcdaab199634299,
                    0x0a0a0774a03bbcc1
                ]
            ) & smart_bitmap_from_data(
                &[1, 3, 4, 7, 10, 13, 14, 16],
                &[
                    0xff11ee22dd11cc22,
                    0x12345677bcdef123,
                    0xeeee1111dddd7777,
                    0x1bb1dd11eeff22cc
                ]
            )
        );
        assert_eq!(
            Some(smart_bitmap_from_data(
                &[1, 3, 7, 10, 13, 16],
                &[0xdeb031ef12345678]
            )),
            smart_bitmap_from_data(
                &[1, 3, 4, 7, 10, 13, 14, 16],
                &[
                    0x1322334455667788,
                    0x1122334455667788,
                    0xddeebb003311eeff,
                    0xddeebb003311eeff,
                ]
            ) & smart_bitmap_from_data(
                &[1, 3, 4, 7, 10, 13, 14, 16],
                &[
                    0xfdffffffffffffff,
                    0xffffffffffffffff,
                    0xffffffffffffffff,
                    0xffffffffffffffff,
                ]
            )
        );

        // joining inputs
        assert_eq!(
            Some(smart_bitmap_from_data(&[1, 2, 3, 4], &[0b0001000010000001])),
            smart_bitmap_from_data(&[1, 2, 3], &[0b10011101])
                & smart_bitmap_from_data(&[2, 3, 4], &[0b01001001])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[1, 2, 3, 4], &[0b0001000010000001])),
            smart_bitmap_from_data(&[2, 3, 4], &[0b01001001])
                & smart_bitmap_from_data(&[1, 2, 3], &[0b10011101])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[1, 2, 3], &[0b00000001])),
            smart_bitmap_from_data(&[1, 2, 3], &[0b00001101])
                & smart_bitmap_from_data(&[2, 3, 4], &[0b01011001])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(
                &[1, 2, 3, 4, 5, 7, 8, 9, 10, 11],
                &[
                    0x000031023102f50a,
                    0x000030003000f000,
                    0x0000000031020000,
                    0x0000000030000000,
                    0x00001022102250aa,
                    0x000032033203fa0f,
                    0x0000000010220000,
                    0x0000000032030000,
                    0xc408f50af50a0000,
                    0xc000f000f0000000,
                    0xc408c40800000000,
                    0xc000c00000000000,
                    0x408850aa50aa0000,
                    0xc80cfa0ffa0f0000,
                    0x4088408800000000,
                    0xc80cc80c00000000
                ]
            )),
            smart_bitmap_from_data(&[2, 5, 7, 9, 11], &[0xa0bc0417])
                & smart_bitmap_from_data(&[1, 3, 4, 8, 10], &[0xe34ac0d2])
        );
        // too many inputs to join
        assert_eq!(
            None,
            smart_bitmap_from_data(
                &[2, 5, 7, 9, 11, 14, 15, 18],
                &[
                    0xbc0a04502a78,
                    0xba0350252577,
                    0xa0a054948899,
                    0xaab0c03af0315
                ]
            ) & smart_bitmap_from_data(
                &[3, 4, 6, 10, 12, 13, 16],
                &[0xbc11466aaa22, 0xba5bb0c22577]
            )
        );
        assert_eq!(
            None,
            smart_bitmap_from_data(&[2, 5, 7, 9, 11, 14, 15], &[0xbc0a04502a78, 0xba0350252577])
                & smart_bitmap_from_data(
                    &[3, 4, 6, 10, 12, 13, 16],
                    &[0xbc11466aaa22, 0xba5bb0c22577]
                )
        );

        // join and reduce
        let exp = Some(smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23],
            &[0x030405060708090a, 0xf0d0b09070503010],
        ));
        let a = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33],
            &[
                0x031405060708090a,
                0xf0d0b09070503011,
                0x030415060708090a,
                0xf0d0b09070503011,
                0x032405060708090a,
                0xf0d0b09070503012,
                0x030425060708090a,
                0xf0d0b09070503012,
                0x033405060708090a,
                0xf0d0b09070503012,
                0x030435060708090a,
                0xf0d0b09070503012,
                0x034405060708090a,
                0xf0d0b09070503013,
                0x030445060708090a,
                0xf0d0b09070503013,
            ],
        );
        let b = smart_bitmap_from_data(
            &[6, 7, 10, 11, 15, 17, 20, 22, 23, 25],
            &[
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffe000ffff,
                0xffffffffffffd000,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xb000ffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffb000,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
            ],
        );
        assert_eq!(exp, a & b);
        assert_eq!(exp, b & a);

        // failure
        let a = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33],
            &[
                0x031405061708090a, // doesn't match
                0xf0d0b09070503011,
                0x030415060708090a,
                0xf0d0b09070503011,
                0x032405060708090a,
                0xf0d0b09070503012,
                0x030425060708090a,
                0xf0d0b09070503012,
                0x033405060708090a,
                0xf0d0b09070503012,
                0x030435060708090a,
                0xf0d0b09070513012, // doesn't match
                0x034405060708090a,
                0xf0d0b09070503013,
                0x030445e60708090a, // doesn't match
                0xf0d0b09070503013,
            ],
        );
        let b = smart_bitmap_from_data(
            &[6, 7, 10, 11, 15, 17, 20, 22, 23, 25],
            &[
                0xfffffffffffffff3,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffe000ffff,
                0xffffffffffffd000,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffffff,
                0xfffffff2ffffffff,
                0xffffffffffffffff,
                0xb000ffffffffffff,
                0xffffffffffffffff,
                0xffffffffffffb000,
                0xffffffffffffffff,
                0xffff1fffffffffff,
                0xffffffffffffffff,
            ],
        );
        assert_eq!(None, a & b);
        assert_eq!(None, b & a);

        // next testcase
        let exp = Some(smart_bitmap_from_data(
            &[10, 11, 15, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0706050403020100,
                0x0f0e0d0c0b0a0908,
                0x1716151413001110,
                0x1f1e1d1c1b1a1918,
                0x2726252423222120,
                0x2f2e002c2b2a2928,
                0x3736353433323130,
                0x3f3e3d3c3b3a3938,
                0x4746454443424100,
                0x4f4e4d4c4b4a4948,
                0x5756555453005150,
                0x5f5e5d5c5b5a5958,
                0x6766656463626160,
                0x6f6e006c6b6a6968,
                0x7776757473727170,
                0x7f7e7d7c7b7a7978,
            ],
        ));
        let a = smart_bitmap_from_data(
            &[6, 17, 20, 22, 23, 25, 29, 33],
            &[
                0b1111111111111101u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111111111111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
            ],
        );
        let b = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0303020201010000,
                0x0707060605050404,
                0x0b0b0a0a09090808,
                0x0f0f0e0e0d0d0c0c,
                0x1313120011111010, // <-
                0x1717161615151414,
                0x1b1b1a1a19191818,
                0x1f1f1e1e1d1d1c1c,
                0x2323222221212020,
                0x2727262625252424,
                0x2b2b2a2a29292828,
                0x2f2f2e2e002d2c2c, // <-
                0x3333323231313030,
                0x3737363635353434,
                0x3b3b3a3a39393838,
                0x3f3f3e3e3d3d3c3c,
                0x4343424241410000,
                0x4747464645454444,
                0x4b4b4a4a49494848,
                0x4f4f4e4e4d4d4c4c,
                0x5353520051515050, // <-
                0x5757565655555454,
                0x5b5b5a5a59595858,
                0x5f5f5e5e5d5d5c5c,
                0x6363626261616060,
                0x6767666665656464,
                0x6b6b6a6a69696868,
                0x6f6f6e6e006d6c6c, // <-
                0x7373727271717070,
                0x7777767675757474,
                0x7b7b7a7a79797878,
                0x7f7f7e7e7d7d7c7c,
            ],
        );
        assert_eq!(exp, a & b);
        assert_eq!(exp, b & a);

        let a = smart_bitmap_from_data(
            &[6, 17, 20, 22, 23, 25, 29, 33],
            &[
                0b1111111111111101u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111111111111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111011111111u64 << 48), // <- doesn't match
            ],
        );
        let b = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0303020201010000,
                0x0707060605050404,
                0x0b0b0a0a09090808,
                0x0f0f0e0e0d0d0c0c,
                0x1313120011111010, // <-
                0x1717161615151414,
                0x1b1b1a1a19191818,
                0x1f1f1e1e1d1d1c1c,
                0x2323002221212020, // <- doesn't match
                0x2727262625252424,
                0x2b2b2a2a29292828,
                0x2f2f2e2e002d2c2c, // <-
                0x3333323231313030,
                0x3737363635353434,
                0x3b3b3a3a39393838,
                0x3f3f3e3e3d3d3c3c,
                0x4343424241410000,
                0x4747464645454444,
                0x4b4b4a4a49494848,
                0x4f4f4e4e4d4d4c4c,
                0x5353520051515050, // <-
                0x5757565655555454,
                0x5b5b5a5a59595858,
                0x5f5f5e5e5d5d5c5c,
                0x6363626261616060,
                0x6767666665656464,
                0x6b6b6a6a69696868,
                0x6f6f6e6e006d6c6c, // <-
                0x7373727271717070,
                0x7777767675757474,
                0x7b7b7a7a79797878,
                0x7f7f7e7e7d7d7c7c,
            ],
        );
        assert_eq!(None, a & b);
        assert_eq!(None, b & a);

        // reduction with this same highest inputs for left and right argument.
        let exp = Some(smart_bitmap_from_data(
            &[10, 11, 15, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0706050403020100,
                0x0f0e0d0c0b0a0908,
                0x1716151413001110,
                0x1f1e1d1c1b1a1918,
                0x2726252423222120,
                0x2f2e002c2b2a2928,
                0x3736353433323130,
                0x3f3e3d3c3b3a3938,
                0x4746454443424100,
                0x4f4e4d4c4b4a4948,
                0x5756555453005150,
                0x5f5e5d5c5b5a5958,
                0x6766656463626160,
                0x6f6e006c6b6a6968,
                0x7776757473727170,
                0x7f7e7d7c7b7a7978,
            ],
        ));
        let a = smart_bitmap_from_data(
            &[6, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0b1111111111111101u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111111111111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111111111111110u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111111111111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
            ],
        );
        let b = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0303020201010000,
                0x0707060605050404,
                0x0b0b0a0a09090808,
                0x0f0f0e0e0d0d0c0c,
                0x1313120011111010, // <-
                0x1717161615151414,
                0x1b1b1a1a19191818,
                0x1f1f1e1e1d1d1c1c,
                0x2323222221212020,
                0x2727262625252424,
                0x2b2b2a2a29292828,
                0x2f2f2e2e002d2c2c, // <-
                0x3333323231313030,
                0x3737363635353434,
                0x3b3b3a3a39393838,
                0x3f3f3e3e3d3d3c3c,
                0x4343424241410000,
                0x4747464645454444,
                0x4b4b4a4a49494848,
                0x4f4f4e4e4d4d4c4c,
                0x5353520051515050, // <-
                0x5757565655555454,
                0x5b5b5a5a59595858,
                0x5f5f5e5e5d5d5c5c,
                0x6363626261616060,
                0x6767666665656464,
                0x6b6b6a6a69696868,
                0x6f6f6e6e006d6c6c, // <-
                0x7373727271717070,
                0x7777767675757474,
                0x7b7b7a7a79797878,
                0x7f7f7e7e7d7d7c7c,
            ],
        );
        assert_eq!(exp, a & b);
        assert_eq!(exp, b & a);

        // failed reduction
        let a = smart_bitmap_from_data(
            &[6, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0b1111111111111101u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111111011111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111101111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111111111111110u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
                0b1111001111111111u64 | // <-
                    (0b1111111111111111u64 << 16) |
                    (0b1111110111111111u64 << 32) |
                    (0b1111111111111111u64 << 48),
                0b1111111111111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111101111111u64 << 32)
                    | (0b1111111111001111u64 << 48), // <-
                0b1111111101111111u64
                    | (0b1111111111111111u64 << 16)
                    | (0b1111111111111111u64 << 32)
                    | (0b1111111111111111u64 << 48),
            ],
        );
        let b = smart_bitmap_from_data(
            &[10, 11, 15, 17, 20, 22, 23, 25, 29, 33, 36],
            &[
                0x0303020201010000,
                0x0707060605050404,
                0x0b0b0a0a09090808,
                0x0f0f0e0e0d0d0c0c,
                0x1313120011111010, // <-
                0x1717161615151414,
                0x1b1b1a6619191818, // <- doesn't match
                0x1f1f1e1e1d1d1c1c,
                0x2323222221212020,
                0x2727262625252424,
                0x2b2b2a2a29292828,
                0x2f2f2e2e002d2c2c, // <-
                0x3333323231313030,
                0x3737363635353434,
                0x3b3b3a3a39393838,
                0x3f3f3e3e3d3d3c3c,
                0x4343424241410000,
                0x4747464645454444,
                0x4b4b4a4a49494848,
                0x4f4f4e4e4d4d4c4c,
                0x5353520051515050, // <-
                0x5757565655555454,
                0x5b5b5a5a59595858,
                0x5f5f5e5e5d5d5c5c,
                0x6363626261616060,
                0x6767666665656464,
                0x6b6b6a6a22696868, // <- doesn't match
                0x6f6f6e6e006d6c6c, // <-
                0x7373727271717070,
                0x7777767675757474,
                0x7b7b7a7a79797878,
                0x7f7f7e7e7d7d7c7c,
            ],
        );
        assert_eq!(None, a & b);
        assert_eq!(None, b & a);
    }

    #[test]
    fn test_smart_bitmap_bitxor() {
        assert_eq!(
            Some(smart_bitmap_from_data::<usize>(&[], &[1])),
            smart_bitmap_from_data::<usize>(&[], &[0]) ^ smart_bitmap_from_data::<usize>(&[], &[1])
        );
        assert_eq!(
            Some(smart_bitmap_from_data::<usize>(&[], &[0])),
            smart_bitmap_from_data::<usize>(&[], &[1]) ^ smart_bitmap_from_data::<usize>(&[], &[1])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[3, 6, 9, 11], &[0x2387])),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b])
                ^ smart_bitmap_from_data(&[3, 6, 9, 11], &[0x3dec])
        );
        assert_eq!(
            Some(smart_bitmap_from_data(&[3, 6, 9], &[0x54])),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b])
                ^ smart_bitmap_from_data(&[3, 6, 9, 11], &[0x4a3f])
        );
    }

    #[test]
    fn test_smart_bitmap_split() {
        assert_eq!(
            (
                smart_bitmap_from_data(&[3, 6, 9], &[0x6b]),
                smart_bitmap_from_data(&[3, 6, 9], &[0x1e])
            ),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b]).split()
        );
        assert_eq!(
            (
                smart_bitmap_from_data(&[3, 6, 9, 11, 12], &[0x55667788]),
                smart_bitmap_from_data(&[3, 6, 9, 11, 12], &[0x11223344])
            ),
            smart_bitmap_from_data(&[3, 6, 9, 11, 12, 14], &[0x1122334455667788]).split()
        );
        assert_eq!(
            (
                smart_bitmap_from_data(
                    &[3, 6, 9, 11, 12, 14, 17, 18],
                    &[
                        0xe300c0009c100020,
                        0x1010420184104021,
                        0xeacc001199414211,
                        0x0a000510a03b20c0,
                    ]
                ),
                smart_bitmap_from_data(
                    &[3, 6, 9, 11, 12, 14, 17, 18],
                    &[
                        0xbc0a04502a784113,
                        0xba03502525770492,
                        0xa0a0549488990490,
                        0xaab0c03af031578d
                    ]
                )
            ),
            smart_bitmap_from_data(
                &[3, 6, 9, 11, 12, 14, 17, 18, 20],
                &[
                    0xe300c0009c100020,
                    0x1010420184104021,
                    0xeacc001199414211,
                    0x0a000510a03b20c0,
                    0xbc0a04502a784113,
                    0xba03502525770492,
                    0xa0a0549488990490,
                    0xaab0c03af031578d
                ]
            )
            .split()
        );
    }

    #[test]
    fn test_smart_bitmap_join() {
        assert_eq!(
            smart_bitmap_from_data(&[3, 6, 9, 13], &[0x1e6b]),
            smart_bitmap_from_data(&[3, 6, 9], &[0x6b])
                .join(smart_bitmap_from_data(&[3, 6, 9], &[0x1e]), 13)
        );
        assert_eq!(
            smart_bitmap_from_data(&[3, 6, 9, 11, 12, 17], &[0x1122334455667788]),
            smart_bitmap_from_data(&[3, 6, 9, 11, 12], &[0x55667788]).join(
                smart_bitmap_from_data(&[3, 6, 9, 11, 12], &[0x11223344]),
                17
            )
        );
        assert_eq!(
            smart_bitmap_from_data(
                &[3, 6, 9, 11, 12, 14, 17, 18, 20],
                &[
                    0xe300c0009c100020,
                    0x1010420184104021,
                    0xeacc001199414211,
                    0x0a000510a03b20c0,
                    0xbc0a04502a784113,
                    0xba03502525770492,
                    0xa0a0549488990490,
                    0xaab0c03af031578d
                ]
            ),
            smart_bitmap_from_data(
                &[3, 6, 9, 11, 12, 14, 17, 18],
                &[
                    0xe300c0009c100020,
                    0x1010420184104021,
                    0xeacc001199414211,
                    0x0a000510a03b20c0,
                ]
            )
            .join(
                smart_bitmap_from_data(
                    &[3, 6, 9, 11, 12, 14, 17, 18],
                    &[
                        0xbc0a04502a784113,
                        0xba03502525770492,
                        0xa0a0549488990490,
                        0xaab0c03af031578d
                    ]
                ),
                20
            ),
        );
    }

    #[test]
    fn test_smart_all_values() {
        //value eq
        assert!(
            !(SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                .value_eq(&SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<
                    usize,
                >(
                    &[], &[1]
                )))))
        );
        assert!(
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1]))).value_eq(
                &SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1])))
            )
        );
        assert!(
            !(SmartAllValues::Unknown.value_eq(&SmartAllValues::Bitmap(Box::new(
                smart_bitmap_from_data::<usize>(&[], &[1])
            ))))
        );
        assert!(
            !(SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                .value_eq(&SmartAllValues::Unknown))
        );
        assert!(!(SmartAllValues::<usize>::Unknown.value_eq(&SmartAllValues::Unknown)));

        // Not
        assert_eq!(SmartAllValues::<usize>::Unknown, !SmartAllValues::Unknown);
        assert_eq!(
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1]))),
            !SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
        );

        // BitAnd
        assert_eq!(
            SmartAllValues::<usize>::Unknown,
            SmartAllValues::Unknown & SmartAllValues::Unknown
        );
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                & SmartAllValues::Unknown
        );
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Unknown
                & SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
        );
        assert_eq!(
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0]))),
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                & SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1])))
        );
        // if bitand returns None
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data(
                &[2, 5, 7, 9, 11, 14, 15, 18],
                &[
                    0xbc0a04502a78,
                    0xba0350252577,
                    0xa0a054948899,
                    0xaab0c03af0315
                ]
            ))) & SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data(
                &[3, 4, 6, 10, 12, 13, 16],
                &[0xbc11466aaa22, 0xba5bb0c22577]
            )))
        );

        // BitXor
        assert_eq!(
            SmartAllValues::<usize>::Unknown,
            SmartAllValues::Unknown ^ SmartAllValues::Unknown
        );
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                ^ SmartAllValues::Unknown
        );
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Unknown
                ^ SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
        );
        assert_eq!(
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1]))),
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[0])))
                ^ SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data::<usize>(&[], &[1])))
        );
        // if bitxor returns None
        assert_eq!(
            SmartAllValues::Unknown,
            SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data(
                &[2, 5, 7, 9, 11, 14, 15, 18],
                &[
                    0xbc0a04502a78,
                    0xba0350252577,
                    0xa0a054948899,
                    0xaab0c03af0315
                ]
            ))) ^ SmartAllValues::Bitmap(Box::new(smart_bitmap_from_data(
                &[3, 4, 6, 10, 12, 13, 16],
                &[0xbc11466aaa22, 0xba5bb0c22577]
            )))
        );
    }
}
