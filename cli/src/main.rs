use clap::{Parser, Subcommand};
use colored::*;
use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

#[derive(Parser)]
#[command(name = "sekizgen")]
#[command(about = "Sekizgen - Full-stack Rust + React framework CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project from the sekizgen template
    New {
        /// Project name (e.g. "blog" or "my_app")
        name: String,
    },
    /// Generate code scaffolding
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        what: GenerateType,
    },
}

/// What type of code to generate.
#[derive(Subcommand)]
enum GenerateType {
    /// Generate a new model
    #[command(alias = "m")]
    Model {
        /// Name of the model (e.g., "post" or "blog_post")
        name: String,
        /// Fields in format: name:type (e.g., title:string content:text published:bool)
        #[arg(trailing_var_arg = true)]
        fields: Vec<String>,
    },
    /// Generate a new controller
    #[command(alias = "c")]
    Controller {
        /// Name of the controller (e.g., "post" or "blog_post")
        name: String,
        /// Actions to generate (e.g., index show create update delete)
        #[arg(trailing_var_arg = true)]
        actions: Vec<String>,
    },
    /// Generate both model and controller (scaffold)
    #[command(alias = "s")]
    Scaffold {
        /// Name of the resource
        name: String,
        /// Fields in format: name:type
        #[arg(trailing_var_arg = true)]
        fields: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => new_project(&name),
        Commands::Generate { what } => match what {
            GenerateType::Model { name, fields } => generate_model(&name, &fields),
            GenerateType::Controller { name, actions } => generate_controller(&name, &actions),
            GenerateType::Scaffold { name, fields } => {
                generate_model(&name, &fields);
                let actions = vec![
                    "index".into(), "show".into(), "create".into(),
                    "update".into(), "delete".into(),
                ];
                generate_controller(&name, &actions);
            }
        },
    }
}

const TEMPLATE_REPO: &str = "https://github.com/hakanersu/axum-react";

fn new_project(name: &str) {
    let snake = name.to_snake_case();

    // Guard: don't overwrite an existing directory
    if Path::new(&snake).exists() {
        eprintln!("{}", format!("Error: directory '{}' already exists.", snake).red());
        exit(1);
    }

    println!("{}", format!("Creating new project: {}", snake).bold());

    // 1. Clone the template (shallow, no history)
    println!("  {} Cloning template...", "→".cyan());
    let status = Command::new("git")
        .args(["clone", "--depth=1", TEMPLATE_REPO, &snake])
        .status()
        .unwrap_or_else(|_| { eprintln!("{}", "Error: git not found in PATH.".red()); exit(1); });

    if !status.success() {
        eprintln!("{}", "Error: git clone failed.".red());
        exit(1);
    }

    // 2. Remove the template's git history
    fs::remove_dir_all(format!("{}/.git", snake))
        .expect("Failed to remove .git directory");

    // 3. Remove runtime artifacts
    let _ = fs::remove_file(format!("{}/data.db", snake));

    // 4. Rename "ruststack" → project name in key files
    println!("  {} Configuring project name...", "→".cyan());
    let files_to_rename = [
        format!("{}/backend/Cargo.toml", snake),
        format!("{}/cli/Cargo.toml", snake),
        format!("{}/Cargo.toml", snake),
    ];
    for path in &files_to_rename {
        if let Ok(content) = fs::read_to_string(path) {
            let updated = content.replace("ruststack", &snake);
            fs::write(path, updated).unwrap_or_else(|_| eprintln!("Warning: could not update {}", path));
        }
    }

    // 5. Init a fresh git repository
    println!("  {} Initialising git repository...", "→".cyan());
    let run = |args: &[&str]| {
        Command::new("git").args(args).current_dir(&snake).status().ok();
    };
    run(&["init"]);
    run(&["add", "."]);
    run(&["commit", "-m", "Initial commit"]);

    // 6. Done
    println!();
    println!("{}", format!("✓ Project '{}' created successfully!", snake).green().bold());
    println!();
    println!("Next steps:");
    println!("  cd {}", snake);
    println!("  make dev");
}

/// Maps CLI field types to Rust types.
/// When you write `title:string`, this converts "string" to "String".
fn map_field_type(type_str: &str) -> &str {
    match type_str {
        "string" | "text" => "String",
        "int" | "integer" => "i64",
        "float" | "double" => "f64",
        "bool" | "boolean" => "bool",
        "date" | "datetime" => "String", // Store as ISO string for DB compatibility
        "uuid" => "String",
        _ => "String", // Default to String for unknown types
    }
}

/// Maps CLI field types to SQL column types.
fn map_sql_type(type_str: &str) -> &str {
    match type_str {
        "string" => "TEXT NOT NULL",
        "text" => "TEXT NOT NULL",
        "int" | "integer" => "INTEGER NOT NULL",
        "float" | "double" => "REAL NOT NULL",
        "bool" | "boolean" => "BOOLEAN NOT NULL DEFAULT FALSE",
        "date" | "datetime" => "TEXT NOT NULL",
        "uuid" => "TEXT NOT NULL",
        _ => "TEXT",
    }
}

