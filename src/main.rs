#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;

use rocket::{Request, State};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

mod config;
mod db;
mod imap_extract;
mod report;

type DbConn = db::DB;

#[derive(Serialize)]
struct TemplateContext {
    title: String,
}

#[derive(Serialize)]
struct TemplateMainContext {
    title: String,
    domains: Vec<String>,
    basic_stats: HashMap<String, db::BasicStats>,
    basic_stats_last_30: HashMap<String, db::BasicStats>,
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
    Template::render("error/404", &map)
}

#[get("/")]
fn index(db_conn: State<DbConn>) -> Template {
    let domains = db::DB::get_domains(&db_conn);
    let basic_stats = db::DB::get_basic_stats(&db_conn, 12000);
    let basic_stats_last_30 = db::DB::get_basic_stats(&db_conn, 30);

    Template::render(
        "index",
        &TemplateMainContext {
            title: String::from("Start"),
            domains,
            basic_stats,
            basic_stats_last_30,
        },
    )
}

#[get("/fetch")]
fn fetch(db_conn: State<DbConn>, config: State<config::Config>) -> Template {
    let imap_extract = imap_extract::ImapExtract::new(&config);

    imap_extract.fetch_reports(&db_conn);
    Template::render(
        "fetched",
        &TemplateContext {
            title: String::from("Fetch"),
        },
    )
}

#[get("/all_reports/<domain>")]
fn all_reports(domain: String, db_conn: State<DbConn>) -> Template {
    Template::render(
        "all_reports",
        &TemplateAllReportsContext {
            title: format!("Report list: {}", domain),
            domain: domain.clone(),
            reports: db::DB::get_all_reports_for_domain(&db_conn, domain),
        },
    )
}

fn rocket() -> rocket::Rocket {
    let config = config::Config::new();
    let conn = db::DB::new(&config.db_path);
    rocket::ignite()
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/", routes![index, fetch, all_reports])
        .register(catchers![not_found])
        .manage(conn)
        .manage(config)
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
