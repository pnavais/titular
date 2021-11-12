use std::cmp::min;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::path::PathBuf;

use crate::{
    error::*,
};

use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

#[cfg(feature = "fetcher")]
/// Downloads asynchronously the resource defined in the given URL and
/// stores it in the supplied path.
pub async fn download_file(url: &str, path: &PathBuf) -> Result<()> {   
    let resp = Client::new()
                .get(url)
                .send()
                .await;
    
    match resp {
        Ok(r) => {
            if r.status().is_success() {
                let total_size = r.content_length() .ok_or(format!("Failed to get content length from '{}'", &url))?;

                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .progress_chars("#>-"));
                pb.set_message(format!("Downloading {}", url));

                let parent_dir = path.parent().ok_or(Error::from(format!("Failed to obtain config directory")))?;
                std::fs::create_dir_all(parent_dir)?;
                let mut file = File::create(path).or(Err(Error::from(format!("Failed to create file '{}'", path.to_string_lossy()))))?;
                let mut downloaded: u64 = 0;
                let mut stream = r.bytes_stream();
                
                while let Some(item) = stream.next().await {            
                    let chunk = item.or(Err(Error::from(format!("Error while downloading file"))))?;
                    file.write(&chunk)
                        .or(Err(Error::from(format!("Error while writing to file"))))?;
                    let new = min(downloaded + (chunk.len() as u64), total_size);
                    downloaded = new;
                    pb.set_position(new);
                }

                pb.finish_with_message(format!("Downloaded {} to {}", url, path.to_string_lossy()));
                thread::sleep(Duration::from_millis(1200));
                pb.finish_and_clear();

                Ok(())
            } else {
                Err(Error::TemplateDownloadError(format!("{}", path.file_name().and_then(|name| name.to_str()).unwrap()), format!("Error {}", r.status())))
            }
        },
        Err(e) => Err(Error::from(e)),
    }
}