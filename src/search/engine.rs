use std::collections::HashMap;
use crate::indexing::tokenizer::tokenize;
use crate::indexing::inverted_index::InvertedIndex;
use crate::search::bm25::{calculate_bm25_idf, calculate_bm25_tf};

#[derive(Debug)]
pub struct SearchResult {
    pub product_id: i32,
    pub score: f64,
}

pub fn search_inverted_products(index: &InvertedIndex, query: &str) -> Vec<SearchResult> {
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

        // now that we have all the product ids and their scores we can return them since we have defined a search result to just be id, score
        for (product_id, score)  in product_id_scores {
            let search_result = SearchResult {
                product_id,
                score,
            };
            results.push(search_result);
        }


        //sort and return search results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
}
