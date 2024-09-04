use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use hex_conservative::DisplayHex;
use reqwest::Client;
use secp256k1::Secp256k1;
use secp256k1::SecretKey;
use secp256k1::XOnlyPublicKey;
use serde_json::json;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::connection::WebsocketConnection;
use super::nostr_event::Event;
use crate::nostr::nostr_event::UnsignedEvent;
use crate::phoenixd::PhoenixdClient;

/// The context for our zap handler.
pub struct ZapHandler {
    /// Every zap requires an event called "zap receipt", that should be published by the lnaddress
    /// server, after it see the payment. As a nostr note, it needs a pubkey and signature to work.
    /// This is the secret key we use to sign those receipts.
    secret_key: SecretKey,
    /// This is the public part of "secret_key", as a string. We use this to build the receipt
    /// event
    public_key_str: String,
    /// This channel weill receive the zap requests from the http server
    receiver: Receiver<PendingZap>,
    /// Our phoenixd we'll use to probe the invoice
    phoenixd: PhoenixdClient,
    /// A list of connected relays that we'll use to send the zap receipts
    connected_relays: Vec<WebsocketConnection>,
    /// A channel with messages from our connected relays
    relays_receiver: Receiver<(usize, Message)>,
    /// The sender we give to relays to connect with us, we keep it here for every time we create a
    /// new connection
    relays_sender: Sender<(usize, Message)>,
    /// Inflight zaps that haven't being paid yet
    inflight: Vec<(PendingZap, Instant)>,
    /// The id of the latest connection we've created
    ids: usize,
}

/// A zap that was requested but haven't being paid yet
pub struct PendingZap {
    /// THe payeer for this zap
    pub sender: XOnlyPublicKey,
    /// THe payee for this zap
    pub receiver: XOnlyPublicKey,
    /// a bolt11 invoice that should be paid for this zap
    pub bolt11: String,
    /// A hash used to identify this payment
    pub payment_hash: String,
    /// The zap request event
    pub event: Event,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IncomingPaymentInfo {
    #[serde(rename = "paymentHash")]
    payment_hash: String,
    preimage: String,
    description: String,
    invoice: String,
    #[serde(rename = "isPaid")]
    is_paid: bool,
    #[serde(rename = "receivedSat")]
    received_sat: u64,
    fees: u64,
    #[serde(rename = "createdAt")]
    created_at: u64,
}

// for now, just use a hard-coded list of relays
const RELAYS: [&str; 4] = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://nostr.mom",
    "wss://nostr.dlsouza.lol",
];

impl ZapHandler {
    pub async fn new(
        phoenixd: PhoenixdClient,
        secret_key: SecretKey,
    ) -> (Self, Sender<PendingZap>) {
        let (sender, receiver) = channel(1024);
        let (ws_send, ws_recv) = channel(1024);

        let mut connected_relays = vec![];
        let mut ids = 0;
        for relay_url in RELAYS {
            let relay = WebsocketConnection::new(ids, relay_url.into(), ws_send.clone()).await;
            if let Ok(relay) = relay {
                connected_relays.push(relay);
            }
            ids += 1;
        }

        (
            Self {
                ids,
                relays_sender: ws_send,
                relays_receiver: ws_recv,
                connected_relays,
                phoenixd,
                receiver,
                inflight: Vec::new(),
                public_key_str: secret_key
                    .x_only_public_key(&Secp256k1::new())
                    .0
                    .serialize()
                    .to_lower_hex_string(),
                secret_key,
            },
            sender,
        )
    }

    async fn check_pending_zaps(&mut self) -> reqwest::Result<()> {
        // remove old zap receipts
        self.inflight.retain(|zap| zap.1.elapsed().as_secs() < 3600);
        let client = Client::default();
        let mut paid_or_expired = Vec::new();

        for (event, when) in self.inflight.iter() {
            let res = client
                .get(format!(
                    "http://{}/payments/incoming/{}",
                    self.phoenixd.host, event.payment_hash
                ))
                .basic_auth("".to_string(), Some(&self.phoenixd.password))
                .send()
                .await?
                .text()
                .await?;

            let res: IncomingPaymentInfo =
                ::serde_json::from_str(&res).expect("Phoenixd is doing something funky");
            if when.elapsed().as_secs() > 3600 {
                paid_or_expired.push(event.payment_hash.clone());
            }

            if !res.is_paid {
                continue;
            }

            paid_or_expired.push(event.payment_hash.clone());

            let payee = event.receiver.serialize().to_lower_hex_string();
            let payer = &event.event.pubkey;
            let e_tag = event
                .event
                .tags
                .iter()
                .position(|tag| tag[0] == "e".to_owned());

            let mut zap_receipt = UnsignedEvent {
                content: "".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                kind: 9735,
                pubkey: self.public_key_str.clone(),
                tags: vec![
                    vec!["p".to_string(), payee],
                    vec!["P".to_string(), payer.to_owned()],
                    vec!["relays".to_string(), "wss://nostr.dlsouza.lol".to_string()],
                    vec![
                        "lnurl".to_string(),
                        "https://dlsouza.lol/callback".to_string(),
                    ],
                    vec!["amount".to_string(), res.received_sat.to_string()],
                    vec![
                        "description".to_string(),
                        ::serde_json::to_string(&event.event).unwrap(),
                    ],
                    vec!["bolt11".to_string(), event.bolt11.clone()],
                ],
            };

            if e_tag.is_some() {
                zap_receipt
                    .tags
                    .push(event.event.tags[e_tag.unwrap()].clone());
            }

            let zap_receipt = zap_receipt.into_signed(&self.secret_key);

            let text_zap = json!(["EVENT", zap_receipt]).to_string();
            let ws_msg = Message::text(text_zap);
            for relay in self.connected_relays.iter_mut() {
                if let Err(e) = relay.write_to_connection(ws_msg.clone()).await {
                    println!("{e:?}");
                }
            }
        }

        self.inflight
            .retain(|event| !paid_or_expired.contains(&event.0.payment_hash));

        Ok(())
    }

    pub fn remove_older_zap(&mut self) {
        self.inflight.sort_by(|first, second| {
            first
                .1
                .elapsed()
                .as_secs()
                .cmp(&second.1.elapsed().as_secs())
        });
        self.inflight.pop();
    }

    pub async fn run(mut self) {
        loop {
            if let Err(e) = self.check_pending_zaps().await {
                println!("{e:?}");
            }

            if let Ok(msg) = self.relays_receiver.try_recv() {
                match msg.1 {
                    Message::Close(_) => {
                        let relay = self
                            .connected_relays
                            .iter()
                            .position(|r| r.id() == msg.0)
                            .expect("got disconnect from a relay not connected?");
                        let relay = self.connected_relays.remove(relay);

                        let relay = WebsocketConnection::new(
                            self.ids,
                            relay.address(),
                            self.relays_sender.clone(),
                        )
                        .await;

                        if let Ok(relay) = relay {
                            self.connected_relays.push(relay);
                        }
                    }

                    _ => {}
                }
            }

            let Ok(Some(event)) = timeout(Duration::from_secs(1), self.receiver.recv()).await
            else {
                continue;
            };
            if self.inflight.len() > 1_000 {
                self.remove_older_zap();
            }
            self.inflight.push((event, Instant::now()));
        }
    }
}
