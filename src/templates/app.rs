use std::path::PathBuf;

use super::{ThulaniDep, render};

pub fn files(name: &str, dep: &ThulaniDep, orm: bool) -> Vec<(PathBuf, String)> {
    let mut v = vec![
        (PathBuf::from("Cargo.toml"), cargo_toml(name, dep, orm)),
        (PathBuf::from("fornthulani.toml"), fornthulani_toml(orm)),
        (PathBuf::from(".gitignore"), gitignore(orm)),
        (PathBuf::from("README.md"), readme(name, orm)),
        (PathBuf::from("src/main.rs"), main_rs(orm)),
        (PathBuf::from("src/app.rs"), APP.into()),
        (PathBuf::from("src/config.rs"), config_rs(orm)),
        (PathBuf::from("src/routes/mod.rs"), ROUTES_MOD.into()),
        (PathBuf::from("src/routes/hello.rs"), ROUTES_HELLO.into()),
        (
            PathBuf::from("src/controllers/mod.rs"),
            CONTROLLERS_MOD.into(),
        ),
        (
            PathBuf::from("src/controllers/hello.rs"),
            CONTROLLERS_HELLO.into(),
        ),
        (PathBuf::from("src/services/mod.rs"), SERVICES_MOD.into()),
        (
            PathBuf::from("src/services/hello_service.rs"),
            SERVICES_HELLO.into(),
        ),
        (
            PathBuf::from("src/plugins/mod.rs"),
            PLUGINS_MOD_LOGGING.into(),
        ),
        (
            PathBuf::from("src/plugins/logging.rs"),
            PLUGINS_LOGGING.into(),
        ),
    ];
    if orm {
        v.push((PathBuf::from("src/db.rs"), DB.into()));
        v.push((PathBuf::from("src/models/mod.rs"), String::new()));
        v.push((PathBuf::from("src/requests/mod.rs"), String::new()));
        v.push((
            PathBuf::from("src/migrations/mod.rs"),
            MIGRATIONS_MOD.into(),
        ));
    }
    v
}

fn cargo_toml(name: &str, dep: &ThulaniDep, orm: bool) -> String {
    let mut extra = String::new();
    // Logging is always included.
    extra.push_str(&dep.plugin_line("fornthulani-plugin-log", "0.1", &[]));
    extra.push('\n');
    if orm {
        extra.push_str(&dep.plugin_line("fornthulani-plugin-orm", "0.1", &["validation"]));
        extra.push('\n');
        extra.push_str(&dep.plugin_line("fornthulani-plugin-validate", "0.1", &[]));
        extra.push('\n');
    }
    render(
        r##"[package]
name = "__NAME__"
version = "0.1.0"
edition = "2024"

[dependencies]
__DEP__
__EXTRA__serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

[profile.release]
overflow-checks = true
panic = "unwind"
"##,
        &[
            ("NAME", name),
            ("DEP", dep.to_cargo_line().as_str()),
            ("EXTRA", extra.as_str()),
        ],
    )
}

fn gitignore(orm: bool) -> String {
    let mut s = String::from("/target\n/storage/logs/*.log\n");
    if orm {
        s.push_str("*.db\n*.db-wal\n*.db-shm\n");
    }
    s
}

fn fornthulani_toml(orm: bool) -> String {
    let mut s = String::from(
        r#"[server]
host = "127.0.0.1"
port = 8080
production = false
max_body_bytes = 1048576

[logging]
# app.log has every line. Channels write to their own file AND to app.log.
app_log = "storage/logs/app.log"
to_stdout = true

# Add channels here. Each `[[logging.channels]]` block becomes a channel:
#   channel!("auth").info(format!("user {} logged in", id));
# writes to storage/logs/auth.log AND to app.log with an [auth] prefix.
[[logging.channels]]
name = "auth"
path = "storage/logs/auth.log"
mirror_to_app = true
"#,
    );
    if orm {
        s.push_str(
            r#"
[database]
path = "app.db"
"#,
        );
    }
    s
}

fn config_rs(orm: bool) -> String {
    let (db_struct, db_field, db_default) = if orm {
        (
            r#"
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub path: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self { path: "app.db".into() }
    }
}
"#,
            "    pub database: DatabaseConfig,\n",
            "            database: DatabaseConfig::default(),\n",
        )
    } else {
        ("", "", "")
    };

    render(
        r##"use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerSection,
    pub logging: LoggingConfig,
__DB_FIELD__}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    pub production: bool,
    pub max_body_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub app_log: String,
    pub to_stdout: bool,
    pub channels: Vec<LoggingChannel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingChannel {
    pub name: String,
    pub path: String,
    #[serde(default = "default_mirror")]
    pub mirror_to_app: bool,
}

