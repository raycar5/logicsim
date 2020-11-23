use smallvec::SmallVec;
pub fn word_mask_64(index: usize) -> (usize, u64) {
    // This is safe because the divisor is a non zero constant.
    // Was 3.14% in the flame graph before unsafe, 2.7% after unsafe.
    let word = unsafe { std::intrinsics::unchecked_div(index, 64) };
    // This is safe because the divisor is a non zero constant and the shift can't be more than 64;
    let mask = unsafe {
        std::intrinsics::unchecked_shl(1u64, std::intrinsics::unchecked_rem(index, 64) as u64)
    };
    (word, mask)
}
pub fn word_mask_8(index: usize) -> (usize, u8) {
    let word = index / 8;
    let mask = 1 << (index % 8);
    (word, mask)
}

#[derive(Debug)]
pub struct BitIter {
    item: SmallVec<[u8; 8]>,
    i: u8,
}
impl BitIter {
    pub fn new<T: Copy>(item: T) -> Self {
        let byte_size = std::mem::size_of::<T>();
        let bit_size = byte_size * 8;

        assert!(
            bit_size <= std::u8::MAX as usize,
            "Item too big to bit iterate, If this is ever hit change the i to u16, bit_size: {}",
            bit_size
        );

        let as_u8s: &[u8] =
            // This is safe because any Copy item can be interpreted as a slice of bytes.
            unsafe { std::slice::from_raw_parts(std::mem::transmute(&item), byte_size) };

        Self {
            item: SmallVec::from_slice(as_u8s),
            i: 0,
        }
    }
}
impl Iterator for BitIter {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.item.len() as u8 * 8 {
            return None;
        }

        let (word_index, word_mask) = word_mask_8(self.i as usize);

        let result = self.item[word_index] & word_mask != 0;
        self.i = self.i + 1;

        Some(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let n = 0b101u8;
        let result = [true, false, true];
        let mut iterations = 0;
        for (i, set) in BitIter::new(n).enumerate() {
            assert_eq!(set, *result.get(i).unwrap_or(&false));
            iterations = iterations + 1;
        }
        assert_eq!(iterations, std::mem::size_of_val(&n) * 8);
    }
    #[test]
    fn test_u128() {
        let n = 0b110u128;
        let result = [false, true, true];
        let mut iterations = 0;
        for (i, set) in BitIter::new(n).enumerate() {
            assert_eq!(set, *result.get(i).unwrap_or(&false));
            iterations = iterations + 1;
        }
        assert_eq!(iterations, std::mem::size_of_val(&n) * 8);
    }
    #[test]
    fn test_struct() {
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Example {
            a: u8,
            b: u8,
        }
        let n = Example { a: 0b1011, b: 0b1 };
        let result = [true, true, false, true, false, false, false, false, true];
        let mut iterations = 0;
        for (i, set) in BitIter::new(n).enumerate() {
            iterations = iterations + 1;
            assert_eq!(set, *result.get(i).unwrap_or(&false));
        }
        assert_eq!(iterations, std::mem::size_of_val(&n) * 8);
    }
}
