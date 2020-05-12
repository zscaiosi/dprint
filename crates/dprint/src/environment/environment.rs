use std::path::PathBuf;
use async_trait::async_trait;
use bytes::Bytes;
use super::super::types::ErrBox;

#[async_trait]
pub trait Environment : std::marker::Sync {
    fn read_file(&self, file_path: &PathBuf) -> Result<String, String>;
    fn read_file_bytes(&self, file_path: &PathBuf) -> Result<Bytes, String>;
    fn write_file(&self, file_path: &PathBuf, file_text: &str) -> Result<(), String>;
    fn write_file_bytes(&self, file_path: &PathBuf, bytes: &[u8]) -> Result<(), String>;
    fn remove_file(&self, file_path: &PathBuf) -> Result<(), String>;
    fn glob(&self, file_patterns: &Vec<String>) -> Result<Vec<PathBuf>, String>;
    fn path_exists(&self, file_path: &PathBuf) -> bool;
    fn log(&self, text: &str);
    fn log_error(&self, text: &str);
    async fn download_file(&self, url: &str) -> Result<Bytes, ErrBox>;
    fn get_user_app_dir(&self) -> Result<PathBuf, String>;
    fn get_plugin_cache_dir(&self) -> Result<PathBuf, String>;
}
