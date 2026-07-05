use std::collections::{HashMap, HashSet};

use database::{
    CheckReportBo, Column, ColumnType, DatasourceInfo, DiffReport, Driver, Schema, StandardCheck,
    Table, database_metadata, diff_report, diff_sql, standard_check,
};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, MySqlConnection, PgConnection, Row, SqliteConnection};

type ResultType<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn database_ping(datasource_info: DatasourceInfo) -> ResultType<()> {
    match datasource_info.driver {
        Driver::Mysql => {
            let mut conn = MySqlConnection::connect(&datasource_info.url())
                .await
                .map_err(|e| e.to_string())?;
            conn.ping().await.map_err(|e| e.to_string())
        }
        Driver::Postgres => {
            let mut conn = PgConnection::connect(&datasource_info.url())
                .await
                .map_err(|e| e.to_string())?;
            conn.ping().await.map_err(|e| e.to_string())
        }
        Driver::Sqlite => {
            let mut conn = SqliteConnection::connect(&datasource_info.url())
                .await
                .map_err(|e| e.to_string())?;
            conn.ping().await.map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
pub async fn database_schemas(datasource_info: DatasourceInfo) -> ResultType<Vec<Schema>> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_schemas(&datasource_info).await;
    }

    let meta = database_metadata(&datasource_info.url()).await;
    meta.schemas().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_tables(datasource_info: DatasourceInfo) -> ResultType<Vec<Table>> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_tables(&datasource_info).await;
    }

    let meta = database_metadata(&datasource_info.url()).await;
    let database = datasource_info.database.as_deref().unwrap_or_default();
    let schemas = table_schemas(&datasource_info).await?;
    let mut data = Vec::new();

    for schema in schemas {
        data.extend(
            meta.tables(database, &schema)
                .await
                .map_err(|e| e.to_string())?,
        );
    }
    Ok(data)
}

/// 表信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnTree {
    pub schema: String,
    pub table_name: String,
    pub table_comment: String,
    pub children: Vec<Column>,
}

#[tauri::command]
pub async fn database_table_tree(
    datasource_info: DatasourceInfo,
) -> ResultType<Vec<TableColumnTree>> {
    if datasource_info.driver == Driver::Postgres {
        return postgres_table_tree(&datasource_info).await;
    }

    let Some(database) = datasource_info.database.as_deref() else {
        return Err("choose database".to_string());
    };
    let meta = database_metadata(&datasource_info.url()).await;
    let schemas = table_schemas(&datasource_info).await?;
    let mut data = Vec::new();

    for schema in schemas {
        let tables = meta
            .tables(database, &schema)
            .await
            .map_err(|e| e.to_string())?;
        for table in tables.into_iter() {
            let columns = meta
                .columns(database, &table.schema, &table.name)
                .await
                .map_err(|e| e.to_string())?;
            data.push(TableColumnTree {
                schema: table.schema,
                table_name: table.name,
                table_comment: table.comment,
                children: columns,
            });
        }
    }
    Ok(data)
}

async fn table_schemas(datasource_info: &DatasourceInfo) -> ResultType<Vec<String>> {
    match datasource_info.driver {
        Driver::Mysql => datasource_info
            .database
            .clone()
            .filter(|database| !database.is_empty())
            .map(|database| vec![database])
            .ok_or_else(|| "choose database".to_string()),
        Driver::Postgres => postgres_schemas(datasource_info)
            .await
            .map(|schemas| schemas.into_iter().map(|schema| schema.name).collect()),
        Driver::Sqlite => Ok(vec![String::new()]),
    }
}

async fn postgres_schemas(datasource_info: &DatasourceInfo) -> ResultType<Vec<Schema>> {
    let mut conn = PgConnection::connect(&datasource_info.url())
        .await
        .map_err(|e| e.to_string())?;
    let rows = sqlx::query(
        r#"
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
          AND schema_name NOT LIKE 'pg_toast%'
        ORDER BY CASE WHEN schema_name = 'public' THEN 0 ELSE 1 END, schema_name
        "#,
    )
    .fetch_all(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| Schema { name: row.get(0) })
        .collect())
}

