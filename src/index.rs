use std::fs::File;
use std::io::Read;
use crate::bloom_filter::BloomFilter;

pub struct Index {
    bloom_filter: BloomFilter
}

impl Index {
    pub fn new() -> Self {
        Index{
            bloom_filter: BloomFilter::new(1000, 0.1)
        }
    }

    pub fn search(&self, keywords: &str) -> bool {
        self.bloom_filter.contains(keywords)
    }

    fn index_sentence(&mut self, words: &str) {
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
                            .replace("(", "")
                            .replace(")", "")
                            .replace("[", "")
                            .replace("]", "")
                            .replace("{", "")
                            .replace("}", "")
                            .replace("'", "")
                            .replace("\"", ""))
            .filter(|word| !word.is_empty());
        for word in words_list {
            self.bloom_filter.insert(&word);
        }
    }

    fn index_file(&mut self, mut file: File) {
        let mut content = String::new();
        file.read_to_string(&mut content).expect("error while reading file");
        for line in content.lines() {
            self.index_sentence(line);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn white_space() {
        let mut index = Index::new();
        index.index_sentence("word1 word2  word3  word4\nword5\tword6");
        assert!(index.search("word1"));
        assert!(index.search("word2"));
        assert!(index.search("word3"));
        assert!(index.search("word4"));
        assert!(index.search("word5"));
        assert!(index.search("word6"));
    }

    #[test]
    fn punctuation() {
        let mut index = Index::new();
        index.index_sentence("word1. word2! word3? word4, word5; word6: word7/ word8& (word9) [word10] {word11} 'word12' \"word13\" ? ");
        assert!(index.search("word1"));
        assert!(index.search("word2"));
        assert!(index.search("word3"));
        assert!(index.search("word4"));
        assert!(index.search("word5"));
        assert!(index.search("word6"));
        assert!(index.search("word7"));
        assert!(index.search("word8"));
        assert!(index.search("word9"));
        assert!(index.search("word10"));
        assert!(index.search("word11"));
        assert!(index.search("word12"));
        assert!(index.search("word13"));
        assert!(!index.search("?"));
    }

    #[test]
    fn file_simple_content() {
        let mut index = Index::new();
        let file = File::open("./test/data/simple_content.txt").unwrap();
        index.index_file(file);
        assert!(index.search("word1"));
        assert!(index.search("word2"));
        assert!(index.search("word3"));
        assert!(index.search("word4"));
    }
}