use std::path::PathBuf;

use super::ThulaniDep;

pub fn files(name: &str, dep: &ThulaniDep) -> Vec<(PathBuf, String)> {
    vec![
        (PathBuf::from("Cargo.toml"), cargo_toml(name, dep)),
        (PathBuf::from("fornthulani.toml"), FORNTHULANI_TOML.into()),
        (PathBuf::from(".gitignore"), GITIGNORE.into()),
        (PathBuf::from("README.md"), readme(name)),
        (PathBuf::from("src/main.rs"), MAIN.into()),
        (PathBuf::from("src/app.rs"), APP.into()),
        (PathBuf::from("src/config.rs"), CONFIG.into()),
        (PathBuf::from("src/routes/mod.rs"), ROUTES_MOD.into()),
        (PathBuf::from("src/routes/hello.rs"), ROUTES_HELLO.into()),
        (PathBuf::from("src/controllers/mod.rs"), CONTROLLERS_MOD.into()),
        (PathBuf::from("src/controllers/hello.rs"), CONTROLLERS_HELLO.into()),
        (PathBuf::from("src/services/mod.rs"), SERVICES_MOD.into()),
        (PathBuf::from("src/services/hello_service.rs"), SERVICES_HELLO.into()),
        (PathBuf::from("src/plugins/mod.rs"), PLUGINS_MOD.into()),
    ]
}

fn cargo_toml(name: &str, dep: &ThulaniDep) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{dep_line}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
toml = "0.8"
"#,
        dep_line = dep.to_cargo_line(),
    )
}

fn readme(name: &str) -> String {
    format!(
        r#"# {name}

A fornthulani application.

## Run

    cargo run

## Layout

    src/
      main.rs           entry point
      app.rs            wires routes into a Router
      config.rs         reads fornthulani.toml
      routes/           URL space (one file per route group)
      controllers/      request/response handlers (one file per endpoint)
      services/         business logic (one file per domain concern)
      plugins/          your own middleware (implements fornthulani::Middleware)

## The controller -> service pattern

Controllers stay thin. They read the request, call a service, and
return a response. Business logic lives in `services/`.

    // controllers/users.rs
    pub fn list(_: &Request, _: &Params) -> Result<Response, ThulaniError> {{
        let users = services::users_service::list()?;
        Ok(Response::json(Status::OK, &users))
    }}

    // services/users_service.rs
    pub fn list() -> Result<Vec<User>, ThulaniError> {{
        // load, filter, sort, cache, whatever
        Ok(vec![])
    }}

Why:

  - Controllers become easy to read (5 lines, no logic).
  - Services are plain Rust - easy to unit test.
  - Services can be reused by CLI tools, scheduled jobs, tests.

This is a hint, not an obligation. Small controllers can stay small
without a service. When a controller starts doing more than reading
the request and calling something, move that work into a service.

## Generate more

    fornthulani generate route <name>       # add a route + controller
    fornthulani generate controller <name>  # add just a controller
    fornthulani generate service <name>     # add a service
    fornthulani generate plugin <name>      # add a middleware in src/plugins/
"#
    )
}

const FORNTHULANI_TOML: &str = r#"[server]
host = "127.0.0.1"
port = 8080

[response]
default_content_type = "application/json"
"#;

const GITIGNORE: &str = "/target\n";

const MAIN: &str = r#"mod app;
mod config;
mod controllers;
mod plugins;
mod routes;
mod services;

use fornthulani::Server;

fn main() -> std::io::Result<()> {
    let cfg = config::Config::load().unwrap_or_default();
    let router = app::build_router();

    let mut server = Server::new(router);
    server = plugins::register(server);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    println!("listening on http://{addr}");
    server.run_blocking(&addr)
}
"#;

const APP: &str = r#"use fornthulani::Router;

use crate::routes;

pub fn build_router() -> Router {
    let mut router = Router::new();
    routes::register(&mut router);
    router
}
"#;

const CONFIG: &str = r#"use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub response: ResponseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ResponseConfig {
    pub default_content_type: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            response: ResponseConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
        }
    }
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            default_content_type: "application/json".into(),
        }
    }
}

impl Config {
    pub fn load() -> std::io::Result<Self> {
        let raw = match std::fs::read_to_string("fornthulani.toml") {
            Ok(s) => s,
            Err(_) => return Ok(Self::default()),
        };
        toml::from_str(&raw).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
"#;

const ROUTES_MOD: &str = r#"pub mod hello;

use fornthulani::Router;

pub fn register(router: &mut Router) {
    hello::register(router);
}
"#;

const ROUTES_HELLO: &str = r#"use fornthulani::Router;

use crate::controllers::hello;

pub fn register(router: &mut Router) {
    router.get("/hello", hello::hello).name("hello");
}
"#;

const CONTROLLERS_MOD: &str = "pub mod hello;\n";

const CONTROLLERS_HELLO: &str = r#"use fornthulani::{Params, Request, Response, Status, ThulaniError};

use crate::services::hello_service;

/// GET /hello
///
/// Notice how the controller just delegates to the service.
/// Business logic lives in `services/hello_service.rs`.
pub fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let greeting = hello_service::greet("fornthulani");
    Ok(Response::json(Status::OK, &greeting))
}
"#;

const SERVICES_MOD: &str = "pub mod hello_service;\n";

const SERVICES_HELLO: &str = r#"/// A service: plain Rust that holds business logic.
///
/// Services do not know about HTTP. They take data, return data.
/// Controllers call services; services call databases, APIs, files, etc.
///
/// This is a hint, not an obligation. Small controllers can stay small
/// without a service. When a controller starts doing more than reading
/// the request and calling something, move that work into a service.

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
"#;

const PLUGINS_MOD: &str = r#"use fornthulani::Server;

/// Register all project plugins against the server.
///
/// Generated plugins add themselves here:
///
///     pub mod my_plugin;
///     ...
///     server = server.with_middleware(my_plugin::MyPlugin);
pub fn register(server: Server) -> Server {
    server
}
"#;
