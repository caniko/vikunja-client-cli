mod auth;
mod client;
mod errors;
mod output;
mod types;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use client::VikunjaClient;
use errors::CliError;
use output::{
    print_json, print_json_items, print_table, CliOutput, CliOutputMeta, OutputFormat, PageMeta,
};
use types::*;

#[derive(Parser, Debug)]
#[command(
    name = "vkc",
    about = "Vikunja API CLI for task/project/label management",
    version,
    max_term_width = 120
)]
struct Cli {
    #[arg(long, env = "VIKUNJA_URL")]
    url: String,

    #[arg(long, env = "VIKUNJA_TOKEN", hide_env_values = true)]
    token: Option<String>,

    #[arg(long)]
    token_file: Option<PathBuf>,

    #[arg(long, env = "VKC_OUTPUT", default_value = "json")]
    output: OutputFormat,

    #[arg(long)]
    accept_invalid_certs: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check API connectivity.
    Health,

    /// Show current user info.
    Whoami,

    /// Manage tasks.
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },

    /// Manage projects.
    Project {
        #[command(subcommand)]
        cmd: ProjectCmd,
    },

    /// Manage labels.
    Label {
        #[command(subcommand)]
        cmd: LabelCmd,
    },

    /// List teams (read-only).
    Team {
        #[command(subcommand)]
        cmd: TeamCmd,
    },

    /// List users.
    User {
        #[command(subcommand)]
        cmd: UserCmd,
    },

    /// Manage Vikunja namespaces.
    Namespace {
        #[command(subcommand)]
        cmd: NamespaceCmd,
    },
}

// ---------------------------------------------------------------------------
// Task subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum TaskCmd {
    /// Create a new task.
    Create {
        title: String,
        #[arg(long)]
        project_id: Option<i64>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<i64>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        start_date: Option<String>,
        #[arg(long)]
        end_date: Option<String>,
        #[arg(long)]
        percent_done: Option<f64>,
    },

    /// List tasks.
    List {
        #[arg(long)]
        project_id: Option<i64>,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
        #[arg(long)]
        filter: Option<String>,
    },

    /// Get a single task by ID.
    Get {
        id: i64,
    },

    /// Update a task.
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        done: Option<bool>,
        #[arg(long)]
        priority: Option<i64>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        remove_due_date: bool,
        #[arg(long)]
        percent_done: Option<f64>,
    },

    /// Delete a task.
    Delete {
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// Project subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum ProjectCmd {
    /// Create a new project.
    Create {
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        namespace_id: Option<i64>,
        #[arg(long)]
        parent_project_id: Option<i64>,
    },

    /// List projects.
    List {
        #[arg(long)]
        namespace_id: Option<i64>,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
    },

    /// Get a single project by ID.
    Get {
        id: i64,
    },

    /// Update a project.
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        archived: Option<bool>,
    },

    /// Delete a project.
    Delete {
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// Label subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum LabelCmd {
    /// Create a new label.
    Create {
        title: String,
        #[arg(long)]
        color: Option<String>,
    },

    /// List labels.
    List {
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
    },

    /// Update a label.
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        color: Option<String>,
    },

    /// Delete a label.
    Delete {
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// Team subcommands (read-only)
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum TeamCmd {
    /// List all teams.
    List,

    /// Get a single team with members.
    Get {
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// User subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum UserCmd {
    /// List users.
    List {
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
    },
}

// ---------------------------------------------------------------------------
// Namespace subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum NamespaceCmd {
    /// List namespaces.
    List {
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
    },
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let token = match auth::resolve_token(cli.token_file.as_deref(), cli.token.as_deref()) {
        Ok(t) => t,
        Err(e) => CliError::new(0, format!("{e}")).exit(),
    };

    let client = match VikunjaClient::new(&cli.url, &token, cli.accept_invalid_certs) {
        Ok(c) => c,
        Err(e) => CliError::new(0, format!("{e}")).exit(),
    };

    let out = &cli.output;

    if let Err(e) = run(cli.command, &client, out).await {
        CliError::new(status_from_error(&e), format!("{e:#}")).exit();
    }
}

async fn run(cmd: Commands, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        Commands::Health => run_health(client, out).await,
        Commands::Whoami => run_whoami(client, out).await,
        Commands::Task { cmd } => run_task(cmd, client, out).await,
        Commands::Project { cmd } => run_project(cmd, client, out).await,
        Commands::Label { cmd } => run_label(cmd, client, out).await,
        Commands::Team { cmd } => run_team(cmd, client, out).await,
        Commands::User { cmd } => run_user(cmd, client, out).await,
        Commands::Namespace { cmd } => run_namespace(cmd, client, out).await,
    }
}

