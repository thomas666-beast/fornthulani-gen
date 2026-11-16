use std::path::PathBuf;

use super::{ThulaniDep, render};

pub fn files(
    name: &str,
    dep: &ThulaniDep,
    orm: bool,
    jwt_secret: &str,
    csrf_secret: &str,
    api_key: &str,
) -> Vec<(PathBuf, String)> {
    let mut v = vec![
        (PathBuf::from("Cargo.toml"), cargo_toml(name, dep, orm)),
        (PathBuf::from(".gitignore"), gitignore(orm)),
        (
            PathBuf::from("fornthulani.toml"),
            fornthulani_toml(orm, jwt_secret, csrf_secret, api_key),
        ),
        (PathBuf::from("README.md"), readme(name)),
        (PathBuf::from("curl.sh"), CURL_SH.to_string()),
        (PathBuf::from("src/main.rs"), if orm { MAIN_ORM } else { MAIN }.to_string()),
        (PathBuf::from("src/app.rs"), if orm { APP_ORM } else { APP_PLAIN }.to_string()),
        (PathBuf::from("src/config.rs"), if orm { CONFIG_ORM } else { CONFIG_PLAIN }.to_string()),
        (PathBuf::from("src/routes/mod.rs"), if orm { ROUTES_MOD_ORM } else { ROUTES_MOD_PLAIN }.to_string()),
        (PathBuf::from("src/routes/hello.rs"), ROUTES_HELLO.to_string()),
        (PathBuf::from("src/controllers/mod.rs"), if orm { "pub mod auth;\npub mod hello;\n" } else { "pub mod hello;\n" }.to_string()),
        (PathBuf::from("src/controllers/hello.rs"), CONTROLLERS_HELLO.to_string()),
        (PathBuf::from("src/services/mod.rs"), "pub mod hello_service;\n".to_string()),
        (PathBuf::from("src/services/hello_service.rs"), SERVICES_HELLO.to_string()),
        (PathBuf::from("src/plugins/mod.rs"), if orm { PLUGINS_MOD_ORM } else { PLUGINS_MOD_PLAIN }.to_string()),
        (PathBuf::from("src/plugins/logging.rs"), PLUGINS_LOGGING.to_string()),
    ];

    if orm {
        v.push((PathBuf::from("src/plugins/auth.rs"), PLUGINS_AUTH.to_string()));
        v.push((PathBuf::from("src/controllers/auth.rs"), CONTROLLERS_AUTH.to_string()));
        v.push((PathBuf::from("src/routes/auth.rs"), ROUTES_AUTH.to_string()));
        v.push((PathBuf::from("src/requests/mod.rs"), "pub mod login;\n".to_string()));
        v.push((PathBuf::from("src/requests/login.rs"), REQUESTS_LOGIN.to_string()));
        v.push((PathBuf::from("src/models/mod.rs"), "pub mod user;\n".to_string()));
        v.push((PathBuf::from("src/models/user.rs"), MODELS_USER.to_string()));
        v.push((PathBuf::from("src/db.rs"), DB.to_string()));
        v.push((PathBuf::from("src/migrations/mod.rs"), MIGRATIONS_MOD.to_string()));
        v.push((
            PathBuf::from("src/migrations/m20260101000001_create_users_table.rs"),
            MIGRATION_CREATE_USERS.to_string(),
        ));
        v.push((
            PathBuf::from("src/migrations/m20260101000002_seed_demo_user.rs"),
            MIGRATION_SEED_USER.to_string(),
        ));
    }
    v
}

