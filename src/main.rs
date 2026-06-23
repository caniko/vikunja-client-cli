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

    /// Manage Vikunja namespaces (deprecated in 2.3).
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
        project_id: i64,
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
        #[arg(long)]
        label_id: Vec<i64>,
        #[arg(long)]
        assignee_id: Vec<i64>,
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

    /// Manage task labels.
    Label {
        #[command(subcommand)]
        cmd: TaskLabelCmd,
    },

    /// Manage task assignees.
    Assignee {
        #[command(subcommand)]
        cmd: TaskAssigneeCmd,
    },
}

#[derive(Subcommand, Debug)]
enum TaskLabelCmd {
    /// Add a label to a task.
    Add {
        task_id: i64,
        label_id: i64,
    },
    /// Remove a label from a task.
    Remove {
        task_id: i64,
        label_id: i64,
    },
}

#[derive(Subcommand, Debug)]
enum TaskAssigneeCmd {
    /// Add an assignee to a task.
    Add {
        task_id: i64,
        user_id: i64,
    },
    /// Remove an assignee from a task.
    Remove {
        task_id: i64,
        user_id: i64,
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
        parent_project_id: Option<i64>,
        #[arg(long)]
        hex_color: Option<String>,
    },

    /// List projects.
    List {
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
        #[arg(long)]
        description: Option<String>,
    },

    /// List labels.
    List {
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        per_page: Option<i64>,
    },

    /// Get a single label by ID.
    Get {
        id: i64,
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
// Namespace subcommands (deprecated)
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
enum NamespaceCmd {
    /// List namespaces (removed in Vikunja 2.3).
    List,
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
            label_id,
            assignee_id,
        } => {
            let task = client
                .create_task(
                    project_id,
                    &CreateTask {
                        title: &title,
                        description: description.as_deref(),
                        priority,
                        due_date: due_date.as_deref(),
                        start_date: start_date.as_deref(),
                        end_date: end_date.as_deref(),
                        percent_done,
                    },
                )
                .await?;
            let task_id = task.id;
            if !label_id.is_empty() {
                client.add_task_labels(task_id, &label_id).await?;
            }
            for uid in &assignee_id {
                client.add_task_assignee(task_id, *uid).await?;
            }
            // Re-fetch to get populated labels/assignees
            let task = client.get_task(task_id).await?;
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
        TaskCmd::Label { cmd } => run_task_label(cmd, client, out).await?,
        TaskCmd::Assignee { cmd } => run_task_assignee(cmd, client, out).await?,
    }
    Ok(())
}

async fn run_task_label(cmd: TaskLabelCmd, client: &VikunjaClient, _out: &OutputFormat) -> Result<()> {
    match cmd {
        TaskLabelCmd::Add { task_id, label_id } => {
            client.add_task_labels(task_id, &[label_id]).await?;
            let msg = serde_json::json!({"added": true, "task_id": task_id, "label_id": label_id});
            print_json(&CliOutput::new(msg))?;
        }
        TaskLabelCmd::Remove { task_id, label_id } => {
            client.remove_task_label(task_id, label_id).await?;
            let msg = serde_json::json!({"removed": true, "task_id": task_id, "label_id": label_id});
            print_json(&CliOutput::new(msg))?;
        }
    }
    Ok(())
}

async fn run_task_assignee(cmd: TaskAssigneeCmd, client: &VikunjaClient, _out: &OutputFormat) -> Result<()> {
    match cmd {
        TaskAssigneeCmd::Add { task_id, user_id } => {
            client.add_task_assignee(task_id, user_id).await?;
            let msg = serde_json::json!({"added": true, "task_id": task_id, "user_id": user_id});
            print_json(&CliOutput::new(msg))?;
        }
        TaskAssigneeCmd::Remove { task_id, user_id } => {
            client.remove_task_assignee(task_id, user_id).await?;
            let msg = serde_json::json!({"removed": true, "task_id": task_id, "user_id": user_id});
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
            parent_project_id,
            hex_color,
        } => {
            let project = client
                .create_project(&CreateProject {
                    title: &title,
                    description: description.as_deref(),
                    parent_project_id,
                    hex_color: hex_color.as_deref(),
                })
                .await?;
            print_json(&CliOutput::new(project))?;
        }
        ProjectCmd::List {
            page,
            per_page,
        } => {
            let page_data = client.list_projects(page, per_page).await?;
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
        LabelCmd::Create {
            title,
            color,
            description: _description,
        } => {
            let label = client.create_label(&title, color.as_deref()).await?;
            print_json(&CliOutput::new(label))?;
        }
        LabelCmd::List {
            search,
            page,
            per_page,
        } => {
            let page_data = client.list_labels(search.as_deref(), page, per_page).await?;
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
        LabelCmd::Get { id } => {
            let label = client.get_label(id).await?;
            print_json(&CliOutput::new(label))?;
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

// -- Namespaces (deprecated) -----------------------------------------------

async fn run_namespace(
    cmd: NamespaceCmd,
    client: &VikunjaClient,
    out: &OutputFormat,
) -> Result<()> {
    match cmd {
        NamespaceCmd::List => {
            let result = client.list_namespaces(None, None).await;
            match result {
                Ok(page_data) => {
                    let meta = PageMeta {
                        page: page_data.page,
                        total_pages: page_data.total_pages,
                        total_items: page_data.total_items,
                    };
                    if matches!(out, OutputFormat::Json) {
                        print_json(&CliOutputMeta::new(page_data.result, meta))?;
                    } else {
                        println!(
                            "Namespaces (page {}/{}):",
                            page_data.page, page_data.total_pages
                        );
                        print_json_items(&page_data.result)?;
                    }
                }
                Err(e) => {
                    let msg = format!("namespaces are not available in Vikunja 2.3: {e}");
                    if matches!(out, OutputFormat::Json) {
                        print_json(&CliOutput::new(
                            serde_json::json!({"error": msg, "hint": "use project list instead"}),
                        ))?;
                    } else {
                        println!("{msg}");
                        println!("  -> use `vkc project list` instead");
                    }
                }
            }
        }
    }
    Ok(())
}
