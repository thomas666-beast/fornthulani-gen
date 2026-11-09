pub fn file(name: &str) -> String {
    format!(
        r#"use fornthulani::Router;

use crate::controllers::{name};

pub fn register(router: &mut Router) {{
    router.get("/{name}", {name}::{name}).name("{name}");
}}
"#
    )
}
