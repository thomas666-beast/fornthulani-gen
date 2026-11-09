pub fn file(name: &str) -> String {
    format!(
        r#"/// A service: plain Rust that holds business logic.
///
/// Services do not know about HTTP. They take data, return data.
/// Controllers call services; services call databases, APIs, files, etc.

use fornthulani::ThulaniError;

pub fn {name}() -> Result<(), ThulaniError> {{
    // TODO: implement
    Ok(())
}}
"#
    )
}
