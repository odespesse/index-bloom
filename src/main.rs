mod bloom_filter;

use crate::bloom_filter::BloomFilter;

fn main() {
    let mut bloom = BloomFilter::new(1000, 0.1);
    bloom.insert("hello");
    assert!(bloom.contains("hello"));
    assert!(!bloom.contains("bar"));
}
