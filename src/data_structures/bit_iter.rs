use smallvec::SmallVec;

/// Returns the index and mask necessary to access the bit at `index` in a ```&[u64]```.
///
/// # Example
///
/// ```
/// # use logicsim::data_structures::word_mask_64;
/// let word_slice = [0u64, 1u64];
/// let bit_index = 64;
///
/// let (word_index, mask) = word_mask_64(bit_index);
/// let bit_set = (word_slice[word_index] & mask) != 0;
///
/// assert_eq!(bit_set, true);
/// ```
// Method was 3.14% in the flame graph before unsafe, 2.7% after unsafe.
#[cfg(feature = "logicsim_unstable")]
pub fn word_mask_64(index: usize) -> (usize, u64) {
    // This is safe because the divisor is a non zero constant.
    let word = unsafe { std::intrinsics::unchecked_div(index, 64) };
    // This is safe because the divisor is a non zero constant
    // and the right operand of the shift can't be more than 64.
    let mask = unsafe {
        std::intrinsics::unchecked_shl(1u64, std::intrinsics::unchecked_rem(index, 64) as u64)
    };
    (word, mask)
}

/// Returns the index and mask necessary to access the bit at `index` in a ```&[u64]```.
///
/// # Example
///
/// ```
/// # use logicsim::data_structures::word_mask_64;
/// let word_slice = [0u64, 1u64];
/// let bit_index = 64;
///
/// let (word_index, mask) = word_mask_64(bit_index);
/// let bit_set = (word_slice[word_index] & mask) != 0;
///
/// assert_eq!(bit_set, true);
/// ```
#[cfg(not(feature = "logicsim_unstable"))]
pub fn word_mask_64(index: usize) -> (usize, u64) {
    // This is safe because the divisor is a non zero constant.
    let word = index / 64;
    // This is safe because the divisor is a non zero constant
    // and the right operand of the shift can't be more than 64.
    let mask = 1 << (index % 64);
    (word, mask)
}

/// Returns the index and mask necessary to access the bit at `index` in a ```&[u8]```.
///
/// # Example
///
/// ```
/// # use logicsim::data_structures::word_mask_8;
/// let word_slice = [0u8, 1u8];
/// let bit_index = 8;
///
/// let (word_index, mask) = word_mask_8(bit_index);
/// let bit_set = (word_slice[word_index] & mask) != 0;
///
/// assert_eq!(bit_set, true);
/// ```
pub fn word_mask_8(index: usize) -> (usize, u8) {
    let word = index / 8;
    let mask = 1 << (index % 8);
    (word, mask)
}

/// Data structure that allows for iterating over the native endian bits of any
/// [Sized] + [Copy] + ['static](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html) value.
///
/// If you are using this data structure with structs, make sure you use a [repr](https://doc.rust-lang.org/nomicon/other-reprs.html) that is defined.
///
/// # Example
/// ```
/// # use logicsim::data_structures::BitIter;
/// let mut bits = BitIter::new(0b00100101u8);
///
/// assert_eq!(bits.next().unwrap(), true);
/// assert_eq!(bits.next().unwrap(), false);
/// assert_eq!(bits.next().unwrap(), true);
/// assert_eq!(bits.next().unwrap(), false);
/// assert_eq!(bits.next().unwrap(), false);
/// assert_eq!(bits.next().unwrap(), true);
/// assert_eq!(bits.next().unwrap(), false);
/// assert_eq!(bits.next().unwrap(), false);
///
/// assert_eq!(bits.next(), None);
///
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BitIter {
    item: SmallVec<[u8; 8]>,
    i: u16,
}
impl BitIter {
    /// Returns a new [BitIter] which will iterate over the native endian bits of `item`.
    ///
    /// # Panics
    ///
    /// Will panic if `item` is bigger than 65535 bits, if this ever happens to you, open an issue or a PR.
    /// It is an arbitrary limit I have set to keep the [BitIter] struct small.
    pub fn new<T: Copy + Sized + 'static>(item: T) -> Self {
        let byte_size = std::mem::size_of::<T>();
        let bit_size = byte_size * 8;

        assert!(
            bit_size <= std::u16::MAX as usize,
            "Item too big to bit iterate, If this is ever hit change the i to u32, bit_size: {}",
            bit_size
        );

        let as_u8s: &[u8] =
            // This is safe because any Copy + Sized + 'static item can be interpreted as a slice of bytes.
            unsafe { std::slice::from_raw_parts(std::mem::transmute(&item), byte_size) };

        Self {
            item: SmallVec::from_slice(as_u8s),
            i: 0,
        }
    }

    /// Returns true if the value used to create the [BitIter] has all its bits set to 0.
    ///
    /// # Example
    /// ```
    /// # use logicsim::data_structures::BitIter;
    /// let zero = BitIter::new(0u64);
    /// assert_eq!(zero.is_zero(), true);
    ///
    /// let non_zero = BitIter::new(32u128);
    /// assert_eq!(non_zero.is_zero(), false);
    /// ```
    pub fn is_zero(&self) -> bool {
        for byte in &self.item {
            if *byte != 0 {
                return false;
            }
        }
        true
    }
}

impl Iterator for BitIter {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.item.len() as u16 * 8 {
            return None;
        }

        let (word_index, word_mask) = word_mask_8(self.i as usize);

        let result = self.item[word_index] & word_mask != 0;
        self.i += 1;

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

    #[test]
    fn test_is_zero() {
        assert_eq!(BitIter::new(8).is_zero(), false);
        assert_eq!(BitIter::new(0u8).is_zero(), true);
        assert_eq!(BitIter::new(0i16).is_zero(), true);
        assert_eq!(BitIter::new(0f32).is_zero(), true);
        assert_eq!(BitIter::new(12.2f64).is_zero(), false);
        assert_eq!(BitIter::new(-0f64).is_zero(), false);
    }
}
