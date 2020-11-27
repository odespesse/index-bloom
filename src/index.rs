use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::bloom_filter::BloomFilter;
use crate::tokens::Tokens;
use crate::errors::Error;

/// An full-text search index.
#[derive(Serialize, Deserialize)]
pub struct Index {
    error_rate: f32,
    bloom_filters: HashMap<String, BloomFilter>
}

impl Index {
    /// Constructs a new, empty `Index` with the specified error_rate.
    ///
    /// The `error_rate` is the probability of false positive when searching for keywords
    ///
    /// # Example
    ///
    /// ```
    /// # use index_bloom::Index;
    /// let mut index = Index::new(0.00001);
    /// ```
    pub fn new(error_rate: f32) -> Self {
        Index {
            error_rate,
            bloom_filters: HashMap::new()
        }
    }

    /// Restore an `Index` from a previous dump.
    ///
    /// A dump is an `Index` serialized in JSON format.
    ///
    /// # Panics
    ///
    /// Panics if the content is not a valid `Index` representation.
    ///
    /// # Example
    ///
    /// ```
    /// # use index_bloom::Index;
    /// let index_dump = "{\"error_rate\":0.1,\"bloom_filters\":{\"file1.txt\":{\"key_size\":4,\"bitfield\":[8,130,65,18,131,164],\"bitfield_size\":48}}}";
    /// let index = Index::restore(&index_dump);
    /// ```
    pub fn restore(content: &str) -> Self {
        let deserialized: Index = serde_json::from_str(&content).expect("Unable to parse dump file");
        return deserialized;
    }

    /// Ingest a new document.
    ///
    /// Insert each word of `content` in the index and identifies them under the given `name`.
    /// To ingest the same key twice will replace its content in the `Index`.
    ///
    /// # Errors
    ///
    /// If a word in the content cannot be hashed then an error is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use index_bloom::Index;
    /// # use index_bloom::Error;
    /// # fn search_index() -> Result<(), Error> {
    /// let mut index = Index::new(0.00001);
    /// let first_content = "A very very long content...";
    /// index.ingest("foo".to_string(), first_content)?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Search keywords in every documents.
    ///
    /// Splits `keywords` and searches for each word in all documents with a boolean AND.
    /// The result may contain false positives (documents not containing all the keywords) according to an error rate set at the creation of the `Index` (see [`Index::new`]).
    ///
    /// # Errors
    ///
    /// If a word in the content cannot be hashed then an error is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use index_bloom::Index;
    /// # use index_bloom::Error;
    /// # fn search_index() -> Result<(), Error> {
    /// # let mut index = Index::new(0.00001);
    /// let hits = index.search("content")?;
    /// match hits {
    ///      Some(documents) => {
    ///          for doc in documents {
    ///             println!("Found at {}", doc);
    ///          }
    ///      },
    ///      None => println!("Not found")
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

