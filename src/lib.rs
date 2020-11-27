//! # index-bloom
//!
//! The `index-bloom` crate provide a lightweight full-text search index focused on speed and space efficiency.
//!
//! It is able to ingest UTF-8 text content and search for matching words at the expense of customizable error probability of false positive (documents not containing all the keywords).
//! By its very nature, the original words are lost in the ingestion process. Therefore, it is not possible to estimate a relevance score for each document based on the index content.
//!
//! _Note_: When the same `name` is used to identify ingested content, the last one replace the previous one in the `Index`.
//!
//! # Quick start
//!
//! ```rust
//! use index_bloom::Index;
//! #   use index_bloom::Error;
//!
//! # fn main() -> Result<(), Error> {
//! let mut index = Index::new(0.00001);
//! index.ingest("foo".to_string(), "A very very long content...")?;
//! let hits = index.search("content")?;
//! println!("{:?}", hits.unwrap());
//! # Ok(())
//! # }
//! ```

mod index;
pub use crate::index::Index;
mod errors;
pub use crate::errors::Error;

mod bloom_filter;
mod tokens;