fn status_from_error(err: &anyhow::Error) -> u16 {
    let msg = format!("{err:#}");
    if msg.contains("unauthorized") || msg.contains("401") {
        401
    } else if msg.contains("403") {
        403
    } else if msg.contains("404") {
        404
    } else if msg.contains("409") {
        409
    } else {
        500
    }
}

// ---------------------------------------------------------------------------
// Command runners
// ---------------------------------------------------------------------------

async fn run_health(client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    client.health().await?;
    let msg = serde_json::json!({"status": "ok"});
    if matches!(out, OutputFormat::Json) {
        print_json(&CliOutput::new(msg))?;
    } else {
        println!("Vikunja API is reachable");
    }
    Ok(())
}

async fn run_whoami(client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    let user = client.whoami().await?;
    if matches!(out, OutputFormat::Json) {
        print_json(&CliOutput::new(user))?;
    } else {
        print_table(&[user]);
    }
    Ok(())
}

// -- Tasks -------------------------------------------------------------------

async fn run_task(cmd: TaskCmd, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        TaskCmd::Create {
            title,
            project_id,
            description,
            priority,
            due_date,
            start_date,
            end_date,
            percent_done,
        } => {
            let task = client
                .create_task(&CreateTask {
                    title: &title,
                    description: description.as_deref(),
                    project_id,
                    priority,
                    due_date: due_date.as_deref(),
                    start_date: start_date.as_deref(),
                    end_date: end_date.as_deref(),
                    percent_done,
                })
                .await?;
            print_json(&CliOutput::new(task))?;
        }
        TaskCmd::List {
            project_id,
            page,
            per_page,
            filter,
        } => {
            let page_data = client
                .list_tasks(project_id, page, per_page, filter.as_deref())
                .await?;
            let meta = PageMeta {
                page: page_data.page,
                total_pages: page_data.total_pages,
                total_items: page_data.total_items,
            };
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutputMeta::new(page_data.result, meta))?;
            } else {
                println!("Tasks (page {}/{}):", page_data.page, page_data.total_pages);
                print_json_items(&page_data.result)?;
            }
        }
        TaskCmd::Get { id } => {
            let task = client.get_task(id).await?;
            print_json(&CliOutput::new(task))?;
        }
        TaskCmd::Update {
            id,
            title,
            description,
            done,
            priority,
            due_date,
            remove_due_date,
            percent_done,
        } => {
            let due = match (due_date, remove_due_date) {
                (Some(d), _) => Some(d),
                (None, true) => Some(String::new()),
                _ => None,
            };
            let task = client
                .update_task(
                    id,
                    &UpdateTask {
                        title: title.as_deref(),
                        description: description.as_deref(),
                        done,
                        priority,
                        due_date: due.as_deref(),
                        percent_done,
                    },
                )
                .await?;
            print_json(&CliOutput::new(task))?;
        }
        TaskCmd::Delete { id } => {
            client.delete_task(id).await?;
            let msg = serde_json::json!({"deleted": true, "id": id});
            print_json(&CliOutput::new(msg))?;
        }
    }
    Ok(())
}

// -- Projects ---------------------------------------------------------------

