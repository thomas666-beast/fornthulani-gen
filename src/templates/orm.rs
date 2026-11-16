//! Templates for `generate model | migration | request | resource`.

use super::render;
use crate::names::{Field, FieldType};

fn quoted_list(items: impl Iterator<Item = String>) -> String {
    items
        .map(|i| format!("\"{i}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn model(pascal: &str, table: &str, fields: &[Field]) -> String {
    let mut field_lines = String::new();
    for f in fields {
        let ty = if f.nullable {
            format!("Option<{}>", f.rust_type())
        } else {
            f.rust_type().to_string()
        };
        field_lines.push_str(&format!("    pub {}: {ty},\n", f.name));
    }
    let fillable = quoted_list(fields.iter().map(|f| f.name.clone()));
    let bools = quoted_list(
        fields
            .iter()
            .filter(|f| f.ty == FieldType::Bool)
            .map(|f| f.name.clone()),
    );
    render(
        r##"use fornthulani_plugin_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct __PASCAL__ {
    pub id: Option<i64>,
__FIELDS__    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Model for __PASCAL__ {
    const TABLE: &'static str = "__TABLE__";

    /// Mass-assignment allow-list: only these keys can be set through
    /// `create()` / `update()`. Add a column here deliberately.
    const FILLABLE: &'static [&'static str] = &[__FILLABLE__];

    /// Columns removed from `public_json()` (e.g. "password").
    const HIDDEN: &'static [&'static str] = &[];

    /// `bool` columns (SQLite stores 0/1).
    const BOOLS: &'static [&'static str] = &[__BOOLS__];

    fn id(&self) -> Option<i64> {
        self.id
    }
}
"##,
        &[
            ("PASCAL", pascal),
            ("TABLE", table),
            ("FIELDS", field_lines.as_str()),
            ("FILLABLE", fillable.as_str()),
            ("BOOLS", bools.as_str()),
        ],
    )
}

pub fn migration(table: &str, fields: &[Field]) -> String {
    let mut cols = String::new();
    for f in fields {
        cols.push_str(&format!("        {};\n", f.column()));
    }
    render(
        r##"use fornthulani_plugin_orm::{Executor, OrmResult, Schema};

pub fn up(db: &dyn Executor) -> OrmResult<()> {
    Schema::create(db, "__TABLE__", |t| {
        t.id();
__COLUMNS__        t.timestamps();
    })
}

pub fn down(db: &dyn Executor) -> OrmResult<()> {
    Schema::drop_if_exists(db, "__TABLE__")
}
"##,
        &[("TABLE", table), ("COLUMNS", cols.as_str())],
    )
}

pub fn blank_migration() -> String {
    r##"use fornthulani_plugin_orm::{Executor, OrmResult};

pub fn up(_db: &dyn Executor) -> OrmResult<()> {
    // Schema::create(db, "table", |t| { ... })
    Ok(())
}

pub fn down(_db: &dyn Executor) -> OrmResult<()> {
    Ok(())
}
"##
    .to_string()
}

pub fn request(pascal: &str, table: &str, fields: &[Field]) -> String {
    let mut store_fields = String::new();
    let mut store_rules = String::new();
    let mut update_fields = String::new();
    let mut update_rules = String::new();

    for f in fields {
        // On create, bool columns have a default, so they are optional.
        let optional_on_store = f.nullable || f.ty == FieldType::Bool;
        let rust = f.rust_type();

        if optional_on_store {
            store_fields.push_str(&format!(
                "    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub {}: Option<{rust}>,\n",
                f.name
            ));
            store_rules.push_str(&format!(
                "            .field(\"{}\", \"nullable|{}\")\n",
                f.name,
                f.type_rule()
            ));
        } else {
            store_fields.push_str(&format!("    pub {}: {rust},\n", f.name));
            store_rules.push_str(&format!(
                "            .field(\"{}\", \"required|{}\")\n",
                f.name,
                f.type_rule()
            ));
        }

        update_fields.push_str(&format!(
            "    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub {}: Option<{rust}>,\n",
            f.name
        ));
        let presence = if f.nullable {
            "sometimes|nullable"
        } else {
            "sometimes|required"
        };
        update_rules.push_str(&format!(
            "            .field(\"{}\", \"{presence}|{}\")\n",
            f.name,
            f.type_rule()
        ));
    }

    render(
        r##"use fornthulani_plugin_validate::{FormRequest, Rules};
use serde::{Deserialize, Serialize};

/// Input for `POST /__TABLE__`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Store__PASCAL__ {
__STORE_FIELDS__}

impl FormRequest for Store__PASCAL__ {
    fn rules() -> Rules {
        Rules::new()
__STORE_RULES__    }

    // Return false to answer 403 before validating:
    // fn authorize(req: &fornthulani::Request) -> bool { req.bearer_token().is_some() }
}

/// Input for `PUT|PATCH /__TABLE__/:id`. Only the fields that were sent
/// are validated and written.
#[derive(Debug, Serialize, Deserialize)]
pub struct Update__PASCAL__ {
__UPDATE_FIELDS__}

impl FormRequest for Update__PASCAL__ {
    fn rules() -> Rules {
        Rules::new()
__UPDATE_RULES__    }
}
"##,
        &[
            ("PASCAL", pascal),
            ("TABLE", table),
            ("STORE_FIELDS", store_fields.as_str()),
            ("STORE_RULES", store_rules.as_str()),
            ("UPDATE_FIELDS", update_fields.as_str()),
            ("UPDATE_RULES", update_rules.as_str()),
        ],
    )
}

