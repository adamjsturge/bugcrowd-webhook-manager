use axum::{routing::get, routing::post, Router, extract::Path, extract::Json};
use std::net::SocketAddr;
use std::env;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use slack_morphism::prelude::*;
use url::Url;

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
                action_webhook_handler(body).await;
                format!("Done")
            } else {
                format!("Unauthorized")
            }
        }
        Err(_) => format!("Error occurred while fetching hash."),
    }
}

async fn action_webhook_handler(body: EventData) -> () {
    let EventData { data, included } = body;
    let Attributes { key, data: attributes_data, created_at } = data.attributes;
    
    let url = create_submission_url(&included[1]);

    match &*key {
        // "blocker.updated" => handle_blocker_updated(attributes_data).await,
        "blocker.created" => handle_blocker_created(attributes_data, url).await,
        "submission.created" => handle_submission_created(attributes_data, url).await,
        "submission.updated" => handle_submission_updated(attributes_data, url).await,
        _ => ()
    }
}

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

async fn handle_blocker_created(attributes_data: AttributesData, url: String) {
    let message = match attributes_data.blocked_by {
        Some(blocked_by) if blocked_by == "bugcrowd_operations" => "Blocker created for Bugcrowd Operations!".to_string(),
        Some(blocked_by) if blocked_by == "researcher" => "Blocker created for researcher!".to_string(),
        Some(blocked_by) if blocked_by == "customer" => "Blocker created for customer!".to_string(),
        _ => "Unknown blocker creator!".to_string()
    };
    send_slack_message("#bugcrowd-rust-info", "Blocker Created", message, url).await;
}

async fn handle_submission_created(attributes_data: AttributesData, url: String) {
    send_slack_message("#bugcrowd-rust-info", "Submission Created", "Created ya".to_string(),url).await;
}

async fn handle_submission_updated(attributes_data: AttributesData, url: String) {
    send_slack_message("#bugcrowd-rust-info", "Submission Updated", "Updated ya".to_string(),url).await;
}

async fn send_slack_message(channel: &'static str, action: &'static str, message: String, url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());
    
    let token_value: SlackApiTokenValue = get_from_env("SLACK_BOT_TOKEN").unwrap().into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let post_chat_req =
        SlackApiChatPostMessageRequest::new(channel.into(),
               SlackMessageContent::new().with_blocks(slack_blocks![
                some_into(SlackHeaderBlock::new(
                    pt!(action)
                )),
                some_into(SlackDividerBlock::new()),
                some_into(
                    SlackSectionBlock::new()
                        .with_text(md!(message))
                ),
                some_into(SlackActionsBlock::new(slack_blocks![some_into(
                    SlackBlockButtonElement::new(
                        "view-submission".into(),
                        pt!("View Submission"),
                    ).with_url(Url::parse(&url).unwrap())
                )]))
            ]),
        );

    let post_chat_resp = session.chat_post_message(&post_chat_req).await?;

    Ok(())
}
