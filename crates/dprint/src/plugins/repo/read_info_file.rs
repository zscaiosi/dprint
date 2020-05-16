use crate::environment::Environment;
use crate::types::ErrBox;
use jsonc_parser::{parse_to_value, JsonValue, JsonObject};

#[derive(PartialEq, Debug)]
pub struct InfoFile {
    pub plugin_system_schema_version: u32,
    pub latest_plugins: Vec<InfoFilePluginInfo>,
}

#[derive(PartialEq, Debug)]
pub struct InfoFilePluginInfo {
    pub name: String,
    pub version: String,
    pub url: String,
    pub config_key: String,
}

const SCHEMA_VERSION: u8 = 1;
pub const REMOTE_INFO_URL: &'static str = "https://plugins.dprint.dev/info.json";

pub async fn read_info_file(environment: &impl Environment) -> Result<InfoFile, ErrBox> {
    let info_bytes = environment.download_file(REMOTE_INFO_URL).await?;
    let info_text = String::from_utf8(info_bytes.to_vec())?;
    let json_value = match parse_to_value(&info_text) {
        Ok(Some(value)) => value,
        Ok(None) => return err!("Expected info.json to have a JSON value."),
        Err(err) => return err!("{}", err.get_message_with_range(&info_text)),
    };
    let mut obj = match json_value {
        JsonValue::Object(obj) => obj,
        _ => return err!("Expected object in root element."),
    };

    // check schema version
    let schema_version = match obj.remove(&String::from("schemaVersion")) {
        Some(JsonValue::Number(value)) => value.parse::<u32>()?,
        _ => return err!("Could not find schema version."),
    };
    if schema_version != SCHEMA_VERSION as u32 {
        return err!("Cannot handle schema version {}. Expected {}. This might mean your dprint CLI version is old and isn't able to get the latest information.", schema_version, SCHEMA_VERSION);
    }

    // get plugin system version
    let plugin_system_schema_version = match obj.remove(&String::from("pluginSystemSchemaVersion")) {
        Some(JsonValue::Number(value)) => value.parse::<u32>()?,
        _ => return err!("Could not find plugin system schema version."),
    };

    let latest_plugins = match obj.remove(&String::from("latest")) {
        Some(JsonValue::Array(arr)) => {
            let mut plugins = Vec::new();
            for value in arr.into_iter() {
                plugins.push(get_latest_plugin(value)?);
            }
            plugins
        },
        _ => return err!("Could not find latest plugins array."),
    };

    Ok(InfoFile {
        plugin_system_schema_version,
        latest_plugins,
    })
}

fn get_latest_plugin(value: JsonValue) -> Result<InfoFilePluginInfo, ErrBox> {
    let mut obj = match value {
        JsonValue::Object(obj) => obj,
        _ => return err!("Expected an object in the latest array."),
    };
    let name = get_string(&mut obj, "name")?;
    let version = get_string(&mut obj, "version")?;
    let url = get_string(&mut obj, "url")?;
    let config_key = get_string(&mut obj, "configKey")?;

    Ok(InfoFilePluginInfo { name, version, url, config_key })
}

fn get_string(value: &mut JsonObject, name: &str) -> Result<String, ErrBox> {
    match value.remove(name) {
        Some(JsonValue::String(text)) => Ok(text),
        _ => return err!("Could not find value: {}", name),
    }
}

#[cfg(test)]
mod test {
    use crate::environment::TestEnvironment;
    use super::*;

    #[tokio::test]
    async fn should_get_info() {
        let environment = TestEnvironment::new();
        environment.add_remote_file(REMOTE_INFO_URL, r#"{
    "schemaVersion": 1,
    "pluginSystemSchemaVersion": 1,
    "latest": [{
        "name": "dprint-plugin-typescript",
        "version": "0.17.2",
        "url": "https://plugins.dprint.dev/typescript-0.17.2.wasm",
        "configKey": "typescript"
    }, {
        "name": "dprint-plugin-jsonc",
        "version": "0.2.3",
        "url": "https://plugins.dprint.dev/json-0.2.3.wasm",
        "configKey": "json"
    }]
}"#.as_bytes());
        let info_file = read_info_file(&environment).await.unwrap();
        assert_eq!(info_file, InfoFile {
            plugin_system_schema_version: 1,
            latest_plugins: vec![InfoFilePluginInfo {
                name: String::from("dprint-plugin-typescript"),
                version: String::from("0.17.2"),
                url: String::from("https://plugins.dprint.dev/typescript-0.17.2.wasm"),
                config_key: String::from("typescript"),
            }, InfoFilePluginInfo {
                name: String::from("dprint-plugin-jsonc"),
                version: String::from("0.2.3"),
                url: String::from("https://plugins.dprint.dev/json-0.2.3.wasm"),
                config_key: String::from("json"),
            }],
        })
    }

    #[tokio::test]
    async fn should_error_if_schema_version_is_different() {
        let environment = TestEnvironment::new();
        environment.add_remote_file(REMOTE_INFO_URL, r#"{
    "schemaVersion": 2,
}"#.as_bytes());
        let message = read_info_file(&environment).await.err().unwrap();
        assert_eq!(
            message.to_string(),
            "Cannot handle schema version 2. Expected 1. This might mean your dprint CLI version is old and isn't able to get the latest information."
        );
    }

    #[tokio::test]
    async fn should_error_if_no_plugin_system_set() {
        let environment = TestEnvironment::new();
        environment.add_remote_file(REMOTE_INFO_URL, r#"{
    "schemaVersion": 1,
}"#.as_bytes());
        let message = read_info_file(&environment).await.err().unwrap();
        assert_eq!(
            message.to_string(),
            "Could not find plugin system schema version."
        );
    }

    #[tokio::test]
    async fn should_error_when_no_internet() {
        let environment = TestEnvironment::new();
        let message = read_info_file(&environment).await.err().unwrap();
        assert_eq!(
            message.to_string(),
            "Could not find file at url https://plugins.dprint.dev/info.json"
        );
    }
}