fn cargo_toml(name: &str, dep: &ThulaniDep, orm: bool) -> String {
    let mut extra = String::new();
    extra.push_str(&dep.plugin_line("fornthulani-plugin-log", "0.1", &[]));
    extra.push('\n');
    extra.push_str(&dep.plugin_line("fornthulani-plugin-cors", "0.1", &[]));
    extra.push('\n');
    extra.push_str(&dep.plugin_line("fornthulani-plugin-json-body", "0.1", &[]));
    extra.push('\n');
    if orm {
        extra.push_str(&dep.plugin_line("fornthulani-plugin-orm", "0.2", &["sqlite", "validation"]));
        extra.push('\n');
        extra.push_str(&dep.plugin_line("fornthulani-plugin-validate", "0.2", &[]));
        extra.push('\n');
        extra.push_str(&dep.plugin_line("fornthulani-plugin-auth", "0.1", &["sqlite"]));
        extra.push('\n');
    }
    render(
        r#"[package]
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
"#,
        &[
            ("NAME", name),
            ("DEP", dep.to_cargo_line().as_str()),
            ("EXTRA", extra.as_str()),
        ],
    )
}

fn gitignore(orm: bool) -> String {
    let mut s = String::from("/target\n/storage/logs/*.log\nfornthulani.toml\n");
    if orm {
        s.push_str("*.db\n*.db-wal\n*.db-shm\n");
    }
    s
}

fn fornthulani_toml(orm: bool, jwt: &str, csrf: &str, api_key: &str) -> String {
    let mut s = String::from(
        r#"[server]
host = "127.0.0.1"
port = 8080
production = false
max_body_bytes = 1048576

[logging]
app_log = "storage/logs/app.log"
to_stdout = true

[[logging.channels]]
name = "auth"
path = "storage/logs/auth.log"
mirror_to_app = true

[[logging.channels]]
name = "db"
path = "storage/logs/db.log"
mirror_to_app = true
"#,
    );
    if orm {
        s.push_str(&format!(
            r#"
[database]
path = "app.db"

[auth]
jwt_secret = "{jwt}"
csrf_secret = "{csrf}"
api_key = "{api_key}"
issuer = "app"
audience = "app-api"
token_ttl_secs = 900
rate_per_minute = 300
login_rate_per_minute = 5
"#
        ));
    }
    s
}

fn readme(name: &str) -> String {
    render(
        r#"# __NAME__

A fornthulani application.

## Run

    cargo run -- migrate
    cargo run

Then in another terminal:

    ./curl.sh

## Auth

Demo user (created by the seed migration):

- email: `demo@example.com`
- password: `hunter2hunter2`

    curl -X POST http://127.0.0.1:8080/login \
         -H 'content-type: application/json' \
         -d '{"email":"demo@example.com","password":"hunter2hunter2"}'

    curl http://127.0.0.1:8080/me \
         -H "Authorization: Bearer <token>"

## Logging

Logs go to `storage/logs/app.log`. Add channels in `fornthulani.toml`:

    [[logging.channels]]
    name = "db"
    path = "storage/logs/db.log"
    mirror_to_app = true
"#,
        &[("NAME", name)],
    )
}

const MAIN: &str = r#"mod app;
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
"#;

const MAIN_ORM: &str = r#"mod app;
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
use fornthulani_plugin_auth::prelude::Jwt;
use std::time::Duration;

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
            std::process::exit(2);
        }
    }

    let server_cfg = if cfg.server.production {
        ServerConfig::production()
    } else {
        ServerConfig::development()
    }
    .max_body_bytes(cfg.server.max_body_bytes);

    let jwt = Jwt::new(cfg.auth.jwt_secret.as_bytes())
        .expect("jwt_secret must be >= 32 bytes")
        .issuer(&cfg.auth.issuer)
        .audience(&cfg.auth.audience)
        .ttl(Duration::from_secs(cfg.auth.token_ttl_secs));

    let server = Server::with_config(app::build_router(jwt.clone()), server_cfg)
        .with_app_config(cfg.clone());
    let server = plugins::register(server, jwt);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let mode = if cfg.server.production { "production" } else { "development" };
    println!("listening on http://{addr} ({mode})");
    server.run_blocking(&addr)
}
"#;

const APP_PLAIN: &str = r#"use fornthulani::Router;

use crate::routes;

pub fn build_router() -> Router {
    let mut router = Router::new();
    routes::register(&mut router);
    router
}
"#;

