use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde::Deserialize;
use serde::Serialize;

use super::config::ServerConfig;
use super::error::ApiError;

#[derive(Default, Serialize, Deserialize)]
/// Data returned to the ".well-known/lnurlp/{username}" endpoint
pub struct LnAddressInfo {
    #[serde(rename = "maxSendable")]
    /// Maximum amount in milisats this user can receive
    max_sendable: u64,
    #[serde(rename = "minSendable")]
    /// Minimum amount in milisats this user can receive
    min_sendable: u64,
    /// What type of lnurl is this? We only do payRequest for now
    tag: String,
    /// The address that should be called to obtain an invoice
    callback: String,
    /// A stringfyed json with some metadata about ourselves
    metadata: String,
    /// Do we support nostr?
    #[serde(rename = "allowsNostr")]
    allows_nostr: bool,
    /// the server's public key
    #[serde(rename = "nostrPubkey")]
    nostr_pubkey: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct UserData {
    /// The address that should be called to obtain an invoice
    callback: String,
    /// A stringfyed json with some metadata about ourselves
    metadata: String,
}

#[get("/.well-known/lnurlp/{user}")]
pub async fn well_known(
    user: web::Path<String>,
    app_data: web::Data<ServerConfig>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();

    // don't allow non-ascii string
    if !user.is_ascii() {
        return Err(ApiError::NonAsciiString);
    }

    // don't allow names with more than 20 chars
    if user.len() > 20 {
        return Err(ApiError::StringTooLong);
    }

    // check for any non-alphanumeric chars
    let all_alph = user
        .chars()
        .all(|ch| (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'A') || (ch >= '0' && ch <= '9'));

    if !all_alph {
        return Err(ApiError::InvalidString);
    }

    let users_dir = &app_data.as_ref().users_dir;
    let user = std::fs::read_to_string(format!("{users_dir}/{user}"))
        .map(|user| serde_json::from_str::<UserData>(&user))
        .map_err(|_| ApiError::UnknownUser)??;

    let response = LnAddressInfo {
        tag: "payRequest".into(),
        callback: user.callback,
        metadata: user.metadata,
        max_sendable: 10_000_000,
        min_sendable: 1,
        nostr_pubkey: app_data.as_ref().zap_pk.clone(),
        allows_nostr: true,
    };

    Ok(HttpResponse::Ok().json(response))
}
