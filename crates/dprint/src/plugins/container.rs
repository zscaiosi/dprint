use core::slice::{Iter};

use dprint_core::plugins::{Plugin};

// todo: create a PluginContainer trait and have this as RealPluginContainer

pub struct PluginContainer {
    plugins: Vec<Box<dyn Plugin>>,
    //libraries: Vec<lib::Library>,
}

impl PluginContainer {
    pub fn new() -> PluginContainer {
        PluginContainer { plugins: Vec::new()/*, libraries: Vec::new()*/ }
    }

    /*
    pub fn add(&mut self, plugin: Box<dyn Plugin>, library: lib::Library) {
        self.plugins.push(plugin);
        self.libraries.push(library);
    }
    */

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

        /*
        for lib in self.libraries.drain(..) {
            drop(lib);
        }
        */
    }
}
