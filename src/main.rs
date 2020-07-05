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
                        .takes_value(true))
                   .arg(Arg::with_name("restore")
                        .short("r")
                        .long("restore")
                        .help("Path to an index dump file")
                        .takes_value(true))
                   .arg(Arg::with_name("dump")
                        .short("d")
                        .long("dump")
                        .help("Path to dump the current index")
                        .takes_value(true))
                   .get_matches();

    let mut index = match matches.value_of("restore") {
        Some(restore_file) => Index::restore(restore_file),
        None => Index::new()
    };
    if let Some(source) = matches.value_of("source") {
        index.index(source);
    }
    if let Some(dump_file) = matches.value_of("dump") {
        index.dump(dump_file);
    }
}

