mod schema;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::fs;
use std::io;
use convert_case::{Case, Casing};
use serde::de::value::Error;
use serde::Deserialize;
use sqlx::FromRow;
use std::io::Write;
pub use schema::{extract_column_info, extract_table_schemas, extract_table_names, Col};
use std::process::{Command, Output};


#[derive(Debug)]
struct Row {
    name: String,
    cols: Vec<Col>,
}

// Helper function to insert multiple PostgreSQL types for a single Rust type
fn insert_multiple(map: &mut HashMap<String, String>, rust_type: &str, postgres_types: &[&str]) {
    for pg_type in postgres_types {
        map.insert(pg_type.to_string(), rust_type.to_string());
    }
}

fn create_type_map() -> HashMap<String, String> {
    let mut type_map = HashMap::new();

    insert_multiple(&mut type_map, "bool", &["BOOL"]);
    insert_multiple(&mut type_map, "i8", &["CHAR"]);
    insert_multiple(&mut type_map, "i16", &["SMALLINT", "SMALLSERIAL", "INT2"]);
    insert_multiple(&mut type_map, "i32", &["INT", "SERIAL", "INT4", "INTEGER"]);
    insert_multiple(&mut type_map, "i64", &["BIGINT", "BIGSERIAL", "INT8"]);
    insert_multiple(&mut type_map, "f32", &["REAL", "FLOAT4"]);
    insert_multiple(&mut type_map, "f64", &["DOUBLE PRECISION", "FLOAT8"]);
    insert_multiple(&mut type_map, "String", &["VARCHAR", "CHAR(N)", "TEXT", "NAME", "CITEXT"]);
    insert_multiple(&mut type_map, "Vec<u8>", &["BYTEA"]);
    insert_multiple(&mut type_map, "()", &["VOID"]);
    insert_multiple(&mut type_map, "PgInterval", &["INTERVAL"]);
    insert_multiple(&mut type_map, "PgMoney", &["MONEY"]);
    insert_multiple(&mut type_map, "PgLTree", &["LTREE"]);
    insert_multiple(&mut type_map, "PgLQuery", &["LQUERY"]);
    insert_multiple(&mut type_map, "PgCiText", &["CITEXT1"]);
    insert_multiple(&mut type_map, "PgCube", &["CUBE"]);
    insert_multiple(&mut type_map, "PgPoint", &["POINT"]);
    insert_multiple(&mut type_map, "PgLine", &["LINE"]);
    insert_multiple(&mut type_map, "PgLSeg", &["LSEG"]);
    insert_multiple(&mut type_map, "PgBox", &["BOX"]);
    insert_multiple(&mut type_map, "PgPath", &["PATH"]);
    insert_multiple(&mut type_map, "PgPolygon", &["POLYGON"]);
    insert_multiple(&mut type_map, "PgCircle", &["CIRCLE"]);
    insert_multiple(&mut type_map, "PgHstore", &["HSTORE"]);

    // Add the new pairs
    // type_map.insert("NUMERIC".to_string(), "bigdecimal::Decimal".to_string());
    type_map.insert("TIMESTAMPTZ".to_string(), "chrono::DateTime<Utc>".to_string());
    type_map.insert("TIMESTAMP".to_string(), "chrono::NaiveDateTime".to_string());
    type_map.insert("DATE".to_string(), "chrono::NaiveDate".to_string());
    type_map.insert("TIME".to_string(), "chrono::NaiveTime".to_string());
    type_map.insert("TIMETZ".to_string(), "PgTimeTz".to_string());
    type_map.insert("UUID".to_string(), "uuid::Uuid".to_string());
    insert_multiple(&mut type_map, "ipnetwork::IpNetwork", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "std::net::IpAddr", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "ipnet::IpNet", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "mac_address::MacAddress", &["MACADDR"]);
    insert_multiple(&mut type_map, "bit_vec::BitVec", &["BIT", "VARBIT"]);
    insert_multiple(&mut type_map, "Json<T>", &["JSON", "JSONB"]); //  *******  TODO:fix ********* //
    insert_multiple(&mut type_map, "serde_json::Value", &["JSON", "JSONB"]);
    insert_multiple(&mut type_map, "&serde_json::value::RawValue", &["JSON", "JSONB"]);

    // Handle PgRange<T> types
    type_map.insert("INT8RANGE".to_string(), "PgRange<i64>".to_string());
    type_map.insert("INT4RANGE".to_string(), "PgRange<i32>".to_string());
    type_map.insert("TSRANGE".to_string(), "PgRange<PgTimestamp>".to_string()); // Assuming you have a PgTimestamp type
    type_map.insert("TSTZRANGE".to_string(), "PgRange<PgTimestampTz>".to_string()); // Assuming you have a PgTimestampTz type
    type_map.insert("DATERANGE".to_string(), "PgRange<PgDate>".to_string()); // Assuming you have a PgDate type
    type_map.insert("NUMRANGE".to_string(), "PgRange<PgNumeric>".to_string()); // Assuming you have a PgNumeric type


    type_map
}


