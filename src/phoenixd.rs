use reqwest::Client;

#[derive(Clone)]
/// A struct that holds all data needed to connect with a running phoenixd,
/// the actual lightning wallet powering this application
pub struct PhoenixdClient {
    /// A reqwest client used to send phoenixd requests
    pub client: Client,
    /// The password we use to authenticate with phoenixd.
    ///
    /// You can find this in $PHOENIXD_DATA_DIR/phoenixd.conf
    pub password: String,
    /// The host where phoenixd is running
    pub host: String
}

#[derive(Default, Serialize, Deserialize)]
/// Data returned from phoenixd when we call "getInvoice"
///
/// The most importanti info here is the "serialized" field, that contains the bolt11
/// invoice
pub struct GetInvoiceResponse {
    #[serde(rename = "amountSat")]
    /// The invoice amount, in sats
    pub amount_sat: u64,
    #[serde(rename = "paymentHash")]
    /// The payment hash for this invoice.
    ///
    /// We may need this information to keep track of whether this invoice was paid or not
    pub payment_hash: String,
    /// The actual bolt11 invoice
    pub serialized: String,
}
