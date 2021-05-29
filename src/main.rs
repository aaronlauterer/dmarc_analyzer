#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;

use chrono::{Duration, Utc};
use rocket::{Request, State};
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

mod config;
mod db;
mod imap_extract;
mod report;

type DbConn = db::DB;
type BasicStats = HashMap<String, db::BasicStats>;
type PolicyEvStats = HashMap<String, HashMap<String, db::PolicyEvaluatedStats>>;

#[derive(Serialize)]
struct FetchTask {
    log: String,
    error: String,
}

#[derive(Serialize)]
struct TemplateFetchContext {
    title: String,
}

#[derive(Serialize)]
struct TemplateMainContext {
    title: String,
    now: String,
    now30_ago: String,
    domains: Vec<String>,
    basic_stats: BasicStats,
    basic_stats_last_30: BasicStats,
    policy_ev_stats_last_30: PolicyEvStats,
}

#[derive(Serialize)]
struct TemplateAllReportsContext {
    title: String,
    domain: String,
    reports: Vec<report::Report>,
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = std::collections::HashMap::new();
    map.insert("path", req.uri().path());
    map.insert("title", "404 - not found");
    Template::render("error/404", &map)
}

#[get("/")]
fn index(db_conn: State<DbConn>) -> Template {
    let domains = db::DB::get_domains(&db_conn).expect("get domains");
    let basic_stats = db::DB::get_basic_stats(&db_conn, 12000).expect("get basic stats");
    let basic_stats_last_30 =
        db::DB::get_basic_stats(&db_conn, 30).expect("get basic last 30 stats");
    let policy_ev_stats_last_30 =
        db::DB::get_policy_evaluated_stats(&db_conn, 30).expect("get basic last 30 stats");

    let now = Utc::now();
    let now30_ago = now - Duration::days(30);

    Template::render(
        "index",
        &TemplateMainContext {
            title: String::from("Start"),
            now: now.format("%Y-%m-%d").to_string(),
            now30_ago: now30_ago.format("%Y-%m-%d").to_string(),
            domains,
            basic_stats,
            basic_stats_last_30,
            policy_ev_stats_last_30,
        },
    )
}

#[get("/fetch")]
fn fetch() -> Template {
    Template::render(
        "fetched",
        &TemplateFetchContext {
            title: String::from("Fetch"),
        },
    )
}

#[get("/fetchdata")]
fn fetchdata(db_conn: State<DbConn>, config: State<config::Config>) -> Json<FetchTask> {
    let imap_extract = imap_extract::ImapExtract::new(&config);
    let mut error = String::new();
    let mut logbuf = Vec::new();

    match imap_extract.fetch_reports(&db_conn, &mut logbuf) {
        Ok(_o) => {}
        Err(e) => error = format!("{:#}", e),
    };
    Json(FetchTask {
        log: String::from_utf8(logbuf).expect("get fetch log"),
        error,
    })
}

#[get("/all_reports/<domain>")]
fn all_reports(domain: String, db_conn: State<DbConn>) -> Template {
    Template::render(
        "all_reports",
        &TemplateAllReportsContext {
            title: format!("Report list: {}", domain),
            domain: domain.clone(),
            reports: db::DB::get_all_reports_for_domain(&db_conn, domain)
                .expect("get all reports for domain"),
        },
    )
}

fn rocket() -> rocket::Rocket {
    let config = config::Config::new();
    let conn = db::DB::new(&config.db_path).expect("get db conn");
    rocket::ignite()
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/", routes![index, fetch, fetchdata, all_reports])
        .register(catchers![not_found])
        .manage(conn)
        .manage(config)
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
