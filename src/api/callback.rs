use std::fmt::Debug;

use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;

use super::config::ServerConfig;
use super::error::ApiError;
use crate::phoenixd::GetInvoiceResponse;

#[derive(Default, Serialize, Deserialize)]
/// The response for a the lnurlpay request. This is returned by the "/callback" endpoint
pub struct LnUrlPayResponse {
    /// The actual invoice
    pr: String,
    /// We don't use this field, but it's required
    routes: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
/// Request sent to phoenixd to get an invoice
pub struct LnUrlPayRequest {
    /// an amount in msats
    amount: u64,
}

#[get("/callback")]
pub async fn ln_url_callback(
    amount: web::Query<LnUrlPayRequest>,
    client: web::Data<ServerConfig>,
) -> Result<impl Responder, ApiError> {
    let amount = amount.into_inner().amount / 1000;
    if amount == 0 {
        return Err(ApiError::AmountTooSmall);
    }

    let amount = format!("{}", amount);
    let values = [
        ("description", "Paying erik's ln address"),
        ("amountSat", &amount),
    ];

    let res = client
        .ph_client
        .client
        .post(format!("http://{}/createinvoice", client.ph_client.host))
        .basic_auth("".to_string(), Some(&client.ph_client.password))
        .form(&values)
        .send()
        .await?
        .text()
        .await?;

    let response: GetInvoiceResponse = serde_json::from_str(&res)?;
    let response = LnUrlPayResponse {
        pr: response.serialized,
        routes: vec![],
    };

    Ok(HttpResponse::Ok().json(response))
}
