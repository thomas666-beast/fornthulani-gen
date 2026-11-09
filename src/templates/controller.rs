pub fn file(name: &str) -> String {
    format!(
        r#"use fornthulani::{{Params, Request, Response, ThulaniError, Status}};

pub fn {name}(_: &Request, _: &Params) -> Result<Response, ThulaniError> {{
    Ok(Response::text(Status::OK, "{name}"))
}}
"#
    )
}
