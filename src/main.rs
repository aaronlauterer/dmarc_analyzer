mod config;
mod db;
mod imap_extract;
mod report;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let config = config::Config::new();
    let db = db::DB::new(&config.db_path);
    println!("Before imap extract creation");
    let imap_extract = imap_extract::ImapExtract::new(&config);
    println!("after imap extract creation");

    imap_extract.fetch_reports(db);

    Ok(())
}
