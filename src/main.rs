use axum::{extract::Json, extract::Path, routing::get, routing::post, Router};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
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
    pub changes: Option<HashMap<String, ChangesStateData>>,
    pub duplicate_ids: Option<Vec<String>>,
    // add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum StateChangeType {
    String(String),
    U8(u8),
    Bool(bool),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChangesStateData {
    pub from: Option<StateChangeType>,
    pub to: Option<StateChangeType>,
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
    pub bug_url: Option<String>,
    pub custom_fields: Option<CustomFields>,
    pub description: Option<String>,
    pub duplication: Option<bool>,
    pub extra_info: Option<String>,
    pub http_request: Option<String>,
    pub last_transitioned_to_informational_at: Option<String>,
    pub last_transitioned_to_not_applicable_at: Option<String>,
    pub last_transitioned_to_not_reproducible_at: Option<String>,
    pub last_transitioned_to_out_of_scope_at: Option<String>,
    pub last_transitioned_to_resolved_at: Option<String>,
    pub last_transitioned_to_triaged_at: Option<String>,
    pub last_transitioned_to_unresolved_at: Option<String>,
    pub remediation_advice: Option<String>,
    pub severity: Option<u8>,
    pub source: Option<String>,
    pub state: Option<String>,
    pub submitted_at: Option<String>,
    pub title: Option<String>,
    pub vrt_id: Option<String>,
    pub vrt_version: Option<String>,
    pub vulnerability_references: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomFields {}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/webhook/:id", post(webhook_handler));

    // Address that server will bind to.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> &'static str {
    "Hello, World!"
}

async fn webhook_handler(Path(id): Path<String>, Json(body): Json<EventData>) -> String {
    if get_from_env("HASH").unwrap() == id {
        action_webhook_handler(body).await;
        format!("Done")
    } else {
        format!("Unauthorized")
    }
}

async fn action_webhook_handler(body: EventData) -> () {
    let EventData { data, included } = body;
    let Attributes {
        key,
        data: attributes_data,
        created_at: _,
    } = data.attributes;

    let temp_include;
    let safe_include;
    if included.len() > 1 {
        safe_include = &included[1];
    } else {
        temp_include = Included {
            id: "".to_string(),
            r#type: "".to_string(),
            links: HashMap::new(),
            attributes: IncludedAttributes {
                name: None,
                email: None,
                staff: None,
                bug_url: None,
                custom_fields: None,
                description: None,
                duplication: None,
                extra_info: None,
                http_request: None,
                last_transitioned_to_informational_at: None,
                last_transitioned_to_not_applicable_at: None,
                last_transitioned_to_not_reproducible_at: None,
                last_transitioned_to_out_of_scope_at: None,
                last_transitioned_to_resolved_at: None,
                last_transitioned_to_triaged_at: None,
                last_transitioned_to_unresolved_at: None,
                remediation_advice: None,
                severity: None,
                source: None,
                state: None,
                submitted_at: None,
                title: None,
                vrt_id: None,
                vrt_version: None,
                vulnerability_references: None,
            },
            relationships: None,
        };
        safe_include = &temp_include;
    }

    match &*key {
        "blocker.updated" => handle_blocker_updated(attributes_data, safe_include).await,
        "blocker.created" => handle_blocker_created(attributes_data, safe_include).await,
        "submission.created" => handle_submission_created(safe_include).await,
        "submission.updated" => handle_submission_updated(attributes_data, safe_include).await,
        _ => (),
    }
}

async fn handle_blocker_created(attributes_data: AttributesData, included: &Included) {
    let url = create_url_blocker(included);

    match attributes_data.blocked_by {
        Some(blocked_by) if blocked_by == "bugcrowd_operations" => {
            println!("Blocker created for Bugcrowd Operations!");
        }
        Some(blocked_by) if blocked_by == "researcher" => {
            println!("Blocker created for researcher!");
        }
        Some(blocked_by) if blocked_by == "customer" => {
            send_slack_message(
                get_from_env("SLACK_NEW_BLOCKER_CHANNEL").unwrap(),
                "Blocker Created",
                format!("Blocker created for customer!"),
                url,
            )
            .await;
        }
        _ => {
            println!("Unknown blocker creator!");
        }
    }
}

async fn handle_blocker_updated(attributes_data: AttributesData, included: &Included) {
    let url = create_url_blocker(included);
    let message = match attributes_data.blocked_by {
        Some(blocked_by) if blocked_by == "bugcrowd_operations" => {
            "Blocker created for Bugcrowd Operations!".to_string()
        }
        Some(blocked_by) if blocked_by == "researcher" => {
            "Blocker created for researcher!".to_string()
        }
        Some(blocked_by) if blocked_by == "customer" => "Blocker created for customer!".to_string(),
        _ => "Unknown blocker creator!".to_string(),
    };
    send_slack_message(
        get_from_env("SLACK_RESOLVED_BLOCKER_CHANNEL").unwrap(),
        "Blocker Created",
        message,
        url,
    )
    .await;
}

