use sqlx::PgPool;

use crate::indexing::inverted_index::{build_inverted_index, InvertedIndex};
use crate::products::repository;

// Asks the database for its current generation (product count + newest `updated_at`) and compares it with the current built index
// - equal: nothing changed, the same index is handed back
// - different: reload every product, rebuild the index, and return the new one
pub async fn refresh_index_if_stale(pool: &PgPool, index: InvertedIndex) -> Result<InvertedIndex, sqlx::Error> {
    let db_generation = repository::get_index_staleness(pool).await?;

    if index.index_generation == db_generation {
        println!("Index is fresh: {:?}", db_generation);
        return Ok(index);
    }

    println!(
        "Index is stale. index: {:?}, database: {:?}. Rebuilding...",
        index.index_generation, db_generation
    );
    let products = repository::fetch_all_products(pool).await?;
    let rebuilt = build_inverted_index(&products);
    println!("Rebuilt index: {:?}", rebuilt.index_generation);

    Ok(rebuilt)
}
