use std::collections::HashMap;
use core::slice::{Iter};
use std::path::PathBuf;

use super::configuration::{ConfigMapValue, ConfigMap};
use super::plugins::{Plugin, PluginContainer};
use super::environment::Environment;

/// A formatter constructed from a collection of plugins.
pub struct Formatter {
    plugin_container: PluginContainer,
}

impl Formatter {
    /// Creates a new formatter
    pub fn new(plugin_container: PluginContainer) -> Formatter {
        Formatter { plugin_container }
    }

    /// Iterates over the plugins.
    pub fn iter_plugins(&self) -> Iter<'_, Box<dyn Plugin>> {
        self.plugin_container.iter()
    }

    /// Formats the file text with one of the plugins.
    ///
    /// Returns the string when a plugin formatted or error. Otherwise None when no plugin was found.
    pub fn format_text(&self, file_path: &PathBuf, file_text: &str) -> Result<Option<String>, String> {
        for plugin in self.iter_plugins() {
            if plugin.should_format_file(file_path, file_text) {
                return plugin.format_text(file_path, file_text).map(|x| Some(x));
            }
        }

        Ok(None)
    }
}

pub fn create_formatter(config_map: ConfigMap, plugins: PluginContainer, environment: &impl Environment) -> Result<Formatter, String> {
    let mut formatter = Formatter::new(plugins);

    match initialize_plugins(config_map, &mut formatter, environment) {
        Ok(()) => Ok(formatter),
        Err(err) => Err(format!("Error initializing from configuration file. {}", err)),
    }
}

fn initialize_plugins(config_map: ConfigMap, formatter: &mut Formatter, environment: &impl Environment) -> Result<(), String> {
    let mut config_map = config_map;

    // get hashmaps per plugin
    let mut plugins_to_config = handle_plugins_to_config_map(&formatter, &mut config_map)?;

    // now get and resolve the global config
    let global_config = get_global_config_from_config_map(config_map)?;
    let global_config_result = dprint_core::configuration::resolve_global_config(global_config);

    // check global diagnostics
    let mut diagnostic_count = 0;
    if !global_config_result.diagnostics.is_empty() {
        for diagnostic in &global_config_result.diagnostics {
            environment.log_error(&diagnostic.message);
            diagnostic_count += 1;
        }
    }

    // intiailize the plugins
    for plugin in formatter.iter_plugins() {
        plugin.initialize(plugins_to_config.remove(&plugin.name()).unwrap_or(HashMap::new()), &global_config_result.config);

        for diagnostic in plugin.get_config_diagnostics() {
            environment.log_error(&format!("[{}]: {}", plugin.name(), diagnostic.message));
            diagnostic_count += 1;
        }
    }

    if diagnostic_count > 0 {
        Err(format!("Had {} diagnostic(s).", diagnostic_count))
    } else {
        Ok(())
    }
}

fn handle_plugins_to_config_map(
    formatter: &Formatter,
    config_map: &mut ConfigMap,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let mut plugin_maps = HashMap::new();
    for plugin in formatter.iter_plugins() {
        let mut key_name = None;
        let config_keys = plugin.config_keys();
        for config_key in config_keys {
            if config_map.contains_key(&config_key) {
                if let Some(key_name) = key_name {
                    return Err(format!("Cannot specify both the '{}' and '{}' configurations for {}.", key_name, config_key, plugin.name()));
                } else {
                    key_name = Some(config_key);
                }
            }
        }
        if let Some(key_name) = key_name {
            let plugin_config_map = config_map.remove(&key_name).unwrap();
            if let ConfigMapValue::HashMap(plugin_config_map) = plugin_config_map {
                plugin_maps.insert(plugin.name(), plugin_config_map);
            } else {
                return Err(format!("Expected the configuration property '{}' to be an object.", key_name));
            }
        }
    }
    Ok(plugin_maps)
}

fn get_global_config_from_config_map(config_map: ConfigMap) -> Result<HashMap<String, String>, String> {
    // at this point, there should only be string values inside the hash map
    let mut global_config = HashMap::new();

    for (key, value) in config_map.into_iter() {
        if let ConfigMapValue::String(value) = value {
            global_config.insert(key, value);
        } else {
            return Err(format!("Unexpected object property '{}'.", key));
        }
    }

    Ok(global_config)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::create_formatter;
    use super::super::environment::{TestEnvironment};
    use super::super::configuration::{ConfigMapValue, ConfigMap};

    #[test]
    fn it_should_get_formatter() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
        config_map.insert(String::from("typescript"), {
            let mut ts_config_map = HashMap::new();
            ts_config_map.insert(String::from("lineWidth"), String::from("40"));
            ConfigMapValue::HashMap(ts_config_map)
        });
        assert_creates(config_map);
    }

    #[test]
    fn it_should_error_when_has_double_plugin_config_keys() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
        config_map.insert(String::from("typescript"), {
            let mut map = HashMap::new();
            map.insert(String::from("lineWidth"), String::from("40"));
            ConfigMapValue::HashMap(map)
        });
        config_map.insert(String::from("javascript"), {
            let mut map = HashMap::new();
            map.insert(String::from("lineWidth"), String::from("40"));
            ConfigMapValue::HashMap(map)
        });
        assert_errors(
            config_map,
            vec![],
            "Error initializing from configuration file. Cannot specify both the 'typescript' and 'javascript' configurations for dprint-plugin-typescript."
        );
    }

    #[test]
    fn it_should_error_plugin_key_is_not_object() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("80")));
        config_map.insert(String::from("typescript"), ConfigMapValue::String(String::from("")));
        assert_errors(
            config_map,
            vec![],
            "Error initializing from configuration file. Expected the configuration property 'typescript' to be an object."
        );
    }

    #[test]
    fn it_should_log_global_diagnostics() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("lineWidth"), ConfigMapValue::String(String::from("null")));
        assert_errors(
            config_map,
            vec!["Error parsing configuration value for 'lineWidth'. Message: invalid digit found in string"],
            "Error initializing from configuration file. Had 1 diagnostic(s)."
        );
    }


    #[test]
    fn it_should_log_unexpected_object_properties() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("test"), ConfigMapValue::HashMap(HashMap::new()));
        assert_errors(
            config_map,
            vec![],
            "Error initializing from configuration file. Unexpected object property 'test'."
        );
    }

    #[test]
    fn it_should_log_plugin_diagnostics() {
        let mut config_map = HashMap::new();
        config_map.insert(String::from("typescript"), {
            let mut map = HashMap::new();
            map.insert(String::from("lineWidth"), String::from("null"));
            ConfigMapValue::HashMap(map)
        });
        assert_errors(
            config_map,
            vec!["[dprint-plugin-typescript]: Error parsing configuration value for 'lineWidth'. Message: invalid digit found in string"],
            "Error initializing from configuration file. Had 1 diagnostic(s)."
        );
    }

    fn assert_creates(config_map: ConfigMap) {
        let test_environment = TestEnvironment::new();
        assert_eq!(true, false); // fail... implement the below
        //assert_eq!(create_formatter(config_map, &test_environment).is_ok(), true);
    }

    fn assert_errors(config_map: ConfigMap, logged_errors: Vec<&'static str>, message: &str) {
        let test_environment = TestEnvironment::new();
        assert_eq!(true, false); // fail... implement the below
        /*let result = create_formatter(config_map, &test_environment);
        assert_eq!(result.err().unwrap(), message);
        assert_eq!(test_environment.get_logged_errors(), logged_errors);*/
    }
}
