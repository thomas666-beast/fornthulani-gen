//! Templates for `fornthulani generate package <name>`.
//!
//! Produces a standalone, publishable plugin crate.
//!
//! IMPORTANT: the README is assembled from an array of lines, NOT from a
//! raw string with embedded ``` fences. That is deliberate: nested triple
//! backticks inside `r#"..."#` are how the last several attempts broke.
//! Here, every fence is its own line, and the balance test below guarantees
//! opens match closes.

use super::render;

/// The full set of files a package contains: `(relative path, contents)`.
pub fn files(pascal: &str, snake: &str, krate: &str, dep: &str) -> Vec<(String, String)> {
    vec![
        ("Cargo.toml".into(), cargo_toml(krate, dep)),
        ("README.md".into(), readme(krate, snake)),
        ("LICENSE-MIT".into(), LICENSE_MIT.into()),
        ("LICENSE-APACHE".into(), LICENSE_APACHE.into()),
        (".gitignore".into(), GITIGNORE.into()),
        ("src/lib.rs".into(), lib_rs(pascal, snake, krate)),
        (
            "tests/integration.rs".into(),
            integration_tests(pascal, krate),
        ),
    ]
}

fn cargo_toml(krate: &str, dep: &str) -> String {
    // format! is used here, so { } must be doubled.
    format!(
        r#"[package]
name = "{krate}"
version = "0.1.0"
edition = "2021"
description = "A plugin for the fornthulani micro HTTP framework."
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourname/{krate}"
keywords = ["fornthulani", "plugin", "middleware"]
categories = ["web-programming::http-server"]

[dependencies]
fornthulani = {{{dep}}}

[dev-dependencies]
reqwest = {{ version = "0.12", features = ["blocking"] }}
"#,
        krate = krate,
        dep = dep
    )
}

// ---------------------------------------------------------------------
// README — built line by line. No raw string contains a ``` fence.
// ---------------------------------------------------------------------

fn readme(krate: &str, snake: &str) -> String {
    let modname = krate.replace('-', "_");
    let pascal = pascal_from_snake(snake);

    // FENCE is the triple-backtick. Writing it as a constant keeps it out
    // of any Rust string literal that also contains other text.
    const FENCE: &str = "```";

    let mut v: Vec<String> = Vec::new();
    v.push(format!("# {krate}"));
    v.push(String::new());
    v.push("A plugin for the [fornthulani](https://crates.io/crates/fornthulani) micro HTTP framework.".into());
    v.push(String::new());

    v.push("## Install".into());
    v.push(String::new());
    v.push("Add the dependency:".into());
    v.push(String::new());
    v.push(format!("{FENCE}toml"));
    v.push("[dependencies]".into());
    v.push(format!("{krate} = \"0.1.0\""));
    v.push(FENCE.to_string());
    v.push(String::new());

    v.push("## Use".into());
    v.push(String::new());
    v.push(format!("{FENCE}rust"));
    v.push("use fornthulani::{Router, Server};".into());
    v.push(format!("use {modname}::{pascal};"));
    v.push(String::new());
    v.push("let server = Server::new(Router::new())".into());
    v.push(format!("    .with_middleware({pascal}::new());"));
    v.push(FENCE.to_string());
    v.push(String::new());

    v.push(format!(
        "`{pascal}` implements `fornthulani::Middleware`, so it runs on every"
    ));
    v.push("request in registration order.".into());
    v.push(String::new());

    v.push("## What it does".into());
    v.push(String::new());
    v.push("Describe the behaviour here once you've implemented `before` / `after`".into());
    v.push("in `src/lib.rs`.".into());
    v.push(String::new());

    v.push("## Testing".into());
    v.push(String::new());
    v.push(FENCE.to_string());
    v.push("cargo test".into());
    v.push(FENCE.to_string());
    v.push(String::new());
    v.push("Three integration tests spin a real server and exercise the plugin".into());
    v.push("through HTTP.".into());
    v.push(String::new());

    v.push("## Publishing".into());
    v.push(String::new());
    v.push("When the implementation is ready:".into());
    v.push(String::new());
    v.push(FENCE.to_string());
    v.push("cargo publish --dry-run".into());
    v.push("cargo publish".into());
    v.push(FENCE.to_string());
    v.push(String::new());

    v.push("## License".into());
    v.push(String::new());
    v.push("Dual-licensed under MIT or Apache-2.0.".into());
    v.push(String::new());

    v.join("\n")
}

// ---------------------------------------------------------------------
// src/lib.rs — raw string is safe here. No ``` anywhere.
// ---------------------------------------------------------------------

