use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use super::super::environment::Environment;
use super::super::types::ErrBox;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CacheManifest {
    pub urls: Vec<UrlCacheEntry>
}

impl CacheManifest {
    pub(super) fn new() -> CacheManifest {
        CacheManifest { urls: vec![] }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UrlCacheEntry {
    pub url: String,
    pub file_name: String,
}

pub fn read_manifest(environment: &impl Environment) -> Result<CacheManifest, ErrBox> {
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

pub fn write_manifest(manifest: &CacheManifest, environment: &impl Environment) -> Result<(), ErrBox> {
    let file_path = get_manifest_file_path(environment)?;
    let serialized_manifest = serde_json::to_string(&manifest).unwrap();
    environment.write_file(&file_path, &serialized_manifest)
}

fn get_manifest_file_path(environment: &impl Environment) -> Result<PathBuf, ErrBox> {
    let app_dir = environment.get_cache_dir()?;
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
            &environment.get_cache_dir().unwrap().join("cache-manifest.json"),
            r#"{ "urls": [{ "url": "a", "file_name": "b" }] }"#
        ).unwrap();

        assert_eq!(read_manifest(&environment).unwrap(), CacheManifest {
            urls: vec![UrlCacheEntry {
                url: String::from("a"),
                file_name: String::from("b"),
            }]
        })
    }

    #[test]
    fn it_should_have_empty_manifest_for_deserialization_error() {
        let environment = TestEnvironment::new();
        environment.write_file(
            &environment.get_cache_dir().unwrap().join("cache-manifest.json"),
            r#"{ "urls": [{ "url": "a", file_name: "b" }] }"#
        ).unwrap();

        assert_eq!(read_manifest(&environment).unwrap(), CacheManifest::new());
        assert_eq!(environment.get_logged_errors(), vec![
            String::from("Error deserializing cache manifest, but ignoring: key must be a string at line 1 column 26")
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
            urls: vec![
                UrlCacheEntry {
                    url: String::from("a"),
                    file_name: String::from("b"),
                },
                UrlCacheEntry {
                    url: String::from("c"),
                    file_name: String::from("d"),
                },
            ]
        };
        write_manifest(&manifest, &environment).unwrap();
        assert_eq!(
            environment.read_file(&environment.get_cache_dir().unwrap().join("cache-manifest.json")).unwrap(),
            r#"{urls":[{"url":"a","file_name":"b"},{"url":"c","file_name":"d"}]}"#
        );
    }
}