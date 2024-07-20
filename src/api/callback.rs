use std::fmt::Debug;

use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;

use super::config::ServerConfig;
use super::error::ApiError;
use crate::nostr::nostr_event::Event;
use crate::nostr::zap_handler::PendingZap;
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
    /// for zaps
    nostr: Option<String>,
}

#[get("/callback")]
pub async fn ln_url_callback(
    amount: web::Query<LnUrlPayRequest>,
    client: web::Data<ServerConfig>,
) -> Result<impl Responder, ApiError> {
    let LnUrlPayRequest { amount, nostr } = amount.into_inner();
    let amount = amount / 1_000;
    if amount == 0 {
        return Err(ApiError::AmountTooSmall);
    }

    let amount = format!("{}", amount);
    let values = [("description", "zap"), ("amountSat", &amount)];

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
    let http_res = LnUrlPayResponse {
        pr: response.serialized,
        routes: vec![],
    };
    if let Some(nostr) = nostr {
        let nostr: Event = ::serde_json::from_str(&nostr)?;
        let zap = PendingZap {
            payment_hash: response.payment_hash,
            bolt11: http_res.pr.clone(),
            receiver: "e0cfb5549d3cf7db4e2736f8e1bc84f62486af7a41295d867c6a313459042528"
                .parse()
                .unwrap(),
            sender: nostr.pubkey.parse().unwrap(),
            event: nostr,
        };

        client.zap_sender.send(zap).await.expect("zap handler died");
    }

    Ok(HttpResponse::Ok().json(http_res))
}
