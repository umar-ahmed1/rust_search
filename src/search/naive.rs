use crate::products::model::Product;
use crate::search::engine::SearchResult;

// rust could be confused if the result is pointing to products or query so use 'a to make it aware
pub fn search_products(products: &Vec<Product>, query: &str) -> Vec<SearchResult> {
    //normalize the query
    let lowercase: String = query.to_lowercase();

    // split the query into words ex: "trail running shoes" -> ["trail", "running", "shoes"]
    let filters: Vec<&str> = lowercase.split_whitespace().collect();

    //if there are no filters then return empty vector
    if filters.is_empty() {
        return vec![];
    }

    let mut results: Vec<SearchResult> = Vec::new();

    //perform the search and return the products
    for product in products {
        // normalize the title/description
        let title: String = product.title.to_lowercase();
        let description: String = product.description.to_lowercase();

        //search through the filters and find the product -> for now if any word matches we can return the product so we r doing an OR over filters
        let matches_title = filters.iter().any(|filter| title.contains(filter));
        let matches_description = filters.iter().any(|filter| description.contains(filter));

        if !(matches_title || matches_description) {
            continue;
        }

        let mut score: f64 = 0.0;
        if matches_title { score += 10.0};
        if matches_description { score += 5.0};

        let search_result = SearchResult {
            product_id: product.id, 
            score
        };

        results.push(search_result);
    }
    // sort search results by highest score first
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}
