#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Feedback {
    pub report_metadata: ReportMetadata,
    pub policy_published: PolicyPublished,
    pub record: Vec<Record>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ReportMetadata {
    pub org_name: String,
    pub email: String,
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_range: DateRange,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DateRange {
    pub begin: i64,
    pub end: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct PolicyPublished {
    pub domain: Option<String>,
    pub adkim: Option<String>,
    pub aspf: Option<String>,
    pub p: Option<String>,
    pub sp: Option<String>,
    pub pct: Option<i8>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Record {
    pub row: Row,
    pub identifiers: Identifiers,
    pub auth_results: AuthResults,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Row {
    pub source_ip: String,
    pub count: i32,
    pub policy_evaluated: PolicyEvaluated,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct PolicyEvaluated {
    pub disposition: String,
    pub dkim: String,
    pub spf: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Identifiers {
    pub header_from: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AuthResults {
    pub dkim: Option<Vec<Dkim>>,
    pub spf: Option<Spf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Dkim {
    pub domain: Option<String>,
    pub result: Option<String>,
    pub selector: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Spf {
    pub domain: Option<String>,
    pub result: Option<String>,
}
