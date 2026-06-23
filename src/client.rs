use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{
    header::AUTHORIZATION,
    Client, Response, StatusCode,
};

use crate::types::*;

const PAGINATION_TOTAL_PAGES: &str = "x-pagination-total-pages";
const PAGINATION_RESULT_COUNT: &str = "x-pagination-result-count";
const PAGINATION_CURRENT_PAGE: &str = "x-pagination-current-page";

pub struct VikunjaClient {
    http: Client,
    api: String,
    auth: String,
}

impl VikunjaClient {
    pub fn new(base_url: &str, token: &str, accept_invalid_certs: bool) -> Result<Self> {
        let base = base_url.trim_end_matches('/');
        let base = base.strip_suffix("/api/v1").unwrap_or(base);
        let api = format!("{base}/api/v1");
        let http = Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("vkc/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("building HTTP client")?;
        Ok(Self {
            http,
            api,
            auth: format!("Bearer {token}"),
        })
    }

    // -- helpers -----------------------------------------------------------

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(AUTHORIZATION, self.auth.parse().unwrap());
        h
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .http
            .get(format!("{}{path}", self.api))
            .headers(self.headers())
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        let resp = ensure_success(resp).await?;
        resp.json()
            .await
            .with_context(|| format!("decoding GET {path} response"))
    }

    async fn get_array<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<Vec<T>> {
        let resp = self
            .http
            .get(format!("{}{path}", self.api))
            .headers(self.headers())
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        let resp = ensure_success(resp).await?;
        let body = resp.text().await?;
        if body == "null" || body.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(&body)
            .with_context(|| format!("decoding GET {path} response (expected array)"))
    }

