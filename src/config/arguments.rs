use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// DMARC Analyzer
///
/// Fetches DMARC reports from IMAP accounts and analyzes them
pub struct Opt {
    #[structopt(long)]
    /// Path to the config file. Defaults to 'config.cfg'
    pub config: Option<String>,

    #[structopt(long, parse(from_os_str))]
    /// Path to the database file. Defaults to 'data.db'
    pub db_path: Option<PathBuf>,

    #[structopt(long)]
    /// Imap user
    pub user: Option<String>,

    #[structopt(long)]
    /// Imap password
    pub password: Option<String>,

    #[structopt(long)]
    /// Imap server
    pub server: Option<String>,

    #[structopt(long)]
    /// Imap server port. Defaults to '993'
    pub port: Option<u16>,

    #[structopt(long)]
    /// The IMAP folder where to place report mails once processed.
    pub store_folder: Option<String>,
}
