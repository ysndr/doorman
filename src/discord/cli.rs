
use clap::Clap;
#[derive(Clap, Debug, Clone)]
pub struct Args {
    /// Discord UserID
    #[clap(short, long, env="DISCORD_USER_ID")]
    pub user: u64,

    /// Discord Bot Token
    #[clap(short, long, env="DISCORD_TOKEN")]
    pub token: String,
}
