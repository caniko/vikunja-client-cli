use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{
    header::AUTHORIZATION,
    Client, Response, StatusCode,
};

use crate::types::*;

pub struct VikunjaClient {
    http: Client,
    api: String,
    auth: String,
}

impl VikunjaClient {
    pub fn new(base_url: &str, token: &str, accept_invalid_certs: bool) -> Result<Self> {
        let base = base_url.trim_end_matches('/');
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
        h.insert(
            AUTHORIZATION,
            self.auth.parse().unwrap(),
        );
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
        self.get_json(&p).await
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
            None => "/tasks/all".into(),
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
        self.get_json(&p).await
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

    // -- projects ----------------------------------------------------------

    pub async fn create_project(&self, project: &CreateProject<'_>) -> Result<Project> {
        self.put_json("/projects", project).await
    }

    pub async fn list_projects(
        &self,
        namespace_id: Option<i64>,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<Project>> {
        let path = match namespace_id {
            Some(nid) => format!("/namespaces/{nid}/projects"),
            None => "/projects".into(),
        };
        self.paginated(&path, page, per_page).await
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

    // -- labels ------------------------------------------------------------

    pub async fn create_label(&self, title: &str, color: Option<&str>) -> Result<Label> {
        let body = serde_json::json!({"title": title, "hex_color": color});
        self.put_json("/labels", &body).await
    }

    pub async fn list_labels(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<Label>> {
        self.paginated("/labels", page, per_page).await
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

    // -- teams (read-only) -------------------------------------------------

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

    // -- namespaces --------------------------------------------------------

    pub async fn list_namespaces(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Paginated<Namespace>> {
        self.paginated("/namespaces", page, per_page).await
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
