use anyhow::{anyhow, Context, Result};
use libflate::gzip::Decoder;
use mailparse::*;
use native_tls::TlsConnector;
use serde_xml_rs::from_reader;
use std::collections::HashMap;
use std::io::prelude::*;
use std::path::PathBuf;
use zip::ZipArchive;

use crate::config::Config;
use crate::db;
use crate::report;
use crate::report::serde_defs;

extern crate libflate;

struct Attachment {
    content: Vec<u8>,
    decompressed: Option<Vec<u8>>,
    mimetype: String,
    name: String,
}

const USABLE_MIMETYPES: [&str; 3] = [
    "application/zip",
    "application/gzip",
    "application/octet-stream",
];

#[derive(Debug)]
pub struct ImapExtract {
    server: String,
    port: u16,
    user: String,
    password: String,
    store_folder: String,
}

impl ImapExtract {
    pub fn new(config: &Config) -> Self {
        Self {
            server: config.server.clone(),
            port: config.port,
            user: config.user.clone(),
            password: config.password.clone(),
            store_folder: config.store_folder.clone(),
        }
    }

    pub fn fetch_reports(self, database: &db::DB, logbuf: &mut Vec<u8>) -> Result<()> {
        writeln!(logbuf, "Starting to fetch reports!")?;
        let tls = TlsConnector::builder().build()?;
        let client = imap::connect(
            (self.server.clone(), self.port as u16),
            self.server.clone(),
            &tls,
        )
        .context("Error connecting to server")?;
        let mut imap_session = client.login(self.user, self.password).map_err(|e| e.0)?;

        match imap_session.select(format!("INBOX/{}", self.store_folder)) {
            Ok(_o) => {}
            Err(_e) => {
                writeln!(logbuf, "Creating store folder: {}", self.store_folder)?;
                imap_session
                    .create(format!("INBOX/{}", self.store_folder))
                    .context("Failed to create store folder")?
            }
        };

        let inbox = imap_session
            .select("INBOX")
            .context("Failed to select INBOX")?;
        let message_count = inbox.exists;

        if message_count == 0 {
            writeln!(logbuf, "No messages found. Finished")?;
            return Ok(());
        }

        let messages = imap_session.fetch("1:*", "RFC822")?;
        let mut log_each_msg = message_count / 20;
        if log_each_msg == 0 {
            log_each_msg = 20;
        }

        let mut count = 0;
        let mut fetch_stats = HashMap::new();
        for message in messages.iter() {
            if count % log_each_msg == 0 {
                writeln!(
                    logbuf,
                    "{:.0} % done",
                    100.00 / message_count as f32 * message.message as f32
                )?;
            }
            count += 1;

            if let Some(body) = message.body() {
                let mail = parse_mail(body)?;
                let message_id = mail.headers.get_first_value("Message-ID").unwrap();

                let attachment = match Self::get_attachment(&mail) {
                    Ok(attachment) => attachment,
                    Err(e) => {
                        writeln!(logbuf, "{} Message: {}", e, message_id)?;
                        continue;
                    }
                };

                let attachment = Self::decompress_attachment(attachment)?;

                let parsed_report: serde_defs::Feedback = from_reader(std::io::Cursor::new(
                    &attachment.decompressed.clone().unwrap(),
                ))
                .unwrap();
                let report = report::Report::from_with_blob(
                    parsed_report,
                    Some(attachment.decompressed.unwrap()),
                );

                match database.insert_report(&report) {
                    Ok(_o) => {
                        let count = fetch_stats
                            .entry(report.policy_domain.unwrap())
                            .or_insert(0);
                        *count += 1;
                    }
                    Err(e) => {
                        writeln!(
                            logbuf,
                            "{} -- Report: '{}' - Organisation: '{}' ",
                            e, report.report_id, report.org_name
                        )?;
                        if e.to_string() != "UNIQUE constraint failed: report.report_id" {
                            continue;
                        }
                    }
                };
                // not every IMAP server supports MOVE
                imap_session.copy(&count.to_string(), format!("INBOX/{}", self.store_folder))?;
                imap_session.store(&count.to_string(), "+FLAGS (\\DELETED)")?;
            }
        }
        imap_session.expunge()?;
        writeln!(logbuf, "100 % done")?;
        if !fetch_stats.is_empty() {
            writeln!(logbuf, "----------")?;
            writeln!(logbuf, "Imported:")?;
            for (domain, val) in fetch_stats.iter() {
                writeln!(logbuf, "{} -> {}", domain, val)?;
            }
        }
        imap_session.logout()?;

        Ok(())
    }

    fn decompress_attachment(mut attachment: Attachment) -> Result<Attachment> {
        // Decompresses the attachment, saves it in te Attachment struct and returns it
        let content = std::io::Cursor::new(&attachment.content);
        let mut decompressed: Vec<u8> = Vec::new();
        // TODO: add function that determines type better, e.g. check file extension if mimetype is
        // octect stream
        if attachment.mimetype == *"application/zip" {
            let mut zip = ZipArchive::new(content).unwrap();
            let mut report = zip.by_index(0)?;
            std::io::copy(&mut report, &mut decompressed)?;
            attachment.name = String::from(report.name());
        } else if attachment.mimetype == *"application/gzip"
            || attachment.mimetype == *"application/octet-stream"
        {
            let mut report = Decoder::new(content).unwrap();
            std::io::copy(&mut report, &mut decompressed)?;
            let mut path = PathBuf::from(attachment.name.clone());
            path = path.with_extension("");
            attachment.name = String::from(path.to_str().unwrap());
        }
        attachment.decompressed = Some(decompressed);

        Ok(attachment)
    }

    fn get_attachment(mail: &ParsedMail) -> Result<Attachment> {
        // Extracts the attachment from the mail

        let mut content_type = mail.ctype.mimetype.clone();
        let mut body: Vec<u8> = vec![];
        let mut name = String::new();

        if USABLE_MIMETYPES.contains(&content_type.as_str()) {
            body = mail.get_body_raw().unwrap();
            name = mail
                .get_content_disposition()
                .params
                .get("filename")
                .unwrap()
                .clone();
        } else if !mail.subparts.is_empty() {
            for subpart in &mail.subparts {
                content_type = subpart.ctype.mimetype.clone();
                if USABLE_MIMETYPES.contains(&content_type.as_str()) {
                    body = subpart.get_body_raw()?;
                    name = subpart
                        .get_content_disposition()
                        .params
                        .get("filename")
                        .unwrap()
                        .clone();
                    break;
                }
            }
        }

        if body.is_empty() {
            return Err(anyhow!("No attachment found."));
        }
        if name.is_empty() {
            return Err(anyhow!("No file name found."));
        }

        Ok(Attachment {
            content: body,
            decompressed: None,
            name,
            mimetype: content_type,
        })
    }
}
