pub mod arguments;
use configparser::ini::Ini;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub db_path: std::path::PathBuf,
    pub server: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub store_folder: String,
}

impl Config {
    pub fn new() -> Self {
        let args = arguments::Opt::from_args();
        let mut config_path = String::from("config.cfg");

        if args.config.is_some() {
            config_path = args.config.clone().unwrap();
        }

        let mut config_file = Ini::new();
        config_file.load(config_path.as_str()).unwrap();

        Self::merge_config_options(&config_file, &args)
    }

    fn merge_config_options(config_file: &Ini, args: &arguments::Opt) -> Self {
        let db_path = args.db_path.clone().unwrap_or_else(|| {
            PathBuf::from(
                config_file
                    .get("global", "db_path")
                    .unwrap_or_else(|| String::from("data.db")),
            )
        });
        let server = args.server.clone().unwrap_or_else(|| {
            config_file
                .get("account", "server")
                .expect("No server specified!")
        });
        let port = args.port.unwrap_or_else(|| {
            config_file
                .getuint("account", "port")
                .unwrap()
                .unwrap_or(993) as u16
        });
        let user = args.user.clone().unwrap_or_else(|| {
            config_file
                .get("account", "user")
                .expect("No user specified")
        });
        let password = args.password.clone().unwrap_or_else(|| {
            config_file
                .get("account", "password")
                .expect("No password specified")
        });
        let store_folder = args.store_folder.clone().unwrap_or_else(|| {
            config_file
                .get("account", "store_folder")
                .unwrap_or_else(|| String::from("processed"))
        });

        Self {
            db_path,
            server,
            port,
            user,
            password,
            store_folder,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_config_options() {
        // default settings
        let mut cf_file = Ini::new();
        cf_file.set("account", "user", Some(String::from("foo")));
        cf_file.set("account", "password", Some(String::from("bar")));
        cf_file.set("account", "server", Some(String::from("testserver.com")));
        let args = arguments::Opt {
            config: None,
            db_path: None,
            server: None,
            port: None,
            user: None,
            password: None,
            store_folder: None,
        };
        assert_eq!(
            Config {
                db_path: PathBuf::from("data.db"),
                server: String::from("testserver.com"),
                port: 993,
                user: String::from("foo"),
                password: String::from("bar"),
                store_folder: String::from("processed"),
            },
            Config::merge_config_options(&cf_file, &args)
        );

        // all config file
        cf_file.set("global", "db_path", Some(String::from("mydata.db")));
        cf_file.set("account", "store_folder", Some(String::from("finished")));
        cf_file.set("account", "port", Some(String::from("123")));
        assert_eq!(
            Config {
                db_path: PathBuf::from("mydata.db"),
                server: String::from("testserver.com"),
                port: 123,
                user: String::from("foo"),
                password: String::from("bar"),
                store_folder: String::from("finished"),
            },
            Config::merge_config_options(&cf_file, &args)
        );

        // all args
        let allargs = arguments::Opt {
            config: None,
            db_path: Some(PathBuf::from("foobar.db")),
            server: Some(String::from("newserver.foo")),
            port: Some(888 as u16),
            user: Some(String::from("newuser")),
            password: Some(String::from("newpassword")),
            store_folder: Some(String::from("newstorefolder")),
        };
        assert_eq!(
            Config {
                db_path: PathBuf::from("foobar.db"),
                server: String::from("newserver.foo"),
                port: 888,
                user: String::from("newuser"),
                password: String::from("newpassword"),
                store_folder: String::from("newstorefolder"),
            },
            Config::merge_config_options(&cf_file, &allargs)
        );
    }
}
