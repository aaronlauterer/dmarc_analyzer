pub mod serde_defs;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Report {
    pub blob: Option<Vec<u8>>,
    pub org_name: String,
    pub email: String,
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_begin: i64,
    pub date_end: i64,
    pub policy_domain: Option<String>,
    pub policy_adkim: Option<String>,
    pub policy_aspf: Option<String>,
    pub policy_p: Option<String>,
    pub policy_sp: Option<String>,
    pub policy_pct: Option<i8>,
    pub records: Vec<Record>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Record {
    pub source_ip: String,
    pub count: i32,
    pub policy_evaluated_disposition: String,
    pub policy_evaluated_dkim: String,
    pub policy_evaluated_spf: String,
    pub identifiers_header_from: String,
    pub auth_results_dkim_domain: Option<String>,
    pub auth_results_dkim_result: Option<String>,
    pub auth_results_dkim_selector: Option<String>,
    pub auth_results_spf_domain: Option<String>,
    pub auth_results_spf_result: Option<String>,
}

impl From<serde_defs::Feedback> for Report {
    fn from(feedback: serde_defs::Feedback) -> Report {
        Report::from_with_blob(feedback, None)
    }
}

impl Report {
    pub fn from_with_blob(feedback: serde_defs::Feedback, blob: Option<Vec<u8>>) -> Report {
        let mut records = Vec::new();

        for i in &feedback.record {
            let dkim = &i.auth_results.dkim.clone().unwrap_or_else(|| {
                vec![serde_defs::Dkim {
                    domain: None,
                    result: None,
                    selector: None,
                }]
            })[0];
            let spf = &i.auth_results.spf.clone().unwrap_or(serde_defs::Spf {
                domain: None,
                result: None,
            });
            records.push(Record {
                source_ip: i.row.source_ip.clone(),
                count: i.row.count,
                policy_evaluated_disposition: i.row.policy_evaluated.disposition.clone(),
                policy_evaluated_dkim: i.row.policy_evaluated.dkim.clone(),
                policy_evaluated_spf: i.row.policy_evaluated.spf.clone(),
                identifiers_header_from: i.identifiers.header_from.clone(),
                auth_results_dkim_domain: dkim.domain.clone(),
                auth_results_dkim_result: dkim.result.clone(),
                auth_results_dkim_selector: dkim.selector.clone(),
                auth_results_spf_domain: spf.domain.clone(),
                auth_results_spf_result: spf.result.clone(),
            });
        }

        Report {
            blob,
            org_name: feedback.report_metadata.org_name,
            email: feedback.report_metadata.email,
            extra_contact_info: feedback.report_metadata.extra_contact_info,
            report_id: feedback.report_metadata.report_id,
            date_begin: feedback.report_metadata.date_range.begin,
            date_end: feedback.report_metadata.date_range.end,
            policy_domain: feedback.policy_published.domain,
            policy_adkim: feedback.policy_published.adkim,
            policy_aspf: feedback.policy_published.aspf,
            policy_p: feedback.policy_published.p,
            policy_sp: feedback.policy_published.sp,
            policy_pct: feedback.policy_published.pct,
            records,
        }
    }
}
