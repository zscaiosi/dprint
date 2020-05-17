use std::collections::HashMap;
use dprint_core::configuration::GlobalConfiguration;

use crate::environment::Environment;
use crate::types::ErrBox;
use super::{Plugin, InitializedPlugin};

pub fn initialize_plugin(
    plugin: Box<dyn Plugin>,
    plugin_config: HashMap<String, String>,
    global_config: &GlobalConfiguration,
    environment: &impl Environment,
) -> Result<Box<dyn InitializedPlugin>, ErrBox> {
    let mut plugin = plugin;
    let initialized_plugin = plugin.initialize(plugin_config, &global_config)?;
    let mut diagnostic_count = 0;

    for diagnostic in initialized_plugin.get_config_diagnostics() {
        environment.log_error(&format!("[{}]: {}", plugin.name(), diagnostic.message));
        diagnostic_count += 1;
    }

    if diagnostic_count > 0 {
        err!("Had {} config diagnostic(s) for {}.", diagnostic_count, plugin.name())
    } else {
        Ok(initialized_plugin)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//     use super::*;
//     use super::super::{TestPlugin, Plugin};
//     use crate::environment::{TestEnvironment};
//     use crate::configuration::{ConfigMapValue, ConfigMap};

//     #[test]
//     fn it_should_initialize_plugins() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
//         config_map.insert(String::from("typescript"), {
//             let mut ts_config_map = HashMap::new();
//             ts_config_map.insert(String::from("lineWidth"), String::from("40"));
//             ConfigMapValue::HashMap(ts_config_map)
//         });
//         assert_creates(config_map);
//     }

//     #[test]
//     fn it_should_error_when_has_double_plugin_config_keys() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
//         config_map.insert(String::from("typescript"), {
//             let mut map = HashMap::new();
//             map.insert(String::from("lineWidth"), String::from("40"));
//             ConfigMapValue::HashMap(map)
//         });
//         config_map.insert(String::from("javascript"), {
//             let mut map = HashMap::new();
//             map.insert(String::from("lineWidth"), String::from("40"));
//             ConfigMapValue::HashMap(map)
//         });
//         assert_errors(
//             config_map,
//             vec![],
//             "Error initializing from configuration file. Cannot specify both the 'typescript' and 'javascript' configurations for dprint-plugin-typescript.",
//             vec![create_plugin()],
//         );
//     }

//     #[test]
//     fn it_should_error_plugin_key_is_not_object() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
//         config_map.insert(String::from("typescript"), ConfigMapValue::String(String::from("")));
//         assert_errors(
//             config_map,
//             vec![],
//             "Error initializing from configuration file. Expected the configuration property 'typescript' to be an object.",
//             vec![create_plugin()],
//         );
//     }

//     #[test]
//     fn it_should_log_global_diagnostics() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("null")));
//         assert_errors(
//             config_map,
//             vec!["Error parsing configuration value for 'lineWidth'. Message: invalid digit found in string"],
//             "Error initializing from configuration file. Had 1 diagnostic(s).",
//             vec![create_plugin()],
//         );
//     }


//     #[test]
//     fn it_should_log_unexpected_object_properties() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("test"), ConfigMapValue::HashMap(HashMap::new()));
//         assert_errors(
//             config_map,
//             vec![],
//             "Error initializing from configuration file. Unexpected object property 'test'.",
//             vec![create_plugin()],
//         );
//     }

//     #[test]
//     fn it_should_log_plugin_diagnostics() {
//         let mut config_map = HashMap::new();
//         config_map.insert(String::from("typescript"), {
//             let mut map = HashMap::new();
//             map.insert(String::from("lineWidth"), String::from("null"));
//             ConfigMapValue::HashMap(map)
//         });
//         let mut plugin = create_plugin();
//         plugin.set_diagnostics(vec![("lineWidth", "Invalid digit found in string")]);
//         assert_errors(
//             config_map,
//             vec!["[dprint-plugin-typescript]: Invalid digit found in string"],
//             "Error initializing from configuration file. Had 1 diagnostic(s).",
//             vec![plugin],
//         );
//     }

//     fn assert_creates(config_map: ConfigMap) {
//         let test_environment = TestEnvironment::new();
//         let mut plugins = get_plugins(vec![create_plugin()]);
//         assert_eq!(initialize_plugins(config_map, &mut plugins, &test_environment).is_ok(), true);
//     }

//     fn assert_errors(config_map: ConfigMap, logged_errors: Vec<&'static str>, message: &str, plugins: Vec<TestPlugin>) {
//         let test_environment = TestEnvironment::new();
//         let mut plugins = get_plugins(plugins);
//         let result = initialize_plugins(config_map, &mut plugins, &test_environment);
//         assert_eq!(result.err().unwrap().to_string(), message);
//         assert_eq!(test_environment.get_logged_errors(), logged_errors);
//     }

//     fn get_plugins(plugins: Vec<TestPlugin>) -> Plugins {
//         plugins.into_iter().map(|plugin| Box::new(plugin) as Box<dyn Plugin>).collect()
//     }

//     fn create_plugin() -> TestPlugin {
//         TestPlugin::new(
//             "dprint-plugin-typescript",
//             vec!["typescript", "javascript"],
//             vec![".ts"]
//         )
//     }
// }
