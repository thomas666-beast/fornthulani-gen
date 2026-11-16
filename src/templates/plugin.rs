use crate::names::pascal;

pub fn file(name: &str) -> String {
    let pascal = pascal(name);
    format!(
        r#"use fornthulani::{{Middleware, Request, Response, ThulaniError}};

pub struct {pascal};

impl Middleware for {pascal} {{
    /// Runs before the controller.
    ///   Ok(None)        -> continue
    ///   Ok(Some(resp))  -> stop here and send `resp`
    ///   Err(e)          -> reject the request
    fn before(&self, _req: &mut Request) -> Result<Option<Response>, ThulaniError> {{
        Ok(None)
    }}

    /// Runs after the controller, in reverse registration order.
    fn after(&self, _req: &Request, _resp: &mut Response) {{}}
}}
"#
    )
}
