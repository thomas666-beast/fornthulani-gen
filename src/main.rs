use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

mod names;
mod templates;

use names::{Field, parse_fields, pascal, plural, snake, timestamp_now};
use templates::{ThulaniDep, app, controller, orm, plugin, route, service};

#[derive(Parser)]
#[command(name = "fornthulani", version, about = "fornthulani project generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate things (app, route, controller, service, plugin, model, ...)
    Generate {
        #[command(subcommand)]
        what: Generate,
    },
}

#[derive(Subcommand)]
enum Generate {
    /// A new project. Add --orm for models, migrations and validation.
    App {
        name: String,

        #[arg(long, conflicts_with = "fornthulani_version")]
        fornthulani_path: Option<String>,

        #[arg(long, conflicts_with = "fornthulani_path")]
        fornthulani_version: Option<String>,

        /// Include the ORM + validation plugins, a SQLite connection and
        /// `migrate` commands.
        #[arg(long)]
        orm: bool,
    },

    Route {
        name: String,
    },

    Controller {
        name: String,
    },

    Service {
        name: String,
    },

    Plugin {
        name: String,
    },

    /// Model + migration:  generate model Post title:string body:text?
    Model {
        name: String,
        /// name:type or name:type? (string, text, integer, float, bool, email)
        #[arg(value_name = "FIELD")]
        fields: Vec<String>,
    },

    /// An empty migration:  generate migration add_index_to_posts
    Migration {
        name: String,
    },

    /// Validation rules (form request) for a model:  generate request Post title:string
    Request {
        name: String,
        #[arg(value_name = "FIELD")]
        fields: Vec<String>,
    },

    /// Model, migration, validation, CRUD controller and routes in one go.
    Resource {
        name: String,
        #[arg(value_name = "FIELD")]
        fields: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { what } => match what {
            Generate::App {
                name,
                fornthulani_path,
                fornthulani_version,
                orm,
            } => {
                let dep = match (fornthulani_path, fornthulani_version) {
                    (Some(p), None) => ThulaniDep::Path(p),
                    (None, Some(v)) => ThulaniDep::Version(v),
                    (None, None) => ThulaniDep::Path("../fornthulani".to_string()),
                    _ => unreachable!("clap prevents this"),
                };
                generate_app(&name, dep, orm)
            }
            Generate::Route { name } => generate_route(&name),
            Generate::Controller { name } => generate_controller(&name),
            Generate::Plugin { name } => generate_plugin(&name),
            Generate::Service { name } => generate_service(&name),
            Generate::Model { name, fields } => generate_model(&name, &fields),
            Generate::Migration { name } => generate_migration(&name),
            Generate::Request { name, fields } => generate_request(&name, &fields),
            Generate::Resource { name, fields } => generate_resource(&name, &fields),
        },
    }
}

// ---------- generate app ----------

fn generate_app(name: &str, dep: ThulaniDep, orm: bool) -> Result<()> {
    if !is_valid_crate_name(name) {
        bail!("`{name}` is not a valid crate name (a-z, 0-9, '-', '_')");
    }
    let root = PathBuf::from(name);
    if root.exists() {
        bail!("`{name}` already exists");
    }

    for (rel, content) in app::files(name, &dep, orm) {
        write_file(&root, &rel, &content)?;
    }
    init_git(&root)?;

    println!(
        "created {name}{}",
        if orm { " (with ORM + validation)" } else { "" }
    );
    println!("  cd {name}");
    if orm {
        println!("  fornthulani generate resource Post title:string body:text?");
        println!("  cargo run -- migrate");
    }
    println!("  cargo run");
    Ok(())
}

// ---------- route / controller / plugin / service ----------

fn generate_route(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name)?;

    let route_rel = PathBuf::from("src/routes").join(format!("{mod_name}.rs"));
    let ctrl_rel = PathBuf::from("src/controllers").join(format!("{mod_name}.rs"));

    if root.join(&route_rel).exists() {
        bail!("route `{mod_name}` already exists");
    }
    if root.join(&ctrl_rel).exists() {
        bail!("controller `{mod_name}` already exists");
    }

    write_file(&root, &route_rel, &route::file(&mod_name))?;
    write_file(&root, &ctrl_rel, &controller::file(&mod_name))?;

    append_mod(&root.join("src/routes/mod.rs"), &mod_name)?;
    append_register(&root.join("src/routes/mod.rs"), &mod_name)?;
    append_mod(&root.join("src/controllers/mod.rs"), &mod_name)?;

    println!("added route {mod_name}");
    Ok(())
}

fn generate_controller(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name)?;
    let rel = PathBuf::from("src/controllers").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("controller `{mod_name}` already exists");
    }
    write_file(&root, &rel, &controller::file(&mod_name))?;
    append_mod(&root.join("src/controllers/mod.rs"), &mod_name)?;
    println!("added controller {mod_name}");
    Ok(())
}

