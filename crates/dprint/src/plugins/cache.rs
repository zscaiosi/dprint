use std::path::PathBuf;

use super::super::environment::Environment;
use super::manifest::*;
use super::super::types::ErrBox;
use super::wasm::compile;

pub struct Cache<'a, TEnvironment> where TEnvironment : Environment {
    environment: &'a TEnvironment,
    cache_manifest: CacheManifest,
}

impl<'a, TEnvironment> Cache<'a, TEnvironment> where TEnvironment : Environment {
    pub fn new(environment: &'a TEnvironment) -> Result<Self, ErrBox> {
        let cache_manifest = read_manifest(environment)?;
        Ok(Cache {
            environment,
            cache_manifest,
        })
    }

    pub async fn get_plugin_file_path(&mut self, url: &str) -> Result<PathBuf, ErrBox> {
        let cache_dir = self.environment.get_cache_dir()?;
        if let Some(cache_entry) = self.get_url_cache_entry(url) {
            let cache_file = cache_dir.join(&cache_entry.file_name);
            return Ok(PathBuf::from(&cache_file));
        }

        let file_bytes = self.environment.download_file(url).await?;
        let file_name = self.get_file_name_from_url_or_path(url, "compiled_wasm");
        let file_path = cache_dir.join(&file_name);
        let url_cache_entry = UrlCacheEntry { url: String::from(url), file_name };

        self.environment.log("Compiling wasm module...");
        let file_bytes = compile(&file_bytes)?;

        self.environment.write_file_bytes(&file_path, &file_bytes)?;

        self.cache_manifest.urls.push(url_cache_entry);
        self.save_manifest()?;

        Ok(file_path)
    }

    pub fn forget_url(&mut self, url: &str) -> Result<(), ErrBox> {
        if let Some(index) = self.get_url_cache_entry_index(url) {
            if let Some(entry) = self.cache_manifest.urls.get(index) {
                let cache_dir = self.environment.get_cache_dir()?;
                let cache_file = cache_dir.join(&entry.file_name);
                match self.environment.remove_file(&cache_file) {
                    _ => {}, // do nothing on success or failure
                }
            }
            self.cache_manifest.urls.remove(index);
            self.save_manifest()?;
        }

        Ok(())
    }

    fn get_file_name_from_url_or_path(&self, text: &str, extension: &str) -> String {
        let text = text.trim_end_matches('/').trim_end_matches('\\');
        let last_slash = std::cmp::max(text.rfind('/').unwrap_or(0), text.rfind('\\').unwrap_or(0));
        if last_slash == 0 {
            self.get_unique_file_name("temp", extension)
        } else {
            let file_name = PathBuf::from(&text[last_slash + 1..]);
            let file_stem = file_name.file_stem().expect("Expected to find the file stem."); // no extension
            self.get_unique_file_name(file_stem.to_str().unwrap(), extension)
        }
    }

    fn get_unique_file_name(&self, prefix: &str, extension: &str) -> String {
        let mut index = 0;
        loop {
            let file_name_with_ext = if index == 0 {
                get_file_name_with_ext(prefix, extension)
            } else {
                get_file_name_with_ext(&format!("{}_{}", prefix, index), extension)
            };
            if self.get_file_name_cache_entry(&file_name_with_ext).is_some() {
                index += 1;
            } else {
                return file_name_with_ext;
            }
        }

        fn get_file_name_with_ext(file_name: &str, extension: &str) -> String {
            format!("{}.{}", file_name, extension)
        }
    }

    fn get_file_name_cache_entry<'b>(&'b self, file_name: &str) -> Option<&'b UrlCacheEntry> {
        self.cache_manifest.urls.iter().filter(|u| u.file_name == file_name).next()
    }

    fn get_url_cache_entry<'b>(&'b self, url: &str) -> Option<&'b UrlCacheEntry> {
        self.cache_manifest.urls.iter().filter(|u| u.url == url).next()
    }

    fn get_url_cache_entry_index(&self, url: &str) -> Option<usize> {
        self.cache_manifest.urls.iter().position(|u| u.url == url)
    }

    fn save_manifest(&self) -> Result<(), ErrBox> {
        write_manifest(&self.cache_manifest, self.environment)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::environment::TestEnvironment;
    use super::super::super::types::ErrBox;

    #[tokio::test]
    async fn it_should_read_file_paths_from_manifest() -> Result<(), ErrBox> {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_cache_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.wasm", "file_path": "/my-file.wasm" }] }"#
        ).unwrap();

        let mut cache = Cache::new(&environment).unwrap();
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.wasm").await?;

        assert_eq!(file_path, PathBuf::from("/my-file.wasm"));
        Ok(())
    }


    #[tokio::test]
    async fn it_should_download_file() -> Result<(), ErrBox> {
        let environment = TestEnvironment::new();
        environment.add_remote_file("https://plugins.dprint.dev/test.wasm", "t".as_bytes());

        let mut cache = Cache::new(&environment).unwrap();
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.wasm").await?;
        let expected_file_path = PathBuf::from("/cache").join("1.wasm");

        assert_eq!(file_path, expected_file_path);

        // should be the same when requesting it again
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.wasm").await?;
        assert_eq!(file_path, expected_file_path);

        // should have saved the manifest
        assert_eq!(
            environment.read_file(&environment.get_cache_dir().unwrap().join("cache-manifest.json")).unwrap(),
            format!(
                r#"{{"count":1,"urls":[{{"url":"https://plugins.dprint.dev/test.wasm","file_path":"{}"}}]}}"#,
                expected_file_path.to_string_lossy().replace("\\", "\\\\")
            )
        );
        Ok(())
    }

    #[test]
    fn it_should_delete_url_from_manifest_when_no_file() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_cache_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.wasm", "file_path": "/my-file.wasm" }] }"#
        ).unwrap();

        let mut cache = Cache::new(&environment).unwrap();
        cache.forget_url("https://plugins.dprint.dev/test.wasm").unwrap();

        assert_eq!(
            environment.read_file(&environment.get_cache_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{"count":1,"urls":[]}"# // count should remain the same
        );
    }

    #[test]
    fn it_should_delete_url_from_manifest_when_file_exists() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_cache_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.wasm", "file_path": "/my-file.wasm" }] }"#
        ).unwrap();
        let dll_file_path = PathBuf::from("/my-file.wasm");
        environment.write_file_bytes(&dll_file_path, "t".as_bytes()).unwrap();

        let mut cache = Cache::new(&environment).unwrap();
        cache.forget_url("https://plugins.dprint.dev/test.wasm").unwrap();

        // should delete the file too
        assert_eq!(environment.read_file(&dll_file_path).is_err(), true);

        assert_eq!(
            environment.read_file(&environment.get_cache_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{"count":1,"urls":[]}"# // count should remain the same
        );
    }
}