fn generate_struct(row: &Row, file_path: &str) -> Result<(), std::io::Error> {
    let type_map = create_type_map();
    let struct_name = row.name.to_case(Case::Pascal); // Convert table name to PascalCase
    let mut struct_string = format!("#[derive(Debug, Deserialize, FromRow)]\nstruct {} {{\n", struct_name);

    for col in &row.cols {
        let field_name = col.name.clone();
        let rust_type = type_map.get(&col.col_type)
            .map(|s| s.as_str())
            .unwrap_or("String"); // Default to String if type not found
        struct_string.push_str(&format!("    {}: {},\n", field_name, rust_type));
    }

    struct_string.push_str("}\n");

    // Write the struct to the file
    // fs::write(file_path, struct_string)?;

    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.

    // Write the data to the file.
    file.write_all(struct_string.as_bytes())?; // comment for testing 
    Ok(())
}

fn create_rows_from_sql(file_path: &str) -> Result<Vec<Row>, io::Error> {
    let table_names = extract_table_names(file_path)?;
    let schemas = extract_table_schemas(file_path)?;
    let mut rows: Vec<Row> = Vec::new();

    if table_names.len() != schemas.len() {
        eprintln!("Warning: Number of table names and schemas do not match!");
    }

    for (table_name, schema) in table_names.iter().zip(schemas.iter()) {
        let cleaned_name = table_name
            .split('.')
            .last()
            .unwrap_or(&table_name)
            .trim_matches('"')
            .to_string();
        let cols = extract_column_info(schema);
        let row = Row {
            name: cleaned_name,
            cols,
        };
        rows.push(row);
    }


    Ok(rows)
}

fn add_insert_func(row: &Row, file_path: &str) -> Result<String, io::Error> {
    let funk_name = format!("add_{}", row.name.clone());
    let struct_name = row.name.clone().to_case(Case::Pascal);
    let table_name = row.name.clone();
    let cols: String = row.cols.iter().map(|col| format!("{}, ", col.name).to_string()).collect::<String>()
        .trim_end_matches(", ").to_string();
    let cols_list = row.cols.iter().map(|col| col.name.clone()).collect::<Vec<_>>();
    
    let bind_feilds = cols_list.iter().enumerate().map(|(i, col)| 
        format!("\t.bind(payload.{})", cols_list[i]))
        .collect::<Vec<_>>().join("\n");
    let feilds = cols_list.iter().enumerate().map(|(i, col)| format!("${}, ", i + 1)).collect::<String>()
        .trim_end_matches(", ").to_string();
    let funk = format!(r###"

async fn {funk_name}(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<{struct_name}>,
) -> Json<Value> {{
    let query = "INSERT INTO {table_name} ({cols}) VALUES ({feilds})";
    let result = sqlx::query(query)
    {bind_feilds}
        .execute(&pool)
        .await;
    match result {{
        Ok(value) => Json(json!({{"res": "added"}})),
        Err(e) => Json(json!({{"res": format!("error: {{}}", e)}}))

    }}
}}
"###);


    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.



    println!("{}", funk);
    file.write_all(funk.as_bytes())?; // comment for testing 

    Ok(funk_name.to_string())
}

// fn add_get_some_func(row: &Row, col: &str, fillter: &str, file_path: &str) -> Result<String, io::Error> {
//     let row_name = row.name.clone();
//     let func_name = format!("get_{}", row.name.clone());
//     let struct_name = row.name.clone().to_case(Case::Pascal);
//     let cols: String = row.cols.iter().map(|col| format!("\t\"{}\": elemint.{}, \n", 
//         col.name, col.name)
//         .to_string()).collect::<String>()
//         .trim_end_matches(", ").to_string();
    

//     let funk_str = format!(r###"

// async fn {func_name}(
//     extract::State(pool): extract::State<PgPool>,
// ) -> Result<Json<Value>, (StatusCode, String)> {{
//     let query = "SELECT * FROM {row_name}";
//     let q = sqlx::query_as::<_, {struct_name}>(query);

//     let elemints: Vec<{struct_name}> = q.fetch_all(&pool).await.map_err(|e| {{
//         (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {{}}", e))
//     }})?;

//     let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {{
//         json!({{
//     {cols}
//         }})
    
//     }}).collect();

//     Ok(Json(json!({{ "payload": res_json }})))
// }}
// "###);

//     let mut file = OpenOptions::new()
//         .write(true) // Enable writing to the file.
//         .append(true) // Set the append mode.  Crucially, this makes it append.
//         .create(true) // Create the file if it doesn't exist.
//         .open(file_path)?; // Open the file, returning a Result.

//     file.write_all(funk_str.as_bytes())?; // comment for testing 

//     println!("{}", funk_str);

//     Ok(func_name.to_string())
// }



fn add_get_all_func(row: &Row, file_path: &str) -> Result<String, io::Error> {
    let row_name = row.name.clone();
    let func_name = format!("get_{}", row.name.clone());
    let struct_name = row.name.clone().to_case(Case::Pascal);
    let cols: String = row.cols.iter().map(|col| format!("\t\"{}\": elemint.{}, \n", 
        col.name, col.name)
        .to_string()).collect::<String>()
        .trim_end_matches(", ").to_string();
    

    let funk_str = format!(r###"

async fn {func_name}(
    extract::State(pool): extract::State<PgPool>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    let query = "SELECT * FROM {row_name}";
    let q = sqlx::query_as::<_, {struct_name}>(query);

    let elemints: Vec<{struct_name}> = q.fetch_all(&pool).await.map_err(|e| {{
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {{}}", e))
    }})?;

    let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {{
        json!({{
    {cols}
        }})
    
    }}).collect();

    Ok(Json(json!({{ "payload": res_json }})))
}}
"###);

    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.

    file.write_all(funk_str.as_bytes())?; // comment for testing 

    println!("{}", funk_str);

    Ok(func_name.to_string())
}

