mod indexer;

use crate::indexer::index::Index;
use clap::{App, Arg};

fn main() {
    let matches = App::new("web-bloom")
                   .version("1.0")
                   .about("A lightweight search engine for the Web.")
                   .arg(Arg::with_name("source")
                        .short("s")
                        .long("source")
                        .help("Path to the file or directory to index")
                        .takes_value(true)
                        .required(true))
                   .get_matches();

    let source = matches.value_of("source").unwrap();
    let mut index = Index::new();
    index.index(source);
    assert!(index.search("word2"));
}

