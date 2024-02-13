use clap::Parser;

/// WEBPUSH-SERVER
/// 
/// Make sure to set the `WEBPUSH_AUTH_BEARER_TOKEN` environment variable to a long, secret token.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct Args {
    /// SQLite database filename where subscriptions are stored.
    #[arg(long, default_value="subscriptions.db")]
    pub db: String,

    /// Port the sever will listen on.
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}