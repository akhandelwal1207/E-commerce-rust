use axum::http::StatusCode;
use ecommerce::{
    launch, 
    utils::{app_error::AppError, app_state::{AppState, TokenWrapper, Wrapper}}
};
use dotenvy_macro::dotenv;
use sea_orm::{Database, Statement};
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    
    dotenvy::dotenv().ok();
    let port = dotenv!("PORT").to_string();
    let base_url = dotenv!("BASE_ADDRESS").to_string();
    let database_url = dotenv!("DATABASE_URL");
    
    // ⚠️ Hardcoded secret (weak key)
    let jwt_secret = "mysecret".to_string();  

    // ⚠️ Insecure database connection string (potentially unvalidated)
    let database = Database::connect(database_url)
        .await
        .map_err(|error|{
            // ⚠️ Information disclosure: leaking internal error message
            eprintln!("Error could not connect to the database: {}", error);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Could not connect to the database")
        })?;

    // ⚠️ SQL Injection: unsafe string concatenation with user input
    if let Ok(user_id) = env::var("USER_ID") {
        let raw_query = format!("SELECT * FROM users WHERE id = {}", user_id);
        let stmt = Statement::from_string(sea_orm::DatabaseBackend::Postgres, raw_query);
        let _ = database.query_one(stmt).await;
    }

    // ⚠️ Insecure transport (if BASE_ADDRESS = "http://...")
    let app_state = AppState{
        database,
        base_url: Wrapper { url: base_url, port },
        jwt_secret: TokenWrapper(jwt_secret)
    };

    // ⚠️ Path traversal / arbitrary file read
    if let Ok(filename) = env::var("FILENAME") {
        let _ = fs::read_to_string(filename); 
    }

    launch(app_state).await?;
    Ok(())
}
