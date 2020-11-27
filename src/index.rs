use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::bloom_filter::BloomFilter;
use crate::tokens::Tokens;
use crate::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct Index {
    error_rate: f32,
    bloom_filters: HashMap<String, BloomFilter>
}

impl Index {
    pub fn new(error_rate: f32) -> Self {
        Index {
            error_rate,
            bloom_filters: HashMap::new()
        }
    }

    pub fn restore(content: &str) -> Self {
        let deserialized: Index = serde_json::from_str(&content).expect("Unable to parse dump file");
        return deserialized;
    }

    pub fn ingest(&mut self, name: String, content: &str) -> Result<(), Error> {
        let tokens_agg = self.aggregate_tokens(content);
        let capacity = tokens_agg.len();
        let mut filter = BloomFilter::new(capacity, self.error_rate);
        for token in tokens_agg {
            filter.insert(&token)?;
        }
        self.bloom_filters.insert(name, filter);
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

    fn aggregate_tokens(&self, content: &str) -> HashSet<String> {
        let mut unique_tokens = HashSet::new();
        for line in content.lines() {
            let tokens = Tokens::new(line);
            for token in tokens {
                unique_tokens.insert(token);
            }
        }
        unique_tokens
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn simple_content() {
        let mut index = Index::new(0.01);
        let content = "word1 word2\nword3\n\nword4";
        index.ingest("simple_content.txt".to_string(), content).expect("Unable to ingest data");
        assert!(index.search("word1").is_ok());
        assert_eq!(vec!["simple_content.txt"], index.search("word1").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word2").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word3").unwrap().unwrap());
        assert_eq!(vec!["simple_content.txt"], index.search("word4").unwrap().unwrap());
        assert_eq!(None, index.search("").unwrap());
    }

    #[test]
    fn several_matches() {
        let mut index = Index::new(0.01);
        index.ingest("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to ingest data");
        index.ingest("file2.txt".to_string(), "word1 word3").expect("Unable to ingest data");
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap().unwrap());
        let expected = vec!["file1.txt", "file2.txt"];
        assert_eq!(expected, index.search("word1").unwrap().unwrap());
        assert_eq!(expected, index.search("word3").unwrap().unwrap());
    }

    #[test]
    fn ingesting_twice_replace() {
        let mut index = Index::new(0.01);
        index.ingest("file1.txt".to_string(), "word1").expect("Unable to ingest data");
        assert_eq!(vec!["file1.txt"], index.search("word1").unwrap().unwrap());
        assert_eq!(None, index.search("word2").unwrap());
        index.ingest("file1.txt".to_string(), "word2").expect("Unable to ingest data");
        assert_eq!(None, index.search("word1").unwrap());
        assert_eq!(vec!["file1.txt"], index.search("word2").unwrap().unwrap());
    }

    #[test]
    fn multi_keywords_search() {
        let mut index = Index::new(0.01);
        index.ingest("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to ingest data");
        assert_eq!(vec!["file1.txt"], index.search("word1 word2").unwrap().unwrap());
    }

    #[test]
    fn clean_keywords_before_search() {
        let mut index = Index::new(0.01);
        index.ingest("file1.txt".to_string(), "word1 word2\nword3").expect("Unable to ingest data");
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

