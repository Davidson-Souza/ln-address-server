#[derive(clap::Parser)]
pub struct Cli {
    /// The password to our phoenix instance. You can find this in "~/.phoenix/phoenix.conf
    pub phoenixd_password: String,
    
    /// The path where we can find users to return
    ///
    /// This should be in absolute path and must exist. Defaults to "./users/"
    #[arg(short, long, value_name = "FILE")]
    pub users_dir: Option<String>,
    
    /// The network address for your phoenixd (only if you've changed it)
    #[arg(short = 'a', long, value_name = "ADDRESS")]
    pub phoenixd_address: Option<String>,
    
    /// The ip address we should listen to
    #[arg(short = 'H', long, value_name = "ADDRESS")]
    pub api_host: Option<String>,

    /// The port we should listen to
    #[arg(short = 'P', long, value_name = "PORT")]
    pub api_port: Option<u16>,
}
