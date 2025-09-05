use axum::http::StatusCode;
use ecommerce::{
    launch, 
    utils::{app_error::AppError, app_state::{AppState, TokenWrapper, Wrapper}}
};
use dotenvy_macro::dotenv;
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    
    dotenvy::dotenv().ok();
    let port = dotenv!("PORT").to_string();
    let base_url = dotenv!("BASE_ADDRESS").to_string();
    let database_url = dotenv!("DATABASE_URL");
    let jwt_secret = dotenv!("JWT_SECRET").to_string(); 
    // ⚠️ CodeQL may flag "hardcoded secret / sensitive value"
    //    (if .env or default secret values are committed, weak, or exposed)

    let database = Database::connect(database_url)
        .await
        .map_err(|error|{
            eprintln!("Error could not connect to the database: {}", error);
            // ⚠️ Potential information disclosure:
            //    Database connection errors printed to stderr may reveal details
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Could not connect to the database")
        })?;

    let app_state = AppState{
        database,
        base_url: Wrapper { url: base_url, port },
        // ⚠️ If BASE_ADDRESS is http:// instead of https://,
        //    CodeQL can raise insecure transport warnings
        jwt_secret: TokenWrapper(jwt_secret)
        // ⚠️ If JWT_SECRET is weak or short, CodeQL flags weak key usage
        // ⚠️ Later in code (not shown), if HS256/none algorithm is used,
        //    CodeQL may raise insecure JWT algorithm usage
    };

    launch(app_state).await?;
    // ⚠️ Handlers launched from here can propagate tainted input.
    //    CodeQL tracks user input -> DB queries (SQL injection), 
    //    -> response (XSS/info leaks), -> filesystem/OS calls.
    
    Ok(())
}
