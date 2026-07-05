use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tabled::settings::Style;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    Json,
    Table,
}

#[derive(Debug, Serialize)]
pub struct CliOutput<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct CliOutputMeta<T: Serialize> {
    pub ok: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PageMeta>,
}

#[derive(Debug, Serialize)]
pub struct PageMeta {
    pub page: i64,
    pub total_pages: i64,
    pub total_items: i64,
}

impl<T: Serialize> CliOutput<T> {
    pub fn new(data: T) -> Self {
        Self { ok: true, data }
    }
}

impl<T: Serialize> CliOutputMeta<T> {
    pub fn new(data: T, meta: PageMeta) -> Self {
        Self {
            ok: true,
            data,
            meta: Some(meta),
        }
    }
}

pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    serde_json::to_writer_pretty(std::io::stdout(), value)?;
    println!();
    Ok(())
}

pub fn print_table<T: Tabled>(rows: &[T]) {
    if rows.is_empty() {
        println!("(empty)");
        return;
    }
    let mut table = tabled::Table::new(rows);
    table.with(Style::rounded());
    println!("{table}");
}

pub fn print_json_items<T: Serialize>(items: &[T]) -> anyhow::Result<()> {
    for item in items {
        serde_json::to_writer(std::io::stdout(), item)?;
        println!();
    }
    Ok(())
}
