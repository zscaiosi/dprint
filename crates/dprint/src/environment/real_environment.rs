use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};

use super::Environment;
use super::super::types::ErrBox;

pub struct RealEnvironment {
    output_lock: Arc<Mutex<u8>>,
}

#[derive(Debug)]
pub struct DownloadError {
    message: String,
}

impl DownloadError {
    pub fn new(message: &str) -> DownloadError {
        DownloadError { message: String::from(message) }
    }
}

impl std::error::Error for DownloadError {}
impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl RealEnvironment {
    pub fn new() -> RealEnvironment {
        RealEnvironment { output_lock: Arc::new(Mutex::new(0)), }
    }
}

const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo { name: "Dprint", author: "Dprint" };

#[async_trait]
impl Environment for RealEnvironment {
    fn read_file(&self, file_path: &PathBuf) -> Result<String, String> {
        match fs::read_to_string(file_path) {
            Ok(text) => Ok(text),
            Err(err) => Err(err.to_string()),
        }
    }

    fn read_file_bytes(&self, file_path: &PathBuf) -> Result<Bytes, String> {
        match fs::read(file_path) {
            Ok(bytes) => Ok(Bytes::from(bytes)),
            Err(err) => Err(err.to_string()),
        }
    }

    fn write_file(&self, file_path: &PathBuf, file_text: &str) -> Result<(), String> {
        match fs::write(file_path, file_text) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    fn write_file_bytes(&self, file_path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
        match fs::write(file_path, bytes) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    fn remove_file(&self, file_path: &PathBuf) -> Result<(), String> {
        match fs::remove_file(file_path) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn download_file(&self, url: &str) -> Result<Bytes, ErrBox> {
        // todo: docs say to use only one client (it has an internal connection pool)
        let client = Client::new();
        let mut resp = client.get(url).send().await?;
        let total_size = {
            if resp.status().is_success() {
                resp.content_length()
            } else {
                return Err(Box::new(DownloadError::new(
                    &format!("Error downloading: {}. Status: {:?}", url, resp.status())
                )));
            }
        };

        // todo: support multiple progress bars downloading many files at the same time
        self.log(&format!("Downloading: {}", url));
        // https://github.com/mitsuhiko/indicatif/blob/master/examples/download.rs
        let pb = ProgressBar::new(total_size.unwrap_or(0));
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"));
        let mut final_bytes = bytes::BytesMut::new();

        while let Some(chunk) = resp.chunk().await? {
            final_bytes.extend_from_slice(&chunk);
            pb.set_position(final_bytes.len() as u64);
        }

        pb.finish_with_message("downloaded");

        Ok(final_bytes.freeze())
    }

    fn glob(&self, file_patterns: &Vec<String>) -> Result<Vec<PathBuf>, String> {
        let walker = match globwalk::GlobWalkerBuilder::from_patterns(&PathBuf::from("."), file_patterns).follow_links(true).build() {
            Ok(walker) => walker,
            Err(err) => return Err(format!("Error parsing file patterns: {}", err)),
        };

        let mut file_paths = Vec::new();
        for result in walker.into_iter() {
            match result {
                Ok(result) => { file_paths.push(result.into_path()); },
                Err(err) => return Err(format!("Error walking files: {}", err)),
            }
        }

        Ok(file_paths)
    }

    fn path_exists(&self, file_path: &PathBuf) -> bool {
        file_path.exists()
    }

    fn log(&self, text: &str) {
        let _g = self.output_lock.lock().unwrap();
        println!("{}", text);
    }

    fn log_error(&self, text: &str) {
        let _g = self.output_lock.lock().unwrap();
        eprintln!("{}", text);
    }

    fn get_user_app_dir(&self) -> Result<PathBuf, String> {
        match app_dirs::app_root(app_dirs::AppDataType::UserConfig, &APP_INFO) {
            Ok(path) => Ok(path),
            Err(err) => Err(format!("Error getting app directory: {:?}", err)),
        }
    }

    fn get_plugin_cache_dir(&self) -> Result<PathBuf, String> {
        match app_dirs::app_dir(app_dirs::AppDataType::UserCache, &APP_INFO, "cache/plugins") {
            Ok(path) => Ok(path),
            Err(err) => Err(format!("Error getting app directory: {:?}", err)),
        }
    }
}
