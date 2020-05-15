use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use globset::{GlobSetBuilder, GlobSet, Glob};
use super::Environment;
use super::super::types::ErrBox;
use async_trait::async_trait;
use bytes::Bytes;

pub struct TestEnvironment {
    files: Arc<Mutex<HashMap<PathBuf, Bytes>>>,
    logged_messages: Arc<Mutex<Vec<String>>>,
    logged_errors: Arc<Mutex<Vec<String>>>,
    remote_files: Arc<Mutex<HashMap<String, Bytes>>>,
}

impl TestEnvironment {
    pub fn new() -> TestEnvironment {
        TestEnvironment {
            files: Arc::new(Mutex::new(HashMap::new())),
            logged_messages: Arc::new(Mutex::new(Vec::new())),
            logged_errors: Arc::new(Mutex::new(Vec::new())),
            remote_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl TestEnvironment {
    pub fn get_logged_messages(&self) -> Vec<String> {
        self.logged_messages.lock().unwrap().clone()
    }

    pub fn clear_logs(&self) {
        self.logged_messages.lock().unwrap().clear();
        self.logged_errors.lock().unwrap().clear();
    }

    pub fn get_logged_errors(&self) -> Vec<String> {
        self.logged_errors.lock().unwrap().clone()
    }

    pub fn add_remote_file(&self, path: &str, bytes: &'static [u8]) {
        let mut remote_files = self.remote_files.lock().unwrap();
        remote_files.insert(String::from(path), Bytes::from(bytes));
    }
}

#[async_trait]
impl Environment for TestEnvironment {
    fn read_file(&self, file_path: &PathBuf) -> Result<String, ErrBox> {
        let file_bytes = self.read_file_bytes(file_path)?;
        Ok(String::from_utf8(file_bytes.to_vec()).unwrap())
    }

    fn read_file_bytes(&self, file_path: &PathBuf) -> Result<Bytes, ErrBox> {
        let files = self.files.lock().unwrap();
        match files.get(file_path) {
            Some(text) => Ok(text.clone()),
            None => err!("Could not find file at path {}", file_path.to_string_lossy()),
        }
    }

    fn write_file(&self, file_path: &PathBuf, file_text: &str) -> Result<(), ErrBox> {
        self.write_file_bytes(file_path, file_text.as_bytes())
    }

    fn write_file_bytes(&self, file_path: &PathBuf, bytes: &[u8]) -> Result<(), ErrBox> {
        let mut files = self.files.lock().unwrap();
        files.insert(file_path.clone(), Bytes::from(bytes.to_vec()));
        Ok(())
    }

    fn remove_file(&self, file_path: &PathBuf) -> Result<(), ErrBox> {
        let mut files = self.files.lock().unwrap();
        files.remove(file_path);
        Ok(())
    }

    async fn download_file(&self, url: &str) -> Result<Bytes, ErrBox> {
        let remote_files = self.remote_files.lock().unwrap();
        match remote_files.get(&String::from(url)) {
            Some(bytes) => Ok(bytes.clone()),
            None => err!("Could not find file at url {}", url),
        }
    }

    fn glob(&self, file_patterns: &Vec<String>) -> Result<Vec<PathBuf>, ErrBox> {
        let mut file_paths = Vec::new();
        let includes_set = file_patterns_to_glob_set(file_patterns.iter().filter(|p| !p.starts_with("!")).map(|p| p.to_owned()))?;
        let excludes_set = file_patterns_to_glob_set(file_patterns.iter().filter(|p| p.starts_with("!")).map(|p| String::from(&p[1..])))?;
        let files = self.files.lock().unwrap();

        for key in files.keys() {
            if includes_set.is_match(key) && !excludes_set.is_match(key) {
                file_paths.push(key.clone());
            }
        }

        Ok(file_paths)
    }

    fn path_exists(&self, file_path: &PathBuf) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(file_path)
    }

    fn log(&self, text: &str) {
        self.logged_messages.lock().unwrap().push(String::from(text));
    }

    fn log_error(&self, text: &str) {
        self.logged_errors.lock().unwrap().push(String::from(text));
    }

    fn get_cache_dir(&self) -> Result<PathBuf, ErrBox> {
        Ok(PathBuf::from("/cache"))
    }
}

fn file_patterns_to_glob_set(file_patterns: impl Iterator<Item = String>) -> Result<GlobSet, ErrBox> {
    let mut builder = GlobSetBuilder::new();
    for file_pattern in file_patterns {
        match Glob::new(&file_pattern) {
            Ok(glob) => { builder.add(glob); },
            Err(err) => return err!("Error parsing glob {}: {}", file_pattern, err),
        }
    }
    match builder.build() {
        Ok(glob_set) => Ok(glob_set),
        Err(err) => err!("Error building glob set: {}", err),
    }
}
