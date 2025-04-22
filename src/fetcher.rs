pub struct TemplateFetcher;

use std::{fmt::Write, io::Write as _, path::PathBuf};

use crate::{
    config::DEFAULT_TEMPLATE_EXT,
    dispatcher::{Dispatcher, URLDispatcher},
    reader::TemplateReader,
    utils::{self, create_backup, remove_backup, restore_backup},
};

use isahc::{
    config::{Configurable, RedirectPolicy},
    AsyncBody, Request, RequestExt, ResponseExt,
};

use nu_ansi_term::Color::{Green, Yellow};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::time::Duration;

use crate::error::*;
use crossterm::cursor::{Hide, Show};

pub struct TargetInfo {
    path: PathBuf,       // The template path
    filename: String,    // Always contains the filename
    exists: bool,        // Checks whether the file was already present in the repository
    total_size: u64,     // The total size of the file to download
    url: Option<String>, // The URL of the template to download
    created: bool,       // Checks whether the file was created successfully
}

impl TemplateFetcher {
    /// Retrieves the template from the given URL and stores it
    /// under the given templates directory.
    ///
    /// # Arguments
    /// * `url` - The URL of the template to download.
    /// * `templates_dir` - The directory where the template will be stored.
    /// * `force` - Whether to force the download even if the template already exists.
    ///
    /// # Returns
    /// * `Result<bool>` - `Ok(true)` if the template was downloaded successfully, `Ok(false)` if the template already exists, or an error if the download failed.
    pub fn fetch(url: &str, templates_dir: &PathBuf, force: bool) -> Result<bool> {
        let url_list = URLDispatcher::process(url)?;
        for url in url_list {
            if let Ok((result, template_name)) = Self::fetch_single(&url, templates_dir, force) {
                if result {
                    println!(
                        "{}",
                        Green.paint(format!(
                            "Template '{}' installed successfully",
                            template_name
                        ))
                    );
                }
            } else {
                return Err(Error::TemplateDownloadError(
                    url.to_string(),
                    "Failed to download template".to_string(),
                ));
            }
        }
        Ok(true)
    }

    /// Fetches a template from a remote repository.
    ///
    /// # Arguments
    /// * `remote` - The remote repository to fetch the template from.
    /// * `template_name` - The name of the template to fetch.
    /// * `input_dir` - The directory to save the template to.
    ///
    /// # Returns
    /// * `Result<bool>`
    /// - `Ok(true)` if the template was downloaded successfully,
    /// - `Ok(false)` if the template already exists, or an error if the download failed.
    pub fn fetch_from_remote<S: AsRef<str>>(
        remote: S,
        template_name: &str,
        input_dir: &PathBuf,
    ) -> Result<bool> {
        let url_list = URLDispatcher::process(remote.as_ref())?;

        // Try to find a URL that matches the template name
        let matching_url = url_list.iter().find(|url| {
            url.ends_with(template_name) || url.ends_with(&format!("{}.tl", template_name))
        });

        match matching_url {
            Some(url) => {
                // Found a matching URL, proceed with fetch_single
                let (result, _) = Self::fetch_single(url, input_dir, true)?;
                Ok(result)
            }
            None => {
                // No matching URL found
                Err(Error::TemplateDownloadError(
                    template_name.to_string(),
                    "No matching template found in remote repository".to_string(),
                ))
            }
        }
    }

