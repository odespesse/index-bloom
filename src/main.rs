mod indexer;

use crate::indexer::index::Index;
use std::path::{PathBuf, Path};

fn main() {
    let mut index = Index::new();
    index.index_directory(PathBuf::from("./test/data/simple_directory"));
    assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word1").unwrap());
}
