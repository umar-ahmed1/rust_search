use sqlx::PgPool;
use crate::config::get_database_url;

pub async fn establish_connection(url: &str) -> Result<PgPool, sqlx::Error> {
    // a connection pool is a "pool" of active db connections postgres creates
    // at query time postgres borrows a connection from the pool, runs the query, and returns the connection back to the pool
    let result: Result<PgPool, sqlx::Error> = PgPool::connect(url).await;
    result
}

// we do a box error here cuz pool can return sqlx::error and the migrate can run sqlx::migrate::migrateerror 
// so box catches all errors
pub async fn setup_db() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url: String = get_database_url();
    let pool = establish_connection(&url).await?;

    // run any pending db migrations if needed
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