const APP_ORM: &str = r#"use fornthulani::Router;
use fornthulani_plugin_auth::prelude::Jwt;

use crate::routes;

pub fn build_router(jwt: Jwt) -> Router {
    let mut router = Router::new();
    routes::register(&mut router, jwt);
    router
}
"#;

const ROUTES_MOD_PLAIN: &str = r#"pub mod hello;

use fornthulani::Router;

pub fn register(router: &mut Router) {
    hello::register(router);
}
"#;

const ROUTES_MOD_ORM: &str = r#"pub mod auth;
pub mod hello;

use fornthulani::Router;
use fornthulani_plugin_auth::prelude::Jwt;

pub fn register(router: &mut Router, jwt: Jwt) {
    hello::register(router);
    auth::register(router, jwt);
}
"#;

const ROUTES_HELLO: &str = r#"use fornthulani::Router;

use crate::controllers::hello;

pub fn register(router: &mut Router) {
    router.get("/hello", hello::hello).name("hello");
}
"#;

const ROUTES_AUTH: &str = r#"use fornthulani::Router;
use fornthulani_plugin_auth::prelude::*;

use crate::controllers::auth;

pub fn register(router: &mut Router, jwt: Jwt) {
    router
        .post("/login", auth::login)
        .with(RateLimit::per_minute(5))
        .name("login");
    router
        .get("/me", auth::me)
        .with(JwtAuth::<NoClaims>::new(jwt))
        .name("me");
}
"#;

const CONTROLLERS_HELLO: &str = r#"use fornthulani::{Params, Request, Response, Status, ThulaniError};

use crate::services::hello_service;

pub fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let greeting = hello_service::greet("fornthulani");
    Ok(Response::json(Status::OK, &greeting))
}
"#;

const CONTROLLERS_AUTH: &str = r#"use fornthulani::{Params, Request, Response, Status, ThulaniError};
use fornthulani_plugin_auth::prelude::*;
use fornthulani_plugin_orm::Model;
use fornthulani_plugin_validate::ValidateExt;
use serde_json::json;

use crate::models::user::User;
use crate::requests::login::LoginInput;

pub fn login(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let input: LoginInput = req.validated()?;

    let hasher = req.provided::<PasswordHasher>()?;
    let user = User::query().where_eq("email", input.email.clone()).first()?;

    let ok = match &user {
        Some(u) => hasher.verify(&input.password, &u.password_hash),
        None => hasher.verify_dummy(&input.password),
    };

    let user = user
        .filter(|_| ok)
        .ok_or_else(|| ThulaniError::Unauthorized("invalid credentials".into()))?;

    let id = user.id.unwrap_or(0);
    let token = req.jwt()?.issue(&id.to_string(), NoClaims {})?;

    Ok(Response::json(
        Status::OK,
        &json!({ "token": token, "expires_in": 900 }),
    ))
}

pub fn me(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let claims = req.claims::<NoClaims>()?;
    let id: i64 = claims
        .sub
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ThulaniError::Unauthorized("invalid subject".into()))?;
    let user = User::find_or_fail(id)?;
    Ok(Response::json(Status::OK, &user.public_json()))
}
"#;

const REQUESTS_LOGIN: &str = r#"use fornthulani_plugin_validate::{FormRequest, Rules};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

impl FormRequest for LoginInput {
    fn rules() -> Rules {
        Rules::new()
            .field("email", "required|email|max:255")
            .field("password", "required|string|min:8")
    }
}
"#;

const MODELS_USER: &str = r#"use fornthulani_plugin_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Model for User {
    const TABLE: &'static str = "users";
    const FILLABLE: &'static [&'static str] = &["email", "password_hash"];
    const HIDDEN: &'static [&'static str] = &["password_hash"];

    fn id(&self) -> Option<i64> {
        self.id
    }
}
"#;

const SERVICES_HELLO: &str = r#"use serde::Serialize;

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
"#;

const PLUGINS_MOD_PLAIN: &str = r#"pub mod logging;

use fornthulani::Server;

