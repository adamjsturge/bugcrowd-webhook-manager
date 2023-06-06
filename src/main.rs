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
    let Attributes {
        key,
        data: attributes_data,
        created_at: _,
    } = data.attributes;

    let temp_include;
    let safe_included;
    if included.len() > 1 {
        safe_included = &included[1];
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
        safe_included = &temp_include;
    }

    match &*key {
        // "blocker.updated" => handle_blocker_updated(attributes_data).await,
        "blocker.created" => handle_blocker_created(attributes_data, safe_included).await,
        "submission.created" => handle_submission_created(attributes_data, safe_included).await,
        "submission.updated" => handle_submission_updated(attributes_data, safe_included).await,
        _ => (),
    }
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

async fn handle_blocker_created(attributes_data: AttributesData, included: &Included) {
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
        "#bugcrowd-rust-info",
        "Blocker Created".to_string(),
        message,
        url,
    )
    .await;
}

async fn handle_submission_created(attributes_data: AttributesData, included: &Included) {
    let url = create_url_submission(&included.id);
    // let action = format!("Submission Created ")
    send_slack_message(
        "#bugcrowd-rust-info",
        "Submission Created".to_string(),
        "Created ya".to_string(),
        url,
    )
    .await;
}

async fn handle_submission_updated(attributes_data: AttributesData, included: &Included) {
    let url = create_url_submission(&included.id);
    let action = format!(
        "Submission Updated: Severity {} ",
        &included.attributes.severity.unwrap_or(0)
    );
    // let message =
    send_slack_message("#bugcrowd-rust-info", action, "Updated ya".to_string(), url).await;
}

async fn send_slack_message(
    channel: &'static str,
    action: String,
    message: String,
    url: String,
) -> () {
    let client = SlackClient::new(SlackClientHyperConnector::new());

    let token_value: SlackApiTokenValue = get_from_env("SLACK_BOT_TOKEN").unwrap().into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let post_chat_req = SlackApiChatPostMessageRequest::new(
        channel.into(),
        SlackMessageContent::new().with_blocks(slack_blocks![
            some_into(SlackHeaderBlock::new(pt!(action))),
            some_into(SlackDividerBlock::new()),
            some_into(SlackSectionBlock::new().with_text(md!(message))),
            some_into(SlackActionsBlock::new(slack_blocks![some_into(
                SlackBlockButtonElement::new("view-submission".into(), pt!("View Submission"),)
                    .with_url(Url::parse(&url).unwrap())
            )]))
        ]),
    );

    let _ = session.chat_post_message(&post_chat_req).await;
}
