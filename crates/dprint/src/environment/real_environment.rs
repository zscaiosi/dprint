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

impl RealEnvironment {
    pub fn new() -> RealEnvironment {
        RealEnvironment { output_lock: Arc::new(Mutex::new(0)), }
    }
}

const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo { name: "Dprint", author: "Dprint" };

#[async_trait]
impl Environment for RealEnvironment {
    fn read_file(&self, file_path: &PathBuf) -> Result<String, ErrBox> {
        let text = fs::read_to_string(file_path)?;
        Ok(text)
    }

    fn read_file_bytes(&self, file_path: &PathBuf) -> Result<Bytes, ErrBox> {
        let bytes = fs::read(file_path)?;
        Ok(Bytes::from(bytes))
    }

    fn write_file(&self, file_path: &PathBuf, file_text: &str) -> Result<(), ErrBox> {
        fs::write(file_path, file_text)?;
        Ok(())
    }

    fn write_file_bytes(&self, file_path: &PathBuf, bytes: &[u8]) -> Result<(), ErrBox> {
        fs::write(file_path, bytes)?;
        Ok(())
    }

    fn remove_file(&self, file_path: &PathBuf) -> Result<(), ErrBox> {
        fs::remove_file(file_path)?;
        Ok(())
    }

    async fn download_file(&self, url: &str) -> Result<Bytes, ErrBox> {
        // todo: docs say to use only one client (it has an internal connection pool)
        let client = Client::new();
        let mut resp = client.get(url).send().await?;
        let total_size = {
            if resp.status().is_success() {
                resp.content_length()
            } else {
                return err!("Error downloading: {}. Status: {:?}", url, resp.status());
            }
        };

        // todo: support multiple progress bars downloading many files at the same time
        self.log(&format!("Downloading {}", url));
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

    fn glob(&self, file_patterns: &Vec<String>) -> Result<Vec<PathBuf>, ErrBox> {
        let walker = match globwalk::GlobWalkerBuilder::from_patterns(&PathBuf::from("."), file_patterns).follow_links(true).build() {
            Ok(walker) => walker,
            Err(err) => return err!("Error parsing file patterns: {}", err),
        };

        let mut file_paths = Vec::new();
        for result in walker.into_iter() {
            match result {
                Ok(result) => { file_paths.push(result.into_path()); },
                Err(err) => return err!("Error walking files: {}", err),
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

    fn get_cache_dir(&self) -> Result<PathBuf, ErrBox> {
        match app_dirs::app_dir(app_dirs::AppDataType::UserCache, &APP_INFO, "cache") {
            Ok(path) => Ok(path),
            Err(err) => err!("Error getting cache directory: {:?}", err),
        }
    }
}
