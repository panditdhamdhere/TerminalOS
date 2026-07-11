use std::ffi::CString;
use std::path::{Path, PathBuf};

use libloading::Library;
use terminalos_shared::{Error, Result};

use crate::api::{
    PLUGIN_ENTRY_SYMBOL, PLUGIN_SUCCESS, PluginEntryFn, PluginExports, PluginInfo,
    descriptor_from_raw,
};

/// A dynamically loaded plugin library.
pub struct DynamicPlugin {
    _library: Library,
    exports: PluginExports,
    info: PluginInfo,
    path: PathBuf,
}

impl DynamicPlugin {
    pub fn load(library_path: impl AsRef<Path>) -> Result<Self> {
        let path = library_path.as_ref();
        let library = unsafe {
            Library::new(path)
                .map_err(|e| Error::Plugin(format!("load library {}: {e}", path.display())))?
        };

        let entry: libloading::Symbol<PluginEntryFn> = unsafe {
            library.get(PLUGIN_ENTRY_SYMBOL).map_err(|e| {
                Error::Plugin(format!("missing plugin entry in {}: {e}", path.display()))
            })?
        };

        let exports = unsafe { entry() };
        let info = unsafe { descriptor_from_raw(&exports.descriptor)? };

        let status = unsafe { (exports.init)() };
        if status != PLUGIN_SUCCESS {
            return Err(Error::Plugin(format!(
                "plugin {} failed to initialize",
                info.name
            )));
        }

        Ok(Self {
            _library: library,
            exports,
            info,
            path: path.to_path_buf(),
        })
    }

    #[must_use]
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn execute(&self, command: &str, args: &[String]) -> Result<String> {
        let command_name = command.to_string();
        let command = CString::new(command_name.as_str())
            .map_err(|e| Error::Plugin(format!("invalid command name: {e}")))?;
        let args_json = serde_json::to_string(args)
            .map_err(|e| Error::Plugin(format!("serialize plugin args: {e}")))?;
        let args_json = CString::new(args_json)
            .map_err(|e| Error::Plugin(format!("invalid args json: {e}")))?;

        let mut buffer = vec![0_u8; 16_384];
        let status = unsafe {
            (self.exports.execute)(
                command.as_ptr(),
                args_json.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };

        if status != PLUGIN_SUCCESS {
            return Err(Error::Plugin(format!(
                "plugin {} command `{command_name}` failed",
                self.info.name
            )));
        }

        let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        let output = String::from_utf8_lossy(&buffer[..nul]).to_string();
        Ok(output)
    }
}

impl Drop for DynamicPlugin {
    fn drop(&mut self) {
        unsafe {
            (self.exports.shutdown)();
        }
    }
}

/// Resolves the platform-specific dynamic library filename for a plugin entry.
#[must_use]
pub fn library_filename(entry: &str) -> String {
    let base = if entry.starts_with("lib") {
        entry.to_string()
    } else {
        format!("lib{entry}")
    };

    if cfg!(target_os = "windows") {
        format!("{base}.dll")
    } else if cfg!(target_os = "macos") {
        format!("{base}.dylib")
    } else {
        format!("{base}.so")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_filename_matches_platform() {
        let name = library_filename("terminalos_plugin_hello");
        #[cfg(target_os = "macos")]
        assert!(name.ends_with(".dylib"));
        #[cfg(target_os = "linux")]
        assert!(name.ends_with(".so"));
        #[cfg(target_os = "windows")]
        assert!(name.ends_with(".dll"));
    }
}