/// Parse a "name:type" field string into (name, type).
fn parse_field(field: &str) -> (String, String) {
    let parts: Vec<&str> = field.split(':').collect();
    let name = parts[0].to_snake_case();
    let type_str = if parts.len() > 1 { parts[1] } else { "string" };
    (name, type_str.to_string())
}

/// Generate a model file with struct, DTOs, and migration SQL.
fn generate_model(name: &str, fields: &[String]) {
    let snake = name.to_snake_case();       // "blog_post"
    let pascal = name.to_upper_camel_case(); // "BlogPost"

    // Parse fields
    let parsed_fields: Vec<(String, String)> = fields.iter().map(|f| parse_field(f)).collect();

    // Build struct fields
    let struct_fields: String = parsed_fields
        .iter()
        .map(|(name, type_str)| format!("    pub {}: {},", name, map_field_type(type_str)))
        .collect::<Vec<_>>()
        .join("\n");

    // Build FromRow field mappings
    let from_row_fields: String = parsed_fields
        .iter()
        .map(|(name, _)| format!("            {}: row.try_get(\"{}\")?,", name, name))
        .collect::<Vec<_>>()
        .join("\n");

    // Build CreateDto fields
    let create_dto_fields: String = parsed_fields
        .iter()
        .map(|(name, type_str)| format!("    pub {}: {},", name, map_field_type(type_str)))
        .collect::<Vec<_>>()
        .join("\n");

    // Build UpdateDto fields (all optional)
    let update_dto_fields: String = parsed_fields
        .iter()
        .map(|(name, type_str)| format!("    pub {}: Option<{}>,", name, map_field_type(type_str)))
        .collect::<Vec<_>>()
        .join("\n");

    // Build SQL columns
    let sql_columns: String = parsed_fields
        .iter()
        .map(|(name, type_str)| format!("                {} {}", name, map_sql_type(type_str)))
        .collect::<Vec<_>>()
        .join(",\n");

    let model_code = format!(
        r#"use serde::{{Deserialize, Serialize}};
use sqlx::any::AnyRow;
use sqlx::Row;
use validator::Validate;

/// {pascal} model - represents a row in the `{snake}s` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal} {{
    pub id: String,
{struct_fields}
    pub created_at: String,
    pub updated_at: String,
}}

impl {pascal} {{
    /// Map a database row to this struct.
    pub fn from_row(row: &AnyRow) -> Result<Self, sqlx::Error> {{
        Ok(Self {{
            id: row.try_get("id")?,
{from_row_fields}
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        }})
    }}
}}

/// DTO for creating a new {pascal}.
#[derive(Debug, Deserialize, Validate)]
pub struct Create{pascal}Dto {{
{create_dto_fields}
}}

/// DTO for updating an existing {pascal}.
/// All fields are optional - only provided fields are updated.
#[derive(Debug, Deserialize)]
pub struct Update{pascal}Dto {{
{update_dto_fields}
}}

/// Response DTO - what the API returns to clients.
#[derive(Debug, Serialize)]
pub struct {pascal}Response {{
    pub id: String,
{struct_fields}
    pub created_at: String,
    pub updated_at: String,
}}

impl From<{pascal}> for {pascal}Response {{
    fn from(item: {pascal}) -> Self {{
        Self {{
            id: item.id,
{from_row_fields_response}
            created_at: item.created_at,
            updated_at: item.updated_at,
        }}
    }}
}}
"#,
        pascal = pascal,
        snake = snake,
        struct_fields = struct_fields,
        from_row_fields = from_row_fields,
        create_dto_fields = create_dto_fields,
        update_dto_fields = update_dto_fields,
        from_row_fields_response = parsed_fields
            .iter()
            .map(|(name, _)| format!("            {}: item.{},", name, name))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    // Generate migration SQL
    let migration_sql = format!(
        r#"-- Migration: Create {snake}s table
CREATE TABLE IF NOT EXISTS {snake}s (
                id TEXT PRIMARY KEY,
{sql_columns},
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#,
        snake = snake,
        sql_columns = sql_columns,
    );

    // Write files
    let model_path = format!("backend/src/models/{}.rs", snake);
    let migration_path = format!("backend/migrations/create_{snake}s.sql");

    write_file(&model_path, &model_code);
    write_file(&migration_path, &migration_sql);

    println!("{}", format!("✓ Generated model: {}", model_path).green());
    println!("{}", format!("✓ Generated migration: {}", migration_path).green());
    println!();
    println!("{}", "Don't forget to:".yellow());
    println!("  1. Add `pub mod {};` to backend/src/models/mod.rs", snake);
    println!("  2. Run the migration against your database");
}

