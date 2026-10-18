use sqlx::{PgPool, QueryBuilder};
use rust_decimal::Decimal;
use chrono::Utc;
use crate::products::model::Product;

// we pass in a reference to the pool here so multiple queries can use the pool at the same time
// remember that one mutable reference or multiple immutable references
pub async fn fetch_all_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products = sqlx::query_as::<_, Product>(
        "select * from products",
        ).fetch_all(pool).await?;

    // println!("products is {:?}", products);

    Ok(products)
}

// fetch a single product (returns None if it doesn't exist) since fetch_optional cud return null
pub async fn get_product_by_id(pool: &PgPool, id: i32) -> Result<Option<Product>, sqlx::Error> {
    // option returns some or none if the product exists or not
    let product: Option<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, title, description, category, price, in_stock, updated_at FROM products WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    // only borrow here so we can still return the product - otherwise rust gets mad
    match &product {
        Some(p) => {
            println!("get product by id got a product: {:?}", p)
        }
        None => {
            println!("get product by id found no product with id: {}", id)
        }
    }

    Ok(product)
}

pub async fn get_products_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<Product>, sqlx::Error> {
    // the type in query_as is of type 1 row so we don't do vec
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, title, description, category, price, in_stock, updated_at FROM products WHERE id = ANY($1)"
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;

    Ok(products)
}

pub async fn create_product(
    pool: &PgPool,
    title: &str,
    description: &str,
    category: &str,
    price: Decimal,
    in_stock: bool,
) -> Result<Product, sqlx::Error> {

    // query_as! checks postgres to get type whenever you build the project, query_as does it at runtime
    // technically query_as! is better cuz it prevents sql typo and stuff but we just use query_as here
    let product: Product = sqlx::query_as::<_, Product>(
        r#"INSERT INTO products (title, description, category, price, in_stock, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, title, description, category, price, in_stock, updated_at"#
    )
    .bind(title)
    .bind(description)
    .bind(category)
    .bind(price)
    .bind(in_stock)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    println!("Successfully created product: {}", product.title);

    Ok(product)
}

/// modify an existing product by id and return the updated record
pub async fn update_product(
    pool: &PgPool,
    id: i32,
    title: &str,
    description: &str,
    category: &str,
    price: Decimal,
    in_stock: bool,
) -> Result<Product, sqlx::Error> {
    let product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products 
        SET title = $1, description = $2, category = $3, price = $4, in_stock = $5, updated_at = $6
        WHERE id = $7
        RETURNING id, title, description, category, price, in_stock, updated_at
        "#
    )
    .bind(title)
    .bind(description)
    .bind(category)
    .bind(price)
    .bind(in_stock)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(pool)
    .await?;

    println!("Successfully updated product: {}", product.title);

    Ok(product)
}

/// remove a product by ID, returns the number of rows deleted (0 or 1)
pub async fn delete_product(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    println!("rows affected by deleting id: {} is {}", id, result.rows_affected());

    Ok(result.rows_affected())
}

// seed a lot of bulk products
pub async fn seed_products_bulk(pool: &PgPool, products: &[Product]) -> Result<(), sqlx::Error> {
    if products.is_empty() {
        return Ok(());
    }

    // SQLx QueryBuilder lets us dynamically build bulk inserts safely in chunks (e.g., 1,000 at a time)
    for chunk in products.chunks(1_000) {
        let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO products (title, description, category, price, in_stock, updated_at) "
        );

        query_builder.push_values(chunk, |mut b, product| {
            b.push_bind(&product.title)
             .push_bind(&product.description)
             .push_bind(&product.category)
             .push_bind(product.price)
             .push_bind(product.in_stock)
             .push_bind(product.updated_at);
        });

        let query = query_builder.build();
        query.execute(pool).await?;
    }

    println!("Successfully seeded {} products into PostgreSQL!", products.len());
    Ok(())
}


/// delete every product and reset the id counter so the next insert gets id 1
pub async fn truncate_products(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE products RESTART IDENTITY")
        .execute(pool)
        .await?;
    Ok(())
}
