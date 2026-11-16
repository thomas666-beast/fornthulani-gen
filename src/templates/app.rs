use std::path::PathBuf;

use super::{ThulaniDep, render};

pub fn files(name: &str, dep: &ThulaniDep, orm: bool) -> Vec<(PathBuf, String)> {
    let mut v = vec![
        (PathBuf::from("Cargo.toml"), cargo_toml(name, dep, orm)),
        (PathBuf::from("fornthulani.toml"), fornthulani_toml(orm)),
        (PathBuf::from(".gitignore"), gitignore(orm)),
        (PathBuf::from("README.md"), readme(name, orm)),
        (
            PathBuf::from("src/main.rs"),
            if orm { MAIN_ORM.into() } else { MAIN.into() },
        ),
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
        (PathBuf::from("src/plugins/mod.rs"), PLUGINS_MOD.into()),
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
panic = "unwind"   # a panicking handler becomes a 500, not a dead server
"##,
        &[
            ("NAME", name),
            ("DEP", dep.to_cargo_line().as_str()),
            ("EXTRA", extra.as_str()),
        ],
    )
}

fn gitignore(orm: bool) -> String {
    let mut s = String::from("/target\n");
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
# Production mode: JSON-only errors, API security headers. Also settable
# with APP_ENV=production. Run behind a TLS-terminating reverse proxy.
production = false
# Largest accepted request body, in bytes.
max_body_bytes = 1048576
"#,
    );
    if orm {
        s.push_str(
            r#"
[database]
# Also settable with DATABASE_PATH.
path = "app.db"
"#,
        );
    }
    s
}

fn config_rs(orm: bool) -> String {
    let (db_struct, db_field, db_default, db_env) = if orm {
        (
            r#"
#[derive(Debug, Clone, Deserialize)]
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
            r#"        if let Ok(v) = std::env::var("DATABASE_PATH") {
            self.database.path = v;
        }
"#,
        )
    } else {
        ("", "", "", "")
    };

    render(
        r##"use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerSection,
__DB_FIELD__}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    pub production: bool,
    pub max_body_bytes: usize,
}
__DB_STRUCT__
impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection::default(),
__DB_DEFAULT__        }
    }
}

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
    /// `fornthulani.toml` (optional) + environment overrides:
    /// `APP_ENV`, `HOST`, `PORT`, `DATABASE_PATH`.
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
__DB_ENV__    }
}

/// The configuration, loaded once. A broken config file stops the program
/// at startup instead of silently falling back to development defaults.
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
            ("DB_ENV", db_env),
        ],
    )
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

`generate resource` creates the model, migration, validation request,
CRUD controller and routes in one go. Pieces on their own:

    fornthulani generate model Post title:string body:text?
    fornthulani generate request Post title:string body:text?
    fornthulani generate migration add_index_to_posts

Field types: `string`, `text`, `integer`, `float`, `bool`, `email`;
add `?` for nullable (`body:text?`).

Security defaults you get for free: every value is a bound parameter,
table/column names are validated, models have a mass-assignment allow-list
(`FILLABLE`), `HIDDEN` columns never reach clients, bulk `update`/`delete`
without a condition is refused, and `paginate` caps `per_page` at 100.
"#
    } else {
        ""
    };

    render(
        r##"# __NAME__

A fornthulani application.

## Run

    cargo run                      # development
    APP_ENV=production cargo run --release

Production mode answers every error as JSON and adds API security
headers. fornthulani speaks plain HTTP: put it behind a TLS-terminating
reverse proxy (nginx, Caddy, a cloud load balancer) and bind it to a
private address.

Configuration lives in `fornthulani.toml`; `APP_ENV`, `HOST`, `PORT`
(and `DATABASE_PATH`) override it.

## Layout

    src/
      main.rs           entry point
      app.rs            wires routes into a Router
      config.rs         fornthulani.toml + environment
      routes/           URL space (one file per route group)
      controllers/      request/response handlers
      services/         business logic
      plugins/          your own middleware
__LAYOUT_ORM__
## Validation

    use fornthulani_plugin_validate::ValidateExt;

    let data = req.validate([
        ("email", "required|email|max:255"),
        ("age",   "nullable|integer|between:18,120"),
    ])?;            // failure -> 422 {"message": ..., "errors": {...}}

## Generate more

    fornthulani generate route <name>       # add a route + controller
    fornthulani generate controller <name>  # add just a controller
    fornthulani generate service <name>     # add a service
    fornthulani generate plugin <name>      # add a middleware in src/plugins/
__ORM__"##,
        &[
            ("NAME", name),
            (
                "LAYOUT_ORM",
                if orm {
                    "      models/           database models\n      requests/         validation rules (form requests)\n      migrations/       schema changes\n      db.rs             connection + migration commands\n"
                } else {
                    ""
                },
            ),
            ("ORM", orm_section),
        ],
    )
}

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

    let server = Server::with_config(app::build_router(), server_cfg);
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

    // `cargo run -- migrate` etc.
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

    let server = Server::with_config(app::build_router(), server_cfg);
    let server = plugins::register(server);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let mode = if cfg.server.production { "production" } else { "development" };
    println!("listening on http://{addr} ({mode})");
    server.run_blocking(&addr)
}
"##;