pub fn register(server: Server) -> Server {
    logging::init_from_server(&server);
    server.with_middleware(logging::middleware())
}
"#;

const PLUGINS_MOD_ORM: &str = r#"pub mod auth;
pub mod logging;

use fornthulani::Server;
use fornthulani_plugin_auth::prelude::Jwt;

pub fn register(server: Server, jwt: Jwt) -> Server {
    logging::init_from_server(&server);
    let server = server.with_middleware(logging::middleware());
    auth::init_from_server(server, jwt)
}
"#;

const PLUGINS_LOGGING: &str = r#"use fornthulani::Server;

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
                eprintln!("[logging] cannot open channel `{}`: {e}", ch.name);
            }
        }
    }
}

pub fn middleware() -> fornthulani_plugin_log::Log {
    fornthulani_plugin_log::Log::new()
}
"#;

const PLUGINS_AUTH: &str = r#"use fornthulani::Server;
use fornthulani_plugin_auth::prelude::*;

pub fn init_from_server(server: Server, jwt: Jwt) -> Server {
    let cfg = match server.app_config::<crate::config::Config>() {
        Some(c) => c.clone(),
        None => {
            eprintln!("[auth] no app config on server");
            return server;
        }
    };

    let hasher = PasswordHasher::new();

    server
        .with_middleware(RateLimit::per_minute(cfg.auth.rate_per_minute))
        .with_middleware(Provide(hasher))
        .with_middleware(Provide(jwt))
}
"#;

const DB: &str = r#"use crate::migrations;
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
"#;

const MIGRATIONS_MOD: &str = r#"use fornthulani_plugin_orm::Migration;

// fornthulani:migration-mods
pub mod m20260101000001_create_users_table;
pub mod m20260101000002_seed_demo_user;

pub fn all() -> Vec<Migration> {
    vec![
        // fornthulani:migration-list
        Migration {
            name: "20260101000001_create_users_table",
            up: m20260101000001_create_users_table::up,
            down: m20260101000001_create_users_table::down,
        },
        Migration {
            name: "20260101000002_seed_demo_user",
            up: m20260101000002_seed_demo_user::up,
            down: m20260101000002_seed_demo_user::down,
        },
    ]
}
"#;

const MIGRATION_CREATE_USERS: &str = r#"use fornthulani_plugin_orm::{Executor, OrmResult, Schema};

pub fn up(db: &dyn Executor) -> OrmResult<()> {
    Schema::create(db, "users", |t| {
        t.id();
        t.string("email", 255).unique();
        t.string("password_hash", 255);
        t.timestamps();
    })
}

pub fn down(db: &dyn Executor) -> OrmResult<()> {
    Schema::drop_if_exists(db, "users")
}
"#;

const MIGRATION_SEED_USER: &str = r#"use fornthulani_plugin_auth::prelude::PasswordHasher;
use fornthulani_plugin_orm::{Executor, OrmResult, Query, SqlValue};

pub fn up(db: &dyn Executor) -> OrmResult<()> {
    let hasher = PasswordHasher::new();
    let phc = hasher
        .hash("hunter2hunter2")
        .map_err(|e| fornthulani_plugin_orm::OrmError::Db(format!("hash: {e}")))?;
    Query::new("users").insert_on(
        db,
        vec![
            ("email".into(), SqlValue::Text("demo@example.com".into())),
            ("password_hash".into(), SqlValue::Text(phc)),
        ],
    )?;
    Ok(())
}

pub fn down(db: &dyn Executor) -> OrmResult<()> {
    Query::new("users")
        .where_eq("email", "demo@example.com")
        .delete_on(db)?;
    Ok(())
}
"#;

const CONFIG_ORM: &str = r#"use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerSection,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub csrf_secret: String,
    pub api_key: String,
    pub issuer: String,
    pub audience: String,
    pub token_ttl_secs: u64,
    pub rate_per_minute: u32,
    pub login_rate_per_minute: u32,
}

fn default_mirror() -> bool { true }