async fn handle_submission_created(included: &Included) {
    let url = create_url_submission(&included.id);
    let title = &included
        .attributes
        .title
        .clone()
        .unwrap_or("Unknown Title".to_string());
    let mut message = String::new();
    if let Some(severity) = &included.attributes.severity {
        message.push_str(&format!("Severity: {}\n", severity));
    }
    send_slack_message(
        get_from_env("SLACK_NEW_SUBMISSION_CHANNEL").unwrap(),
        title,
        message,
        url,
    )
    .await;
}

async fn handle_submission_updated(attributes_data: AttributesData, included: &Included) {
    let url = create_url_submission(&included.id);
    let mut channel = get_from_env("SLACK_PENDING_SUBMISSION_UPDATE_CHANNEL").unwrap();
    if let Some(changes) = &attributes_data.changes {
        if let Some(change) = changes.get("state") {
            if let Some(StateChangeType::String(s)) = &change.to {
                if s == "not-applicable" {
                    channel = get_from_env("SLACK_DUPLICATE_NA_CHANNEL").unwrap();
                }
            }
        }
        if let Some(change) = changes.get("duplicate") {
            if let Some(StateChangeType::Bool(s)) = &change.to {
                if s == &true {
                    channel = get_from_env("SLACK_DUPLICATE_NA_CHANNEL").unwrap();
                }
            }
        }
    }

    let title = &included
        .attributes
        .title
        .clone()
        .unwrap_or("Unknown Title".to_string());
    let message = generate_change_message(&attributes_data, &included);
    send_slack_message(channel, title, message, url).await;
}

fn generate_change_message(attributes: &AttributesData, included: &Included) -> String {
    let mut message = String::new();

    if let Some(severity) = &included.attributes.severity {
        message.push_str(&format!("Severity: {}\n", severity));
    }

    message.push_str("Changes:\n");

    if let Some(changes) = &attributes.changes {
        for (key, change) in changes {
            let from_value = match &change.from {
                Some(StateChangeType::String(s)) => s.clone(),
                Some(StateChangeType::U8(n)) => n.to_string(),
                Some(StateChangeType::Bool(b)) => b.to_string(),
                None => "None".to_string(),
            };
            let to_value = match &change.to {
                Some(StateChangeType::String(s)) => s.clone(),
                Some(StateChangeType::U8(n)) => n.to_string(),
                Some(StateChangeType::Bool(b)) => b.to_string(),
                None => "None".to_string(),
            };

            message.push_str(&format!("{}: {} -> {}\n", key, from_value, to_value));
        }
    }

    message
}

fn create_url_submission(submission_id: &str) -> String {
    format!(
        "https://tracker.bugcrowd.com/{}/submissions/{}",
        get_from_env("BUGCROWD_ORG").unwrap(),
        submission_id
    )
}

fn create_url_blocker(included: &Included) -> String {
    match &included.relationships {
        Some(relations) => match &relations.resource {
            Some(resource) => format!(
                "https://tracker.bugcrowd.com/{}/submissions/{}",
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

async fn send_slack_message(channel: String, title: &str, message: String, url: String) -> () {
    println!("Sending slack message");
    let client = SlackClient::new(SlackClientHyperConnector::new());

    let token_value: SlackApiTokenValue = get_from_env("SLACK_BOT_TOKEN").unwrap().into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let post_chat_req = SlackApiChatPostMessageRequest::new(
        channel.into(),
        SlackMessageContent::new().with_blocks(slack_blocks![
            some_into(SlackHeaderBlock::new(pt!(title
                .chars()
                .take(150)
                .collect::<String>()))),
            some_into(SlackDividerBlock::new()),
            some_into(SlackSectionBlock::new().with_text(md!(message))),
            some_into(SlackActionsBlock::new(slack_blocks![some_into(
                SlackBlockButtonElement::new("view-submission".into(), pt!("View Submission"),)
                    .with_url(Url::parse(&url).unwrap())
            )]))
        ]),
    );

    // let _ = session.chat_post_message(&post_chat_req).await;
    match session.chat_post_message(&post_chat_req).await {
        Ok(response) => {
            println!("Message sent successfully: {:?}", response);
        }
        Err(e) => {
            eprintln!("Failed to send message: {:?}", e);
        }
    }
}
