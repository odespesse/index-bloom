use std::collections::HashMap;

use crate::bloom_filter::BloomFilter;
use crate::tokens::Tokens;

pub struct Index {
    capacity: u32,
    error_rate: f32,
    bloom_filters: HashMap<String, BloomFilter>
}

impl Index {
    pub fn new() -> Self {
        Index {
            capacity: 1000,
            error_rate: 0.1,
            bloom_filters: HashMap::new()
        }
    }

    pub fn index(&mut self, name: String, content: &str) {
        let filter = self.bloom_filters.entry(name).or_insert(BloomFilter::new(self.capacity, self.error_rate));
        for line in content.lines() {
            let tokens = Tokens::new(line);
            for token in tokens {
                filter.insert(&token);
            }
        }
    }

    pub fn search(&self, keywords: &str) -> Option<Vec<&String>> {
        let mut result :Vec<&String> = Vec::new();
        for (name, filter) in &self.bloom_filters {
            let tokens = Tokens::new(keywords);
            let mut all_tokens_match = true;
            let mut in_loop = false;
            for token in tokens {
                in_loop = true;
                if !filter.contains(&token) {
                    all_tokens_match = false;
                    break;
                }
            }
            if in_loop && all_tokens_match {
                result.push(name);
            }
        }
        return if result.len() > 0 {
            result.sort();
            Some(result)
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_content() {
        let mut index = Index::new();
        let content = "word1 word2\nword3\n\nword4";
        index.index("simple_content.txt".to_string(), content);
        assert_eq!(vec!["simple_content.txt"], index.search("word1").unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word2").unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word3").unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word4").unwrap());
        assert_eq!(None, index.search(""));
    }

    #[test]
    fn several_matches() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3");
        index.index("file2.txt".to_string(), "word1 word3");
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap());
        let expected = vec!["file1.txt", "file2.txt"];
        assert_eq!(expected, index.search("word1").unwrap());
        assert_eq!(expected, index.search("word3").unwrap());
    }

    #[test]
    fn two_steps_indexing() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1");
        assert_eq!(vec!["file1.txt"], index.search("word1").unwrap());
        assert_eq!(None, index.search("word2"));
        index.index("file1.txt".to_string(), "word2");
        assert_eq!(vec!["file1.txt"], index.search("word1").unwrap());
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap());
    }

    #[test]
    fn multi_keywords_search() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3");
        assert_eq!(vec!["file1.txt"], index.search("word1 word2").unwrap());
    }

    #[test]
    fn clean_keywords_before_search() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3");
        assert_eq!(vec!["file1.txt"], index.search("(word1) Word2, word3?").unwrap());
    }
}

