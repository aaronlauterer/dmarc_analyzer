use crate::report;
use log::info;
use rusqlite::{params, Connection, Result, Transaction, TransactionBehavior};
use std::collections::HashMap;
use std::path::Path;

use std::sync::Mutex;

#[derive(Debug)]
pub struct DB {
    conn: Mutex<Connection>,
}

#[derive(Debug, Serialize)]
pub struct BasicStats {
    dkim_passed: u32,
    spf_passed: u32,
    dkim_failed: u32,
    spf_failed: u32,
}

impl DB {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path).expect("Error opening database");

        Self::init_db(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS report (
                id                    INTEGER PRIMARY KEY,
                report_id             TEXT NOT NULL,
                blob                  BLOB NOT NULL,
                org_name              TEXT NOT NULL,
                email                 TEXT NOT NULL,
                extra_contact_info    TEXT,
                date_begin            INTEGER NOT NULL,
                date_end              INTEGER NOT NULL,
                policy_domain         TEXT NOT NULL,
                policy_adkim          TEXT,
                policy_aspf           TEXT,
                policy_p              TEXT,
                policy_sp             TEXT,
                policy_pct            INTEGER NOT NULL
                  )",
            params![],
        )?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS report_id_index
        on report (report_id)",
            params![],
        )?;

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
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS domains (
                id                  INTEGER PRIMARY KEY,
                domain              TEXT NOT NULL
                )",
            params![],
        )?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS domain_index
        on domains (domain)",
            params![],
        )?;

        Ok(())
    }

    pub fn insert_report(&self, report: &report::Report) -> Result<()> {
        let conn = &self.conn.lock().expect("Could not get DB lock");
        conn.execute(
            "INSERT OR IGNORE INTO domains (domain) VALUES (?1)",
            params![report.policy_domain.clone()],
        )?;

        match conn.execute(
            "SELECT report_id FROM report WHERE report_id = '?1'",
            params![report.report_id],
        ) {
            Ok(_num) => {
                info!("Report {} already exists", report.report_id);
                return Ok(());
            }
            Err(_r) => {}
        }

        let tx = Transaction::new_unchecked(conn, TransactionBehavior::Deferred)?;
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
                report.report_id,
                report.blob,
                report.org_name,
                report.email,
                report.extra_contact_info,
                report.date_begin,
                report.date_end,
                report.policy_domain,
                report.policy_adkim,
                report.policy_aspf,
                report.policy_p,
                report.policy_sp,
                report.policy_pct,
            ],
        )?;

        for record in &report.records {
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
                    report.report_id,
                    record.source_ip,
                    record.count,
                    record.policy_evaluated_disposition,
                    record.policy_evaluated_dkim,
                    record.policy_evaluated_spf,
                    record.identifiers_header_from,
                    record.auth_results_dkim_domain,
                    record.auth_results_dkim_result,
                    record.auth_results_dkim_selector,
                    record.auth_results_spf_domain,
                    record.auth_results_spf_result,
                ],
            )?;
        }

        tx.commit()?;

        Ok(())
    }

    pub fn get_domains(&self) -> Result<Vec<String>> {
        let conn = &self.conn.lock().expect("Could not get DB lock");
        let mut stmt = conn.prepare("SELECT domain FROM domains ORDER BY domain")?;

        let rows = stmt.query_map(params![], |row| row.get(0))?;

        let mut domains = Vec::new();
        for domain_result in rows {
            domains.push(domain_result?);
        }
        Ok(domains)
    }

    pub fn get_basic_stats(&self, last_days: u16) -> Result<HashMap<String, BasicStats>> {
        let domains = Self::get_domains(&self)?;

        #[derive(Debug)]
        struct ResRow {
            count: u32,
            domain: String,
        }

        let mut stats = HashMap::new();
        for domain in domains {
            stats.entry(domain).or_insert(BasicStats {
                dkim_passed: 0,
                spf_passed: 0,
                dkim_failed: 0,
                spf_failed: 0,
            });
        }

        let conn = &self.conn.lock().expect("Could not get DB lock");
        let mut stmt = conn
            .prepare(
                "SELECT
                report.policy_domain,
                sum(record.count)
            FROM report
            LEFT OUTER JOIN record
            ON record.report = report.report_id
            WHERE record.policy_ev_dkim = 'pass'
            AND date(report.date_begin, 'unixepoch') >= date('now', ?)
            GROUP BY report.policy_domain",
        )?;
        let rows = stmt
            .query_map(params![format!("-{} days", last_days)], |row| {
                Ok(ResRow {
                    count: row.get(1)?,
                    domain: row.get(0)?,
                })
            })
        })?;

        for row in rows {
            let d = row?;
            if let Some(cur) = stats.get_mut(&d.domain) {
                cur.dkim_passed = d.count;
            }
        }

        let mut stmt = conn
            .prepare(
                "SELECT
                report.policy_domain,
                sum(record.count)
            FROM report
            LEFT OUTER JOIN record
            ON record.report = report.report_id
            WHERE record.policy_ev_spf = 'pass'
            AND date(report.date_begin, 'unixepoch') >= date('now', ?)
            GROUP BY report.policy_domain",
        )?;
        let rows = stmt
            .query_map(params![format!("-{} days", last_days)], |row| {
                Ok(ResRow {
                    count: row.get(1)?,
                    domain: row.get(0)?,
                })
            })
        })?;

        for row in rows {
            let d = row?;
            if let Some(cur) = stats.get_mut(&d.domain) {
                cur.spf_passed = d.count;
            }
        }

        let mut stmt = conn
            .prepare(
                "SELECT
                report.policy_domain,
                sum(record.count)
            FROM report
            LEFT OUTER JOIN record
            ON record.report = report.report_id
            WHERE record.policy_ev_dkim != 'pass'
            AND date(report.date_begin, 'unixepoch') >= date('now', ?)
            GROUP BY report.policy_domain",
        )?;
        let rows = stmt
            .query_map(params![format!("-{} days", last_days)], |row| {
                Ok(ResRow {
                    count: row.get(1)?,
                    domain: row.get(0)?,
                })
            })
        })?;

        for row in rows {
            let d = row?;
            if let Some(cur) = stats.get_mut(&d.domain) {
                cur.dkim_failed = d.count;
            }
        }

        let mut stmt = conn
            .prepare(
                "SELECT
                report.policy_domain,
                sum(record.count)
            FROM report
            LEFT OUTER JOIN record
            ON record.report = report.report_id
            WHERE record.policy_ev_spf != 'pass'
            AND date(report.date_begin, 'unixepoch') >= date('now', ?)
            GROUP BY report.policy_domain",
        )?;
        let rows = stmt
            .query_map(params![format!("-{} days", last_days)], |row| {
                Ok(ResRow {
                    count: row.get(1)?,
                    domain: row.get(0)?,
                })
            })
        })?;

        for row in rows {
            let d = row?;
            if let Some(cur) = stats.get_mut(&d.domain) {
                cur.spf_failed = d.count;
            }
        }

        Ok(stats)
    }

    pub fn get_all_reports_for_domain(&self, domain: String) -> Result<Vec<report::Report>> {
        let mut reports: Vec<report::Report> = Vec::new();

        let report_ids = Self::get_report_ids_for_domain(&self, domain)?;

        for report in report_ids {
            let id = report;
            reports.push(Self::get_report(&self, id)?);
        }

        Ok(reports)
    }

    fn get_report_ids_for_domain(&self, domain: String) -> Result<Vec<String>> {
        let conn = &self.conn.lock().expect("Could not get DB lock");

        let mut stmt = conn
            .prepare(
                "SELECT report_id
                     FROM report
                     WHERE policy_domain = ?
                     ORDER BY date_begin DESC",
        )?;
        let reports_iter = stmt.query_map(params![domain], |row| Ok(row.get(0)))?;

        let mut report_ids: Vec<String> = Vec::new();
        for report in reports_iter {
            let id = report??;
            report_ids.push(id);
        }

        Ok(report_ids)
    }

    pub fn get_report(&self, report_id: String) -> Result<report::Report> {
        let conn = &self.conn.lock().expect("Could not get DB lock");

        let mut records: Vec<report::Record> = Vec::new();

        let mut stmt = conn
            .prepare(
                "SELECT
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
            FROM record
            WHERE report = ?",
        )?;

        let record_iter = stmt.query_map(params![report_id], |row| {
            Ok(report::Record {
                source_ip: row.get(0)?,
                count: row.get(1)?,
                policy_evaluated_disposition: row.get(2)?,
                policy_evaluated_dkim: row.get(3)?,
                policy_evaluated_spf: row.get(4)?,
                identifiers_header_from: row.get(5)?,
                auth_results_dkim_domain: row.get(6)?,
                auth_results_dkim_result: row.get(7)?,
                auth_results_dkim_selector: row.get(8)?,
                auth_results_spf_domain: row.get(9)?,
                auth_results_spf_result: row.get(10)?,
            })
        })?;

        for record in record_iter {
            records.push(record?);
        }

        conn.query_row(
            "SELECT
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
            FROM report
            WHERE report_id = ?",
            params![report_id],
            |row| {
                Ok(report::Report {
                    blob: row.get(0)?,
                    org_name: row.get(1)?,
                    email: row.get(2)?,
                    extra_contact_info: row.get(3)?,
                    report_id: report_id.clone(),
                    date_begin: row.get(4)?,
                    date_end: row.get(5)?,
                    policy_domain: row.get(6)?,
                    policy_adkim: row.get(7)?,
                    policy_aspf: row.get(8)?,
                    policy_p: row.get(9)?,
                    policy_sp: row.get(10)?,
                    policy_pct: row.get(11)?,
                    records,
                })
            },
        )
    }
}
