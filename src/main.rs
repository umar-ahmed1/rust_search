mod config;
mod db;
mod index_manager;
mod indexing;
mod products;
mod search;
mod seed;

use sqlx::PgPool;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::db::setup_db;
use crate::indexing::inverted_index::{build_inverted_index, InvertedIndex, IndexGeneration};
use crate::products::model::Product;
use crate::products::repository::{self};
use crate::search::engine::{search_inverted_products, SearchResult};
use crate::search::naive::search_products;

#[tokio::main] // <-- This macro transforms the async main for Rust - native rust doesnt know how to run async and await
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup the db and query products
    let pool: PgPool = setup_db().await?; // remember the .await? bubbles the error to mains error so we are just left with the good case

    // `cargo run -- seed` wipes the table and reloads the deterministic catalog, then exits
    if std::env::args().nth(1).as_deref() == Some("seed") {
        seed::reset_and_seed(&pool, seed::CATALOG_SIZE).await?;
        return Ok(());
    }

    compare(&pool).await?;
    inverted_flow(&pool).await?;

    Ok(())
}

async fn compare(pool: &PgPool) -> Result<(), sqlx::Error> {
    let query: &str = "jacket";

    println!("\n--- Benchmarking products ---");
    let catalog = repository::fetch_all_products(pool).await?;

    // 1. Benchmark Index Build Time
    let start_build: Instant = Instant::now();
    let index: InvertedIndex = build_inverted_index(&catalog);
    let duration_build: Duration = start_build.elapsed();
    println!("Index build time:       {:?}", duration_build);

    // 2. Benchmark Naive Search (search_products)
    let start_naive: Instant = Instant::now();
    let naive_results: Vec<SearchResult> = search_products(&catalog, query);
    let duration_naive: Duration = start_naive.elapsed();
    println!("Naive search time:      {:?} (Found {} results)", duration_naive, naive_results.len());

    // 3. Benchmark Indexed Search (search_inverted_products)
    let start_query: Instant = Instant::now();
    let indexed_results: Vec<SearchResult> = search_inverted_products(&index, query);
    let duration_query: Duration = start_query.elapsed();
    println!("Indexed search time:    {:?} (Found {} results)", duration_query, indexed_results.len());
    
    Ok(())
}

async fn inverted_flow(pool: &PgPool) -> Result<(), sqlx::Error> {
    let query: &str = "shoes";

    println!("\n--- Benchmarking actual flow ---");
    let catalog = repository::fetch_all_products(pool).await?;

    // 1. Benchmark Index Build Time
    let start_build: Instant = Instant::now();
    let index: InvertedIndex = build_inverted_index(&catalog);
    let duration_build: Duration = start_build.elapsed();
    println!("Index build time:       {:?}", duration_build);

    // 2. Make sure the index still matches PostgreSQL (rebuilds it if the database changed)
    let index: InvertedIndex = index_manager::refresh_index_if_stale(pool, index).await?;

    // 3. Benchmark Indexed Search (search_inverted_products)
    let start_query: Instant = Instant::now();
    let indexed_results: Vec<SearchResult> = search_inverted_products(&index, query);
    let duration_query: Duration = start_query.elapsed();
    println!("Indexed search time:    {:?} (Found {} results)", duration_query, indexed_results.len());

    // 4. Take the top 10 ranked results (slice is safe even if there are fewer than 10 matches)
    let top: &[SearchResult] = &indexed_results[..indexed_results.len().min(10)];

    // 5. Fetch just those products from PostgreSQL (one query, arbitrary row order)
    let start_get: Instant = Instant::now();
    let ids: Vec<i32> = top.iter().map(|result| result.product_id).collect();
    let products: Vec<Product> = repository::get_products_by_ids(pool, &ids).await?;
    let duration_get: Duration = start_get.elapsed();
    println!("Indexed get time:       {:?} (Fetched {} products index products count {} index updated at {:?})", duration_get, products.len(), &index.index_generation.product_count, index.index_generation.max_updated_at);

    // 6. product id -> product map, so we can restore the ranked order
    let mut products_map: HashMap<i32, Product> = HashMap::with_capacity(products.len());
    for product in products {
        products_map.insert(product.id, product);
    }

    // 7. print the results
    println!("\nTop results for \"{}\":", query);
    let mut missing: usize = 0;
    for (rank, result) in top.iter().enumerate() {
        match products_map.get(&result.product_id) {
            Some(product) => println!("{}. {} score {}", rank + 1, product.title, result.score),
            None => missing += 1,
        }
    }
    if missing > 0 {
        println!("({} ranked product(s) no longer exist in PostgreSQL, skipped)", missing);
    }

    Ok(())
}