fn default_mirror() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            app_log: "storage/logs/app.log".into(),
            to_stdout: true,
            channels: Vec::new(),
        }
    }
}
__DB_STRUCT__
impl Default for ServerSection {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            production: false,
            max_body_bytes: 1024 * 1024,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let mut cfg = match std::fs::read_to_string("fornthulani.toml") {
            Ok(raw) => toml::from_str(&raw).map_err(|e| e.to_string())?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(e.to_string()),
        };
        cfg.apply_env();
        Ok(cfg)
    }

    fn apply_env(&mut self) {
        if let Ok(v) = std::env::var("APP_ENV") {
            self.server.production = v.eq_ignore_ascii_case("production");
        }
        if let Ok(v) = std::env::var("HOST") {
            self.server.host = v;
        }
        if let Some(p) = std::env::var("PORT").ok().and_then(|p| p.parse().ok()) {
            self.server.port = p;
        }
        if let Ok(v) = std::env::var("LOGGING_APP_LOG") {
            self.logging.app_log = v;
        }
    }
}

pub fn get() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fatal: cannot load configuration: {e}");
            std::process::exit(1);
        }
    })
}
"##,
        &[
            ("DB_FIELD", db_field),
            ("DB_STRUCT", db_struct),
            ("DB_DEFAULT", db_default),
        ],
    )
}

fn main_rs(orm: bool) -> String {
    if orm { MAIN_ORM.into() } else { MAIN.into() }
}

fn readme(name: &str, orm: bool) -> String {
    let orm_section = if orm {
        r#"
## Database (Eloquent-style)

    cargo run -- migrate             # run pending migrations
    cargo run -- migrate:rollback    # undo the last batch
    cargo run -- migrate:status

    fornthulani generate resource Post title:string body:text? published:bool
    cargo run -- migrate
    cargo run

Field types: `string`, `text`, `integer`, `float`, `bool`, `email`;
add `?` for nullable (`body:text?`).
"#
    } else {
        ""
    };

    render(
        r##"# __NAME__

A fornthulani application.

## Run

    cargo run
    APP_ENV=production cargo run --release

## Layout

    src/
      main.rs           entry point
      app.rs            wires routes into a Router
      config.rs         reads fornthulani.toml
      routes/           URL space
      controllers/      request/response handlers
      services/         business logic
      plugins/          middleware (logging is wired in by default)
      plugins/logging.rs  HTTP request logging + channel logging

## Logging

Already configured. Every request is logged to `storage/logs/app.log`.

To log from anywhere in the project:

    use fornthulani_plugin_log::{info, warn, channel};

    info!("server started");
    channel!("auth").info("user logged in");

**Add a channel** — edit `fornthulani.toml`:

    [[logging.channels]]
    name = "db"
    path = "storage/logs/db.log"
    mirror_to_app = true

Then `channel!("db").info(...)` writes to `storage/logs/db.log` AND
`storage/logs/app.log` (with a `[db]` prefix).
__ORM__"##,
        &[("NAME", name), ("ORM", orm_section)],
    )
}

// ---------------------------------------------------------------------
// templates
// ---------------------------------------------------------------------

const MAIN: &str = r##"mod app;
mod config;
mod controllers;
mod plugins;
mod routes;
mod services;

use fornthulani::{Server, ServerConfig};

fn main() -> std::io::Result<()> {
    let cfg = config::get();

    let server_cfg = if cfg.server.production {
        ServerConfig::production()
    } else {
        ServerConfig::development()
    }
    .max_body_bytes(cfg.server.max_body_bytes);

    let server = Server::with_config(app::build_router(), server_cfg)
        .with_app_config(cfg.clone());

    let server = plugins::register(server);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let mode = if cfg.server.production { "production" } else { "development" };
    println!("listening on http://{addr} ({mode})");
    server.run_blocking(&addr)
}
"##;

const MAIN_ORM: &str = r##"mod app;
mod config;
mod controllers;
mod db;
mod migrations;
mod models;
mod plugins;
mod requests;
mod routes;
mod services;

use fornthulani::{Server, ServerConfig};

