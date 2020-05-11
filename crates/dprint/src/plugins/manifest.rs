use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use super::super::environment::Environment;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CacheManifest {
    pub count: u32,
    pub urls: Vec<UrlCacheEntry>
}

impl CacheManifest {
    pub(super) fn new() -> CacheManifest {
        CacheManifest { count: 0, urls: vec![] }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UrlCacheEntry {
    pub url: String,
    pub file_path: String,
}

pub fn read_manifest(environment: &impl Environment) -> Result<CacheManifest, String> {
    let file_path = get_manifest_file_path(environment)?;
    let manifest_file_text = match environment.read_file(&file_path) {
        Ok(text) => Some(text),
        Err(_) => None,
    };

    if let Some(text) = manifest_file_text {
        let deserialized_manifest = serde_json::from_str(&text);
        match deserialized_manifest {
            Ok(manifest) => Ok(manifest),
            Err(err) => {
                environment.log_error(&format!("Error deserializing cache manifest, but ignoring: {}", err));
                Ok(CacheManifest::new())
            }
        }
    } else {
        Ok(CacheManifest::new())
    }
}

pub fn write_manifest(manifest: &CacheManifest, environment: &impl Environment) -> Result<(), String> {
    let file_path = get_manifest_file_path(environment)?;
    let serialized_manifest = serde_json::to_string(&manifest).unwrap();
    environment.write_file(&file_path, &serialized_manifest)
}

fn get_manifest_file_path(environment: &impl Environment) -> Result<PathBuf, String> {
    let app_dir = environment.get_user_app_dir()?;
    Ok(app_dir.join("cache-manifest.json"))
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::environment::TestEnvironment;

    #[test]
    fn it_should_read_ok_manifest() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_user_app_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 0, "urls": [{ "url": "a", "file_path": "b" }] }"#
        ).unwrap();

        assert_eq!(read_manifest(&environment).unwrap(), CacheManifest {
            count: 0,
            urls: vec![UrlCacheEntry {
                url: String::from("a"),
                file_path: String::from("b"),
            }]
        })
    }

    #[test]
    fn it_should_have_empty_manifest_for_deserialization_error() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_user_app_dir().unwrap().join("cache-manifest.json"),
            r#"{ "count": 0, "urls": [{ "url": "a", file_path: "b" }] }"#
        ).unwrap();

        assert_eq!(read_manifest(&environment).unwrap(), CacheManifest::new());
        assert_eq!(environment.get_logged_errors(), vec![
            String::from("Error deserializing cache manifest, but ignoring: key must be a string at line 1 column 38")
        ]);
    }

    #[test]
    fn it_should_deal_with_non_existent_manifest() {
        let environment = TestEnvironment::new();

        assert_eq!(read_manifest(&environment).unwrap(), CacheManifest::new());
        assert_eq!(environment.get_logged_errors().len(), 0);
    }

    #[test]
    fn it_save_manifest() {
        let environment = TestEnvironment::new();
        let manifest = CacheManifest {
            count: 2,
            urls: vec![
                UrlCacheEntry {
                    url: String::from("a"),
                    file_path: String::from("b"),
                },
                UrlCacheEntry {
                    url: String::from("c"),
                    file_path: String::from("d"),
                },
            ]
        };
        write_manifest(&manifest, &environment).unwrap();
        assert_eq!(
            environment.read_file(&environment.get_user_app_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{"count":2,"urls":[{"url":"a","file_path":"b"},{"url":"c","file_path":"d"}]}"#
        );
    }
}