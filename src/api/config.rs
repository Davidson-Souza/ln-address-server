use crate::phoenixd::PhoenixdClient;

#[derive(Clone)]
/// General configuration and state for our server
///
/// THis struct will be used as the app data inside the server, so we'll aways
/// have access to it inside every endpoint
pub struct ServerConfig {
    /// A phoenixd client we use to talk with phoenixd
    pub ph_client: PhoenixdClient,
    /// Where we can find our user's data
    pub users_dir: String,
    /// The ip and port the API should listen to
    pub host: String,
}
