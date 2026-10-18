mod products;
mod index;
mod db;

use chrono::{Utc};
use products::{search_products};
use index::build_inverted_index;
use rust_decimal::Decimal;
use sqlx::PgPool;
use crate::db::setup_db;
use crate::products::{Product, SearchResult};
use crate::index::{InvertedIndex, search_inverted_products};
use std::time::{Duration,Instant};
use db::{fetch_all_products};

#[tokio::main] // <-- This macro transforms the async main for Rust - native rust doesnt know how to run async and await
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup the db and query products
    let pool: PgPool = setup_db().await?; // remember the .await? bubbles the error to mains error so we are just left with the good case

    println!("Generating 100,000 products in memory...");
    let catalog = generate_large_catalog(100_000);

    println!("Inserting products into PostgreSQL...");
    let start = Instant::now();
    db::seed_products_bulk(&pool, &catalog).await?;
    println!("Database seeding took: {:?}", start.elapsed());

    // Now build your inverted index from the database or catalog and benchmark!
    let index = build_inverted_index(&catalog);
    let results = search_inverted_products(&index, &catalog, "wireless keyboard");
    println!("Found {} results for search query.", results.len());

    // test_db(&pool).await?;
    // compare();

    Ok(())
}

async fn test_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    let _: Vec<Product> = fetch_all_products(&pool).await?;

    let _ = db::create_product(
        &pool,
        "Mechanical Keyboard",
        "A tactile and responsive mechanical keyboard.",
        "Electronics",
        Decimal::new(12999, 2),
        true,
    )
    .await?;

    let product_id: i32 = 5;

    let _ = db::get_product_by_id(&pool, product_id).await?;

    let _ = db::update_product(
        &pool,
        1,
        "Wireless Mechanical Keyboard",
        "An upgraded wireless mechanical keyboard with Bluetooth.",
        "Electronics",
        Decimal::new(14999, 2), // Represents 149.99
        true,
    )
    .await?;

    let _ = db::delete_product(&pool, product_id).await?;

    let _ = db::get_product_by_id(&pool, product_id).await?;

    Ok(())
}

fn generate_large_catalog(size: usize) -> Vec<Product> {
    let mut products = Vec::with_capacity(size);

    let adjectives: [&str; 15] = [
        "wireless", "ergonomic", "portable", "durable", "lightweight",
        "mechanical", "waterproof", "insulated", "smart", "adjustable",
        "reusable", "compact", "premium", "heavy", "smooth"
    ];

    let nouns: [&str; 15] = [
        "keyboard", "mouse", "headphones", "backpack", "bottle",
        "jacket", "shoes", "tent", "lamp", "processor",
        "desk", "coffee", "maker", "t-shirt", "watch"
    ];

    let contexts: [&str; 5] = [
        "designed for gaming and office productivity.",
        "great for hiking, camping, and outdoor adventures.",
        "perfect for everyday use, studying, and travel.",
        "engineered for high performance and long-lasting comfort.",
        "keeps drinks cold and weather out during winter trails."
    ];

    for i in 1..=size {
        // Cycle through our arrays to mix and match words uniquely
        let adj: &str = adjectives[i % adjectives.len()];
        let noun: &str = nouns[(i / adjectives.len()) % nouns.len()];
        let context: &str = contexts[i % contexts.len()];

        products.push(Product {
            id: i as i32,
            title: format!("{} {} #{}", adj, noun, i),
            description: format!("A reliable product made with quality materials. It is {} {}", adj, context),
            category: "abc".to_string(),
            price: Decimal::new(5, 1),
            in_stock: true,
            updated_at: Utc::now(),
        });
    }

    products
}

fn compare() {
    let sizes: [usize; 4] = [100, 1_000, 10_000, 50_000]; // add 50,000 and 1_000_000 later if needed
    let query: &str = "jacket";

    for &size in &sizes {
        println!("\n--- Benchmarking with {} products ---", size);
        let catalog = generate_large_catalog(size);

        // 1. Benchmark Index Build Time
        let start_build: Instant = Instant::now();
        let index: InvertedIndex = build_inverted_index(&catalog);
        let duration_build: Duration = start_build.elapsed();
        println!("Index build time:       {:?}", duration_build);

        // 2. Benchmark Naive Search (search_products)
        let start_naive: Instant = Instant::now();
        let naive_results: Vec<SearchResult<'_>> = search_products(&catalog, query);
        let duration_naive: Duration = start_naive.elapsed();
        println!("Naive search time:      {:?} (Found {} results)", duration_naive, naive_results.len());

        // 3. Benchmark Indexed Search (search_inverted_products)
        let start_query: Instant = Instant::now();
        let indexed_results: Vec<SearchResult<'_>> = search_inverted_products(&index, &catalog, query);
        let duration_query: Duration = start_query.elapsed();
        println!("Indexed search time:    {:?} (Found {} results)", duration_query, indexed_results.len());
    }
}