fn lib_rs(pascal: &str, snake: &str, krate: &str) -> String {
    let modname = krate.replace('-', "_");
    let _ = snake; // kept in the signature for symmetry with the caller

    // No triple backticks in this template. The doc-comment example is
    // stated in prose plus a plain `use` line. If you want a fenced code
    // example in the generated lib.rs, the right place to add it is a
    // separate template file, not this one.
    render(
        r##"//! __PASCAL__ — a middleware plugin for fornthulani.
//!
//! Attach it to a Server:
//!
//!     use fornthulani::{{Router, Server}};
//!     use __MODNAME__::__PASCAL__;
//!
//!     let server = Server::new(Router::new())
//!         .with_middleware(__PASCAL__::new());
//!

use fornthulani::{{Middleware, Request, Response, ThulaniError}};

/// One sentence about what this plugin does.
pub struct __PASCAL__ {{
    // configuration fields go here
}}

impl __PASCAL__ {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}

impl Default for __PASCAL__ {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl Middleware for __PASCAL__ {{
    fn before(&self, _req: &mut Request) -> Result<Option<Response>, ThulaniError> {{
        // Three outcomes:
        //   Ok(None)           -> continue to the controller
        //   Ok(Some(response)) -> short-circuit: this response is sent
        //   Err(e)             -> reject the request with e's status
        Ok(None)
    }}

    fn after(&self, _req: &Request, _resp: &mut Response) {{
        // Runs after the controller for a buffered response.
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn constructible() {{
        let _ = __PASCAL__::new();
    }}
}}
"##,
        &[("PASCAL", pascal), ("MODNAME", &modname)],
    )
}

// ---------------------------------------------------------------------
// tests/integration.rs — raw string, no ``` anywhere.
// ---------------------------------------------------------------------

fn integration_tests(pascal: &str, krate: &str) -> String {
    let modname = krate.replace('-', "_");
    render(
        r##"use fornthulani::{{Params, Request, Response, Router, Server, Status, ThulaniError}};
use __MODNAME__::__PASCAL__;
use std::net::TcpListener;
use std::thread;

fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {{
    Ok(Response::text(Status::OK, "hello"))
}}

fn spawn() -> (String, fornthulani::ServerHandle, thread::JoinHandle<()>) {{
    let mut router = Router::new();
    router.get("/hello", hello);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router)
        .with_middleware(__PASCAL__::new())
        .spawn(&addr.to_string())
        .unwrap();
    thread::sleep(std::time::Duration::from_millis(50));
    (format!("http://{{addr}}"), handle, join)
}}

#[test]
fn attaches_and_request_succeeds() {{
    let (base, handle, join) = spawn();
    let res = reqwest::blocking::get(format!("{{base}}/hello")).unwrap();
    assert_eq!(res.status(), 200);
    handle.shutdown();
    join.join().unwrap();
}}

#[test]
fn does_not_change_the_body() {{
    let (base, handle, join) = spawn();
    let res = reqwest::blocking::get(format!("{{base}}/hello")).unwrap();
    assert_eq!(res.text().unwrap(), "hello");
    handle.shutdown();
    join.join().unwrap();
}}

#[test]
fn is_reusable_across_servers() {{
    let (base1, h1, j1) = spawn();
    let (base2, h2, j2) = spawn();

    let r1 = reqwest::blocking::get(format!("{{base1}}/hello")).unwrap();
    let r2 = reqwest::blocking::get(format!("{{base2}}/hello")).unwrap();
    assert_eq!(r1.status(), 200);
    assert_eq!(r2.status(), 200);

    h1.shutdown();
    j1.join().unwrap();
    h2.shutdown();
    j2.join().unwrap();
}}
"##,
        &[("PASCAL", pascal), ("MODNAME", &modname)],
    )
}

fn pascal_from_snake(snake: &str) -> String {
    snake
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

const GITIGNORE: &str = "/target\nCargo.lock\n";

const LICENSE_MIT: &str = r#"MIT License

Copyright (c) 2026 yourname

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

const LICENSE_APACHE: &str = r#"                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<(String, String)> {
        files(
            "MyRateLimit",
            "my_rate_limit",
            "fornthulani-plugin-my-rate-limit",
            "path = \"../fornthulani\", version = \"0.2.0\"",
        )
    }

    #[test]
    fn files_are_present() {
        let f = sample();
        let names: Vec<&str> = f.iter().map(|(p, _)| p.as_str()).collect();
        assert!(names.contains(&"Cargo.toml"));
        assert!(names.contains(&"README.md"));
        assert!(names.contains(&"src/lib.rs"));
        assert!(names.contains(&"tests/integration.rs"));
        assert!(names.contains(&"LICENSE-MIT"));
        assert!(names.contains(&"LICENSE-APACHE"));
        assert!(names.contains(&".gitignore"));
    }

    #[test]
    fn cargo_toml_names_the_crate_and_dep() {
        let f = sample();
        let cargo = f.iter().find(|(p, _)| p == "Cargo.toml").unwrap();
        assert!(
            cargo
                .1
                .contains("name = \"fornthulani-plugin-my-rate-limit\"")
        );
        assert!(
            cargo
                .1
                .contains("fornthulani = {path = \"../fornthulani\", version = \"0.2.0\"}")
        );
    }

    #[test]
    fn lib_rs_references_the_type() {
        let f = sample();
        let lib = f.iter().find(|(p, _)| p == "src/lib.rs").unwrap();
        assert!(lib.1.contains("pub struct MyRateLimit"));
        assert!(lib.1.contains("impl Middleware for MyRateLimit"));
        assert!(!lib.1.contains("__"));
    }

    #[test]
    fn tests_reference_the_plugin() {
        let f = sample();
        let t = f.iter().find(|(p, _)| p == "tests/integration.rs").unwrap();
        assert!(t.1.contains("use fornthulani_plugin_my_rate_limit::MyRateLimit;"));
        assert!(t.1.contains("with_middleware(MyRateLimit::new())"));
        assert!(t.1.contains("#[test]"));
        assert!(!t.1.contains("__"));
    }

    #[test]
    fn readme_fences_are_balanced() {
        let f = sample();
        let r = f.iter().find(|(p, _)| p == "README.md").unwrap();
        let fence_count = r.1.matches("```").count();

        // Invariant 1: the count must be even (every open has a close).
        assert!(
            fence_count % 2 == 0,
            "unbalanced code fences in README ({fence_count} found)"
        );

        // Invariant 2: there must be at least the three blocks we know about
        // (toml install, rust use, plus one plain for commands).
        assert!(
            fence_count >= 6,
            "expected at least 3 fenced blocks, found {fence_count} fence markers"
        );

        // Invariant 3: the two typed blocks must be present.
        assert!(r.1.contains("```toml"), "README missing toml block opener");
        assert!(r.1.contains("```rust"), "README missing rust block opener");
    }
}
