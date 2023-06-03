use axum::{routing::get, routing::post, Router, extract::Path, extract::Json};
use std::net::SocketAddr;
use std::env;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct EventData {
    pub data: Data,
    pub included: Vec<Included>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
    pub relationships: Relationships,
    pub links: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    pub created_at: String,
    pub key: String,
    pub data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relationships {
    pub actor: Actor,
    pub resource: Resource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    pub data: ActorData,
    pub links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActorData {
    pub r#type: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    pub related: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub data: ResourceData,
    pub links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceData {
    pub r#type: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Included {
    pub id: String,
    pub r#type: String,
    pub links: HashMap<String, String>,
    pub attributes: IncludedAttributes,
    pub relationships: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedAttributes {
    pub name: String,
    pub email: String,
    pub staff: bool,
}


#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index_handler))
    .route("/webhook/:id", post(webhook_handler));

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

async fn webhook_handler(Path(id): Path<String>, Json(body): Json<EventData>) -> &'static str {
    match get_hash_from_env() {
        Ok(hash) => {
            if hash == id {
                action_webhook_handler(body).await
            } else {
                "Hashes don't match!"
            }
        }
        Err(_) => "Error occurred while fetching hash.",
    }
}

async fn action_webhook_handler(body: EventData) -> &'static str {
    if body.data.attributes.key == "blocker.updated" {
        "Blocker updated!"
    } else {
        "Not a blocker updated event!"
    }
}

fn get_hash_from_env() -> Result<String, env::VarError> {
    dotenv().ok();
    env::var("HASH")
}