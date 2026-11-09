pub mod app;
pub mod controller;
pub mod plugin;
pub mod route;
pub mod service;

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
}
