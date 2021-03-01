use crate::report;
use log::info;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new(db_path: &PathBuf) -> Self {
        let conn = Connection::open(db_path).expect("Error opening database");

        Self::init_db(&conn);

        Self { conn: conn }
    }

    fn init_db(conn: &Connection) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS report (
                id                    INTEGER PRIMARY KEY,
                report_id             TEXT NOT NULL,
                blob                  BLOB NOT NULL,
                org_name              TEXT NOT NULL,
                email                 TEXT NOT NULL,
                extra_contact_info    TEXT,
                date_begin            TEXT NOT NULL,
                date_end              TEXT NOT NULL,
                policy_domain         TEXT NOT NULL,
                policy_adkim          TEXT,
                policy_aspf           TEXT,
                policy_p              TEXT,
                policy_sp             TEXT,
                policy_pct            INTEGER NOT NULL
                  )",
            params![],
        )
        .unwrap();

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS report_id_index
        on report (report_id)",
            params![],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS record (
                id                      INTEGER PRIMARY KEY,
                report                  TEXT NOT NULL,
                source_ip               TEXT NOT NULL,
                count                   INTEGER NOT NULL,
                policy_ev_disposition   TEXT NOT NULL,
                policy_ev_dkim         TEXT NOT NULL,
                policy_ev_spf          TEXT NOT NULL,
                identifier_header_from  TEXT NOT_NULL,
                auth_dkim_domain        TEXT,
                auth_dkim_result        TEXT,
                auth_dkim_selector      TEXT,
                auth_spf_domain         TEXT,
                auth_spf_result         TEXT
            )",
            params![],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS domains (
                id                  INTEGER PRIMARY KEY,
                domain              TEXT NOT NULL
                )",
            params![],
        )
        .unwrap();

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS domain_index
        on domains (domain)",
            params![],
        )
        .unwrap();
    }

    pub fn insert_report(&mut self, report: &report::Feedback, blob: &Vec<u8>) -> Result<()> {
        // println!("{:#?}", report);
        self.conn
            .execute(
                "INSERT OR IGNORE INTO domains (domain) VALUES (?1)",
                params![report.policy_published.domain.clone().unwrap()],
            )
            .unwrap();

        match self.conn.execute(
            "SELECT report_id FROM report WHERE report_id = '?1'",
            params![report.report_metadata.report_id],
        ) {
            Ok(_num) => {
                info!("Report {} already exists", report.report_metadata.report_id);
                return Ok(());
            }
            Err(_r) => {}
        }

        let tx = self.conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO report (
                report_id,
                blob,
                org_name,
                email,
                extra_contact_info,
                date_begin,
                date_end,
                policy_domain,
                policy_adkim,
                policy_aspf,
                policy_p,
                policy_sp,
                policy_pct
            )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                report.report_metadata.report_id,
                blob,
                report.report_metadata.org_name,
                report.report_metadata.email,
                report.report_metadata.extra_contact_info,
                report.report_metadata.date_range.begin,
                report.report_metadata.date_range.end,
                report.policy_published.domain,
                report.policy_published.adkim,
                report.policy_published.aspf,
                report.policy_published.p,
                report.policy_published.sp,
                report.policy_published.pct,
            ],
        )?;

        for record in &report.record {
            let dkim = &record
                .auth_results
                .dkim
                .clone()
                .unwrap_or(vec![report::DKIM {
                    domain: None,
                    result: None,
                    selector: None,
                }])[0];

            let spf = &record.auth_results.spf.clone().unwrap_or(report::SPF {
                domain: None,
                result: None,
            });

            tx.execute(
                "INSERT INTO record (
                report,
                source_ip,
                count,
                policy_ev_disposition,
                policy_ev_dkim,
                policy_ev_spf,
                identifier_header_from,
                auth_dkim_domain,
                auth_dkim_result,
                auth_dkim_selector,
                auth_spf_domain,
                auth_spf_result
                )
                VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    report.report_metadata.report_id,
                    record.row.source_ip,
                    record.row.count,
                    record.row.policy_evaluated.disposition,
                    record.row.policy_evaluated.dkim,
                    record.row.policy_evaluated.spf,
                    record.identifiers.header_from,
                    dkim.domain,
                    dkim.result,
                    dkim.selector,
                    spf.domain,
                    spf.result,
                ],
            )?;
        }

        tx.commit().unwrap();

        Ok(())
    }
}
