use core::slice::{Iter};

use dprint_core::plugins::{Plugin};
use libloading as lib;

pub struct PluginContainer {
    plugins: Vec<Box<dyn Plugin>>,
    libraries: Vec<lib::Library>,
}

impl PluginContainer {
    pub fn new() -> PluginContainer {
        let ts_lib = lib::Library::new(r#"V:/dprint/crates/dprint-plugin-typescript/target/debug/dprint_plugin_typescript.dll"#).unwrap();
        let ts_plugin = unsafe {
            let create: lib::Symbol<unsafe extern fn() -> *mut dyn Plugin> = ts_lib.get(b"create_plugin").unwrap();
            Box::from_raw(create())
        } as Box<dyn Plugin>;

        PluginContainer {
            plugins: vec![
                //Box::new(dprint_plugin_typescript::TypeScriptPlugin::new()),
                ts_plugin,
                Box::new(dprint_plugin_jsonc::JsoncPlugin::new())
            ],
            libraries: vec![ts_lib]
        }
    }

    /// Iterates over the plugins.
    pub fn iter_plugins(&self) -> Iter<'_, Box<dyn Plugin>> {
        self.plugins.iter()
    }
}

impl Drop for PluginContainer {
    fn drop(&mut self) {
        for plugin in self.plugins.drain(..) {
            plugin.dispose();
            drop(plugin)
        }

        for lib in self.libraries.drain(..) {
            drop(lib);
        }
    }
}
