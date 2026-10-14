mod products;
mod index;

use products::{sample_products, search_products};
use index::build_inverted_index;
use crate::products::{Product, SearchResult};
use crate::index::{search_inverted_products};
use std::time::Instant;

fn main() {
    println!("Hello, world!");
    let products: Vec<Product> = sample_products();

    let query: &str = "gaming";
    // results is now borrowing products using the reference (since searchresult contains &product) so we cant modify products until results is done using it
    let results: Vec<SearchResult> = search_products(&products, query);

    println!("Found {} matches for '{}':", results.len(), query);

    for search_result in results {
        println!("{:?}", search_result);
    }

    let index = build_inverted_index(&products);
    println!("index is {:?}", index);

    let inverted_results = search_inverted_products(&index, &products, query);

    println!("Found {} matches for '{}':", inverted_results.len(), query);

    for search_result in inverted_results {
        println!("inverted {:?}", search_result);
    }

    compare();
}

fn generate_large_catalog(size: usize) -> Vec<Product> {
    let mut products = Vec::with_capacity(size);

    let adjectives = [
        "wireless", "ergonomic", "portable", "durable", "lightweight",
        "mechanical", "waterproof", "insulated", "smart", "adjustable",
        "reusable", "compact", "premium", "heavy", "smooth"
    ];

    let nouns = [
        "keyboard", "mouse", "headphones", "backpack", "bottle",
        "jacket", "shoes", "tent", "lamp", "processor",
        "desk", "coffee", "maker", "t-shirt", "watch"
    ];

    let contexts = [
        "designed for gaming and office productivity.",
        "great for hiking, camping, and outdoor adventures.",
        "perfect for everyday use, studying, and travel.",
        "engineered for high performance and long-lasting comfort.",
        "keeps drinks cold and weather out during winter trails."
    ];

    for i in 1..=size {
        // Cycle through our arrays to mix and match words uniquely
        let adj = adjectives[i % adjectives.len()];
        let noun = nouns[(i / adjectives.len()) % nouns.len()];
        let context = contexts[i % contexts.len()];

        products.push(Product {
            id: i as i32,
            title: format!("{} {} #{}", adj, noun, i),
            description: format!("A reliable product made with quality materials. It is {} {}", adj, context),
            category: "abc".to_string()
        });
    }

    products
}

fn compare() {
    let sizes = [100, 1_000, 10_000, 50_000, 1_000_000];
    let query = "jacket";

    for &size in &sizes {
        println!("\n--- Benchmarking with {} products ---", size);
        let catalog = generate_large_catalog(size);

        // 1. Benchmark Index Build Time
        let start_build = Instant::now();
        let index = build_inverted_index(&catalog);
        let duration_build = start_build.elapsed();
        println!("Index build time:       {:?}", duration_build);

        // 2. Benchmark Naive Search (search_products)
        let start_naive = Instant::now();
        let naive_results = search_products(&catalog, query); // Using your existing function!
        let duration_naive = start_naive.elapsed();
        println!("Naive search time:      {:?} (Found {} results)", duration_naive, naive_results.len());

        // 3. Benchmark Indexed Search (search_inverted_products)
        let start_query = Instant::now();
        let indexed_results = search_inverted_products(&index, &catalog, query);
        let duration_query = start_query.elapsed();
        println!("Indexed search time:    {:?} (Found {} results)", duration_query, indexed_results.len());
    }
}
