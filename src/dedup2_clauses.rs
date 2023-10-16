use gatesim::*;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr, BitXor, Not};

const BITMAP_BITS: usize = 2048;
const BITMAP_BITS_BITS: usize = 11;
const FALSE_INPUT_MAXLEN: usize = 4;
const TRUE_INPUT_MAXLEN: usize = 4;

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

struct SmallVec<T, const N: usize> {
    data: [T; N],
    len: u8,
}

impl<T, const N: usize> SmallVec<T, N>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
{
    #[inline]
    fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    #[inline]
    fn data(&self) -> &[T] {
        &self.data[0..self.len as usize]
    }

    #[inline]
    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data[0..self.len as usize]
    }

    #[inline]
    fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    fn insert(&mut self, e: T) {
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
    fn remove(&mut self, e: T) {
        let p = match self.data().binary_search(&e) {
            Ok(p) => {
                let old_len = self.len as usize;
                self.data_mut().copy_within(p + 1..old_len, p);
                self.len -= 1;
            }
            Err(p) => {}
        };
    }
}

enum SmalAllValues<T> {
    Unknown,
    Bitmap(SmartBitmap<T>),
}

struct SmartBitmap<T> {
    // all inputs must be ordered.
    inputs: SmallVec<T, BITMAP_BITS_BITS>,
    // false_inputs, true_inputs - boolean value indicates where is falses and trues
    // opposite position is place where is data.
    // false and true inputs are used if no free inputs.
    false_inputs: SmallVec<(T, bool), FALSE_INPUT_MAXLEN>,
    true_inputs: SmallVec<(T, bool), TRUE_INPUT_MAXLEN>,
    bitmap: [u64; BITMAP_BITS >> 6],
}

impl<T> SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
{
    fn from_input(input: T, value: bool) -> Self {
        let mut out = Self {
            inputs: SmallVec::new(),
            false_inputs: SmallVec::new(),
            true_inputs: SmallVec::new(),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        out.inputs.insert(input);
        out.bitmap[0] = if value { 0b10 } else { 0 };
        out
    }

    #[inline]
    fn bitmap_bitlen(&self) -> usize {
        1 << self.inputs.len()
    }

    #[inline]
    fn bitmap_u64len(&self) -> usize {
        let bl = self.bitmap_bitlen();
        (bl + 63) >> 6
    }

    #[inline]
    fn bitmap(&self) -> &[u64] {
        &self.bitmap[0..self.bitmap_u64len()]
    }

    #[inline]
    fn bitmap_mut(&mut self) -> &mut [u64] {
        let l = self.bitmap_u64len();
        &mut self.bitmap[0..l]
    }

    fn remove_unused_inputs(&mut self) {
        //for i in
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
}