fn fail(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

fn main() -> std::io::Result<()> {
    let cfg = config::get();

    let command = std::env::args().nth(1);
    db::connect(&cfg.database.path).map_err(fail)?;
    match command.as_deref() {
        None | Some("serve") => {}
        Some("migrate") => return db::migrate().map_err(fail),
        Some("migrate:rollback") => return db::rollback().map_err(fail),
        Some("migrate:status") => return db::status().map_err(fail),
        Some(other) => {
            eprintln!("unknown command `{other}`");
            eprintln!("commands: serve (default), migrate, migrate:rollback, migrate:status");
            std::process::exit(2);
        }
    }

    let server_cfg = if cfg.server.production {
        ServerConfig::production()
    } else {
        ServerConfig::development()
    }
    .max_body_bytes(cfg.server.max_body_bytes);

    let server = Server::with_config(app::build_router(), server_cfg)
        .with_app_config(cfg.clone());

    let server = plugins::register(server);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let mode = if cfg.server.production { "production" } else { "development" };
    println!("listening on http://{addr} ({mode})");
    server.run_blocking(&addr)
}
"##;

const APP: &str = r##"use fornthulani::Router;

use crate::routes;

pub fn build_router() -> Router {
    let mut router = Router::new();
    routes::register(&mut router);
    router
}
"##;

const ROUTES_MOD: &str = r##"pub mod hello;

use fornthulani::Router;

pub fn register(router: &mut Router) {
    hello::register(router);
}
"##;

const ROUTES_HELLO: &str = r##"use fornthulani::Router;

use crate::controllers::hello;

pub fn register(router: &mut Router) {
    router.get("/hello", hello::hello).name("hello");
}
"##;

const CONTROLLERS_MOD: &str = "pub mod hello;\n";

const CONTROLLERS_HELLO: &str = r##"use fornthulani::{Params, Request, Response, Status, ThulaniError};

use crate::services::hello_service;

pub fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let greeting = hello_service::greet("fornthulani");
    Ok(Response::json(Status::OK, &greeting))
}
"##;

const SERVICES_MOD: &str = "pub mod hello_service;\n";

const SERVICES_HELLO: &str = r##"//! A service: plain Rust that holds business logic.

use serde::Serialize;

#[derive(Serialize)]
pub struct Greeting {
    pub message: String,
    pub from: String,
}

pub fn greet(name: &str) -> Greeting {
    Greeting {
        message: format!("hello, {name}"),
        from: "fornthulani".into(),
    }
}
"##;

const PLUGINS_MOD_LOGGING: &str = r##"pub mod logging;

use fornthulani::Server;

pub fn register(server: Server) -> Server {
    logging::init_from_server(&server);
    server.with_middleware(logging::middleware())
}
"##;

const PLUGINS_LOGGING: &str = r##"//! Logging: HTTP request logging + channel logging.
//!
//! Wired in by the generator. Reads `[logging]` from fornthulani.toml.
//!
//! To add a channel, edit fornthulani.toml:
//!
//!     [[logging.channels]]
//!     name = "auth"
//!     path = "storage/logs/auth.log"
//!     mirror_to_app = true
//!
//! Then log with:
//!
//!     use fornthulani_plugin_log::channel;
//!     channel!("auth").info("user logged in");

use fornthulani::Server;

pub fn init_from_server(server: &Server) {
    let cfg = match server.app_config::<crate::config::Config>() {
        Some(c) => c,
        None => {
            eprintln!("[logging] no app config on server");
            return;
        }
    };

    if let Err(e) =
        fornthulani_plugin_log::init_both(cfg.logging.app_log.as_str(), cfg.logging.to_stdout)
    {
        eprintln!("[logging] cannot open `{}`: {e}", cfg.logging.app_log);
        return;
    }

    for ch in &cfg.logging.channels {
        match fornthulani_plugin_log::add_channel(ch.name.clone(), ch.path.as_str()) {
            Ok(b) => {
                b.mirror_to_app(ch.mirror_to_app);
            }
            Err(e) => {
                eprintln!(
                    "[logging] cannot open channel `{}` at `{}`: {e}",
                    ch.name, ch.path
                );
            }
        }
    }
}

pub fn middleware() -> fornthulani_plugin_log::Log {
    fornthulani_plugin_log::Log::new()
}
"##;

const DB: &str = r##"use crate::migrations;
use fornthulani_plugin_orm::{Db, Migrator, OrmResult, SqliteConnection};

