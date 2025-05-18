use std::io;
use std::fs;

pub fn extract_table_schemas(file_path: &str) -> Result<Vec<String>, io::Error> {
    let contents = fs::read_to_string(file_path)?;
    let mut schemas = Vec::new();
    let lower_contents = contents.to_lowercase();
    let mut start_index = 0;

    while let Some(create_index) = lower_contents[start_index..].find("create table if not exists") {
        let start = start_index + create_index;
        if let Some(open_paren_index) = contents[start..].find('(') {
            let schema_start = start + open_paren_index + 1;
            if let Some(close_paren_index) = contents[schema_start..].find(");") {
                let schema_end = schema_start + close_paren_index;
                let schema = contents[schema_start..schema_end].trim().to_string();
                schemas.push(schema);
                start_index = schema_end + 2; // Move past ");"
            } else {
                break; // Handle potential errors if closing parenthesis isn't found
            }
        } else {
            break; // Handle potential errors if opening parenthesis isn't found
        }
        start_index = start + 1;
    }

    Ok(schemas)
}

#[derive(Debug)]
pub struct Col {
    pub name: String,
    pub col_type: String,
    pub auto_gen: bool
}

pub fn extract_column_info(schema: &str) -> Vec<Col> {
    let column_definitions: Vec<&str> = schema.split(',')
    .map(|s| s.trim())
    .filter(|&s| !s.contains("FOREIGN")).collect();
    let mut columns_info = Vec::new();
    
    for definition in column_definitions {
        let parts: Vec<&str> = definition.split_whitespace().collect();
        let auto_gen = if parts.contains(&"DEFAULT") {
            true
        } else {
            false
        };
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            let mut col_type = parts[1].to_string();
            if col_type == "DOUBLE" {
                col_type.push(' ');
                col_type.push_str(parts[2])
            }
            columns_info.push(Col { name, col_type, auto_gen});
        } else if parts.len() == 1 {
            // Handle cases with only a name (e.g., constraints)
            let name = parts[0].to_string();
            columns_info.push(Col { name, col_type: "".to_string(), auto_gen});
        }
    }

    columns_info
}


pub fn extract_table_names(file_path: &str) -> Result<Vec<String>, io::Error> {
    let contents = fs::read_to_string(file_path)?;
    let mut table_names = Vec::new();
    let lower_contents = contents.to_lowercase();
    let mut start_index = 0;

    while let Some(create_index) = lower_contents[start_index..].find("create table if not exists") {
        let start = start_index + create_index + "create table if not exists".len();
        // Find the start of the table name
        let name_start = contents[start..].trim_start();
        // Extract the table name
        let mut table_name = String::new();
        for c in name_start.chars() {
            if c == '(' || c == ' ' || c == '\n' || c == '\r' {
                break;
            }
            if table_name == "foreign"{
                break;
            }
            table_name.push(c);
        }

        table_names.push(table_name);

        // Find the end of the current CREATE TABLE statement (look for ");")
        if let Some(end_index) = contents[start..].find(");") {
            start_index = start + end_index + 2; // Move past ");"
        } else {
            break; // Handle case where ");" is not found (malformed SQL)
        }
    }

    Ok(table_names)
}
