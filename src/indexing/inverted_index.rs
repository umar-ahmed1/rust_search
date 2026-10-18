use std::collections::HashMap;
use crate::products::model::Product;
use crate::indexing::tokenizer::tokenize;

#[derive(Debug)]
pub struct InvertedIndex {
    // the key is the word and the value is a tuple of (product_id, how many times the word showed up in that product)
    pub postings: HashMap<String, Vec<(i32, u32)>>,
    // hashmap that maps a product to its length for quick lookup
    pub doc_lengths: HashMap<i32, usize>,
    // fields for bm25 calculation
    pub average_dl: f64,
    pub total_docs: usize,
}

pub fn build_inverted_index(products: &[Product]) -> InvertedIndex {
    let mut inverted_index: HashMap<String, Vec<(i32,u32)>> = HashMap::new();
    let mut doc_lengths: HashMap<i32, usize> = HashMap::new();

    // track total length for BM25 
    let mut total_length: usize = 0;

    //go through each product
    for product in products {
        // tokenize the two relevant fields
        let title_tokenized: Vec<String> = tokenize(&product.title); // can do &product.title here cuz its a string and its an owned type so we let tokenize borrow it
        let description_tokenized: Vec<String> = tokenize(&product.description);

        // combine the two so we have all the words related to a product and add this length to our products length hashmap
        let words: Vec<String> = title_tokenized.into_iter().chain(description_tokenized).collect();
        total_length += words.len();
        *doc_lengths.entry(product.id).or_insert(0) = words.len();
        
        // count how many times each term appears in title + description
        let mut term_counts: HashMap<String, u32> = HashMap::new();
        for word in words {
            *term_counts.entry(word).or_insert(0) += 1;
        }
        //insert into the inverted index with entry key of word and append the id of the product and its freq to the array of product ids for that word. ex for "running" -> (1,2) we need to insert id 3 and its freq
        for (word, freq) in term_counts {
            inverted_index.entry(word).or_default().push((product.id, freq)); //i32 is a primitize so we dont need &product its on the stack and it'll get copied automatically - ownership doesnt matter
        }

    }

    let average_dl = if products.is_empty() {
        0.0
    } else {
        total_length as f64 / products.len() as f64
    };

    let index_struct = InvertedIndex {
        postings: inverted_index,
        doc_lengths,
        average_dl: average_dl,
        total_docs: products.len()
    };
    index_struct
}
