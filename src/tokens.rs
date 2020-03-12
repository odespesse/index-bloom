use std::str::SplitWhitespace;
use unidecode::unidecode;

pub struct Tokens<'a> {
    words: SplitWhitespace<'a>
}

impl<'a> Tokens<'a> {
    pub fn new(words: &'a str) -> Self {
        Tokens {
            words: words.split_whitespace()
        }
    }

    fn clean_word(&self, word: &str) -> String {
        word.replace(".", "")
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
            .replace("\"", "")
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(word) = self.words.next() {
            let ascii_word = unidecode(word);
            let token = self.clean_word(&ascii_word).to_lowercase();
            if !token.is_empty() {
                return Some(token)
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_space() {
        let mut tokens = Tokens::new("word1 word2  word3  word4\nword5\tword6");
        assert_eq!(tokens.next().unwrap(), "word1");
        assert_eq!(tokens.next().unwrap(), "word2");
        assert_eq!(tokens.next().unwrap(), "word3");
        assert_eq!(tokens.next().unwrap(), "word4");
        assert_eq!(tokens.next().unwrap(), "word5");
        assert_eq!(tokens.next().unwrap(), "word6");
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn punctuation() {
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
        let mut tokens = Tokens::new(&sentence);
        assert_eq!(tokens.next().unwrap(), "word1");
        assert_eq!(tokens.next().unwrap(), "word2");
        assert_eq!(tokens.next().unwrap(), "word3");
        assert_eq!(tokens.next().unwrap(), "word4");
        assert_eq!(tokens.next().unwrap(), "word5");
        assert_eq!(tokens.next().unwrap(), "word6");
        assert_eq!(tokens.next().unwrap(), "word7");
        assert_eq!(tokens.next().unwrap(), "word8");
        assert_eq!(tokens.next().unwrap(), "word9");
        assert_eq!(tokens.next().unwrap(), "word10");
        assert_eq!(tokens.next().unwrap(), "word11");
        assert_eq!(tokens.next().unwrap(), "word12");
        assert_eq!(tokens.next().unwrap(), "word13");
        assert_eq!(tokens.next().unwrap(), "word14");
        assert_eq!(tokens.next().unwrap(), "word15");
        assert_eq!(tokens.next().unwrap(), "word16");
        assert_eq!(tokens.next().unwrap(), "word17");
        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn normalize_words() {
        let mut tokens = Tokens::new("WORD1 word2 éèêàïùç");
        assert_eq!(tokens.next().unwrap(), "word1");
        assert_eq!(tokens.next().unwrap(), "word2");
        assert_eq!(tokens.next().unwrap(), "eeeaiuc");
        assert_eq!(tokens.next(), None);
    }
}