async fn postgres_tables(datasource_info: &DatasourceInfo) -> ResultType<Vec<Table>> {
    let mut conn = PgConnection::connect(&datasource_info.url())
        .await
        .map_err(|e| e.to_string())?;
    let schemas = postgres_schemas(datasource_info).await?;
    let mut tables = Vec::new();

    for schema in schemas {
        let rows = sqlx::query(
            r#"
            SELECT n.nspname, c.relname, COALESCE(obj_description(c.oid, 'pg_class'), '')
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind IN ('r', 'p')
              AND n.nspname = $1
            ORDER BY c.relname
            "#,
        )
        .bind(&schema.name)
        .fetch_all(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

        tables.extend(rows.into_iter().map(|row| Table {
            schema: row.get(0),
            name: row.get(1),
            comment: row.get(2),
        }));
    }

    Ok(tables)
}

async fn postgres_table_tree(datasource_info: &DatasourceInfo) -> ResultType<Vec<TableColumnTree>> {
    let mut conn = PgConnection::connect(&datasource_info.url())
        .await
        .map_err(|e| e.to_string())?;
    let tables = postgres_tables(datasource_info).await?;
    let mut data = Vec::with_capacity(tables.len());

    for table in tables {
        let rows = sqlx::query(
            r#"
            SELECT
                current_database(),
                n.nspname,
                c.relname,
                a.attname,
                NOT a.attnotnull,
                pg_get_expr(ad.adbin, ad.adrelid),
                COALESCE(col_description(c.oid, a.attnum), ''),
                format_type(a.atttypid, a.atttypmod),
                t.typname,
                CASE
                    WHEN a.atttypmod > 0 AND t.typname IN ('varchar', 'bpchar') THEN a.atttypmod - 4
                    ELSE NULL
                END
            FROM pg_attribute a
            JOIN pg_class c ON c.oid = a.attrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_type t ON t.oid = a.atttypid
            LEFT JOIN pg_attrdef ad ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
            WHERE a.attnum > 0
              AND NOT a.attisdropped
              AND n.nspname = $1
              AND c.relname = $2
            ORDER BY a.attnum
            "#,
        )
        .bind(&table.schema)
        .bind(&table.name)
        .fetch_all(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

        let children = rows
            .into_iter()
            .map(|row| {
                let formatted_type: String = row.get(7);
                let type_name: String = row.get(8);
                let default: Option<String> = row.get(5);
                Column {
                    database: row.get(0),
                    schema: row.get(1),
                    table_name: row.get(2),
                    name: row.get(3),
                    r#type: Some(pg_column_type(&type_name, &formatted_type)),
                    length: row.get(9),
                    scale: None,
                    default: default.clone(),
                    enum_values: None,
                    comment: row.get(6),
                    is_null: row.get(4),
                    is_auto_incr: default
                        .as_deref()
                        .is_some_and(|value| value.contains("nextval(")),
                    is_unique: false,
                    is_primary_key: false,
                    is_unsigned: false,
                    rust_type: pg_rust_type(&type_name).to_string(),
                }
            })
            .collect();

        data.push(TableColumnTree {
            schema: table.schema,
            table_name: table.name,
            table_comment: table.comment,
            children,
        });
    }

    Ok(data)
}

fn pg_column_type(type_name: &str, formatted_type: &str) -> ColumnType {
    match type_name {
        "int2" | "smallserial" => ColumnType::SmallInt,
        "int4" | "serial" => ColumnType::Int,
        "int8" | "bigserial" => ColumnType::Bigint,
        "numeric" => ColumnType::Decimal,
        "float4" => ColumnType::Float,
        "float8" => ColumnType::Double,
        "bool" => ColumnType::TinyInt,
        "date" => ColumnType::Date,
        "time" | "timetz" => ColumnType::Time,
        "timestamp" | "timestamptz" => ColumnType::Timestamp,
        "varchar" => ColumnType::VarChar,
        "bpchar" => ColumnType::Char,
        "text" => ColumnType::Text,
        "json" | "jsonb" => ColumnType::Json,
        "bytea" => ColumnType::Blob,
        "uuid" => ColumnType::Char,
        _ if formatted_type.contains("char") => ColumnType::VarChar,
        _ => ColumnType::Text,
    }
}

fn pg_rust_type(type_name: &str) -> &'static str {
    match type_name {
        "bool" => "bool",
        "int2" | "smallserial" => "i16",
        "int4" | "serial" => "i32",
        "int8" | "bigserial" => "i64",
        "numeric" | "float4" | "float8" => "f64",
        "bytea" => "Vec<u8>",
        "uuid" => "Uuid",
        _ => "String",
    }
}

#[tauri::command]
pub async fn database_diff_report(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> ResultType<DiffReport> {
    diff_report(source, target).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_diff_sql(
    source: DatasourceInfo,
    target: DatasourceInfo,
) -> ResultType<Vec<String>> {
    diff_sql(source, target).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_standard_check_codes() -> Vec<HashMap<String, String>> {
    StandardCheck::codes()
}

#[tauri::command]
pub async fn database_standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> ResultType<Vec<CheckReportBo>> {
    standard_check(source, check_codes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn database_databases() -> Vec<SelectOption> {
    Vec::new()
}

#[tauri::command]
pub async fn database_columns(
    _database: Option<String>,
    _schema: Option<String>,
    _table: Option<String>,
) -> Vec<SelectOption> {
    Vec::new()
}

#[derive(Serialize)]
pub struct SelectOption {
    label: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeResponse {
    success: bool,
    message: String,
    data: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeRequest {
    datasource_info: DatasourceInfo,
    language: String,
    #[serde(default)]
    table_names: Vec<String>,
    #[serde(default)]
    file_types: Vec<String>,
    #[serde(default)]
    package_names: HashMap<String, String>,
}

#[tauri::command]
pub async fn generate_code_from_db(request: GenerateCodeRequest) -> GenerateCodeResponse {
    match generate_code(request).await {
        Ok(data) => GenerateCodeResponse {
            success: true,
            message: "代码生成成功".to_string(),
            data,
        },
        Err(message) => GenerateCodeResponse {
            success: false,
            message,
            data: HashMap::new(),
        },
    }
}

async fn generate_code(request: GenerateCodeRequest) -> ResultType<HashMap<String, String>> {
    let GenerateCodeRequest {
        datasource_info,
        language,
        table_names,
        file_types,
        package_names,
    } = request;
    let tables = database_table_tree(datasource_info).await?;
    let selected_tables = select_generate_tables(tables, &table_names);

    if selected_tables.is_empty() {
        return Err("未找到可生成的表，请先选择表".to_string());
    }

    let file_types = if file_types.is_empty() {
        default_file_types(&language)
    } else {
        file_types
    };

    let mut data = HashMap::new();
    for table in selected_tables {
        for file_type in &file_types {
            let Some((file_name, code)) =
                render_generate_file(&language, file_type, &table, &package_names)
            else {
                continue;
            };
            data.insert(format!("{}.{}", table.table_name, file_name), code);
        }
    }

    if data.is_empty() {
        return Err("没有生成任何文件，请检查语言和文件类型".to_string());
    }

    Ok(data)
}

fn select_generate_tables(
    tables: Vec<TableColumnTree>,
    table_names: &[String],
) -> Vec<TableColumnTree> {
    if table_names.is_empty() {
        return tables;
    }

    let selected = table_names
        .iter()
        .map(|name| name.to_lowercase())
        .collect::<HashSet<_>>();

    tables
        .into_iter()
        .filter(|table| {
            let table_name = table.table_name.to_lowercase();
            let schema_table = format!("{}.{}", table.schema, table.table_name).to_lowercase();
            selected.contains(&table_name) || selected.contains(&schema_table)
        })
        .collect()
}

fn default_file_types(language: &str) -> Vec<String> {
    match language {
        "rust" => vec!["model.rs".to_string()],
        _ => vec![
            "entity.java".to_string(),
            "mapper.java".to_string(),
            "mapper.xml".to_string(),
            "service.java".to_string(),
            "serviceImpl.java".to_string(),
            "controller.java".to_string(),
        ],
    }
}

fn render_generate_file(
    language: &str,
    file_type: &str,
    table: &TableColumnTree,
    package_names: &HashMap<String, String>,
) -> Option<(String, String)> {
    match language {
        "rust" => render_rust_file(file_type, table),
        _ => render_java_file(file_type, table, package_names),
    }
}

fn render_java_file(
    file_type: &str,
    table: &TableColumnTree,
    package_names: &HashMap<String, String>,
) -> Option<(String, String)> {
    let class_name = upper_camel(&table.table_name);
    let entity_package = package_name(package_names, "entity.java", "com.example.entity");
    let mapper_package = package_name(package_names, "mapper.java", "com.example.mapper");
    let service_package = package_name(package_names, "service.java", "com.example.service");
    let service_impl_package = package_name(
        package_names,
        "serviceImpl.java",
        "com.example.service.impl",
    );
    let controller_package =
        package_name(package_names, "controller.java", "com.example.controller");

    match file_type {
        "entity.java" => Some((
            format!("{class_name}.java"),
            java_entity(table, &class_name, &entity_package),
        )),
        "mapper.java" => Some((
            format!("{class_name}Mapper.java"),
            format!(
                "package {mapper_package};\n\nimport com.baomidou.mybatisplus.core.mapper.BaseMapper;\nimport {entity_package}.{class_name};\n\n/**\n * {}\n */\npublic interface {class_name}Mapper extends BaseMapper<{class_name}> {{\n}}\n",
                table.table_comment
            ),
        )),
        "mapper.xml" => Some((
            format!("{class_name}Mapper.xml"),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<!DOCTYPE mapper PUBLIC \"-//mybatis.org//DTD Mapper 3.0//EN\" \"http://mybatis.org/dtd/mybatis-3-mapper.dtd\">\n<mapper namespace=\"{mapper_package}.{class_name}Mapper\">\n</mapper>\n"
            ),
        )),
        "service.java" => Some((
            format!("I{class_name}Service.java"),
            format!(
                "package {service_package};\n\nimport com.baomidou.mybatisplus.extension.service.IService;\nimport {entity_package}.{class_name};\n\n/**\n * {}\n */\npublic interface I{class_name}Service extends IService<{class_name}> {{\n}}\n",
                table.table_comment
            ),
        )),
        "serviceImpl.java" => Some((
            format!("{class_name}ServiceImpl.java"),
            format!(
                "package {service_impl_package};\n\nimport com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;\nimport {entity_package}.{class_name};\nimport {mapper_package}.{class_name}Mapper;\nimport {service_package}.I{class_name}Service;\nimport org.springframework.stereotype.Service;\n\n/**\n * {}\n */\n@Service\npublic class {class_name}ServiceImpl extends ServiceImpl<{class_name}Mapper, {class_name}> implements I{class_name}Service {{\n}}\n",
                table.table_comment
            ),
        )),
        "controller.java" => Some((
            format!("{class_name}Controller.java"),
            format!(
                "package {controller_package};\n\nimport org.springframework.web.bind.annotation.RequestMapping;\nimport org.springframework.web.bind.annotation.RestController;\n\n/**\n * {}\n */\n@RestController\n@RequestMapping(\"/{route}\")\npublic class {class_name}Controller {{\n}}\n",
                table.table_comment,
                route = kebab_case(&table.table_name)
            ),
        )),
        _ => None,
    }
}

fn java_entity(table: &TableColumnTree, class_name: &str, package: &str) -> String {
    let imports = java_imports(&table.children);
    let fields = table
        .children
        .iter()
        .map(|column| {
            let field_type = java_type(column);
            let field_name = lower_camel(&column.name);
            let comment = if column.comment.is_empty() {
                column.name.as_str()
            } else {
                column.comment.as_str()
            };
            let table_id = if column.is_primary_key {
                "    @TableId\n"
            } else {
                ""
            };
            format!("    /** {comment} */\n{table_id}    private {field_type} {field_name};\n")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "package {package};\n\nimport com.baomidou.mybatisplus.annotation.TableId;\nimport com.baomidou.mybatisplus.annotation.TableName;\nimport lombok.Data;\n{imports}\n/**\n * {}\n */\n@Data\n@TableName(\"{}\")\npublic class {class_name} {{\n\n{fields}\n}}\n",
        table.table_comment, table.table_name
    )
}

fn java_imports(columns: &[Column]) -> String {
    let mut imports = HashSet::new();
    for column in columns {
        match column.r#type {
            Some(ColumnType::Date) => {
                imports.insert("import java.time.LocalDate;".to_string());
            }
            Some(ColumnType::DateTime) | Some(ColumnType::Timestamp) | Some(ColumnType::Time) => {
                imports.insert("import java.time.LocalDateTime;".to_string());
            }
            Some(ColumnType::Decimal) | Some(ColumnType::Numeric) => {
                imports.insert("import java.math.BigDecimal;".to_string());
            }
            _ => {}
        }
    }
    if imports.is_empty() {
        String::new()
    } else {
        let mut imports = imports.into_iter().collect::<Vec<_>>();
        imports.sort();
        format!("{}\n", imports.join("\n"))
    }
}

fn java_type(column: &Column) -> &'static str {
    match column.r#type {
        Some(ColumnType::Bigint) => "Long",
        Some(ColumnType::SmallInt) | Some(ColumnType::TinyInt) => "Integer",
        Some(ColumnType::Int) | Some(ColumnType::Integer) | Some(ColumnType::MediumInt) => {
            "Integer"
        }
        Some(ColumnType::Float) | Some(ColumnType::Double) | Some(ColumnType::Real) => "Double",
        Some(ColumnType::Decimal) | Some(ColumnType::Numeric) => "BigDecimal",
        Some(ColumnType::Date) => "LocalDate",
        Some(ColumnType::DateTime) | Some(ColumnType::Timestamp) | Some(ColumnType::Time) => {
            "LocalDateTime"
        }
        Some(ColumnType::Bit) if column.length == Some(1) => "Boolean",
        Some(ColumnType::Blob)
        | Some(ColumnType::LongBlob)
        | Some(ColumnType::MediumBlob)
        | Some(ColumnType::TinyBlob)
        | Some(ColumnType::Binary)
        | Some(ColumnType::Varbinary) => "byte[]",
        _ => "String",
    }
}

fn render_rust_file(file_type: &str, table: &TableColumnTree) -> Option<(String, String)> {
    if file_type != "model.rs" {
        return None;
    }

    let struct_name = upper_camel(&table.table_name);
    let fields = table
        .children
        .iter()
        .map(|column| {
            let field_name = rust_field_name(&column.name);
            let field_type = if column.is_null {
                format!("Option<{}>", rust_type(column))
            } else {
                rust_type(column)
            };
            let comment = if column.comment.is_empty() {
                column.name.as_str()
            } else {
                column.comment.as_str()
            };
            format!("    /// {comment}\n    pub {field_name}: {field_type},")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Some((
        format!("{}.rs", snake_case(&table.table_name)),
        format!(
            "use serde::{{Deserialize, Serialize}};\n\n/// {}\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct {struct_name} {{\n{fields}\n}}\n",
            table.table_comment
        ),
    ))
}

fn rust_type(column: &Column) -> String {
    if column.rust_type == "Uuid" {
        return "uuid::Uuid".to_string();
    }
    if !column.rust_type.is_empty() {
        return column.rust_type.clone();
    }
    match column.r#type {
        Some(ColumnType::Bigint) => "i64",
        Some(ColumnType::SmallInt) | Some(ColumnType::TinyInt) => "i16",
        Some(ColumnType::Int) | Some(ColumnType::Integer) | Some(ColumnType::MediumInt) => "i32",
        Some(ColumnType::Float) | Some(ColumnType::Double) | Some(ColumnType::Real) => "f64",
        Some(ColumnType::Blob)
        | Some(ColumnType::LongBlob)
        | Some(ColumnType::MediumBlob)
        | Some(ColumnType::TinyBlob)
        | Some(ColumnType::Binary)
        | Some(ColumnType::Varbinary) => "Vec<u8>",
        _ => "String",
    }
    .to_string()
}

fn package_name(package_names: &HashMap<String, String>, key: &str, default: &str) -> String {
    package_names
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn upper_camel(value: &str) -> String {
    split_words(value)
        .into_iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn lower_camel(value: &str) -> String {
    let upper = upper_camel(value);
    let mut chars = upper.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_lowercase(), chars.as_str()),
        None => String::new(),
    }
}

fn snake_case(value: &str) -> String {
    split_words(value).join("_").to_lowercase()
}

fn kebab_case(value: &str) -> String {
    split_words(value).join("-").to_lowercase()
}

fn rust_field_name(value: &str) -> String {
    let name = snake_case(value);
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "union", "unsafe", "use", "where", "while",
    ];
    if KEYWORDS.contains(&name.as_str()) {
        format!("r#{name}")
    } else {
        name
    }
}

fn split_words(value: &str) -> Vec<String> {
    value
        .split(|c: char| c == '_' || c == '-' || c == ' ' || c == '.')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}
