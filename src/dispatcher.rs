use crate::{
    error::{Error, Result},
    github::GitHubDispatcher,
};

/// Trait for URL dispatchers that process URLs and return a list of URLs to fetch.
pub trait Dispatcher {
    /// Processes a URL and returns a list of URLs to fetch.
    ///
    /// # Arguments
    /// * `url` - The URL to process
    ///
    /// # Returns
    /// A `Result` containing a vector of URLs to fetch or an error
    fn process(url: &str) -> Result<Vec<String>>;
}

pub struct URLDispatcher {}

impl URLDispatcher {
    /// Dispatches HTTP/HTTPS URLs, ensuring all URLs in the list use these schemes.
    ///
    /// # Arguments
    /// * `url` - The URL or comma-separated list of URLs to process
    ///
    /// # Returns
    /// A vector of valid HTTP/HTTPS URLs
    ///
    /// # Errors
    /// Returns an error if any URL in the list doesn't use HTTP or HTTPS scheme
    fn dispatch_http(url: &str) -> Result<Vec<String>> {
        let url_list = url.split(',').collect::<Vec<&str>>();
        let mut result = Vec::new();

        for url in url_list {
            let trimmed_url = url.trim();
            if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
                return Err(Error::TemplateDownloadError(
                    trimmed_url.to_string(),
                    "Only HTTP and HTTPS URLs are supported".to_string(),
                ));
            }
            result.push(trimmed_url.to_string());
        }

        Ok(result)
    }
}

impl Dispatcher for URLDispatcher {
    /// Dispatches the URL to the appropriate resolver.
    ///
    /// If a single URL is provided, it will be returned as is.
    /// If a comma-separated list of URLs is provided, each URL will be processed
    /// and the results will be returned as a list.
    ///
    /// # Arguments
    /// * `url` - The URL to dispatch.
    ///
    /// # Returns
    /// A `Result` containing a vector of URLs to fetch or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - The URL scheme is not supported (not github:, http://, or https://)
    /// - Any URL in a comma-separated list doesn't use HTTP or HTTPS
    fn process(url: &str) -> Result<Vec<String>> {
        match url.split_once(':') {
            Some(("github", _)) => GitHubDispatcher::process(url),
            Some(("http", _)) | Some(("https", _)) => Self::dispatch_http(url),
            Some((scheme, _)) => Err(Error::TemplateDownloadError(
                url.to_string(),
                format!(
                    "URL scheme '{}' is not supported. Only github:, http://, and https:// are supported.",
                    scheme
                ),
            )),
            None => Err(Error::TemplateDownloadError(
                url.to_string(),
                "Invalid URL format. Expected scheme:// or github: prefix".to_string(),
            )),
        }
    }
}