const DB: &str = r##"use crate::migrations;
use fornthulani_plugin_orm::{Db, Migrator, OrmResult, SqliteConnection};

/// Open the database and make it the connection every model uses.
pub fn connect(path: &str) -> OrmResult<()> {
    Db::set(SqliteConnection::open(path)?);
    // Enables the `unique:` and `exists:` validation rules.
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

/// Every migration. They run sorted by name; `generate migration` and
/// `generate model` register themselves here.
pub fn all() -> Vec<Migration> {
    vec![
        // fornthulani:migration-list
    ]
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

/// GET /hello
///
/// The controller just delegates to the service.
/// Business logic lives in `services/hello_service.rs`.
pub fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let greeting = hello_service::greet("fornthulani");
    Ok(Response::json(Status::OK, &greeting))
}
"##;

const SERVICES_MOD: &str = "pub mod hello_service;\n";

const SERVICES_HELLO: &str = r##"//! A service: plain Rust that holds business logic.
//!
//! Services do not know about HTTP. They take data, return data.
//! Controllers call services; services call databases, APIs, files, etc.
//!
//! This is a hint, not an obligation. Small controllers can stay small
//! without a service.

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

const PLUGINS_MOD: &str = r##"use fornthulani::Server;

/// Register all project plugins against the server.
///
/// Generated plugins print the line to add here:
///
///     server = server.with_middleware(my_plugin::MyPlugin);
pub fn register(server: Server) -> Server {
    server
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    fn dep() -> ThulaniDep {
        ThulaniDep::Path("../fornthulani".into())
    }

    #[test]
    fn plain_app_has_no_orm_bits() {
        let f = files("demo", &dep(), false);
        let names: Vec<String> = f.iter().map(|(p, _)| p.display().to_string()).collect();
        assert!(
            !names
                .iter()
                .any(|n| n.contains("db.rs") || n.contains("models"))
        );
        let cargo = &f.iter().find(|(p, _)| p.ends_with("Cargo.toml")).unwrap().1;
        assert!(cargo.contains("edition = \"2024\""));
        assert!(!cargo.contains("orm"));
        let cfg = &f.iter().find(|(p, _)| p.ends_with("config.rs")).unwrap().1;
        assert!(!cfg.contains("DatabaseConfig"));
        assert!(!cfg.contains("__"));
    }

    #[test]
    fn orm_app_has_db_and_markers() {
        let f = files("demo", &dep(), true);
        let get = |suffix: &str| &f.iter().find(|(p, _)| p.ends_with(suffix)).unwrap().1;
        assert!(get("Cargo.toml").contains("fornthulani-plugin-orm = { path = \"../fornthulani-plugin-orm\", features = [\"validation\"] }"));
        assert!(get("migrations/mod.rs").contains("// fornthulani:migration-mods"));
        assert!(get("migrations/mod.rs").contains("// fornthulani:migration-list"));
        assert!(get("config.rs").contains("DatabaseConfig"));
        assert!(get("config.rs").contains("DATABASE_PATH"));
        assert!(!get("config.rs").contains("__"));
        assert!(get("fornthulani.toml").contains("[database]"));
        assert!(get("main.rs").contains("migrate:rollback"));
    }
}
