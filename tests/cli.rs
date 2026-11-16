//! Drives the real binary in a temp directory.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_fornthulani")
}

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fth-gen-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap_or_else(|_| panic!("missing {rel}"))
}

#[test]
fn orm_app_and_resource() {
    let dir = tmp("orm");
    let out = run(&dir, &["generate", "app", "demo", "--orm"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let app = dir.join("demo");

    assert!(read(&app, "Cargo.toml").contains("fornthulani-plugin-orm"));
    assert!(read(&app, "src/migrations/mod.rs").contains("fornthulani:migration-list"));

    let out = run(
        &app,
        &[
            "generate",
            "resource",
            "Post",
            "title:string",
            "body:text?",
            "published:bool",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(read(&app, "src/models/post.rs").contains("pub struct Post"));
    assert!(read(&app, "src/requests/post.rs").contains("StorePost"));
    assert!(read(&app, "src/controllers/post.rs").contains("pub fn store"));
    assert!(read(&app, "src/routes/post.rs").contains("\"/posts\""));
    assert!(read(&app, "src/models/mod.rs").contains("pub mod post;"));
    assert!(read(&app, "src/routes/mod.rs").contains("post::register(router);"));

    let migs = read(&app, "src/migrations/mod.rs");
    assert!(migs.contains("create_posts_table"));
    assert!(migs.contains("::up,"));
    // marker comments are kept so more migrations can be added
    assert!(migs.contains("fornthulani:migration-list"));

    // a duplicate is refused
    let out = run(&app, &["generate", "resource", "Post", "title:string"]);
    assert!(!out.status.success());
}

#[test]
fn model_requires_orm_project_and_valid_fields() {
    let dir = tmp("plain");
    assert!(run(&dir, &["generate", "app", "plain"]).status.success());
    let app = dir.join("plain");
    let out = run(&app, &["generate", "model", "Post", "title:string"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("ORM"));

    let dir = tmp("orm2");
    assert!(
        run(&dir, &["generate", "app", "demo2", "--orm"])
            .status
            .success()
    );
    let app = dir.join("demo2");
    for bad in ["id:integer", "type:string", "x:blob", "Bad:string"] {
        let out = run(&app, &["generate", "model", "Thing", bad]);
        assert!(!out.status.success(), "{bad}");
    }
    assert!(!app.join("src/models/thing.rs").exists());
}

#[test]
fn plugin_template_matches_the_middleware_trait() {
    let dir = tmp("plugin");
    assert!(run(&dir, &["generate", "app", "p"]).status.success());
    let app = dir.join("p");
    assert!(run(&app, &["generate", "plugin", "audit"]).status.success());
    let src = read(&app, "src/plugins/audit.rs");
    assert!(src.contains("Result<Option<Response>, ThulaniError>"));
    assert!(src.contains("fn after(&self, _req: &Request, _resp: &mut Response)"));
}

#[test]
fn path_like_names_cannot_escape() {
    let dir = tmp("escape");
    assert!(run(&dir, &["generate", "app", "e"]).status.success());
    let app = dir.join("e");
    let _ = run(&app, &["generate", "controller", "../../evil"]);
    // sanitised into src/controllers/evil.rs, never outside the project
    assert!(!dir.join("evil.rs").exists());
    assert!(!app.join("evil.rs").exists());
}
