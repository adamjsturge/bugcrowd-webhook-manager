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
    pub data: AttributesData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttributesData {
    pub source: Option<String>,
    pub current_substate: Option<String>,
    pub blocked_by: Option<String>,
    // add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relationships {
    pub actor: Actor,
    pub resource: Option<Resource>,
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
    pub relationships: Option<IncludedRelationships>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedRelationships {
    pub resource: Option<IncludedRelationshipsResource>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedRelationshipsResource {
    pub data: IncludedRelationshipsResourceData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedRelationshipsResourceData {
    pub r#type: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedAttributes {
    pub name: Option<String>,
    pub email: Option<String>,
    pub staff: Option<bool>,
}


#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index_handler))
    .route("/webhook/:id", post(webhook_handler));

    // Address that server will bind to.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> &'static str  {
    "Hello, World!"
}

async fn webhook_handler(Path(id): Path<String>, Json(body): Json<EventData>) -> String {
    match get_from_env("HASH") {
        Ok(hash) => {
            if hash == id {
                action_webhook_handler(body).await
            } else {
                format!("Hashes don't match!")
            }
        }
        Err(_) => format!("Error occurred while fetching hash."),
    }
}

async fn action_webhook_handler(body: EventData) -> String {
    let EventData { data, included } = body;
    let Attributes { key, data: attributes_data, created_at } = data.attributes;
    
    let url = create_submission_url(&included[1]);

    match &*key {
        // "blocker.updated" => format!("{} {}", handle_blocker_updated(body), url),
        "blocker.created" => format!("{} {}", handle_blocker_created(attributes_data), url),
        // "submission.created" => handle_submission_created(body),
        // "submission.updated" => handle_submission_updated(body),
        _ => format!("Unknown event!")
    }
}

// fn create_submission_url(submission_id: String) -> String {
//     format!("https://tracker.bugcrowd.com/{}/submissions/{}", get_from_env("BUGCROWD_ORG").unwrap(), submission_id)
// }

fn create_submission_url(included: &Included) -> String {
    match &included.relationships {
        Some(relations) => match &relations.resource {
            Some(resource) => format!("https://tracker.bugcrowd.com/{}/submissions/{}",
                get_from_env("BUGCROWD_ORG").unwrap(),
                resource.data.id
            ),
            None => String::from("Resource is None"),
        },
        None => String::from("Relationships are None"),
    }
}



fn get_from_env(key: &str) -> Result<String, env::VarError> {
    dotenv().ok();
    env::var(key)
}

fn handle_blocker_created(attributes_data: AttributesData) -> String {
    match attributes_data.blocked_by {
        Some(blocked_by) if blocked_by == "bugcrowd_operations" => format!("Blocker created for Bugcrowd Operations!"),
        Some(blocked_by) if blocked_by == "researcher" => format!("Blocker created for researcher!"),
        Some(blocked_by) if blocked_by == "customer" => format!("Blocker created for customer!"),
        _ => format!("Unknown blocker creator!")
    }
}

fn handle_blocker_updated(body: EventData) -> String {
    match body.data.attributes.data.blocked_by {
        Some(blocked_by) if blocked_by == "bugcrowd_operations" => format!("Blocker created for Bugcrowd Operations!"),
        Some(blocked_by) if blocked_by == "researcher" => format!("Blocker created for researcher!"),
        Some(blocked_by) if blocked_by == "customer" => format!("Blocker created for customer!"),
        _ => format!("Unknown blocker creator!")
    }
}

fn handle_submission_created(body: EventData) -> String {
    format!("Submission created!")
}

fn handle_submission_updated(body: EventData) -> String {
    format!("Submission updated!")
}