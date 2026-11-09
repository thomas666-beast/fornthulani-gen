pub fn file(name: &str) -> String {
    let pascal = pascal(name);
    format!(
        r#"use fornthulani::{{Middleware, Request, Response, ThulaniError}};

pub struct {pascal};

impl Middleware for {pascal} {{
    fn before(&self, _req: &mut Request) -> Result<(), ThulaniError> {{
        // inspect or mutate the request here
        Ok(())
    }}

    fn after(&self, _resp: &mut Response) {{
        // inspect or mutate the response here
    }}
}}
"#
    )
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
