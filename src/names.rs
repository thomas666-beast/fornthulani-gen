//! Name handling and field-spec parsing shared by the generators.

use std::time::{SystemTime, UNIX_EPOCH};

/// `BlogPost`, `blog-post`, `blog post` -> `blog_post`.
pub fn snake(s: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_lower = true;
        } else {
            out.push('_');
            prev_lower = false;
        }
    }
    let mut collapsed = String::new();
    for c in out.chars() {
        if c == '_' && collapsed.ends_with('_') {
            continue;
        }
        collapsed.push(c);
    }
    collapsed.trim_matches('_').to_string()
}

/// `blog_post` -> `BlogPost`.
pub fn pascal(s: &str) -> String {
    snake(s)
        .split('_')
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

/// A small English pluraliser, good enough for table names.
pub fn plural(s: &str) -> String {
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    let consonant_y_stem = s
        .strip_suffix('y')
        .filter(|stem| stem.chars().last().is_some_and(|c| !vowels.contains(&c)));
    if let Some(stem) = consonant_y_stem {
        return format!("{stem}ies");
    }
    if s.ends_with('s')
        || s.ends_with('x')
        || s.ends_with('z')
        || s.ends_with("ch")
        || s.ends_with("sh")
    {
        return format!("{s}es");
    }
    format!("{s}s")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String,
    Text,
    Integer,
    Float,
    Bool,
    Email,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
    pub nullable: bool,
}

impl Field {
    pub fn rust_type(&self) -> &'static str {
        match self.ty {
            FieldType::String | FieldType::Text | FieldType::Email => "String",
            FieldType::Integer => "i64",
            FieldType::Float => "f64",
            FieldType::Bool => "bool",
        }
    }

    /// The `Blueprint` call (without the trailing `;`).
    pub fn column(&self) -> String {
        let n = &self.name;
        let base = match self.ty {
            FieldType::String | FieldType::Email => format!("t.string(\"{n}\", 255)"),
            FieldType::Text => format!("t.text(\"{n}\")"),
            FieldType::Integer => format!("t.integer(\"{n}\")"),
            FieldType::Float => format!("t.float(\"{n}\")"),
            FieldType::Bool => format!("t.boolean(\"{n}\")"),
        };
        let mut s = base;
        if self.ty == FieldType::Bool && !self.nullable {
            s.push_str(".default(false)");
        }
        if self.nullable {
            s.push_str(".nullable()");
        }
        s
    }

    /// Validation rule body for this type, e.g. `string|max:255`.
    pub fn type_rule(&self) -> &'static str {
        match self.ty {
            FieldType::String => "string|max:255",
            FieldType::Text => "string",
            FieldType::Email => "string|email|max:255",
            FieldType::Integer => "integer",
            FieldType::Float => "numeric",
            FieldType::Bool => "boolean",
        }
    }
}

const RESERVED_COLUMNS: &[&str] = &["id", "created_at", "updated_at", "deleted_at"];
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while",
];

/// Parse `name:type` / `name:type?` specs. A missing type means `string`.
pub fn parse_fields(specs: &[String]) -> Result<Vec<Field>, String> {
    let mut out: Vec<Field> = Vec::new();
    for spec in specs {
        let (name, ty) = match spec.split_once(':') {
            Some((n, t)) => (n, t),
            None => (spec.as_str(), "string"),
        };
        let (ty, nullable) = match ty.strip_suffix('?') {
            Some(t) => (t, true),
            None => (ty, false),
        };
        let valid = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid {
            return Err(format!(
                "invalid field name `{name}` (use snake_case: a-z, 0-9, _)"
            ));
        }
        if RESERVED_COLUMNS.contains(&name) {
            return Err(format!("`{name}` is added automatically, remove it"));
        }
        if KEYWORDS.contains(&name) {
            return Err(format!("`{name}` is a Rust keyword; pick another name"));
        }
        if out.iter().any(|f| f.name == name) {
            return Err(format!("field `{name}` is listed twice"));
        }
        let ty = match ty {
            "string" => FieldType::String,
            "text" => FieldType::Text,
            "integer" | "int" => FieldType::Integer,
            "float" => FieldType::Float,
            "bool" | "boolean" => FieldType::Bool,
            "email" => FieldType::Email,
            other => {
                return Err(format!(
                    "unknown type `{other}` for `{name}` (string, text, integer, float, bool, email)"
                ));
            }
        };
        out.push(Field {
            name: name.to_string(),
            ty,
            nullable,
        });
    }
    Ok(out)
}

/// `YYYYMMDDHHMMSS` in UTC, for migration file names.
pub fn timestamp_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_timestamp(secs)
}

fn format_timestamp(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!(
        "{y:04}{m:02}{d:02}{:02}{:02}{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_conversion() {
        assert_eq!(snake("BlogPost"), "blog_post");
        assert_eq!(snake("blog-post"), "blog_post");
        assert_eq!(snake("Blog  Post!"), "blog_post");
        assert_eq!(snake("HTTPServer"), "httpserver");
        assert_eq!(snake("__x__"), "x");
        assert_eq!(pascal("blog_post"), "BlogPost");
        assert_eq!(pascal("post"), "Post");
    }

    #[test]
    fn pluralisation() {
        assert_eq!(plural("post"), "posts");
        assert_eq!(plural("category"), "categories");
        assert_eq!(plural("day"), "days");
        assert_eq!(plural("box"), "boxes");
        assert_eq!(plural("address"), "addresses");
        assert_eq!(plural("match"), "matches");
    }

    #[test]
    fn field_parsing() {
        let f = parse_fields(&[
            "title:string".into(),
            "body:text?".into(),
            "published:bool".into(),
            "age".into(),
        ])
        .unwrap();
        assert_eq!(f.len(), 4);
        assert!(f[1].nullable);
        assert_eq!(f[1].ty, FieldType::Text);
        assert_eq!(f[3].ty, FieldType::String);
        assert_eq!(f[2].column(), "t.boolean(\"published\").default(false)");
        assert_eq!(f[1].column(), "t.text(\"body\").nullable()");
        assert_eq!(f[0].rust_type(), "String");
    }

    #[test]
    fn field_parsing_rejects_bad_input() {
        for bad in [
            "Title:string",
            "ti tle:string",
            "id:integer",
            "type:string",
            "x:blob",
            "1a:string",
            "created_at:string",
        ] {
            assert!(parse_fields(&[bad.to_string()]).is_err(), "{bad}");
        }
        assert!(parse_fields(&["a:string".into(), "a:text".into()]).is_err());
        // names end up inside generated Rust and SQL: no injection via the spec
        assert!(parse_fields(&["a\";drop:string".into()]).is_err());
    }

    #[test]
    fn timestamps() {
        assert_eq!(format_timestamp(0), "19700101000000");
        assert_eq!(format_timestamp(1_709_210_096), "20240229123456");
        assert_eq!(timestamp_now().len(), 14);
    }
}
