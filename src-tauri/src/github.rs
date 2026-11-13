//! GitHub's public API, without an account (60 requests an hour).

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Serialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
    /// "sha256:<hex>" when GitHub gives it.
    pub digest: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Release {
    pub tag: String,
    pub title: String,
    /// ISO 8601, as GitHub gives it.
    pub published: String,
    pub prerelease: bool,
    pub notes: String,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

#[derive(Deserialize)]
struct RawRelease {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    body: Option<String>,
    #[serde(default)]
    assets: Vec<RawAsset>,
}

pub fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("Lanceur-Burnout3/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("an HTTP client")
    })
}

/// The repository's releases, newest first, prereleases included
/// (releases/latest leaves them out, and upstream publishes nothing else).
pub async fn releases(repository: &str) -> Result<Vec<Release>, String> {
    // LANCEUR_GITHUB_API: another address for the API, to try updates locally.
    let api = std::env::var("LANCEUR_GITHUB_API").unwrap_or_else(|_| "https://api.github.com".into());
    let url = format!("{api}/repos/{repository}/releases?per_page=10");
    let response = client()
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub a répondu {}", response.status()));
    }
    let raw: Vec<RawRelease> = response.json().await.map_err(|e| e.to_string())?;
    let mut list: Vec<Release> = raw
        .into_iter()
        .filter(|r| !r.draft)
        .map(|r| Release {
            title: r.name.unwrap_or_else(|| r.tag_name.clone()),
            tag: r.tag_name,
            published: r.published_at.unwrap_or_default(),
            prerelease: r.prerelease,
            notes: r.body.unwrap_or_default(),
            assets: r
                .assets
                .into_iter()
                .map(|a| Asset { name: a.name, url: a.browser_download_url, size: a.size, digest: a.digest })
                .collect(),
        })
        .collect();
    // ISO 8601 sorts as text.
    list.sort_by(|a, b| b.published.cmp(&a.published));
    Ok(list)
}
