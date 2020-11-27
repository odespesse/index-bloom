# index-bloom

A fast and lightweight full-text search engine aims to provide basic search functionality.

`index-bloom` is an alternative to a heavy full-text search engine when you need to retrieve only ids instead of documents.
It is best used in a scenario where the memory footprint is a strong constraint and a low percentage of false positives is acceptable.
At its core `index-bloom` uses Bloom filters to store a reduced representation of the document while minimizing search time.
Such a filter ensures that a negative response (absence of the document) is certain, while a positive response (presence of the document) is accurate according to a customizable probability.

## Example
An example of how to create a new index, ingest content and search for keywords :

```rust
    use index_bloom::Index;
    use index_bloom::Error;
    
    fn main() -> Result<(), Error>{
        // Create a new index
        let mut index = Index::new(0.00001);
        // Index contents
        let first_content = "A very very long content...";
        index.ingest("foo".to_string(), first_content)?;
        let second_content = "Another content !";
        index.ingest("bar".to_string(), second_content)?;
        // Search for various words
        let hits = index.search("content")?;
        println!("{:?}", hits.unwrap()); // ["bar", "foo"]
        let hits = index.search("very")?;
        println!("{:?}", hits.unwrap()); // ["foo"]
        let hits = index.search("unknown")?;
        println!("{:?}", hits); // None
        Ok(())
    }
```

## License

`index-bloom` is released under the MIT license ([LICENSE](https://github.com/odespesse/index-bloom/blob/master/LICENSE)).

## Resources

- Inspired by the article [Writing a full-text search engine using Bloom filters](https://www.stavros.io/posts/bloom-filter-search-engine/)
- Bloom filters originally published in this paper [Burton H. Bloom, Space/Time Trade-offs in Hash Coding with Allowable Errors](https://dl.acm.org/doi/10.1145/362686.362692)
- If you want to evaluate a filter theoretical properties you can use the online tool [Bloom Filter Calculator](https://hur.st/bloomfilter/)