pub fn controller(snake: &str, pascal: &str, fields: &[Field]) -> String {
    let mut sortable = vec!["id".to_string(), "created_at".to_string()];
    sortable.extend(fields.iter().map(|f| f.name.clone()));
    let sortable = quoted_list(sortable.into_iter());
    render(
        r##"use fornthulani::{Params, Request, Response, Status, ThulaniError};
use fornthulani_plugin_orm::Model;
use fornthulani_plugin_validate::ValidateExt;

use crate::models::__SNAKE__::__PASCAL__;
use crate::requests::__SNAKE__::{Store__PASCAL__, Update__PASCAL__};

/// GET /__SNAKE__?page=1&per_page=15&sort=-created_at
pub fn index(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let page = req.query_param("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page = req.query_param("per_page").and_then(|p| p.parse().ok()).unwrap_or(15);

    // `sort` is client input: it must be in this allow-list.
    let results = __PASCAL__::query()
        .sort_by(req.query_param("sort").as_deref(), &[__SORTABLE__], ("id", "desc"))
        .paginate(page, per_page)?;
    Ok(results.json_response())
}

/// GET /__SNAKE__/:id
pub fn show(_: &Request, params: &Params) -> Result<Response, ThulaniError> {
    let item = __PASCAL__::find_or_fail(params.parse("id")?)?;
    Ok(item.json_response(Status::OK))
}

/// POST /__SNAKE__
pub fn store(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let input: Store__PASCAL__ = req.validated()?;
    let item = __PASCAL__::create(&input)?;
    Ok(item.json_response(Status::CREATED))
}

/// PUT|PATCH /__SNAKE__/:id
pub fn update(req: &Request, params: &Params) -> Result<Response, ThulaniError> {
    let mut item = __PASCAL__::find_or_fail(params.parse("id")?)?;
    let input: Update__PASCAL__ = req.validated()?;
    item.update(&input)?;
    Ok(item.json_response(Status::OK))
}

/// DELETE /__SNAKE__/:id
pub fn destroy(_: &Request, params: &Params) -> Result<Response, ThulaniError> {
    let item = __PASCAL__::find_or_fail(params.parse("id")?)?;
    item.delete()?;
    Ok(Response::empty(Status::NO_CONTENT))
}
"##,
        &[
            ("SNAKE", snake),
            ("PASCAL", pascal),
            ("SORTABLE", sortable.as_str()),
        ],
    )
}

pub fn routes(snake: &str, table: &str) -> String {
    render(
        r##"use fornthulani::{Resource, Router};

use crate::controllers::__SNAKE__;

pub fn register(router: &mut Router) {
    router.resource(
        "/__TABLE__",
        Resource::new()
            .index(__SNAKE__::index)
            .show(__SNAKE__::show)
            .store(__SNAKE__::store)
            .update(__SNAKE__::update)
            .destroy(__SNAKE__::destroy),
    );
}
"##,
        &[("SNAKE", snake), ("TABLE", table)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::parse_fields;

    fn fields() -> Vec<Field> {
        parse_fields(&[
            "title:string".into(),
            "body:text?".into(),
            "published:bool".into(),
            "views:integer".into(),
        ])
        .unwrap()
    }

    fn balanced(s: &str) -> bool {
        let mut depth = 0i32;
        for c in s.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            if depth < 0 {
                return false;
            }
        }
        depth == 0
    }

    #[test]
    fn model_template() {
        let s = model("Post", "posts", &fields());
        assert!(s.contains("pub struct Post {"));
        assert!(s.contains("pub title: String,"));
        assert!(s.contains("pub body: Option<String>,"));
        assert!(s.contains("pub published: bool,"));
        assert!(s.contains("&[\"title\", \"body\", \"published\", \"views\"]"));
        assert!(s.contains("BOOLS: &'static [&'static str] = &[\"published\"]"));
        assert!(!s.contains("__"));
        assert!(balanced(&s));
    }

    #[test]
    fn migration_template() {
        let s = migration("posts", &fields());
        assert!(s.contains("t.string(\"title\", 255);"));
        assert!(s.contains("t.text(\"body\").nullable();"));
        assert!(s.contains("t.boolean(\"published\").default(false);"));
        assert!(s.contains("t.timestamps();"));
        assert!(!s.contains("__"));
        assert!(balanced(&s));
    }

    #[test]
    fn request_template() {
        let s = request("Post", "posts", &fields());
        assert!(s.contains("pub struct StorePost {"));
        assert!(s.contains("pub struct UpdatePost {"));
        assert!(s.contains(".field(\"title\", \"required|string|max:255\")"));
        assert!(s.contains(".field(\"body\", \"nullable|string\")"));
        assert!(s.contains(".field(\"published\", \"nullable|boolean\")"));
        assert!(s.contains(".field(\"title\", \"sometimes|required|string|max:255\")"));
        assert!(s.contains(".field(\"body\", \"sometimes|nullable|string\")"));
        assert!(!s.contains("__"));
        assert!(balanced(&s));
    }

    #[test]
    fn controller_and_routes_templates() {
        let s = controller("post", "Post", &fields());
        assert!(
            s.contains("&[\"id\", \"created_at\", \"title\", \"body\", \"published\", \"views\"]")
        );
        assert!(s.contains("use crate::models::post::Post;"));
        assert!(!s.contains("__"));
        assert!(balanced(&s));
        let r = routes("post", "posts");
        assert!(r.contains("\"/posts\""));
        assert!(r.contains("post::destroy"));
        assert!(balanced(&r));
    }
}
