use serde::{Deserialize, Serialize};
use tabled::Tabled;

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct Paginated<T> {
    pub result: Vec<T>,
    #[serde(default)]
    pub total_pages: i64,
    #[serde(default)]
    pub total_items: i64,
    #[serde(alias = "current_page", default)]
    pub page: i64,
}

// ---------------------------------------------------------------------------
// User
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub hex_color: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub done: bool,
    pub done_at: Option<String>,
    pub due_date: Option<String>,
    #[serde(default)]
    pub priority: i64,
    pub project_id: i64,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub assignees: Vec<User>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub percent_done: f64,
    pub position: Option<f64>,
    pub created_by: Option<User>,
}

#[derive(Debug, Serialize)]
pub struct CreateTask<'a> {
    pub title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_done: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateTask<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_done: Option<f64>,
}

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub namespace_id: i64,
    pub parent_project_id: Option<i64>,
    #[serde(default)]
    pub hex_color: Option<String>,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
    pub owner: Option<User>,
}

#[derive(Debug, Serialize)]
pub struct CreateProject<'a> {
    pub title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_project_id: Option<i64>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateProject<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
}

// ---------------------------------------------------------------------------
// Team
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Team {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDetail {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub members: Vec<User>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
}

// ---------------------------------------------------------------------------
// Namespace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
    pub owner: Option<User>,
}