    async fn put_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self
            .http
            .put(format!("{}{path}", self.api))
            .headers(self.headers())
            .json(body)
            .send()
            .await
            .with_context(|| format!("PUT {path}"))?;
        let resp = ensure_success(resp).await?;
        resp.json()
            .await
            .with_context(|| format!("decoding PUT {path} response"))
    }

    async fn post_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self
            .http
            .post(format!("{}{path}", self.api))
            .headers(self.headers())
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        let resp = ensure_success(resp).await?;
        resp.json()
            .await
            .with_context(|| format!("decoding POST {path} response"))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let resp = self
            .http
            .delete(format!("{}{path}", self.api))
            .headers(self.headers())
            .send()
            .await
            .with_context(|| format!("DELETE {path}"))?;
        ensure_success(resp).await?;
        Ok(())
    }

    /// Fetch a paginated list endpoint.
    /// Vikunja 2.3 returns plain arrays with pagination in response headers.
    async fn paginated<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<T>> {
        let mut p = path.to_owned();
        let sep = if p.contains('?') { '&' } else { '?' };
        if let Some(page) = page {
            p.push_str(&format!("{sep}page={page}"));
        }
        if let Some(per) = per_page {
            let sep2 = if page.is_some() { '&' } else { sep };
            p.push_str(&format!("{sep2}per_page={per}"));
        }

        let resp = self
            .http
            .get(format!("{}{}", self.api, p))
            .headers(self.headers())
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        let page_num = resp
            .headers()
            .get(PAGINATION_CURRENT_PAGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let total_pages = resp
            .headers()
            .get(PAGINATION_TOTAL_PAGES)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let total_items = resp
            .headers()
            .get(PAGINATION_RESULT_COUNT)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let resp = ensure_success(resp).await?;
        let body = resp.text().await?;
        let result: Vec<T> = if body == "null" || body.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&body)
                .with_context(|| format!("decoding GET {path} response (expected array)"))?
        };

        Ok(Paginated {
            result,
            total_pages,
            total_items,
            page: page_num,
        })
    }

    // -- health ------------------------------------------------------------

    pub async fn health(&self) -> Result<()> {
        let resp = self
            .http
            .get(format!("{}/info", self.api))
            .headers(self.headers())
            .send()
            .await
            .context("requesting Vikunja info")?;
        ensure_success(resp).await?;
        Ok(())
    }

    // -- tasks -------------------------------------------------------------

    pub async fn create_task(&self, task: &CreateTask<'_>) -> Result<Task> {
        self.put_json("/tasks", task).await
    }

    pub async fn list_tasks(
        &self,
        project_id: Option<i64>,
        page: Option<i64>,
        per_page: Option<i64>,
        filter: Option<&str>,
    ) -> Result<Paginated<Task>> {
        let path = match project_id {
            Some(pid) => format!("/projects/{pid}/tasks"),
            None => "/tasks".into(),
        };
        let mut p = path;
        let mut have_q = false;
        if let Some(page) = page {
            p.push_str(&format!("?page={page}"));
            have_q = true;
        }
        if let Some(per) = per_page {
            let sep = if have_q { '&' } else { '?' };
            p.push_str(&format!("{sep}per_page={per}"));
            have_q = true;
        }
        if let Some(f) = filter {
            let sep = if have_q { '&' } else { '?' };
            p.push_str(&format!("{sep}filter={}", urlencode(f)));
        }
        self.paginated(&p, None, None).await
    }

    pub async fn get_task(&self, id: i64) -> Result<Task> {
        self.get_json(&format!("/tasks/{id}")).await
    }

    pub async fn update_task(&self, id: i64, updates: &UpdateTask<'_>) -> Result<Task> {
        self.post_json(&format!("/tasks/{id}"), updates).await
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        self.delete(&format!("/tasks/{id}")).await
    }

    pub async fn add_task_labels(&self, task_id: i64, label_ids: &[i64]) -> Result<()> {
        for lid in label_ids {
            let body = serde_json::json!({"label_id": lid});
            self.put_json::<serde_json::Value, _>(&format!("/tasks/{task_id}/labels"), &body)
                .await?;
        }
        Ok(())
    }

    pub async fn remove_task_label(&self, task_id: i64, label_id: i64) -> Result<()> {
        self.delete(&format!("/tasks/{task_id}/labels/{label_id}"))
            .await
    }

    pub async fn add_task_assignee(&self, task_id: i64, user_id: i64) -> Result<()> {
        let body = serde_json::json!({"user_id": user_id});
        self.put_json::<serde_json::Value, _>(&format!("/tasks/{task_id}/assignees"), &body)
            .await?;
        Ok(())
    }

    pub async fn remove_task_assignee(&self, task_id: i64, user_id: i64) -> Result<()> {
        self.delete(&format!("/tasks/{task_id}/assignees/{user_id}"))
            .await
    }

    // -- projects ----------------------------------------------------------

    pub async fn create_project(&self, project: &CreateProject<'_>) -> Result<Project> {
        self.put_json("/projects", project).await
    }

    pub async fn list_projects(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<Project>> {
        self.paginated("/projects", page, per_page).await
    }

    pub async fn get_project(&self, id: i64) -> Result<Project> {
        self.get_json(&format!("/projects/{id}")).await
    }

    pub async fn update_project(&self, id: i64, updates: &UpdateProject<'_>) -> Result<Project> {
        self.post_json(&format!("/projects/{id}"), updates).await
    }

    pub async fn delete_project(&self, id: i64) -> Result<()> {
        self.delete(&format!("/projects/{id}")).await
    }

    pub async fn add_project_team(&self, project_id: i64, team_id: i64) -> Result<()> {
        let body = serde_json::json!({"team_id": team_id});
        self.put_json::<serde_json::Value, _>(&format!("/projects/{project_id}/teams"), &body)
            .await?;
        Ok(())
    }

    pub async fn get_project_teams(&self, project_id: i64) -> Result<Vec<ProjectTeam>> {
        self.get_array(&format!("/projects/{project_id}/teams"))
            .await
    }

    // -- labels ------------------------------------------------------------

    pub async fn create_label(&self, title: &str, color: Option<&str>) -> Result<Label> {
        let body = serde_json::json!({"title": title, "hex_color": color});
        self.put_json("/labels", &body).await
    }

    pub async fn list_labels(
        &self,
        search: Option<&str>,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<Label>> {
        let mut path = "/labels".to_string();
        if let Some(s) = search {
            path.push_str(&format!("?s={}", urlencode(s)));
        }
        self.paginated(&path, page, per_page).await
    }

    pub async fn get_label(&self, id: i64) -> Result<Label> {
        self.get_json(&format!("/labels/{id}")).await
    }

    pub async fn update_label(
        &self,
        id: i64,
        title: Option<&str>,
        color: Option<&str>,
    ) -> Result<Label> {
        let mut body = serde_json::Map::new();
        if let Some(t) = title {
            body.insert("title".into(), t.into());
        }
        if let Some(c) = color {
            body.insert("hex_color".into(), c.into());
        }
        self.post_json(&format!("/labels/{id}"), &body).await
    }

    pub async fn delete_label(&self, id: i64) -> Result<()> {
        self.delete(&format!("/labels/{id}")).await
    }

    // -- teams -------------------------------------------------------------

    pub async fn list_teams(&self) -> Result<Vec<Team>> {
        let resp = self
            .http
            .get(format!("{}/teams", self.api))
            .headers(self.headers())
            .send()
            .await
            .context("GET /teams")?;
        let resp = ensure_success(resp).await?;
        resp.json().await.context("decoding teams list")
    }

    pub async fn get_team(&self, id: i64) -> Result<TeamDetail> {
        self.get_json(&format!("/teams/{id}")).await
    }

    // -- users -------------------------------------------------------------

    pub async fn whoami(&self) -> Result<User> {
        self.get_json("/user").await
    }

    pub async fn list_users(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<User>> {
        self.paginated("/users", page, per_page).await
    }

    // -- namespaces (removed in Vikunja 2.3) --------------------------------

    pub async fn list_namespaces(
        &self,
        _page: Option<i64>,
        _per_page: Option<i64>,
    ) -> Result<Paginated<Namespace>> {
        anyhow::bail!("namespaces were removed in Vikunja 2.3 — work directly with projects instead")
    }
}

// -- HTTP response helpers ---------------------------------------------------

async fn ensure_success(resp: Response) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    if status == StatusCode::UNAUTHORIZED {
        anyhow::bail!("unauthorized ({status}) — check the API token: {body}");
    }
    anyhow::bail!("request failed with HTTP {status}: {body}");
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