impl Default for ServerSection {
    fn default() -> Self {
        Self { host: "127.0.0.1".into(), port: 8080, production: false, max_body_bytes: 1024 * 1024 }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { app_log: "storage/logs/app.log".into(), to_stdout: true, channels: Vec::new() }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self { path: "app.db".into() }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            csrf_secret: String::new(),
            api_key: String::new(),
            issuer: "app".into(),
            audience: "app-api".into(),
            token_ttl_secs: 900,
            rate_per_minute: 300,
            login_rate_per_minute: 5,
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
        if let Ok(v) = std::env::var("HOST") { self.server.host = v; }
        if let Some(p) = std::env::var("PORT").ok().and_then(|p| p.parse().ok()) {
            self.server.port = p;
        }
        if let Ok(v) = std::env::var("LOGGING_APP_LOG") { self.logging.app_log = v; }
        if let Ok(v) = std::env::var("DATABASE_PATH") { self.database.path = v; }
        if let Ok(v) = std::env::var("JWT_SECRET") { self.auth.jwt_secret = v; }
        if let Ok(v) = std::env::var("CSRF_SECRET") { self.auth.csrf_secret = v; }
        if let Ok(v) = std::env::var("API_KEY") { self.auth.api_key = v; }
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
"#;

const CONFIG_PLAIN: &str = r#"use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerSection,
    pub logging: LoggingConfig,
}

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

fn default_mirror() -> bool { true }

impl Default for ServerSection {
    fn default() -> Self {
        Self { host: "127.0.0.1".into(), port: 8080, production: false, max_body_bytes: 1024 * 1024 }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { app_log: "storage/logs/app.log".into(), to_stdout: true, channels: Vec::new() }
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
        if let Ok(v) = std::env::var("HOST") { self.server.host = v; }
        if let Some(p) = std::env::var("PORT").ok().and_then(|p| p.parse().ok()) {
            self.server.port = p;
        }
        if let Ok(v) = std::env::var("LOGGING_APP_LOG") { self.logging.app_log = v; }
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
"#;

const CURL_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail
BASE="${BASE:-http://127.0.0.1:8080}"
EMAIL="${EMAIL:-demo@example.com}"
PASSWORD="${PASSWORD:-hunter2hunter2}"

PASS=0
FAIL=0
ok() { echo "  ok $1"; PASS=$((PASS+1)); }
no() { echo "  FAIL $1"; FAIL=$((FAIL+1)); }
status() { curl -s -o /dev/null -w '%{http_code}' "$@"; }
jget() { python3 -c "import sys,json;print(json.load(sys.stdin)$1)" 2>/dev/null || echo ""; }

echo "== Hello =="
S=$(status "$BASE/hello"); [[ "$S" == "200" ]] && ok "GET /hello" || no "GET /hello ($S)"

echo "== Login =="
S=$(status -X POST "$BASE/login" -H 'content-type: application/json' -d '{"email":"wrong@x.io","password":"xxxxxxxx"}')
[[ "$S" == "401" ]] && ok "POST /login (bad creds)" || no "POST /login bad ($S)"

LOGIN=$(curl -s -X POST "$BASE/login" -H 'content-type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
TOKEN=$(echo "$LOGIN" | jget "['token']")
[[ -n "$TOKEN" ]] && ok "POST /login (good creds)" || no "POST /login good"

echo "== Me =="
S=$(status "$BASE/me"); [[ "$S" == "401" ]] && ok "GET /me (no token)" || no "GET /me no token ($S)"
S=$(status "$BASE/me" -H "Authorization: Bearer $TOKEN")
[[ "$S" == "200" ]] && ok "GET /me (with token)" || no "GET /me with token ($S)"

echo "== Logging =="
[[ -f storage/logs/app.log ]] && ok "app.log exists" || no "app.log missing"
[[ -f storage/logs/auth.log ]] && ok "auth.log exists" || no "auth.log missing"

echo
echo "Passed: $PASS  Failed: $FAIL"
[[ $FAIL -eq 0 ]] || exit 1
"#;