fn generate_plugin(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name)?;
    let rel = PathBuf::from("src/plugins").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("plugin `{mod_name}` already exists");
    }
    write_file(&root, &rel, &plugin::file(&mod_name))?;
    append_mod(&root.join("src/plugins/mod.rs"), &mod_name)?;
    println!("added plugin {mod_name}");
    println!("  add to src/plugins/mod.rs::register:");
    println!(
        "      let server = server.with_middleware({mod_name}::{});",
        pascal(&mod_name)
    );
    Ok(())
}

fn generate_service(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name)?;
    let rel = PathBuf::from("src/services").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("service `{mod_name}` already exists");
    }
    write_file(&root, &rel, &service::file(&mod_name))?;
    append_mod(&root.join("src/services/mod.rs"), &mod_name)?;
    println!("added service {mod_name}");
    Ok(())
}

// ---------- ORM generators ----------

struct Names {
    snake: String,
    pascal: String,
    table: String,
}

fn model_names(name: &str) -> Result<Names> {
    let s = sanitize_ident(name)?;
    Ok(Names {
        pascal: pascal(&s),
        table: plural(&s),
        snake: s,
    })
}

fn require_orm(root: &Path) -> Result<()> {
    let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    if !cargo.contains("fornthulani-plugin-orm") {
        bail!(
            "this project does not use the ORM plugin.\n\
             Create projects with `fornthulani generate app <name> --orm`, or add\n\
             fornthulani-plugin-orm and fornthulani-plugin-validate to Cargo.toml and create\n\
             src/models, src/requests, src/migrations (see the plugin READMEs)."
        );
    }
    Ok(())
}

fn fields_or_bail(specs: &[String]) -> Result<Vec<Field>> {
    parse_fields(specs).map_err(|e| anyhow!(e))
}

/// Write the migration file and register it in `src/migrations/mod.rs`.
fn create_migration(root: &Path, slug: &str, body: &str) -> Result<String> {
    let full = format!("{}_{slug}", timestamp_now());
    let module = format!("m{full}");
    let rel = PathBuf::from("src/migrations").join(format!("{module}.rs"));
    if root.join(&rel).exists() {
        bail!("migration `{module}` already exists (wait a second and retry)");
    }
    write_file(root, &rel, body)?;

    let mod_rs = root.join("src/migrations/mod.rs");
    let mut s = fs::read_to_string(&mod_rs)
        .with_context(|| format!("reading {} (is this an --orm project?)", mod_rs.display()))?;
    insert_before_marker(
        &mut s,
        "// fornthulani:migration-mods",
        &format!("pub mod {module};\n"),
    )?;
    insert_before_marker(
        &mut s,
        "// fornthulani:migration-list",
        &format!(
            "        Migration {{ name: \"{full}\", up: {module}::up, down: {module}::down }},\n"
        ),
    )?;
    fs::write(&mod_rs, s)?;
    Ok(full)
}

fn generate_model(name: &str, specs: &[String]) -> Result<()> {
    let root = project_root()?;
    require_orm(&root)?;
    let n = model_names(name)?;
    let fields = fields_or_bail(specs)?;

    let rel = PathBuf::from("src/models").join(format!("{}.rs", n.snake));
    if root.join(&rel).exists() {
        bail!("model `{}` already exists", n.snake);
    }
    write_file(&root, &rel, &orm::model(&n.pascal, &n.table, &fields))?;
    append_mod(&root.join("src/models/mod.rs"), &n.snake)?;

    let migration = create_migration(
        &root,
        &format!("create_{}_table", n.table),
        &orm::migration(&n.table, &fields),
    )?;
    println!("added model {} (table `{}`)", n.pascal, n.table);
    println!("added migration {migration}");
    println!("  cargo run -- migrate");
    Ok(())
}

fn generate_migration(name: &str) -> Result<()> {
    let root = project_root()?;
    require_orm(&root)?;
    let slug = sanitize_ident(name)?;
    let full = create_migration(&root, &slug, &orm::blank_migration())?;
    println!("added migration {full}");
    Ok(())
}

fn generate_request(name: &str, specs: &[String]) -> Result<()> {
    let root = project_root()?;
    require_orm(&root)?;
    let n = model_names(name)?;
    let fields = fields_or_bail(specs)?;

    let rel = PathBuf::from("src/requests").join(format!("{}.rs", n.snake));
    if root.join(&rel).exists() {
        bail!("request `{}` already exists", n.snake);
    }
    write_file(&root, &rel, &orm::request(&n.pascal, &n.table, &fields))?;
    append_mod(&root.join("src/requests/mod.rs"), &n.snake)?;
    println!("added requests Store{0} / Update{0}", n.pascal);
    Ok(())
}

