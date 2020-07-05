use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::indexer::bloom_filter::BloomFilter;
use crate::indexer::tokens::Tokens;

#[derive(Serialize, Deserialize)]
pub struct Index {
    capacity: u32,
    error_rate: f32,
    bloom_filters: HashMap<PathBuf, BloomFilter>
}

impl Index {
    pub fn new() -> Self {
        Index {
            capacity: 1000,
            error_rate: 0.1,
            bloom_filters: HashMap::new()
        }
    }

    pub fn index(&mut self, source: &str) {
        let src_path = PathBuf::from(source);
        if src_path.is_file() {
            self.index_file(src_path);
        } else if src_path.is_dir() {
            self.index_directory(src_path);
        } else {
            panic!("source type must be file or directory");
        }
    }

    pub fn search(&self, keywords: &str) -> Option<Vec<&PathBuf>> {
        let mut result :Vec<&PathBuf> = Vec::new();
        for (path, filter) in &self.bloom_filters {
            let mut tokens = Tokens::new(keywords);
            if  tokens.all(|token| filter.contains(&token) ) {
                result.push(path);
            }
        }
        return if result.len() > 0 {
            result.sort();
            Some(result)
        } else {
            None
        }
    }

    pub fn restore(path :&str) -> Self {
        if Path::new(path).is_file() {
            let serialized = std::fs::read_to_string(path).unwrap();
            let deserialized: Index = serde_json::from_str(&serialized).unwrap();
            deserialized
        } else {
            panic!("file not found");
        }
    }

    pub fn dump(&self, path: &str) {
        let dest = Path::new(&path);
        let mut output_file = File::create(dest).unwrap();
        let serialized = serde_json::to_string(&self).unwrap();
        write!(output_file, "{}\n", serialized).unwrap();
    }

    fn index_directory(&mut self, path: PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                self.index_file(path);
            }
        }
    }

    fn index_file(&mut self, path: PathBuf) {
        let mut content = String::new();
        let mut file = File::open(&path).unwrap();
        match file.read_to_string(&mut content) {
            Ok(_) => {
                let mut filter = BloomFilter::new(self.capacity, self.error_rate);
                for line in content.lines() {
                    self.index_sentence(line, &mut filter);
                }
                self.bloom_filters.insert(path, filter);
            },
            Err(_) => eprintln!("Error reading file")
        }
    }

    fn index_sentence(&mut self, words: &str, filter: &mut BloomFilter) {
        let tokens = Tokens::new(words);
        for token in tokens {
            filter.insert(&token);
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn white_space() {
        let mut index = Index::new();
        let mut filter = BloomFilter::new(1000, 0.1);
        index.index_sentence("Word1 word2, word3. Word4!", &mut filter);
        assert!(filter.contains("word1"));
        assert!(filter.contains("word2"));
        assert!(filter.contains("word3"));
        assert!(filter.contains("word4"));
    }

    #[test]
    fn index_source_is_file() {
        let mut index = Index::new();
        index.index("./test/data/simple_content.txt");
        assert_eq!(vec![Path::new("./test/data/simple_content.txt")], index.search("word1").unwrap());
    }

    #[test]
    fn index_source_is_directory() {
        let mut index = Index::new();
        index.index("./test/data/simple_directory");
        assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word1").unwrap());
        assert_eq!(vec![Path::new("./test/data/simple_directory/file2.txt")], index.search("word4").unwrap());
    }

    #[test]
    #[should_panic(expected="source type must be file or directory")]
    fn index_source_is_unsupported() {
        let mut index = Index::new();
        index.index("./test/unknown_source");
    }

    #[test]
    fn file_simple_content() {
        let mut index = Index::new();
        index.index_file(PathBuf::from("./test/data/simple_content.txt"));
        assert_eq!(vec![Path::new("./test/data/simple_content.txt")], index.search("word1").unwrap());
        assert_eq!(vec![Path::new("./test/data/simple_content.txt")], index.search("word2").unwrap());
        assert_eq!(vec![Path::new("./test/data/simple_content.txt")], index.search("word3").unwrap());
        assert_eq!(vec![Path::new("./test/data/simple_content.txt")], index.search("word4").unwrap());
    }

    #[test]
    fn simple_directory_content() {
       let mut index = Index::new();
       index.index_directory(PathBuf::from("./test/data/simple_directory"));
       assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word1").unwrap());
       assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word2").unwrap());
       assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word3").unwrap());
       assert_eq!(vec![Path::new("./test/data/simple_directory/file2.txt")], index.search("word4").unwrap());
       assert_eq!(vec![Path::new("./test/data/simple_directory/file2.txt")], index.search("word5").unwrap());
    }

    #[test]
    fn random_directory_content() {
        let mut index = Index::new();
        index.index_directory(PathBuf::from("./test/data/random_directory"));
        assert_eq!(vec![Path::new("./test/data/random_directory/file1.txt")], index.search("word1").unwrap());
        assert_eq!(vec![Path::new("./test/data/random_directory/file1.txt")], index.search("word2").unwrap());
        assert_eq!(vec![Path::new("./test/data/random_directory/file1.txt")], index.search("word3").unwrap());
        assert_eq!(None, index.search("word4"));
        assert_eq!(None, index.search("word5"));
    }

    #[test]
    fn several_matches() {
        let mut index = Index::new();
        index.index_directory(PathBuf::from("./test/data/several_matches_directory"));
        let expected = vec![Path::new("./test/data/several_matches_directory/file1.txt")];
        assert_eq!(expected, index.search("word2").unwrap());
        let expected = vec![Path::new("./test/data/several_matches_directory/file1.txt"), Path::new("./test/data/several_matches_directory/file2.txt")];
        assert_eq!(index.search("word1").unwrap(), expected);
        assert_eq!(index.search("word3").unwrap(), expected);
    }

    #[test]
    fn multi_keywords_search() {
        let mut index = Index::new();
        index.index_directory(PathBuf::from("./test/data/several_matches_directory"));
        let expected = vec![Path::new("./test/data/several_matches_directory/file1.txt")];
        assert_eq!(expected, index.search("word1 word2").unwrap());
    }

    #[test]
    fn clean_keywords_before_search() {
        let mut index = Index::new();
        index.index_directory(PathBuf::from("./test/data/simple_directory"));
        let expected = vec![Path::new("./test/data/simple_directory/file1.txt")];
        assert_eq!(index.search("(word1) Word2, word3?").unwrap(), expected);
    }

    #[test]
    fn restore_index() {
        let index = Index::restore("./test/data/simple_dump.json");
        let expected = vec![Path::new("./test/data/simple_directory/file1.txt")];
        assert_eq!(index.search("(word1) Word2, word3?").unwrap(), expected);
    }

    #[test]
    fn dump_index() {
        let mut index = Index {
            capacity: 5,
            error_rate: 0.1,
            bloom_filters: HashMap::new()
        };
        index.index("./test/data/simple_content.txt");
        let mut dest_file = std::env::temp_dir();
        dest_file.push("bloom_dump.json");
        index.dump(dest_file.as_path().to_str().unwrap());
        let expected = "{\"capacity\":5,\"error_rate\":0.1,\"bloom_filters\":{\"./test/data/simple_content.txt\":{\"key_size\":4,\"bitfield\":[true,false,false,true,false,true,true,true,true,true,true,false,true,false,false,false,true,false,false,false,false,true,false,false]}}}\n";
        let actual = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(actual, expected);
        fs::remove_file(dest_file).unwrap();
    }
}
