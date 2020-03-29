mod indexer;

use crate::indexer::index::Index;
use std::path::Path;

fn main() {
    let mut index = Index::new();
    index.index("./test/data/simple_directory");
    assert_eq!(vec![Path::new("./test/data/simple_directory/file1.txt")], index.search("word1").unwrap());
}
