use crate::client::API_CLIENT;
use std::error::Error;
use serde_json::Value;
use crate::projects::fetch_project_id_by_name;
use chrono::{DateTime, Utc, Duration};
use comfy_table::{Table, Cell, presets::UTF8_FULL, Color, ContentArrangement}; // Import required modules
use comfy_table::Attribute; // Import Attribute for cell styling
use colored::Colorize;
// Define ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const CHECKMARK: &str = "✅";
const CROSS: &str = "❌";

pub async fn fetch_pipelines_for_project(
    token: &str,
    project_name: &str,
    last: Option<String>,
    branch: Option<String>,
    show_jobs: bool,
) -> Result<(), Box<dyn Error>> {
    let id = fetch_project_id_by_name(token, project_name).await?;
    let response = API_CLIENT
        .get(format!("https://gitlab.com/api/v4/projects/{}/pipelines/?ref={}", id, branch.unwrap().to_string()))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    if response.status().is_success() {
        let pipelines: Vec<Value> = response.json()?;

        // Optionally filter pipelines by the last duration (if provided)
        let pipelines_to_display = if let Some(duration_str) = last {
            let duration = parse_duration(&duration_str)?;
            filter_pipelines(pipelines, duration)
        } else {
            pipelines
        };

        // Iterate through each pipeline to display
        for pipeline in pipelines_to_display {
            // Colorize the status for console output
            if let Some(status) = pipeline.get("status").and_then(Value::as_str) {
                let colored_status = match status {
                    "success" => format!("{}{} success{}", GREEN, CHECKMARK, RESET),
                    "failed" => format!("{}{} failed{}", RED, CROSS, RESET),
                    _ => status.to_string(), // Fallback for other statuses
                };

                // Print the pipeline details
                println!("\nPipeline ID {}: {}", pipeline["id"], colored_status);
                println!("Triggered at \"{}\" on \"{}\" branch", pipeline["created_at"], pipeline["ref"]);
                println!("Web URL \"{}\"", pipeline["web_url"]);

                // If show_jobs is true, fetch and print the jobs array
                if show_jobs {
                    if let Some(pipeline_id) = pipeline.get("id").and_then(Value::as_i64) {
                        let jobs_url = format!("https://gitlab.com/api/v4/projects/{}/pipelines/{}/jobs", id, pipeline_id);
                        let jobs_response = API_CLIENT
                            .get(&jobs_url)
                            .header("Authorization", format!("Bearer {}", token))
                            .send()?;

                        if jobs_response.status().is_success() {
                            let jobs: Vec<Value> = jobs_response.json()?;

                            // Create a comfy-table instance to display job details

                            // Create a comfy-table instance to display job details
                            let mut table = Table::new();
                            table.load_preset(UTF8_FULL)
                                .set_content_arrangement(ContentArrangement::Dynamic)
                                // Use UTF8_FULL for solid lines
                                .set_header(vec![
                                    Cell::new("Job ID").add_attribute(Attribute::Bold),
                                    Cell::new("Name").add_attribute(Attribute::Bold),
                                    Cell::new("Stage").add_attribute(Attribute::Bold),
                                    Cell::new("Status").add_attribute(Attribute::Bold),
                                    Cell::new("Ref").add_attribute(Attribute::Bold),
                                    Cell::new("Duration").add_attribute(Attribute::Bold),
                                    Cell::new("Created At").add_attribute(Attribute::Bold),
                                    Cell::new("Finished At").add_attribute(Attribute::Bold),
                                    Cell::new("Web URL").add_attribute(Attribute::Bold),

                                ]);

                            // Add job details to the table
                            for job in jobs {
                                let status_cell = if job["status"].as_str() == Some("success") {
                                    Cell::new(job["status"].as_str().unwrap_or("")).fg(Color::Green) // Green for success
                                } else {
                                    Cell::new(job["status"].as_str().unwrap_or("")).fg(Color::Red) // Red for failed
                                };
                                table.add_row(vec![
                                    Cell::new(&job["id"].as_i64().unwrap_or_default().to_string()),
                                    Cell::new(job["name"].as_str().unwrap_or("")),
                                    Cell::new(job["stage"].as_str().unwrap_or("")),
                                    status_cell,
                                    Cell::new(job["ref"].as_str().unwrap_or("")),
                                    Cell::new(&job["duration"].as_f64().unwrap_or_default().to_string()),
                                    Cell::new(job["created_at"].as_str().unwrap_or("")),
                                    Cell::new(job["finished_at"].as_str().unwrap_or("")),
                                    Cell::new(job["web_url"].as_str().unwrap_or("")),

                                ]);
                            }
                            // Print the table with a nice format
                            println!("{table}");

                        } else {
                            eprintln!("Failed to fetch jobs for pipeline {}: {}", pipeline_id, jobs_response.status());
                        }
                    }
                }

                // Print a newline to separate each pipeline output
                println!();
            }
        }

        Ok(())
    } else {
        let error_message = format!("Failed to fetch project pipelines: {}", response.status());
        Err(Box::from(error_message))
    }
}

pub async fn fetch_job_logs(token: &str, project_name: &str, job_id: i64) -> Result<(), Box<dyn Error>> {
    // Fetch the project ID by name (implement this function as needed)
    let project_id = fetch_project_id_by_name(token, project_name).await?;
    // Set up the API client and make the GET request
    let response = API_CLIENT
        .get(format!(
            "https://gitlab.com/api/v4/projects/{}/jobs/{}/trace",
            project_id, job_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    // Check if the response is successful
    if response.status().is_success() {
        let job_trace = response.text()?;
        // Print the job trace with color based on content
        for line in job_trace.lines() {
            if line.contains("ERROR") {
                println!("{}", line.red()); // Red for errors
            } else if line.contains("WARNING") {
                println!("{}", line.yellow()); // Yellow for warnings
            } else {
                println!("{}", line.green()); // Green for regular logs
            }
        }

        Ok(())
    } else {
        // If the response isn't successful, return an error message
        let error_message = format!("Failed to fetch job trace: {}", response.status());
        Err(Box::from(error_message))
    }
}


fn parse_duration(last: &str) -> Result<Duration, Box<dyn Error>> {
    let num_str = &last[..last.len() - 1]; // Get the numeric part
    let unit = last.chars().last().ok_or("Invalid format: No unit provided")?; // Get the last character as the unit

    // Try to parse the numeric part
    let num: usize = num_str.parse().map_err(|_| Box::<dyn Error>::from("Invalid number format"))?;

    match unit {
        'h' => Ok(Duration::hours(num as i64)),
        'd' => Ok(Duration::days(num as i64)),
        _ => Err(Box::<dyn Error>::from("Invalid time unit, use 'h' for hours or 'd' for days.")),
    }
}

fn filter_pipelines(pipelines: Vec<Value>, duration: Duration) -> Vec<Value> {
    let now: DateTime<Utc> = Utc::now();
    pipelines.into_iter()
        .filter(|pipeline| {
            if let Some(created_at_str) = pipeline.get("created_at").and_then(Value::as_str) {
                if let Ok(created_at) = DateTime::parse_from_rfc3339(created_at_str) {
                    let created_at = created_at.with_timezone(&Utc);
                    return now - created_at <= duration; // Filter based on duration
                }
            }
            false
        })
        .collect()
}
