use std::path::PathBuf;

use super::super::environment::Environment;
use super::manifest::*;
use super::super::types::ErrBox;

pub struct PluginCache<'a, TEnvironment> where TEnvironment : Environment {
    environment: &'a TEnvironment,
    cache_manifest: CacheManifest,
}

impl<'a, TEnvironment> PluginCache<'a, TEnvironment> where TEnvironment : Environment {
    pub fn new(environment: &'a TEnvironment) -> Result<Self, String> {
        let cache_manifest = read_manifest(environment)?;
        Ok(PluginCache {
            environment,
            cache_manifest,
        })
    }

    pub async fn get_plugin_file_path(&mut self, url: &str) -> Result<PathBuf, ErrBox> {
        if let Some(cache_entry) = self.get_url_cache_entry(url) {
            return Ok(PathBuf::from(&cache_entry.file_path));
        }

        let cache_dir = self.environment.get_plugin_cache_dir()?;
        let file_bytes = self.environment.download_file(url).await?;
        let cache_count = self.cache_manifest.count + 1;
        let file_path = cache_dir.join(&format!("{}.{}", cache_count, get_url_extension(url)?));
        let url_cache_entry = UrlCacheEntry { url: String::from(url), file_path: file_path.to_string_lossy().to_string() };

        self.environment.write_file_bytes(&file_path, &file_bytes)?;

        self.cache_manifest.count += 1;
        self.cache_manifest.urls.push(url_cache_entry);
        self.save_manifest()?;

        Ok(file_path)
    }

    pub fn forget_url(&mut self, url: &str) -> Result<(), ErrBox> {
        if let Some(index) = self.get_url_cache_entry_index(url) {
            if let Some(entry) = self.cache_manifest.urls.get(index) {
                match self.environment.remove_file(&PathBuf::from(&entry.file_path)) {
                    _ => {}, // do nothing on success or failure
                }
            }
            self.cache_manifest.urls.remove(index);
            self.save_manifest()?;
        }

        Ok(())
    }

    fn get_url_cache_entry<'b>(&'b self, url: &str) -> Option<&'b UrlCacheEntry> {
        self.cache_manifest.urls.iter().filter(|u| u.url == url).next()
    }

    fn get_url_cache_entry_index(&self, url: &str) -> Option<usize> {
        self.cache_manifest.urls.iter().position(|u| u.url == url)
    }

    fn save_manifest(&self) -> Result<(), String> {
        write_manifest(&self.cache_manifest, self.environment)
    }
}

fn get_url_extension(url: &str) -> Result<String, String> {
    if let Some(ext_index) = url.rfind('.') {
        Ok(String::from(&url[ext_index + 1..]))
    } else {
        Err(format!("Could not find extension for url: {}", url))
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
            &environment.get_user_app_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.dll", "file_path": "/my-file.dll" }] }"#
        ).unwrap();

        let mut cache = PluginCache::new(&environment).unwrap();
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.dll").await?;

        assert_eq!(file_path, PathBuf::from("/my-file.dll"));
        Ok(())
    }


    #[tokio::test]
    async fn it_should_download_file() -> Result<(), ErrBox> {
        let environment = TestEnvironment::new();
        environment.add_remote_file("https://plugins.dprint.dev/test.dll", "t".as_bytes());

        let mut cache = PluginCache::new(&environment).unwrap();
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.dll").await?;
        let expected_file_path = PathBuf::from("/cache").join("1.dll");

        assert_eq!(file_path, expected_file_path);

        // should be the same when requesting it again
        let file_path = cache.get_plugin_file_path("https://plugins.dprint.dev/test.dll").await?;
        assert_eq!(file_path, expected_file_path);

        // should have saved the manifest
        assert_eq!(
            environment.read_file(&environment.get_user_app_dir().unwrap().join("cache-manifest.json")).unwrap(),
            format!(
                r#"{{"count":1,"urls":[{{"url":"https://plugins.dprint.dev/test.dll","file_path":"{}"}}]}}"#,
                expected_file_path.to_string_lossy().replace("\\", "\\\\")
            )
        );
        Ok(())
    }

    #[test]
    fn it_should_delete_url_from_manifest_when_no_file() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_user_app_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.dll", "file_path": "/my-file.dll" }] }"#
        ).unwrap();

        let mut cache = PluginCache::new(&environment).unwrap();
        cache.forget_url("https://plugins.dprint.dev/test.dll").unwrap();

        assert_eq!(
            environment.read_file(&environment.get_user_app_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{"count":1,"urls":[]}"# // count should remain the same
        );
    }

    #[test]
    fn it_should_delete_url_from_manifest_when_file_exists() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_user_app_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 1, "urls": [{ "url": "https://plugins.dprint.dev/test.dll", "file_path": "/my-file.dll" }] }"#
        ).unwrap();
        let dll_file_path = PathBuf::from("/my-file.dll");
        environment.write_file_bytes(&dll_file_path, "t".as_bytes()).unwrap();

        let mut cache = PluginCache::new(&environment).unwrap();
        cache.forget_url("https://plugins.dprint.dev/test.dll").unwrap();

        // should delete the file too
        assert_eq!(environment.read_file(&dll_file_path).is_err(), true);

        assert_eq!(
            environment.read_file(&environment.get_user_app_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{"count":1,"urls":[]}"# // count should remain the same
        );
    }
}