fn generate_resource(name: &str, specs: &[String]) -> Result<()> {
    let root = project_root()?;
    require_orm(&root)?;
    let n = model_names(name)?;
    let fields = fields_or_bail(specs)?;
    if fields.is_empty() {
        bail!("give at least one field, e.g. `title:string`");
    }

    let model_rel = PathBuf::from("src/models").join(format!("{}.rs", n.snake));
    let req_rel = PathBuf::from("src/requests").join(format!("{}.rs", n.snake));
    let ctrl_rel = PathBuf::from("src/controllers").join(format!("{}.rs", n.snake));
    let route_rel = PathBuf::from("src/routes").join(format!("{}.rs", n.snake));
    for rel in [&model_rel, &req_rel, &ctrl_rel, &route_rel] {
        if root.join(rel).exists() {
            bail!("{} already exists", rel.display());
        }
    }

    write_file(&root, &model_rel, &orm::model(&n.pascal, &n.table, &fields))?;
    write_file(&root, &req_rel, &orm::request(&n.pascal, &n.table, &fields))?;
    write_file(
        &root,
        &ctrl_rel,
        &orm::controller(&n.snake, &n.pascal, &fields),
    )?;
    write_file(&root, &route_rel, &orm::routes(&n.snake, &n.table))?;

    append_mod(&root.join("src/models/mod.rs"), &n.snake)?;
    append_mod(&root.join("src/requests/mod.rs"), &n.snake)?;
    append_mod(&root.join("src/controllers/mod.rs"), &n.snake)?;
    append_mod(&root.join("src/routes/mod.rs"), &n.snake)?;
    append_register(&root.join("src/routes/mod.rs"), &n.snake)?;

    let migration = create_migration(
        &root,
        &format!("create_{}_table", n.table),
        &orm::migration(&n.table, &fields),
    )?;

    println!("added resource {} -> /{}", n.pascal, n.table);
    println!("  model, requests, controller, routes, migration {migration}");
    println!("  cargo run -- migrate && cargo run");
    Ok(())
}

// ---------- helpers ----------

fn project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("src/main.rs").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!("not inside a fornthulani project (no Cargo.toml + src/main.rs found)");
        }
    }
}

fn write_file(root: &Path, rel: &Path, content: &str) -> Result<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn append_mod(mod_rs: &Path, mod_name: &str) -> Result<()> {
    let mut s = fs::read_to_string(mod_rs).unwrap_or_default();
    let line = format!("pub mod {mod_name};\n");
    if !s.contains(&line) {
        if !s.is_empty() && !s.ends_with('\n') {
            s.push('\n');
        }
        s.push_str(&line);
        fs::write(mod_rs, s)?;
    }
    Ok(())
}

fn append_register(mod_rs: &Path, mod_name: &str) -> Result<()> {
    let mut s = fs::read_to_string(mod_rs)?;
    let marker = "    hello::register(router);\n";
    let line = format!("    {mod_name}::register(router);\n");
    if !s.contains(&line) {
        if let Some(idx) = s.find(marker) {
            let insert = idx + marker.len();
            s.insert_str(insert, &line);
        } else if let Some(idx) = s.rfind('}') {
            s.insert_str(idx, &line);
        } else {
            s.push_str(&line);
        }
        fs::write(mod_rs, s)?;
    }
    Ok(())
}

/// Insert `text` on its own line(s) directly above the line holding `marker`.
fn insert_before_marker(s: &mut String, marker: &str, text: &str) -> Result<()> {
    let idx = s
        .find(marker)
        .ok_or_else(|| anyhow!("marker `{marker}` not found in src/migrations/mod.rs"))?;
    let line_start = s[..idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
    s.insert_str(line_start, text);
    Ok(())
}

fn init_git(root: &Path) -> Result<()> {
    let _ = std::process::Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(root)
        .status();
    Ok(())
}

fn is_valid_crate_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

/// A safe snake_case module/identifier built from user input.
/// Rejects names that would not make a valid Rust module.
fn sanitize_ident(s: &str) -> Result<String> {
    let id = snake(s);
    let valid = id.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    if !valid {
        bail!("`{s}` does not make a valid name (start with a letter; letters, digits, _ only)");
    }
    if matches!(
        id.as_str(),
        "mod" | "self" | "super" | "crate" | "type" | "match" | "use" | "fn" | "gen" | "async"
    ) {
        bail!("`{id}` is a Rust keyword; pick another name");
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_insertion() {
        let mut s = "a\n// m1\nb\n        // m2\n".to_string();
        insert_before_marker(&mut s, "// m1", "pub mod x;\n").unwrap();
        insert_before_marker(&mut s, "// m2", "        item,\n").unwrap();
        assert_eq!(s, "a\npub mod x;\n// m1\nb\n        item,\n        // m2\n");
        assert!(insert_before_marker(&mut s, "// nope", "x").is_err());
    }

    #[test]
    fn ident_sanitising() {
        assert_eq!(sanitize_ident("BlogPost").unwrap(), "blog_post");
        assert_eq!(sanitize_ident("my-thing").unwrap(), "my_thing");
        assert!(sanitize_ident("1abc").is_err());
        assert!(sanitize_ident("mod").is_err());
        assert!(sanitize_ident("../../etc").is_ok_and(|s| !s.contains('/') && !s.contains('.')));
        assert!(sanitize_ident("").is_err());
    }
}
