use std::error::Error;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Table};
use serde_json::Value;
use std::io::{self, Write};

use crate::client::API_CLIENT;
use crate::projects::fetch_project_id_by_name;

pub async fn list_project_variables(token: &str, project_name: &str) -> Result<(), Box<dyn Error>> {
    let project_id: u64 = fetch_project_id_by_name(token, project_name).await?;
    let response = API_CLIENT
        .get(format!("https://gitlab.com/api/v4/projects/{}/variables", project_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;
    if response.status().is_success() {
        let variables: Vec<Value> = response.json()?;
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
            .set_header(vec![
                Cell::new("Key").add_attribute(Attribute::Bold),
                Cell::new("Value").add_attribute(Attribute::Bold)
            ]);

        for variable in variables {
            table.add_row(vec![
                Cell::new(variable["key"].as_str().unwrap_or("")),
                Cell::new(variable["value"].as_str().unwrap_or("")),
            ]);
        }
        println!("{table}");
    } else {
        eprintln!("Failed to fetch variables for this project. Response={}", response.status());
    }
    Ok(())
}

pub async fn set_project_variables(token: &str, project_name: &str, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let project_id: u64 = fetch_project_id_by_name(token, project_name).await?;
    let response = API_CLIENT
        .post(format!("https://gitlab.com/api/v4/projects/{}/variables/?key={}&value={}", project_id, key, value))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;
    if response.status().is_success() {
        println!("variable {} was set successfully", key);
    } else {
        println!("{}", response.text()?);
        eprintln!("Failed to set variable {}", key);
    }
    Ok(())
}

pub async fn update_project_variables(token: &str, project_name: &str, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let project_id: u64 = fetch_project_id_by_name(token, project_name).await?;
    let response = API_CLIENT
        .put(format!("https://gitlab.com/api/v4/projects/{}/variables/{}?value={}", project_id, key, value))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;
    if response.status().is_success() {
        println!("variable {} was updated successfully", key);
    } else {
        eprintln!("Failed to update variable {}", key);
    }
    Ok(())
}
pub async fn delete_project_variables(token: &str, project_name: &str, key: &str) -> Result<(), Box<dyn Error>> {
    println!("Are you sure you want to delete the variable `{}` in project `{}`? Type 'DELETE' to confirm:", key, project_name);
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    if confirmation.trim() != "DELETE" {
        println!("Delete operation aborted.");
        return Ok(());
    }
    let project_id: u64 = fetch_project_id_by_name(token, project_name).await?;

    // Proceed with the delete request
    let response = API_CLIENT
        .delete(format!("https://gitlab.com/api/v4/projects/{}/variables/{}", project_id, key))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    if response.status().is_success() {
        println!("Variable `{}` deleted successfully.", key);
    } else {
        eprintln!("Failed to delete variable: {}", response.status());
    }

    Ok(())
}