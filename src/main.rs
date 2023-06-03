use axum::{routing::get, Router, extract::Path};
use std::net::SocketAddr;
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index_handler))
    .route("/webhook/:id", get(webhook_handler));

    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> &'static str  {
    "Hello, World!"
}

async fn webhook_handler(Path(id): Path<String>) -> &'static str {
    match get_hash_from_env() {
        Ok(hash) => {
            if hash == id {
                "Hashes match!"
            } else {
                "Hashes don't match!"
            }
        }
        Err(_) => "Error occurred while fetching hash.",
    }
}

fn get_hash_from_env() -> Result<String, env::VarError> {
    dotenv().ok();
    env::var("HASH")
}