fn add_top_boilerplate(file_path: &str) -> Result<(), io::Error> {
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.

    let top_boiler = r###"
use axum::{                                                                                                                                                                      
    extract::{self, Extension, Path},                                                                                                                                            
    routing::{get, post},                                                                                                                                                        
    Json, Router,                                                                                                                                                                
};                                                                                                                                                                               
use serde::Deserialize;                                                                                                                                                          
use serde_json::{json, Value};                                                                                                                                                   
use sqlx::PgPool;                                                                                                                                                                
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};                                                                                                                           
use std::env;                                                                                                                                                                    
use std::net::SocketAddr;                                                                                                                                                        
use std::result::Result;                                                                                                                                                         
use std::sync::Arc;                                                                                                                                                              
use axum::http::StatusCode;                                                                                                                                                      
use sqlx::types::chrono::Utc; 


"###;
    file.write_all(top_boiler.as_bytes())?; // comment for testing 

    Ok(())
} 

fn add_axum_end(funcs: Vec<String>, file_path: &str) -> Result<(), io::Error> {

    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.
    
    let routs: String = funcs.iter().map(|func| {
        let http_method = if func.starts_with("get") { "get" } else { "post" };
        format!("\t.route(\"/{func}\", {http_method}({func}))\n").to_string()
    }).collect::<String>();
    let ending = format!(r###"


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let db_url = "postgres://dbuser:p@localhost:1111/work";
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    let app = Router::new()
    {routs}
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}}


"###);
    
    file.write_all(ending.as_bytes())?; // comment for testing 
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let rows = create_rows_from_sql("../testing/migrations/0001_data.sql")?;
    // println!("Table names: {:?}", rows.iter().map(|row| row.name.clone()).collect::<Vec<String>>());

    let path = "src/generated_struct.rs";
    let mut func_names = Vec::new();
    add_top_boilerplate(path);
    for row in rows {
        println!("Row: {:?} \n", row);
        generate_struct(&row, path)?;
        func_names.push(add_insert_func(&row, path)?);
        func_names.push(add_get_all_func(&row, path)?);
    }
    add_axum_end(func_names, path);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::fs;

    #[test]
    fn test_extract_table_schemas() -> Result<(), io::Error> {
        let sql_content = r#"
        CREATE TABLE public."user" (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            favorite_color VARCHAR(50),
            height NUMERIC,
            age INTEGER,
            job VARCHAR(100)
        );

        CREATE TABLE product_details (
            product_id INTEGER PRIMARY KEY,
            description TEXT,
            price DECIMAL(10, 2)
        );


        create table order_items (
            order_id INTEGER,
            item_id INTEGER,
            quantity INTEGER
        );
        "#;
        fs::write("test.sql", sql_content)?;

        let expected_schemas = vec![
            "id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            favorite_color VARCHAR(50),
            height NUMERIC,
            age INTEGER,
            job VARCHAR(100)",
            "product_id INTEGER PRIMARY KEY,
            description TEXT,
            price DECIMAL(10, 2)",
            "order_id INTEGER,
            item_id INTEGER,
            quantity INTEGER",
        ];

        let schemas = extract_table_schemas("test.sql")?;
        assert_eq!(schemas.len(), expected_schemas.len());
        for (i, schema) in schemas.iter().enumerate() {
            assert_eq!(schema.trim(), expected_schemas[i].trim());
        }

        Ok(())
    }
}

