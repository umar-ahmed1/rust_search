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
