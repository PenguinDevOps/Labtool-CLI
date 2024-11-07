use clap::{arg, Parser, Subcommand};
use std::{fs, io::Error as IoError};

#[derive(Parser)]
#[command(name = "labtool")]
#[command(about = "CLI for managing GitLab via their API", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Log in to GitLab by storing a token in memory
    Login {
        #[arg(short, long)]
        token: String,
    },
    /// Fetch and print GitLab projects using the stored token
    Projects{
       #[command(subcommand)]
       action: ProjectActions
    },
    /// View and manage pipelines in projects
    Pipelines {
        #[command(subcommand)]
        action:PipelineActions,
    },
}


#[derive(Subcommand)]
pub enum ProjectActions{
    List{

    },
    View{
        #[arg(short, long)]
        name: String
    },
    //labtoool projects variables --project-name
    Variables{
        #[command(subcommand)]
        action: VariablesActions
    }
}
#[derive(Subcommand)]
pub enum VariablesActions{
    //labtoool projects variables list --project-name
    List{
        #[arg(short,long)]
        project_name: String
    },
}
#[derive(Subcommand,Clone)]
pub enum PipelineActions {
    // labtool pipelines list --project testproject --last 2h --status failed --show-jobs
    /// list pipeliens for specific project
    List {
        #[arg(short, long)]
        project: String,
        #[arg(short, long)]
        last: Option<String>,
        #[arg(short,long)]
        branch: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long, default_value_t = false)]
        show_jobs: bool,
    },
    // labtool pipelines trigger --project testproject --branch main
    Trigger{
        #[arg(short, long)]
        project: String,
        #[arg(short, long)]
        branch: String
    },
    // labtool pipelines jobs logs --project testproject --job-id 2924792047
    ///view job logs
    Jobs{
        #[command(subcommand)]
        action:JobAction,
    },
}
#[derive(Subcommand,Clone)]
pub enum JobAction {
    Logs {
        #[arg(short, long)]
        project: String,
        #[arg(short, long)]
        job_id: i64,
    },
}
