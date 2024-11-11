mod commands;
mod projects;
mod gitlab_login;
mod pipelines;
mod client;
mod variables;

use std::io::Read;
use serde_json::{json, Value};
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use std::{error::Error, sync::Mutex};
use commands::Commands;
use crate::commands::{JobAction, PipelineActions, ProjectActions, VariablesActions};
use crate::gitlab_login::{fetch_stored_token, login};
use crate::pipelines::{fetch_job_logs, fetch_pipelines_for_project};
use crate::projects::{fetch_all_gitlab_projects, fetch_project_by_name};
use crate::variables::{delete_project_variables, list_project_variables, set_project_variables, update_project_variables};
lazy_static::lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}
const TOKEN_FILE: &str = ".gitlab_token";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = commands::Cli::parse();
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        match &cli.command {
            Commands::Login { token } => {
                login(token)?;  // Ensure login is async
            }
            Commands::Projects { action } => {
                // Fetch the token from storage
                match fetch_stored_token() {
                    Ok(Some(token)) => {
                        match action {
                            ProjectActions::List {} => {
                                fetch_all_gitlab_projects(&token.trim()).await?;
                            }
                            ProjectActions::View { name } => {
                                fetch_project_by_name(&token.trim(), name.clone()).await?;
                            }
                        }
                    }
                    Ok(None) => {
                        println!("No token found. Please login first using 'devopscli login --token \"X\"'");
                    }
                    Err(e) => {
                        println!("Error fetching token: {}", e);
                    }
                }
            }
            Commands::Variables { action } => {
                match fetch_stored_token() {
                    Ok(Some(token)) => {
                        match action{
                            VariablesActions::List { project_name } => {
                                list_project_variables(&token.trim(), project_name).await?
                            }
                            VariablesActions::Set { project_name, key, value } =>{
                                set_project_variables(&token.trim(), project_name, key, value).await?
                            }
                            VariablesActions::Update {project_name, key, value}=>{
                                update_project_variables(&token.trim(), project_name, key, value).await?
                            }
                            VariablesActions::Delete {project_name, key}=>{
                                delete_project_variables(&token.trim(), project_name, key).await?
                            }
                        }
                    }Ok(None) => {
                        println!("No token found. Please login first using 'devopscli login --token \"X\"'");
                    }
                    Err(e) => {
                        println!("Error fetching token: {}", e);
                    }
                }
            }
            Commands::Pipelines { action } => {
                match fetch_stored_token() {
                    Ok(Some(token)) => {
                        match action {
                            PipelineActions::List { project, last, branch, status, show_jobs } => {
                                fetch_pipelines_for_project(&token.trim(), project, last.clone(), branch.clone(), show_jobs.clone()).await?;
                            }
                            PipelineActions::Trigger { project, branch } => {
                                //TODO implement function to trigger pipeline
                            }
                            PipelineActions::Jobs { action } => {
                                match action {
                                    JobAction::Logs { project, job_id } => {
                                        fetch_job_logs(&token.trim(), project, *job_id).await;
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        println!("No token found. Please login first using 'devopscli login --token \"X\"'");
                    }
                    Err(e) => {
                        println!("Error fetching token: {}", e);
                    }
                }
            }
        }
        Ok(()) // Return Ok for the async block
    })
}
