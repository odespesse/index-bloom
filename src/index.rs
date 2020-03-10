use std::fs::File;
use std::io::Read;
use std::fs;
use std::collections::HashMap;

use crate::bloom_filter::BloomFilter;
use std::path::PathBuf;

pub struct Index {
    bloom_filters: HashMap<PathBuf, BloomFilter>
}

impl Index {
    pub fn new() -> Self {
        Index{
            bloom_filters: HashMap::new()
        }
    }

    pub fn search(&self, keywords: &str) -> Option<&PathBuf> {
        for (path, filter) in &self.bloom_filters {
            if filter.contains(keywords) {
                return Some(path);
            }
        }
        return None;
    }

    fn index_sentence(&mut self, words: &str, filter: &mut BloomFilter) {
        let words_list = words
            .split_whitespace()
            .map(|word| word.replace(".", "")
                            .replace("!", "")
                            .replace("?", "")
                            .replace(",", "")
                            .replace(";", "")
                            .replace(":", "")
                            .replace("/", "")
                            .replace("&", "")
                            .replace("#", "")
                            .replace("*", "")
                            .replace("_", "")
                            .replace("(", "")
                            .replace(")", "")
                            .replace("[", "")
                            .replace("]", "")
                            .replace("{", "")
                            .replace("}", "")
                            .replace("<", "")
                            .replace(">", "")
                            .replace("'", "")
                            .replace("`", "")
                            .replace("\"", ""))
            .filter(|word| !word.is_empty());
        for word in words_list {
            filter.insert(&word);
        }
    }

    fn index_file(&mut self, path: PathBuf) {
        let mut content = String::new();
        let mut file = File::open(&path).unwrap();
        match file.read_to_string(&mut content) {
            Ok(_) => {
                let mut filter = BloomFilter::new(1000, 0.1);
                for line in content.lines() {
                    self.index_sentence(line, &mut filter);
                }
                self.bloom_filters.insert(path, filter);
            },
            Err(_) => eprintln!("Error reading file")
        }
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn white_space() {
        let mut index = Index::new();
        let mut filter = BloomFilter::new(1000, 0.1);
        index.index_sentence("word1 word2  word3  word4\nword5\tword6", &mut filter);
        assert!(filter.contains("word1"));
        assert!(filter.contains("word2"));
        assert!(filter.contains("word3"));
        assert!(filter.contains("word4"));
        assert!(filter.contains("word5"));
        assert!(filter.contains("word6"));
    }

    #[test]
    fn punctuation() {
        let mut index = Index::new();
        let mut filter = BloomFilter::new(1000, 0.1);
        let sentence = [
            "word1.",
            "word2!",
            "word3?",
            "word4,",
            "word5;",
            "word6:",
            "word7/",
            "word8&",
            "(word9)",
            "[word10]",
            "{word11}",
            "'word12'",
            "\"word13\"",
            "<word14>",
            "`word15`",
            "*word16*",
            "__word17__",
            "?",
            "#"
        ].join(" ");
        index.index_sentence(&sentence, &mut filter);
        assert!(filter.contains("word1"));
        assert!(filter.contains("word2"));
        assert!(filter.contains("word3"));
        assert!(filter.contains("word4"));
        assert!(filter.contains("word5"));
        assert!(filter.contains("word6"));
        assert!(filter.contains("word7"));
        assert!(filter.contains("word8"));
        assert!(filter.contains("word9"));
        assert!(filter.contains("word10"));
        assert!(filter.contains("word11"));
        assert!(filter.contains("word12"));
        assert!(filter.contains("word13"));
        assert!(filter.contains("word14"));
        assert!(filter.contains("word15"));
        assert!(filter.contains("word16"));
        assert!(filter.contains("word17"));
        assert!(!filter.contains("?"));
        assert!(!filter.contains("#"));
    }

    #[test]
    fn file_simple_content() {
        let mut index = Index::new();
        index.index_file(PathBuf::from("./test/data/simple_content.txt"));
        assert_eq!(Path::new("./test/data/simple_content.txt"), index.search("word1").unwrap().as_path());
        assert_eq!(Path::new("./test/data/simple_content.txt"), index.search("word2").unwrap().as_path());
        assert_eq!(Path::new("./test/data/simple_content.txt"), index.search("word3").unwrap().as_path());
        assert_eq!(Path::new("./test/data/simple_content.txt"), index.search("word4").unwrap().as_path());
    }

    #[test]
    fn simple_directory_content() {
       let mut index = Index::new();
       index.index_directory(PathBuf::from("./test/data/simple_directory"));
       assert_eq!(Path::new("./test/data/simple_directory/file1.txt"), index.search("word1").unwrap().as_path());
       assert_eq!(Path::new("./test/data/simple_directory/file1.txt"), index.search("word2").unwrap().as_path());
       assert_eq!(Path::new("./test/data/simple_directory/file1.txt"), index.search("word3").unwrap().as_path());
       assert_eq!(Path::new("./test/data/simple_directory/file2.txt"), index.search("word4").unwrap().as_path());
       assert_eq!(Path::new("./test/data/simple_directory/file2.txt"), index.search("word5").unwrap().as_path());
    }

    #[test]
    fn random_directory_content() {
        let mut index = Index::new();
        index.index_directory(PathBuf::from("./test/data/random_directory"));
        assert_eq!(Path::new("./test/data/random_directory/file1.txt"), index.search("word1").unwrap().as_path());
        assert_eq!(Path::new("./test/data/random_directory/file1.txt"), index.search("word2").unwrap().as_path());
        assert_eq!(Path::new("./test/data/random_directory/file1.txt"), index.search("word3").unwrap().as_path());
        assert_eq!(None, index.search("word4"));
        assert_eq!(None, index.search("word5"));
    }
}

