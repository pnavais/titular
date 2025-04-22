use isahc::{
    config::{Configurable, RedirectPolicy},
    Request, RequestExt,
};
use nu_ansi_term::Color::Yellow;
use serde_json::Value;
use smol::io::AsyncReadExt;

use crate::{
    dispatcher::Dispatcher,
    error::{Error, Result},
};

/// Dispatcher for handling GitHub-specific URLs.
///
/// This dispatcher handles URLs that start with the "github:" prefix.
/// It converts GitHub repository URLs into GitHub API content URLs.
pub struct GitHubDispatcher {}

impl Dispatcher for GitHubDispatcher {
    /// Processes a GitHub URL and returns a list of GitHub API content URLs.
    ///
    /// # Arguments
    /// * `url` - The GitHub URL to process (must start with "github:")
    ///
    /// # Returns
    /// A `Result` containing a vector of GitHub API content URLs or an error
    ///
    /// # Errors
    /// Returns an error if the URL doesn't start with "github:" or has an invalid format
    fn process(url: &str) -> Result<Vec<String>> {
        // Remove the github: prefix
        let repo_path = url.strip_prefix("github:").ok_or_else(|| {
            Error::TemplateDownloadError(
                url.to_string(),
                "URL must start with 'github:' prefix".to_string(),
            )
        })?;

        // Split into owner/repo and optional branch
        let (repo_part, branch) = match repo_path.split_once('@') {
            Some((repo, branch)) => (repo, Some(branch)),
            None => (repo_path, None),
        };

        // Split into owner/repo/path
        let parts: Vec<&str> = repo_part.split('/').collect();

        if parts.len() < 2 {
            return Err(Error::TemplateDownloadError(
                url.to_string(),
                "Invalid GitHub URL format. Expected github:owner/repo[@branch]".to_string(),
            ));
        }

        let owner = parts[0];
        let repo = parts[1];
        let path = if parts.len() > 2 {
            parts[2..].join("/")
        } else {
            String::new()
        };

        // Construct the GitHub API content URL
        let api_url = if let Some(branch) = branch {
            format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                owner, repo, path, branch
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                owner, repo, path
            )
        };

        GitHubDispatcher::fetch_templates(&api_url)
    }
}

impl GitHubDispatcher {
    fn fetch_templates(api_url: &str) -> Result<Vec<String>> {
        smol::block_on(GitHubDispatcher::fetch_templates_async(api_url))
    }

    /// Fetches templates from a GitHub API URL asynchronously.
    ///
    /// # Arguments
    /// * `api_url` - The GitHub API URL to fetch templates from
    ///
    /// # Returns
    /// A `Result` containing a vector of template URLs or an error
    async fn fetch_templates_async(api_url: &str) -> Result<Vec<String>> {
        let mut response = Request::get(api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "titular")
            .redirect_policy(RedirectPolicy::Follow)
            .body(())?
            .send_async()
            .await?;

        if !response.status().is_success() {
            return Err(Error::TemplateDownloadError(
                api_url.to_string(),
                format!("Server returned status {}", response.status()),
            ));
        }

        // Read the entire response body
        let mut body = Vec::new();
        let response_body = response.body_mut();
        response_body.read_to_end(&mut body).await?;

        // Parse the JSON response
        let json: Value = serde_json::from_slice(&body)?;

        // Get template names from the JSON response
        let templates = Self::fetch_template_names(&json);

        // If no templates found, try the templates subdirectory
        if templates.is_empty() {
            // Split the URL into base and query parts
            let (base_url, query) = api_url.split_once('?').unwrap_or((api_url, ""));

            // Check if the URL already ends with /templates
            let fallback_url = if base_url.ends_with("/templates") {
                api_url.to_string()
            } else {
                // Insert /templates before any query parameters
                if query.is_empty() {
                    format!("{}/templates", base_url)
                } else {
                    format!("{}/templates?{}", base_url, query)
                }
            };

            // Recursively call with the fallback URL
            return Box::pin(GitHubDispatcher::fetch_templates_async(&fallback_url)).await;
        }

        // Show number of templates found
        if !templates.is_empty() {
            println!(
                "{}",
                Yellow.paint(format!("Found {} template(s)", templates.len()))
            );
        }

        Ok(templates)
    }

    /// Extracts template names from a GitHub API JSON response.
    /// Only includes files that end with .tl extension.
    ///
    /// # Arguments
    /// * `json` - The JSON response from the GitHub API
    ///
    /// # Returns
    /// A `Vec` of template URLs
    fn fetch_template_names(json: &Value) -> Vec<String> {
        let mut templates = Vec::new();
        if let Value::Array(items) = json {
            for item in items {
                if let Some(path) = item.get("path").and_then(|p| p.as_str()) {
                    // Only add files that end with .tl extension
                    if path.ends_with(".tl") {
                        if let Some(download_url) =
                            item.get("download_url").and_then(|p| p.as_str())
                        {
                            templates.push(download_url.to_string());
                        }
                    }
                }
            }
        }
        templates
    }
}
