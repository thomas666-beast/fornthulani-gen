use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

mod templates;

use templates::{app, controller, plugin, route, service, ThulaniDep};

#[derive(Parser)]
#[command(name = "fornthulani", version, about = "fornthulani project generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate things (app, route, controller, plugin)
    Generate {
        #[command(subcommand)]
        what: Generate,
    },
}

#[derive(Subcommand)]
enum Generate {
    App {
        name: String,

        #[arg(long, conflicts_with = "fornthulani_version")]
        fornthulani_path: Option<String>,

        #[arg(long, conflicts_with = "fornthulani_path")]
        fornthulani_version: Option<String>,
    },

    Route { name: String },

    Controller { name: String },

    Service { name: String },

    Plugin { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { what } => match what {
            Generate::App {
                name,
                fornthulani_path,
                fornthulani_version,
            } => {
                let dep = match (fornthulani_path, fornthulani_version) {
                    (Some(p), None) => ThulaniDep::Path(p),
                    (None, Some(v)) => ThulaniDep::Version(v),
                    (None, None) => ThulaniDep::Path("../fornthulani".to_string()),
                    _ => unreachable!("clap prevents this"),
                };
                generate_app(&name, dep)
            }
            Generate::Route { name } => generate_route(&name),
            Generate::Controller { name } => generate_controller(&name),
            Generate::Plugin { name } => generate_plugin(&name),
            Generate::Service { name } => generate_service(&name),
        },
    }
}

// ---------- generate app ----------

fn generate_app(name: &str, dep: ThulaniDep) -> Result<()> {
    if !is_valid_crate_name(name) {
        bail!("`{name}` is not a valid crate name (a-z, 0-9, '-', '_')");
    }
    let root = PathBuf::from(name);
    if root.exists() {
        bail!("`{name}` already exists");
    }

    let files = app::files(name, &dep);
    for (rel, content) in files {
        write_file(&root, &rel, &content)?;
    }
    init_git(&root)?;

    println!("created {name}");
    println!("  cd {name}");
    println!("  cargo run");
    Ok(())
}

// ---------- generate route ----------

fn generate_route(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name);

    let route_rel = PathBuf::from("src/routes").join(format!("{mod_name}.rs"));
    let ctrl_rel = PathBuf::from("src/controllers").join(format!("{mod_name}.rs"));

    if root.join(&route_rel).exists() {
        bail!("route `{mod_name}` already exists");
    }

    write_file(&root, &route_rel, &route::file(&mod_name))?;
    write_file(&root, &ctrl_rel, &controller::file(&mod_name))?;

    append_mod(&root.join("src/routes/mod.rs"), &mod_name)?;
    append_register(&root.join("src/routes/mod.rs"), &mod_name)?;
    append_mod(&root.join("src/controllers/mod.rs"), &mod_name)?;

    println!("added route {mod_name}");
    Ok(())
}

// ---------- generate controller ----------

fn generate_controller(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name);
    let rel = PathBuf::from("src/controllers").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("controller `{mod_name}` already exists");
    }
    write_file(&root, &rel, &controller::file(&mod_name))?;
    append_mod(&root.join("src/controllers/mod.rs"), &mod_name)?;
    println!("added controller {mod_name}");
    Ok(())
}

// ---------- generate plugin ----------

fn generate_plugin(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name);
    let rel = PathBuf::from("src/plugins").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("plugin `{mod_name}` already exists");
    }
    write_file(&root, &rel, &plugin::file(&mod_name))?;
    append_mod(&root.join("src/plugins/mod.rs"), &mod_name)?;
    println!("added plugin {mod_name}");
    println!("  add to src/plugins/mod.rs::register:");
    println!(
        "      server = server.with_middleware({mod_name}::{});",
        pascal(&mod_name)
    );
    Ok(())
}

// ---------- generate service ----------

fn generate_service(name: &str) -> Result<()> {
    let root = project_root()?;
    let mod_name = sanitize_ident(name);
    let rel = PathBuf::from("src/services").join(format!("{mod_name}.rs"));
    if root.join(&rel).exists() {
        bail!("service `{mod_name}` already exists");
    }
    write_file(&root, &rel, &service::file(&mod_name))?;
    append_mod(&root.join("src/services/mod.rs"), &mod_name)?;
    println!("added service {mod_name}");
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

fn sanitize_ident(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_ascii_lowercase()
}

fn pascal(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
