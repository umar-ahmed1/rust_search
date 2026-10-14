use crate::products::{Product, SearchResult};
use std::collections::{HashMap};

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

pub fn tokenize(text: &str) -> Vec<String> {
    // 1. Convert to lowercase once (allocates one String for the text)
    let lowercase = text.to_lowercase();

    // 2. Stream through whitespace, trim, filter, and only then allocate final Strings
    lowercase
        .split_whitespace()
        .map(|word| word.trim_matches(|ch: char| !ch.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string()) // Only allocate for words that actually survive!
        .collect()
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

// idf is basically how rare a word is, so we get the total docs and docs using the term to figure out how rare the term is
pub fn calculate_bm25_idf(total_docs: usize, docs_using_term: usize) -> f64 {
    let n = docs_using_term as f64;
    let n_docs = total_docs as f64;
    // Standard BM25 IDF formula
    let idf = ((n_docs - n + 0.5) / (n + 0.5) + 1.0).ln();
    // Sometimes IDF can dip negative for extremely common words; clamping to 0.0 is a safe guard
    idf.max(0.0)
}

// tf is the frequency of a term within a product, so we get the length of that products title + description and the frequency of that word and the average length of a products title + description
pub fn calculate_bm25_tf(freq: u32, doc_len: usize, avg_dl: f64) -> f64 {
    // edge cases
    if freq == 0 || avg_dl <= 0.0 {
        return 0.0;
    }
    // Constants for BM25
    let k1 = 1.2;
    let b = 0.75;

    //variables 
    let f = freq as f64;
    let dl = doc_len as f64;

    let numerator = f * (k1 + 1.0);
    let denominator = f + k1 * (1.0 - b + b * (dl / avg_dl));

    numerator / denominator
}

pub fn search_inverted_products<'a>(index: &InvertedIndex, products: &'a [Product], query: &str) -> Vec<SearchResult<'a>> {
        //normalize the query
        let query_terms = tokenize(query);
    
        //if there are no filters then return empty vector
        if query_terms.is_empty() {
            return vec![];
        }
        
        //hashmap of id -> score, we will calculate this score using the BM25 algorithm
        let mut product_id_scores: HashMap<i32, f64> = HashMap::new();
        let mut results: Vec<SearchResult> = Vec::new();

        // we have to go through our hashmap and find all the product ids for these words then combine/dedup (using hashset) and return the products
        for term in query_terms {
            if let Some(product_ids) = index.postings.get(&term) {
                // calculate idf for this query term
                let bm25_idf: f64 = calculate_bm25_idf(index.total_docs, product_ids.len());
                for (id, freq) in product_ids {

                    // calculate bm25 tf
                    let doc_len: usize = *index.doc_lengths.get(&id).unwrap_or(&0);
                    let bm25_tf: f64 = calculate_bm25_tf(*freq, doc_len, index.average_dl);

                    // calculate bm 25 with everything we have and insert into the id scores
                    // tf tells us if product talks about this term a lot
                    // idf tells us if this term is actually a rare term or not
                    let score = bm25_idf * bm25_tf;
                    *product_id_scores.entry(*id).or_insert(0.0) += score; // use dereference cuz we have a reference but we want the actual i32 number
                }
            }
        }

        // now that we have all the product ids and their scores we need to get the corresponding products and create the search results with the scores
        for (product_id, score)  in product_id_scores {
            let product = &products[(product_id - 1) as usize];
                let search_result = SearchResult {
                    product,
                    score,
                };
            results.push(search_result);
        }


        //sort and return search results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
}