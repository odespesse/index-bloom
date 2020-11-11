use std::str;
use std::cell::RefCell;
use std::convert::TryFrom;
use blake2::VarBlake2b;
use blake2::digest::{Update, VariableOutput};
use serde::{Serialize, Deserialize};
use crate::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct BloomFilter {
    key_size: u32,
    bitfield: Vec<u8>,
    bitfield_size: usize
}

impl BloomFilter {
    pub fn new(capacity: usize, err_rate: f32) -> Self {
        if capacity == 0 {
            panic!("Invalid Bloom filter capacity: cannot be 0");
        }
        let capacity_float = capacity as f32;
        let factor = (1.0/2.0_f32.powf(2.0_f32.ln())).ln();
        let bitfield_size = ((capacity_float * err_rate.ln()) / factor).ceil();
        let key_size = ((bitfield_size / capacity_float) * 2.0_f32.ln()).ceil() as u32;
        let mut bitfield = Vec::with_capacity((bitfield_size / 8.0).ceil() as usize);
        for _ in 0..(bitfield.capacity()) {
            bitfield.push(0);
        }
        BloomFilter {
            key_size,
            bitfield,
            bitfield_size: bitfield_size as usize
        }
    }

    pub fn insert(&mut self, key: &str) -> Result<(), Error> {
        let positions = self.hash_word(key)?;
        for position in positions {
            let array_index = position / 8;
            let bit_index = position % 8;
            self.bitfield[array_index] |= (2u8).pow(u32::try_from(bit_index).unwrap());
        }
        Ok(())
    }

    pub fn contains(&self, key: &str) -> Result<bool, Error> {
        let positions = self.hash_word(key)?;
        Ok(positions.into_iter().all(|position| {
            let array_index = position / 8;
            let bit_index = u8::try_from(position % 8).unwrap();
            let mask = (2u8).pow(u32::try_from(bit_index).unwrap());
            self.bitfield[array_index] & mask == mask
        }))
    }

    fn hash_word(&self, key: &str) -> Result<Vec<usize>, Error> {
        let mut result = Vec::new();
        let mut keys_buffer = Vec::new();
        for _ in 0..self.key_size {
            keys_buffer.push(key.to_string());
            let mut hasher = VarBlake2b::new(4).unwrap();
            let k = keys_buffer.join("");
            hasher.update(&k);
            let digest_vec: RefCell<Vec<u8>> = RefCell::new(vec![]);
            hasher.finalize_variable(|digest| {
                *digest_vec.borrow_mut() = digest.to_vec();
            });
            let byte = digest_vec.into_inner().iter().map(|d| format!("{:x}", d)).collect::<Vec<String>>().join("");
            let position = match usize::from_str_radix(&byte, 16) {
                Ok(num) => num % self.bitfield_size,
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
            assert_eq!(0, bit);
        }
    }

    #[test]
    fn check_bloom_filter_configuration() {
        let filter = BloomFilter::new(5, 0.1);
        assert_eq!(4, filter.key_size);
        assert_eq!(24, filter.bitfield_size);
        assert_eq!(3, filter.bitfield.len());

        let filter = BloomFilter::new(100, 0.5);
        assert_eq!(2, filter.key_size);
        assert_eq!(145, filter.bitfield_size);
        assert_eq!(19, filter.bitfield.len());
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
        assert_eq!(vec![43, 0], filter.bitfield);
        filter.insert("world").expect("Unable to insert token in filter");
        assert_eq!(vec![107, 1], filter.bitfield);
    }

    #[test]
    fn filter_contains_a_key() {
        let mut filter = BloomFilter::new(2, 0.1);
        filter.bitfield = vec![43, 0];
        assert!(filter.contains("hello").is_ok());
        assert!(filter.contains("hello").unwrap());
        filter.bitfield = vec![107, 1];
        assert!(filter.contains("world").unwrap());
        assert!(!filter.contains("foobar").unwrap());
    }
}