/// Generate a controller file with CRUD handlers.
fn generate_controller(name: &str, actions: &[String]) {
    let snake = name.to_snake_case();
    let pascal = name.to_upper_camel_case();

    let default_actions = if actions.is_empty() {
        vec!["index".to_string(), "show".to_string(), "create".to_string(),
             "update".to_string(), "delete".to_string()]
    } else {
        actions.to_vec()
    };

    let mut handler_code = String::new();
    let mut route_code = String::new();

    for action in &default_actions {
        match action.as_str() {
            "index" => {
                handler_code.push_str(&format!(r#"
/// List all {snake}s.
pub async fn index_{snake}s(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {{
    let rows = sqlx::query("SELECT * FROM {snake}s ORDER BY created_at DESC")
        .fetch_all(&state.db.pool)
        .await?;

    let items: Vec<{pascal}> = rows
        .iter()
        .filter_map(|row| {pascal}::from_row(row).ok())
        .collect();

    Ok(Json(json!({{ "{snake}s": items }})))
}}
"#, snake = snake, pascal = pascal));
                route_code.push_str(&format!(
                    "        .route(\"/{snake}s\", get(index_{snake}s))\n", snake = snake
                ));
            }
            "show" => {
                handler_code.push_str(&format!(r#"
/// Get a single {snake} by ID.
pub async fn show_{snake}(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {{
    let row = sqlx::query("SELECT * FROM {snake}s WHERE id = $1")
        .bind(&id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or(AppError::NotFound("{pascal} not found".into()))?;

    let item = {pascal}::from_row(&row)?;
    Ok(Json(json!({{ "{snake}": item }})))
}}
"#, snake = snake, pascal = pascal));
                route_code.push_str(&format!(
                    "        .route(\"/{snake}s/:id\", get(show_{snake}))\n", snake = snake
                ));
            }
            "create" => {
                handler_code.push_str(&format!(r#"
/// Create a new {snake}.
pub async fn create_{snake}(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Create{pascal}Dto>,
) -> Result<Json<Value>, AppError> {{
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // TODO: Add INSERT query with your model's fields
    // sqlx::query("INSERT INTO {snake}s (id, ..., created_at, updated_at) VALUES ($1, ..., $N, $N+1)")
    //     .bind(&id)
    //     .bind(...)
    //     .bind(&now)
    //     .bind(&now)
    //     .execute(&state.db.pool)
    //     .await?;

    Ok(Json(json!({{ "message": "{pascal} created", "id": id }})))
}}
"#, snake = snake, pascal = pascal));
                route_code.push_str(&format!(
                    "        .route(\"/{snake}s\", post(create_{snake}))\n", snake = snake
                ));
            }
            "update" => {
                handler_code.push_str(&format!(r#"
/// Update an existing {snake}.
pub async fn update_{snake}(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Update{pascal}Dto>,
) -> Result<Json<Value>, AppError> {{
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // TODO: Add UPDATE query with your model's fields
    // sqlx::query("UPDATE {snake}s SET ..., updated_at = $N WHERE id = $N+1")
    //     .bind(...)
    //     .bind(&now)
    //     .bind(&id)
    //     .execute(&state.db.pool)
    //     .await?;

    Ok(Json(json!({{ "message": "{pascal} updated" }})))
}}
"#, snake = snake, pascal = pascal));
                route_code.push_str(&format!(
                    "        .route(\"/{snake}s/:id\", put(update_{snake}))\n", snake = snake
                ));
            }
            "delete" => {
                handler_code.push_str(&format!(r#"
/// Delete a {snake}.
pub async fn delete_{snake}(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {{
    sqlx::query("DELETE FROM {snake}s WHERE id = $1")
        .bind(&id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json(json!({{ "message": "{pascal} deleted" }})))
}}
"#, snake = snake, pascal = pascal));
                route_code.push_str(&format!(
                    "        .route(\"/{snake}s/:id\", delete(delete_{snake}))\n", snake = snake
                ));
            }
            _ => {
                println!("{}", format!("⚠ Unknown action: {}", action).yellow());
            }
        }
    }

    let controller_code = format!(
        r#"use axum::{{extract::{{Path, State}}, Json}};
use serde_json::{{json, Value}};
use std::sync::Arc;

use crate::controllers::AppState;
use crate::errors::AppError;
use crate::models::{snake}::*;
{handler_code}
/// Build routes for the {snake} resource.
/// Add this to your routes/mod.rs:
///   `use crate::controllers::{snake}_controller;`
///   `.nest("/api", {snake}_controller::routes(state.clone()))`
pub fn routes(state: std::sync::Arc<AppState>) -> axum::Router<Arc<AppState>> {{
    use axum::routing::*;
    axum::Router::new()
{route_code}
}}
"#,
        snake = snake,
        handler_code = handler_code,
        route_code = route_code,
    );

    let controller_path = format!("backend/src/controllers/{}_controller.rs", snake);
    write_file(&controller_path, &controller_code);

    println!("{}", format!("✓ Generated controller: {}", controller_path).green());
    println!();
    println!("{}", "Don't forget to:".yellow());
    println!("  1. Add `pub mod {}_controller;` to backend/src/controllers/mod.rs", snake);
    println!("  2. Add routes to backend/src/routes/mod.rs");
}

/// Write content to a file, creating parent directories if needed.
fn write_file(path: &str, content: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }
    fs::write(path, content).expect(&format!("Failed to write {}", path));
}
