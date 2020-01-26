fn main() {
    let bloom = BloomFilter::new(1000, 0.1);
    bloom.insert("hello");
    assert!(bloom.contains("hello"));
}

struct BloomFilter {
    key_size: u32,
    bitfield: Vec<bool>
}

impl BloomFilter {
    fn new(capacity: u32, err_rate: f32) -> Self {
        let factor = (1.0/2.0_f32.powf(2.0_f32.ln())).ln();
        let bitfield_size = ((capacity as f32 * err_rate.ln()) / factor).ceil();
        let key_size = ((bitfield_size / capacity as f32) * 2.0_f32.ln()).ceil();
        BloomFilter {
            key_size: key_size as u32,
            bitfield: Vec::with_capacity(bitfield_size as usize)
        }
    }

    fn insert(&self, key: &str) {
        unimplemented!();
    }

    fn contains(&self, key: &str) -> bool {
        unimplemented!();
    }

    fn hash_word(&self, key: &str, key_size: u32, bitfield_size: usize) -> Vec<usize> {
        let mut result = Vec::new();
        for i in 0..key_size {
            let position = 0;
            result.push(position);
        }
        result
    }
}
