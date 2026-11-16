pub mod app;
pub mod controller;
pub mod orm;
pub mod plugin;
pub mod route;
pub mod service;

use std::path::Path;

/// How the generated project depends on `fornthulani`.
pub enum ThulaniDep {
    /// `fornthulani = { path = "<absolute-or-relative>" }`
    Path(String),
    /// `fornthulani = "<version>"`
    Version(String),
}

impl ThulaniDep {
    pub fn to_cargo_line(&self) -> String {
        match self {
            ThulaniDep::Path(p) => format!("fornthulani = {{ path = \"{p}\" }}"),
            ThulaniDep::Version(v) => format!("fornthulani = \"{v}\""),
        }
    }

    /// A dependency line for a plugin that lives next to the core crate
    /// (path mode) or on crates.io (version mode).
    pub fn plugin_line(&self, krate: &str, version: &str, features: &[&str]) -> String {
        let feats = if features.is_empty() {
            String::new()
        } else {
            let list: Vec<String> = features.iter().map(|f| format!("\"{f}\"")).collect();
            format!(", features = [{}]", list.join(", "))
        };
        match self {
            ThulaniDep::Path(p) => {
                let dir = Path::new(p)
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(krate);
                let dir = dir.display().to_string().replace('\\', "/");
                format!("{krate} = {{ path = \"{dir}\"{feats} }}")
            }
            ThulaniDep::Version(_) => {
                format!("{krate} = {{ version = \"{version}\"{feats} }}")
            }
        }
    }
}

/// Replace `__KEY__` placeholders. Used instead of `format!` so templates
/// full of Rust braces stay readable.
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("__{k}__"), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_lines() {
        let p = ThulaniDep::Path("../fornthulani".into());
        assert_eq!(
            p.plugin_line("fornthulani-plugin-orm", "0.1", &["validation"]),
            "fornthulani-plugin-orm = { path = \"../fornthulani-plugin-orm\", features = [\"validation\"] }"
        );
        let v = ThulaniDep::Version("0.2".into());
        assert_eq!(
            v.plugin_line("fornthulani-plugin-validate", "0.1", &[]),
            "fornthulani-plugin-validate = { version = \"0.1\" }"
        );
    }

    #[test]
    fn render_replaces_all() {
        assert_eq!(
            render("__A__-__B__-__A__", &[("A", "x"), ("B", "y")]),
            "x-y-x"
        );
    }
}