    /// Retrieves the template from the given URL and stores it
    /// under the given templates directory.
    ///
    /// # Arguments
    /// * `url` - The URL of the template to download.
    /// * `templates_dir` - The directory where the template will be stored.
    /// * `force` - Whether to force the download even if the template already exists.
    ///
    /// # Returns
    /// * `Result<bool>` - `Ok(true)` if the template was downloaded successfully, `Ok(false)` if the template already exists, or an error if the download failed.
    pub fn fetch_single(url: &str, templates_dir: &PathBuf, force: bool) -> Result<(bool, String)> {
        let result =
            async { TemplateFetcher::download_file(&url, &templates_dir, force, true).await };
        match smol::block_on(result) {
            Ok(mut target_info) => {
                if target_info.created {
                    Self::process_fetched_template(&mut target_info, force)?;
                    Ok((true, target_info.filename))
                } else {
                    if target_info.exists {
                        println!(
                            "{}",
                            Yellow.paint(format!(
                                "A template with the same name [{}] already exists",
                                target_info.filename
                            ))
                        );
                    }
                    restore_backup(&target_info.path)?;
                    Ok((false, target_info.filename))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Downloads asynchronously the resource defined in the given URL and
    /// stores it in the supplied path.
    ///
    /// # Arguments
    /// * `url` - The URL of the resource to download.
    /// * `path` - The path where the resource should be stored.
    /// * `force` - Whether to force the download even if the template already exists.
    ///
    /// # Returns
    /// Returns a Result indicating success or failure.
    ///
    /// # Errors
    /// Returns an error if the download fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use titular::fetcher::TemplateFetcher;
    ///
    /// fn main() {
    ///     let url = "https://example.com/template.tl";
    ///     let path = PathBuf::from("/templates/");
    ///     smol::block_on(async { TemplateFetcher::download_file(url, &path, true, true).await; });
    /// }
    /// ```
    pub async fn download_file(
        url: &str,
        path: &PathBuf,
        force: bool,
        show_progress: bool,
    ) -> Result<TargetInfo> {
        Self::ensure_directory_exists(path)?;

        // Pre-process the URL to handle redirects and get content information
        let mut target_info = Self::pre_process_url(url, path).await?;

        if target_info.exists && !force {
            target_info.created = false;
            return Ok(target_info);
        } else if target_info.exists {
            create_backup(&target_info.path)?;
        }

        let mut response = Request::get(target_info.url.as_deref().unwrap_or(url))
            .redirect_policy(RedirectPolicy::Follow)
            .body(())?
            .send_async()
            .await?;

        if !response.status().is_success() {
            return Err(Error::TemplateDownloadError(
                "HTTP Request".to_string(),
                format!("Server returned status {}", response.status()),
            ));
        }

        // Update total_size if not already set
        target_info.total_size = if target_info.total_size == 0 {
            response
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        } else {
            target_info.total_size
        };

        let mut file = std::fs::File::create(&target_info.path)?;
        let mut body = response.body_mut();

        if show_progress {
            Self::download_with_progress(
                &mut body,
                &mut file,
                target_info.total_size,
                &target_info.filename,
            )
            .await?;
        } else {
            Self::download_without_progress(&mut body, &mut file).await?;
        }

        if target_info.exists {
            remove_backup(&target_info.path)?;
        }

        target_info.created = true;
        Ok(target_info)
    }

    /// Handles a newly created template, ensuring it has the correct extension
    /// and renaming it if necessary.
    ///
    /// # Arguments
    /// * `target_info` - Information about the downloaded template
    /// * `force` - Whether to force overwriting the template if it already exists.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    fn process_fetched_template(target_info: &mut TargetInfo, force: bool) -> Result<bool> {
        match TemplateReader::get_template_name(&target_info.path) {
            Ok(mut template_name) => {
                template_name = template_name.to_lowercase().replace(" ", "_");
                if !template_name.ends_with(DEFAULT_TEMPLATE_EXT) {
                    template_name = format!("{}{}", template_name, DEFAULT_TEMPLATE_EXT);
                }

                if target_info.created && target_info.filename != template_name {
                    if force {
                        std::fs::rename(
                            &target_info.path,
                            &target_info.path.with_file_name(&template_name),
                        )?;
                    } else {
                        println!(
                            "{}",
                            Yellow.paint(format!(
                                "A template with the same name [{}] already exists",
                                target_info.filename
                            ))
                        );
                    }
                    return Ok(false);
                }

                Ok(true)
            }
            Err(_) => {
                std::fs::remove_file(&target_info.path)?;
                Err(Error::TemplateDownloadError(
                    target_info
                        .url
                        .as_ref()
                        .map_or_else(|| target_info.filename.clone(), |url| url.clone()),
                    "Error parsing template".to_string(),
                ))
            }
        }
    }

    /// Builds the target path for the template.
    ///
    /// # Arguments
    /// * `url` - The URL of the template.
    /// * `templates_dir` - The directory where the template will be saved.
    ///
    /// # Returns
    /// A `TargetInfo` struct containing the path and filename of the template.
    fn build_target_path(url: &str, templates_dir: &PathBuf) -> Result<TargetInfo> {
        match TemplateFetcher::extract_filename_from_url(url) {
            Some(filename) => {
                // We have a filename from the URL
                let path = templates_dir.join(&filename);
                Ok(TargetInfo {
                    exists: path.exists(),
                    path,
                    filename, //: TemplateFetcher::ensure_extension(filename),
                    total_size: 0,
                    url: None,
                    created: false,
                })
            }
            None => Err(Error::TemplateDownloadError(
                url.to_string(),
                "Invalid URL supplied".to_string(),
            )),
        }
    }

    /// Extracts the filename from the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL of the template.
    ///
    /// # Returns
    /// The filename extracted from the URL.
    fn extract_filename_from_url(url: &str) -> Option<String> {
        url::Url::parse(url).ok().and_then(|parsed_url| {
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .filter(|s| !s.is_empty())
                .map(|s| s.split('?').next().unwrap_or(s).to_string())
        })
    }

    /// Pre-processes the URL to handle redirects and get content information.
    ///
    /// # Arguments
    /// * `url` - The original URL of the resource.
    /// * `path` - The original path where the resource should be stored.
    ///
    /// # Returns
    /// A tuple containing the final URL, final path, and total size of the resource.
    async fn pre_process_url(url: &str, path: &PathBuf) -> Result<TargetInfo> {
        // Send request without compression to get headers and actual content-length
        let response = Request::head(url)
            .header("Accept-Encoding", "identity")
            .redirect_policy(RedirectPolicy::Follow)
            .body(())?
            .send_async()
            .await;

        let (total_size, final_url) = match response {
            Ok(r) => {
                // Get content length
                let total_size = r
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let final_url = r.effective_uri().map_or(url.to_string(), |u| u.to_string());
                (total_size, final_url)
            }
            Err(_) => (0, url.to_string()),
        };

        // Get effective path
        let mut target_info = TemplateFetcher::build_target_path(&final_url, &path)?;

        target_info.total_size = total_size;
        target_info.url = Some(final_url);

        Ok(target_info)
    }

    /// Ensures that the directory where the file will be downloaded exists.
    /// If the directory doesn't exist, it will be created.
    ///
    /// # Arguments
    /// * `path` - The path where the file will be downloaded.
    ///
    /// # Returns
    /// Returns a Result indicating success or failure.
    fn ensure_directory_exists(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Downloads a file while showing a progress bar with detailed information.
    /// The progress bar includes the filename, download progress, percentage,
    /// estimated time remaining, and elapsed time.
    ///
    /// # Arguments
    /// * `body` - The response body to read from.
    /// * `file` - The file to write to.
    /// * `total_size` - The total size of the file to download.
    /// * `filename` - The name of the file being downloaded.
    ///
    /// # Returns
    /// Returns a Result indicating success or failure.
    async fn download_with_progress(
        body: &mut AsyncBody,
        file: &mut std::fs::File,
        total_size: u64,
        filename: &str,
    ) -> Result<()> {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template("{prefix:.yellow} {spinner:.green} {bar:40.blue/white.dim} ({percent}%) {msg} ({eta}) [{elapsed_precise}]")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("\u{2501}\u{2501}")
        );

        // Ensure the progress bar appears even for small files
        pb.enable_steady_tick(Duration::from_millis(50)); // More frequent updates for small files
        pb.set_prefix(filename.to_string());

        crossterm::execute!(std::io::stdout(), Hide)?;

        let mut buffer = [0u8; 8192];
        let mut downloaded = 0;

        while let Ok(n) = smol::io::AsyncReadExt::read(body, &mut buffer).await {
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])?;
            downloaded += n as u64;
            pb.set_position(downloaded);
            pb.set_message(format!(
                "{}/{}",
                utils::format_bytes(downloaded),
                utils::format_bytes(total_size)
            ));
        }

        // Change progress bar to green only after download is complete
        pb.set_style(
            ProgressStyle::with_template("{prefix:.yellow} {spinner:.green} {bar:40.green/white.dim} ({percent}%) {msg} ({eta}) [{elapsed_precise}]")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("\u{2501}\u{2501}")
        );

        pb.finish_with_message(format!(
            "{}",
            Green.paint(format!(
                "Downloaded {} ({})",
                filename,
                utils::format_bytes(downloaded)
            ))
        ));

        // Show the green completion message and bar for 800ms
        smol::Timer::after(Duration::from_millis(100)).await;
        pb.finish_and_clear();

        // Show cursor again after clearing the progress bar
        crossterm::execute!(std::io::stdout(), Show)?;

        Ok(())
    }

    /// Downloads a file without showing any progress information.
    /// This is used when progress display is not needed or not possible.
    ///
    /// # Arguments
    /// * `body` - The response body to read from.
    /// * `file` - The file to write to.
    ///
    /// # Returns
    /// Returns a Result indicating success or failure.
    async fn download_without_progress(
        body: &mut AsyncBody,
        file: &mut std::fs::File,
    ) -> Result<()> {
        let mut buffer = [0u8; 8192];
        while let Ok(n) = smol::io::AsyncReadExt::read(body, &mut buffer).await {
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])?;
        }
        Ok(())
    }
}
