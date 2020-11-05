use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::bloom_filter::BloomFilter;
use crate::tokens::Tokens;
use crate::errors::Error;

#[derive(Serialize, Deserialize)]
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

    pub fn with_params(capacity: u32, error_rate: f32) -> Self {
        Index {
            capacity,
            error_rate,
            bloom_filters: HashMap::new()
        }
    }

    pub fn restore(content: &str) -> Self {
        let deserialized: Index = serde_json::from_str(&content).expect("Unable to parse dump file");
        return deserialized;
    }

    pub fn index(&mut self, name: String, content: &str) -> Result<(), Error> {
        let filter = self.bloom_filters.entry(name).or_insert(BloomFilter::new(self.capacity, self.error_rate));
        for line in content.lines() {
            let tokens = Tokens::new(line);
            for token in tokens {
                filter.insert(&token)?;
            }
        }
        Ok(())
    }

    pub fn search(&self, keywords: &str) -> Result<Option<Vec<&String>>, Error> {
        let mut result :Vec<&String> = Vec::new();
        for (name, filter) in &self.bloom_filters {
            let tokens = Tokens::new(keywords);
            let mut all_tokens_match = true;
            let mut in_loop = false;
            for token in tokens {
                in_loop = true;
                if !filter.contains(&token)? {
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
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn simple_content() {
        let mut index = Index::new();
        let content = "word1 word2\nword3\n\nword4";
        index.index("simple_content.txt".to_string(), content).expect("Unable to index data");
        assert!(index.search("word1").is_ok());
        assert_eq!(vec!["simple_content.txt"], index.search("word1").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word2").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word3").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word4").unwrap().unwrap());
        assert_eq!(None, index.search("").unwrap());
    }

    #[test]
    fn several_matches() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to index data");
        index.index("file2.txt".to_string(), "word1 word3").expect("Unable to index data");
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap().unwrap());
        let expected = vec!["file1.txt", "file2.txt"];
        assert_eq!(expected, index.search("word1").unwrap().unwrap());
        assert_eq!(expected, index.search("word3").unwrap().unwrap());
    }

    #[test]
    fn two_steps_indexing() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1").expect("Unable to index data");
        assert_eq!(vec!["file1.txt"], index.search("word1").unwrap().unwrap());
        assert_eq!(None, index.search("word2").unwrap());
        index.index("file1.txt".to_string(), "word2").expect("Unable to index data");
        assert_eq!(vec!["file1.txt"], index.search("word1").unwrap().unwrap());
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap().unwrap());
    }

    #[test]
    fn multi_keywords_search() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to index data");
        assert_eq!(vec!["file1.txt"], index.search("word1 word2").unwrap().unwrap());
    }

    #[test]
    fn clean_keywords_before_search() {
        let mut index = Index::new();
        index.index("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to index data");
        assert_eq!(vec!["file1.txt"], index.search("(word1) Word2, word3?").unwrap().unwrap());
    }

    #[test]
    fn restore_from_str() {
        let path = "./test/data/test_restore.json";
        let index_content = fs::read_to_string(path).expect(format!("Unable to read dump file {}", &path).as_str());
        let index = Index::restore(&index_content);
        assert_eq!(vec!["file1.txt"], index.search("word1 word2 word3").unwrap().unwrap());
    }
}

