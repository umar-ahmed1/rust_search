pub fn get_database_url() -> String {
    dotenvy::dotenv().ok();

    // Pull the DATABASE_URL variable we set in .env
    let database_url: String = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in the .env file");

    database_url
}