pub fn connect(path: &str) -> OrmResult<()> {
    Db::set(SqliteConnection::open(path)?);
    fornthulani_plugin_orm::register_validation_rules();
    Ok(())
}

pub fn migrate() -> OrmResult<()> {
    let conn = Db::get()?;
    let ran = Migrator::new(migrations::all()).run(&*conn)?;
    if ran.is_empty() {
        println!("nothing to migrate");
    }
    for name in ran {
        println!("migrated {name}");
    }
    Ok(())
}

pub fn rollback() -> OrmResult<()> {
    let conn = Db::get()?;
    let undone = Migrator::new(migrations::all()).rollback(&*conn)?;
    if undone.is_empty() {
        println!("nothing to roll back");
    }
    for name in undone {
        println!("rolled back {name}");
    }
    Ok(())
}

pub fn status() -> OrmResult<()> {
    let conn = Db::get()?;
    for (name, applied) in Migrator::new(migrations::all()).status(&*conn)? {
        println!("[{}] {name}", if applied { "x" } else { " " });
    }
    Ok(())
}
"##;

const MIGRATIONS_MOD: &str = r##"use fornthulani_plugin_orm::Migration;

// fornthulani:migration-mods

pub fn all() -> Vec<Migration> {
    vec![
        // fornthulani:migration-list
    ]
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    fn dep() -> ThulaniDep {
        ThulaniDep::Path("../fornthulani".into())
    }

    #[test]
    fn plain_app_has_logging_but_no_orm() {
        let f = files("demo", &dep(), false);
        let names: Vec<String> = f.iter().map(|(p, _)| p.display().to_string()).collect();
        assert!(names.iter().any(|n| n.contains("plugins/logging.rs")));
        assert!(names.iter().any(|n| n.contains("plugins/mod.rs")));
        assert!(!names.iter().any(|n| n.contains("db.rs")));
        assert!(!names.iter().any(|n| n.contains("models")));
    }

    #[test]
    fn cargo_toml_has_log_plugin_always() {
        let f = files("demo", &dep(), false);
        let cargo = &f.iter().find(|(p, _)| p.ends_with("Cargo.toml")).unwrap().1;
        assert!(cargo.contains("fornthulani-plugin-log"));
    }

    #[test]
    fn toml_has_logging_section() {
        let f = files("demo", &dep(), false);
        let toml = &f
            .iter()
            .find(|(p, _)| p.ends_with("fornthulani.toml"))
            .unwrap()
            .1;
        assert!(toml.contains("[logging]"));
        assert!(toml.contains("app_log"));
        assert!(toml.contains("[[logging.channels]]"));
        assert!(toml.contains("name = \"auth\""));
    }

    #[test]
    fn config_has_logging_struct() {
        let f = files("demo", &dep(), false);
        let cfg = &f.iter().find(|(p, _)| p.ends_with("config.rs")).unwrap().1;
        assert!(cfg.contains("pub struct LoggingConfig"));
        assert!(cfg.contains("pub struct LoggingChannel"));
        assert!(cfg.contains("pub logging: LoggingConfig"));
        assert!(!cfg.contains("__"));
    }

    #[test]
    fn main_attaches_app_config() {
        let f = files("demo", &dep(), false);
        let main = &f.iter().find(|(p, _)| p.ends_with("main.rs")).unwrap().1;
        assert!(main.contains(".with_app_config(cfg.clone())"));
        assert!(main.contains("plugins::register(server)"));
    }

    #[test]
    fn plugins_mod_calls_init_and_middleware() {
        let f = files("demo", &dep(), false);
        let pm = &f
            .iter()
            .find(|(p, _)| p.ends_with("plugins/mod.rs"))
            .unwrap()
            .1;
        assert!(pm.contains("logging::init_from_server(&server)"));
        assert!(pm.contains("server.with_middleware(logging::middleware())"));
    }

    #[test]
    fn plugins_logging_matches_demo() {
        let f = files("demo", &dep(), false);
        let pl = &f
            .iter()
            .find(|(p, _)| p.ends_with("plugins/logging.rs"))
            .unwrap()
            .1;
        assert!(pl.contains("fornthulani_plugin_log::init_both"));
        assert!(pl.contains("fornthulani_plugin_log::add_channel"));
        assert!(pl.contains("fornthulani_plugin_log::Log::new"));
    }
}