async fn run_project(cmd: ProjectCmd, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        ProjectCmd::Create {
            title,
            description,
            namespace_id,
            parent_project_id,
        } => {
            let project = client
                .create_project(&CreateProject {
                    title: &title,
                    description: description.as_deref(),
                    namespace_id,
                    parent_project_id,
                })
                .await?;
            print_json(&CliOutput::new(project))?;
        }
        ProjectCmd::List {
            namespace_id,
            page,
            per_page,
        } => {
            let page_data = client.list_projects(namespace_id, page, per_page).await?;
            let meta = PageMeta {
                page: page_data.page,
                total_pages: page_data.total_pages,
                total_items: page_data.total_items,
            };
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutputMeta::new(page_data.result, meta))?;
            } else {
                println!("Projects (page {}/{}):", page_data.page, page_data.total_pages);
                print_json_items(&page_data.result)?;
            }
        }
        ProjectCmd::Get { id } => {
            let project = client.get_project(id).await?;
            print_json(&CliOutput::new(project))?;
        }
        ProjectCmd::Update {
            id,
            title,
            description,
            archived,
        } => {
            let project = client
                .update_project(
                    id,
                    &UpdateProject {
                        title: title.as_deref(),
                        description: description.as_deref(),
                        is_archived: archived,
                    },
                )
                .await?;
            print_json(&CliOutput::new(project))?;
        }
        ProjectCmd::Delete { id } => {
            client.delete_project(id).await?;
            let msg = serde_json::json!({"deleted": true, "id": id});
            print_json(&CliOutput::new(msg))?;
        }
    }
    Ok(())
}

// -- Labels -----------------------------------------------------------------

async fn run_label(cmd: LabelCmd, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        LabelCmd::Create { title, color } => {
            let label = client.create_label(&title, color.as_deref()).await?;
            print_json(&CliOutput::new(label))?;
        }
        LabelCmd::List { page, per_page } => {
            let page_data = client.list_labels(page, per_page).await?;
            let meta = PageMeta {
                page: page_data.page,
                total_pages: page_data.total_pages,
                total_items: page_data.total_items,
            };
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutputMeta::new(page_data.result, meta))?;
            } else {
                println!("Labels (page {}/{}):", page_data.page, page_data.total_pages);
                print_json_items(&page_data.result)?;
            }
        }
        LabelCmd::Update { id, title, color } => {
            let label = client
                .update_label(id, title.as_deref(), color.as_deref())
                .await?;
            print_json(&CliOutput::new(label))?;
        }
        LabelCmd::Delete { id } => {
            client.delete_label(id).await?;
            let msg = serde_json::json!({"deleted": true, "id": id});
            print_json(&CliOutput::new(msg))?;
        }
    }
    Ok(())
}

// -- Teams ------------------------------------------------------------------

async fn run_team(cmd: TeamCmd, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        TeamCmd::List => {
            let teams = client.list_teams().await?;
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutput::new(teams))?;
            } else {
                if teams.is_empty() {
                    println!("(no teams)");
                } else {
                    print_table(&teams);
                }
            }
        }
        TeamCmd::Get { id } => {
            let detail = client.get_team(id).await?;
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutput::new(detail))?;
            } else {
                println!("Team: {} (id={})", detail.name, detail.id);
                println!("Description: {}", detail.description);
                println!("Members:");
                for member in &detail.members {
                    println!("  {} (id={})", member.username, member.id);
                }
            }
        }
    }
    Ok(())
}

// -- Users ------------------------------------------------------------------

async fn run_user(cmd: UserCmd, client: &VikunjaClient, out: &OutputFormat) -> Result<()> {
    match cmd {
        UserCmd::List { page, per_page } => {
            let page_data = client.list_users(page, per_page).await?;
            let meta = PageMeta {
                page: page_data.page,
                total_pages: page_data.total_pages,
                total_items: page_data.total_items,
            };
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutputMeta::new(page_data.result, meta))?;
            } else {
                if page_data.result.is_empty() {
                    println!("(no users)");
                } else {
                    print_table(&page_data.result);
                }
            }
        }
    }
    Ok(())
}

// -- Namespaces -------------------------------------------------------------

async fn run_namespace(
    cmd: NamespaceCmd,
    client: &VikunjaClient,
    out: &OutputFormat,
) -> Result<()> {
    match cmd {
        NamespaceCmd::List { page, per_page } => {
            let page_data = client.list_namespaces(page, per_page).await?;
            let meta = PageMeta {
                page: page_data.page,
                total_pages: page_data.total_pages,
                total_items: page_data.total_items,
            };
            if matches!(out, OutputFormat::Json) {
                print_json(&CliOutputMeta::new(page_data.result, meta))?;
            } else {
                println!("Namespaces (page {}/{}):", page_data.page, page_data.total_pages);
                print_json_items(&page_data.result)?;
            }
        }
    }
    Ok(())
}
