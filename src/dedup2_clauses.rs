use gatesim::*;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitXor, Not};

use crate::utils::*;

const BITMAP_BITS: usize = 2048;
const BITMAP_BITS_BITS: usize = 11;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

    fn from_iter(iter: impl IntoIterator<Item = T>) -> Self
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

enum SmartAllValues<T> {
    Unknown,
    Bitmap(SmartBitmap<T>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SmartBitmap<T> {
    // all inputs must be ordered.
    inputs: SmallVec<T, BITMAP_BITS_BITS>,
    bitmap: [u64; BITMAP_BITS >> 6],
}

impl<T> SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
{
    fn from_bool(value: bool) -> Self {
        let mut out = Self {
            inputs: SmallVec::new(),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        out.bitmap[0] = u64::from(value);
        out
    }

    fn from_input(input: T) -> Self {
        let mut out = Self {
            inputs: SmallVec::new(),
            bitmap: [0; BITMAP_BITS >> 6],
        };
        out.inputs.insert(input);
        out.bitmap[0] = 0b10;
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
        let bitlen = self.bitmap_bitlen();
        let bitmap = self.bitmap();
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

    fn apply_new_inputs(&self, self_input_num: usize, bit_start: usize, b_inputs: &[T]) -> Self {
        assert!(self_input_num + b_inputs.len() <= BITMAP_BITS_BITS);
        // create merged list of inputs
        let merged_inputs = merge_sorted_by_key(
            self.inputs.data()[0..self_input_num]
                .iter()
                .enumerate()
                .map(|(i, x)| (i, (*x, true))),
            b_inputs.iter().enumerate().map(|(i, x)| (i, (*x, false))),
            |(_, (x, _))| *x,
        );
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
        Self {
            inputs: SmallVec::from_iter(merged_inputs.into_iter().map(|(_, (x, _))| x)),
            bitmap: out_bitmap,
        }
    }

    fn make_op(self, rhs: Self, op: impl Fn(&mut [u64], &[u64], &[u64])) -> Option<Self> {
        if self.inputs.len() + rhs.inputs.len() <= BITMAP_BITS_BITS {
            let ext_self = self.apply_new_inputs(self.inputs.len() as usize, 0, rhs.inputs.data());
            let ext_rhs = rhs.apply_new_inputs(rhs.inputs.len() as usize, 0, self.inputs.data());
            let mut out = SmartBitmap {
                inputs: ext_self.inputs,
                bitmap: [0; BITMAP_BITS >> 6],
            };
            op(out.bitmap_mut(), ext_self.bitmap(), ext_rhs.bitmap());
            out.remove_unused_inputs();
            Some(out)
        } else if self.inputs.len() + rhs.inputs.len() <= BITMAP_BITS_BITS + 3 {
            let merged_inputs = merge_sorted_by_key(
                self.inputs
                    .data()
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (i, (*x, true))),
                rhs.inputs
                    .data()
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (i, (*x, false))),
                |(_, (x, _))| *x,
            );
            let merged_inputs_lasts = &merged_inputs[BITMAP_BITS_BITS..];
            let merged_inputs = &merged_inputs[0..BITMAP_BITS_BITS];
            // match self.inputs.data().binary_search(&merged_inputs.last().unwrap()) {
            //     Ok(p) = merged_
            // }
            let self_last_input_index = merged_inputs
                .iter()
                .rev()
                .find(|(_, (_, isself))| *isself)
                .map(|(i, (_, _))| *i);
            let rhs_last_input_index = merged_inputs
                .iter()
                .rev()
                .find(|(_, (_, isself))| !*isself)
                .map(|(i, (_, _))| *i);
            if self_last_input_index.is_none() || rhs_last_input_index.is_none() {
                // if inputs are not overlapping.
                // because for values where is at least one false and true (T)
                // T AND false != T AND true, T OR false != T OR true,
                // T XOR false != T XOR true then ignore such case.
                return None;
            }
            let self_next_input_index = self_last_input_index.unwrap() + 1;
            let rhs_next_input_index = rhs_last_input_index.unwrap() + 1;

            // let mut out = SmartBitmap {
            //     inputs: ext_self.inputs,
            //     bitmap: [0; BITMAP_BITS >> 6],
            // };
            let mut all_parts = vec![];
            let mut input_mask = u64::MAX;
            for i in 0..1 << merged_inputs_lasts.len() {
                // determine start bit index for bitmap
                let self_bi = merged_inputs_lasts.iter().enumerate().fold(
                    0,
                    |si, (sbit, (dbit, (_, selfbmap)))| {
                        if *selfbmap {
                            si | (((i >> sbit) & 1) << dbit)
                        } else {
                            si
                        }
                    },
                ) << self_next_input_index;
                let rhs_bi = merged_inputs_lasts.iter().enumerate().fold(
                    0,
                    |si, (sbit, (dbit, (_, selfbmap)))| {
                        if !*selfbmap {
                            si | (((i >> sbit) & 1) << dbit)
                        } else {
                            si
                        }
                    },
                ) << rhs_next_input_index;
                // generate part of bitmap to operation
                let ext_self = self.apply_new_inputs(
                    self_next_input_index,
                    self_bi,
                    &rhs.inputs.data()[0..rhs_next_input_index],
                );
                let ext_rhs = rhs.apply_new_inputs(
                    rhs_next_input_index,
                    rhs_bi,
                    &self.inputs.data()[0..self_next_input_index],
                );
                let mut out = SmartBitmap {
                    inputs: ext_self.inputs,
                    bitmap: [0; BITMAP_BITS >> 6],
                };
                // make operation
                op(out.bitmap_mut(), ext_self.bitmap(), ext_rhs.bitmap());
                input_mask &= out.check_all_unused_inputs();
                all_parts.push((out, self_bi, rhs_bi));
            }
            {
                // remove higher inputs and check
            }
            None
        } else {
            None
        }
    }
}

impl<T> Not for SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
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
        self
    }
}

impl<T> BitAnd for SmartBitmap<T>
where
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
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
    T: Default + Clone + Copy + Ord + PartialEq + Eq,
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

    fn small_vec_from_slice<T, const N: usize>(t: &[T]) -> SmallVec<T, N>
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

    fn smart_bitmap_from_data<T>(inputs: &[T], bitmap: &[u64]) -> SmartBitmap<T>
    where
        T: Default + Clone + Copy + PartialEq + Eq + Ord,
    {
        let mut bmap = SmartBitmap {
            inputs: small_vec_from_slice(inputs),
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
    fn test_apply_new_inputs() {
        assert_eq!(
            smart_bitmap_from_data(
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
            ),
            smart_bitmap_from_data(&[3, 4, 6, 9, 11, 14], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[0, 1, 5, 12]
            )
        );

        assert_eq!(
            smart_bitmap_from_data(
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
            ),
            smart_bitmap_from_data(&[3, 6, 9, 14, 17], &[0xbcda2135]).apply_new_inputs(
                5,
                0,
                &[4, 7, 8, 15, 16]
            )
        );

        assert_eq!(
            smart_bitmap_from_data(
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
            ),
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
            smart_bitmap_from_data(&[3, 4, 6, 7, 9], &[0b01011010010110101010111110101111]),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b]).apply_new_inputs(3, 0, &[4, 7])
        );

        assert_eq!(
            smart_bitmap_from_data(&[3, 4, 6, 7, 9], &[0b00000101000001011111101011111010]),
            smart_bitmap_from_data(&[3, 6, 9, 11], &[0x1e6b]).apply_new_inputs(3, 8, &[4, 7])
        );
    }
}
