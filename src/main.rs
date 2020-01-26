use crypto::digest::Digest;
use crypto::blake2b::Blake2b;

fn main() {
    let mut bloom = BloomFilter::new(1000, 0.1);
    bloom.insert("hello");
    assert!(bloom.contains("hello"));
}

struct BloomFilter {
    key_size: u32,
    bitfield: Vec<bool>
}

impl BloomFilter {
    fn new(capacity: u32, err_rate: f32) -> Self {
        if capacity == 0 {
            panic!("Invalid Bloom filter capacity: cannot be 0");
        }
        let factor = (1.0/2.0_f32.powf(2.0_f32.ln())).ln();
        let bitfield_size = ((capacity as f32 * err_rate.ln()) / factor).ceil();
        let key_size = ((bitfield_size / capacity as f32) * 2.0_f32.ln()).ceil();
        let mut bitfield = Vec::with_capacity(bitfield_size as usize);
        for _ in 0..(bitfield_size as usize) {
            bitfield.push(false);
        }
        BloomFilter {
            key_size: key_size as u32,
            bitfield
        }
    }

    fn insert(&mut self, key: &str) {
        let positions = self.hash_word(key, self.key_size, self.bitfield.len());
        for position in positions {
            self.bitfield[position] = true
        }
    }

    fn contains(&self, key: &str) -> bool {
        unimplemented!();
    }

    fn hash_word(&self, key: &str, key_size: u32, bitfield_size: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut keys_buffer = vec![key.to_string()];
        for _ in 0..key_size {
            let mut hasher = Blake2b::new(8);
            let k = keys_buffer.join("");
            hasher.input_str(&k);
            let digest = hasher.result_str();
            let position = usize::from_str_radix(&digest, 16).unwrap() % bitfield_size;
            keys_buffer.push(key.to_string());
            result.push(position);
        }
        result
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
        filter.insert("hello");
        assert_eq!(vec![false, true, false, false, false, false, true, false, false, false], filter.bitfield);
        filter.insert("world");
        assert_eq!(vec![true, true, false, true, false, false, true, true, true, false], filter.bitfield);
    }
}
