use std::str;
use std::cell::RefCell;
use blake2::VarBlake2b;
use blake2::digest::{Update, VariableOutput};
use serde::{Serialize, Deserialize};
use bitvec::prelude::*;
use crate::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct BloomFilter {
    key_size: u32,
    bitfield: BitVec
}

impl BloomFilter {
    pub fn new(capacity: u32, err_rate: f32) -> Self {
        if capacity == 0 {
            panic!("Invalid Bloom filter capacity: cannot be 0");
        }
        let factor = (1.0/2.0_f32.powf(2.0_f32.ln())).ln();
        let bitfield_size = ((capacity as f32 * err_rate.ln()) / factor).ceil();
        let key_size = ((bitfield_size / capacity as f32) * 2.0_f32.ln()).ceil();
        let mut bitfield = BitVec::with_capacity(bitfield_size as usize);
        for _ in 0..(bitfield_size as usize) {
            bitfield.push(false);
        }
        BloomFilter {
            key_size: key_size as u32,
            bitfield
        }
    }

    pub fn insert(&mut self, key: &str) -> Result<(), Error> {
        let positions = self.hash_word(key, self.key_size, self.bitfield.len())?;
        for position in positions {
            self.bitfield.set(position, true);
        }
        Ok(())
    }

    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let positions = self.hash_word(key, self.key_size, self.bitfield.len())?;
        Ok(positions.into_iter().all(|position| self.bitfield[position] == true))
    }

    fn hash_word(&self, key: &str, key_size: u32, bitfield_size: usize) -> Result<Vec<usize>, Error> {
        let mut result = Vec::new();
        let mut keys_buffer = Vec::new();
        for _ in 0..key_size {
            keys_buffer.push(key.to_string());
            let mut hasher = VarBlake2b::new(4).unwrap();
            let k = keys_buffer.join("");
            hasher.update(&k);
            let test: RefCell<Vec<u8>> = RefCell::new(vec![]);
            hasher.finalize_variable(|digest| {
                *test.borrow_mut() = digest.to_vec();
            });
            let byte = test.into_inner().iter().map(|d| format!("{:x}", d)).collect::<Vec<String>>().join("");
            let position = match usize::from_str_radix(&byte, 16) {
                Ok(num) => num % bitfield_size,
                Err(error) => return Err(Error::HashWord(error))
            };
            result.push(position);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bloom_filter() {
        let filter = BloomFilter::new(1000, 0.1);
        for bit in filter.bitfield {
            assert!(!bit);
        }
    }

    #[test]
    fn check_bloom_filter_configuration() {
        let filter = BloomFilter::new(5, 0.1);
        assert_eq!(4, filter.key_size);
        assert_eq!(24, filter.bitfield.len());

        let filter = BloomFilter::new(100, 0.5);
        assert_eq!(2, filter.key_size);
        assert_eq!(145, filter.bitfield.len());
    }

    #[test]
    #[should_panic]
    fn check_invalid_bloom_filter_capacity() {
        BloomFilter::new(0, 1.0);
    }

    #[test]
    fn insert_new_key() {
        let mut filter = BloomFilter::new(2, 0.1);
        filter.insert("hello").expect("Unable to insert token in filter");
        assert_eq!(bitvec![1, 1, 0, 1, 0, 1, 0, 0, 0, 0], filter.bitfield);
        filter.insert("world").expect("Unable to insert token in filter");
        assert_eq!(bitvec![1, 1, 0, 1, 0, 1, 1, 0, 1, 0], filter.bitfield);
    }

    #[test]
    fn filter_contains_a_key() {
        let mut filter = BloomFilter::new(2, 0.1);
        filter.bitfield = bitvec![1, 1, 0, 1, 0, 1, 0, 0, 0, 0];
        assert!(filter.contains("hello").is_ok());
        assert!(filter.contains("hello").unwrap());
        filter.bitfield = bitvec![1, 1, 0, 1, 0, 1, 1, 0, 1, 0];
        assert!(filter.contains("world").unwrap());

        assert!(!filter.contains("foobar").unwrap());
    }
}
