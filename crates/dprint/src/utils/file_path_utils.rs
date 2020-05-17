use std::path::PathBuf;
use std::collections::HashSet;

pub fn get_lowercase_file_extension(file_path: &PathBuf) -> Option<String> {
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        Some(String::from(ext).to_lowercase())
    } else {
        None
    }
}

pub fn get_unique_extensions_from_file_paths(file_paths: &Vec<PathBuf>) -> HashSet<String> {
    let mut extensions = HashSet::new();

    for file_path in file_paths {
        if let Some(ext) = get_lowercase_file_extension(file_path) {
            extensions.insert(ext);
        }
    }

    extensions
}