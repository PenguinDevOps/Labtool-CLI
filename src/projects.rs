use std::error::Error;
use comfy_table::{Table, Cell, presets::UTF8_FULL, Color, ContentArrangement}; // Import required modules
use comfy_table::Attribute; // Import Attribute for cell styling
use serde_json::{json, to_string_pretty, Value};
use serde_json::ser::CharEscape::Tab;
use crate::client::API_CLIENT;
// Fetch all GitLab projects asynchronously
pub async fn fetch_all_gitlab_projects(token: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let response = API_CLIENT
        .get("https://gitlab.com/api/v4/groups/66653699/projects")
        .header("Authorization", format!("Bearer {}", token))
        .send()?; // Await the response

    if response.status().is_success() {
        // Parse the response body as JSON
        let projects: Value = response.json()?; // Await JSON parsing

        // Filter and display only the required fields
        let mut project_list = Vec::new(); // To store projects
        if let Some(projects_array) = projects.as_array() {
            for project in projects_array {
                let id = project.get("id").unwrap_or(&json!(null));
                let name = project.get("name").unwrap_or(&json!(null));
                let http_url_to_repo = project.get("http_url_to_repo").unwrap_or(&json!(null));

                // Print project details
                println!("Project ID: {}", id);
                println!("Name: {}", name);
                println!("Repository URL: {}\n", http_url_to_repo);

                // Store project in the list
                project_list.push(project.clone());
            }
        }
        return Ok(project_list); // Return the list of projects
    } else {
        println!("Failed to fetch projects: {}", response.status());
    }
    Ok(Vec::new()) // Return an empty Vec if no projects found
}


// Fetch a project by name asynchronously

pub async fn fetch_project_id_by_name(token: &str, project_name: &str) -> Result<u64, Box<dyn Error>> {
    let projects = fetch_all_gitlab_projects(token).await?;

    for project in projects {
        if let Some(name) = project.get("name").and_then(Value::as_str) {
            if name == project_name {
                if let Some(id) = project.get("id").and_then(Value::as_u64) {
                    return Ok(id); // Return the ID when found
                }
            }
        }
    }

    // If the project was not found, return an error
    Err(Box::from(format!("Project '{}' not found", project_name)))
}

pub async fn fetch_project_by_name(token: &str, project_name: String) -> Result<(), Box<dyn Error>> {
    let id = fetch_project_id_by_name(token, &project_name).await?;
    let response = API_CLIENT
        .get(format!("https://gitlab.com/api/v4/projects/{}", id))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;
    if response.status().is_success() {
        let project_info: Value = response.json()?; // Await JSON parsing
        let formatted_response = to_string_pretty(&project_info)?; // Format the JSON response
        println!("{}", formatted_response);
        Ok(()) // Return Ok after successful printing
    } else {
        // Handle the case where the request was not successful
        let error_message = format!("Failed to fetch project details: {}", response.status());
        Err(Box::from(error_message)) // Return an error with a descriptive message
    }
   }
