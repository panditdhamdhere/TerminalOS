use std::ffi::CStr;
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};
use terminalos_shared::Result;

/// Stable plugin API version for dynamic libraries.
pub const PLUGIN_API_VERSION: u32 = 1;

/// Symbol exported by plugin dynamic libraries.
pub const PLUGIN_ENTRY_SYMBOL: &[u8] = b"terminalos_plugin_entry\0";

/// Plugin initialization/execution success code.
pub const PLUGIN_SUCCESS: i32 = 0;

/// Plugin initialization/execution failure code.
pub const PLUGIN_ERROR: i32 = -1;

/// Metadata describing an installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// A command exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
}

/// Plugin manifest parsed from plugin.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub info: PluginInfo,
    pub entry: String,
    pub enabled: bool,
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
}

/// C-compatible plugin metadata for dynamic libraries.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginDescriptor {
    pub api_version: u32,
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub author: *const c_char,
}

pub type PluginInitFn = unsafe extern "C" fn() -> i32;
pub type PluginExecuteFn = unsafe extern "C" fn(
    command: *const c_char,
    args_json: *const c_char,
    out: *mut u8,
    out_cap: usize,
) -> i32;
pub type PluginShutdownFn = unsafe extern "C" fn();

/// Function table exported by dynamic plugin libraries.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginExports {
    pub descriptor: PluginDescriptor,
    pub init: PluginInitFn,
    pub execute: PluginExecuteFn,
    pub shutdown: PluginShutdownFn,
}

pub type PluginEntryFn = unsafe extern "C" fn() -> PluginExports;

/// Trait implemented by in-process TerminalOS plugins.
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;

    fn on_load(&mut self) -> Result<()>;

    fn on_unload(&mut self) -> Result<()>;

    fn execute(&self, command: &str, args: &[String]) -> Result<String>;
}

/// Reads a plugin descriptor from raw C pointers into safe Rust types.
///
/// # Safety
/// All pointer fields must reference valid null-terminated UTF-8 strings.
pub unsafe fn descriptor_from_raw(raw: &PluginDescriptor) -> Result<PluginInfo> {
    if raw.api_version != PLUGIN_API_VERSION {
        return Err(terminalos_shared::Error::Plugin(format!(
            "unsupported plugin API version {}",
            raw.api_version
        )));
    }

    Ok(PluginInfo {
        name: read_cstr(raw.name)?,
        version: read_cstr(raw.version)?,
        description: read_cstr(raw.description)?,
        author: read_cstr(raw.author)?,
    })
}

fn read_cstr(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        return Err(terminalos_shared::Error::Plugin(
            "plugin descriptor contained null string".to_string(),
        ));
    }
    let value = unsafe { CStr::from_ptr(ptr) };
    value
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| terminalos_shared::Error::Plugin(format!("invalid plugin string: {e}")))
}

/// Exports a dynamic plugin library entry point.
#[macro_export]
macro_rules! export_plugin {
    ($name:expr, $version:expr, $description:expr, $author:expr, $init:path, $execute:path, $shutdown:path) => {
        const PLUGIN_NAME: &str = $name;
        const PLUGIN_VERSION: &str = $version;
        const PLUGIN_DESCRIPTION: &str = $description;
        const PLUGIN_AUTHOR: &str = $author;

        #[unsafe(no_mangle)]
        pub extern "C" fn terminalos_plugin_entry() -> $crate::PluginExports {
            $crate::PluginExports {
                descriptor: $crate::PluginDescriptor {
                    api_version: $crate::PLUGIN_API_VERSION,
                    name: PLUGIN_NAME.as_ptr().cast(),
                    version: PLUGIN_VERSION.as_ptr().cast(),
                    description: PLUGIN_DESCRIPTION.as_ptr().cast(),
                    author: PLUGIN_AUTHOR.as_ptr().cast(),
                },
                init: $init,
                execute: $execute,
                shutdown: $shutdown,
            }
        }
    };
}

/// Helper for plugin authors to write command output into a host buffer.
///
/// # Safety
/// `out` must point to a valid buffer of at least `out_cap` bytes.
pub unsafe fn write_plugin_output(out: *mut u8, out_cap: usize, message: &str) -> i32 {
    if out.is_null() || out_cap == 0 {
        return PLUGIN_ERROR;
    }

    let bytes = message.as_bytes();
    let len = bytes.len().min(out_cap.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, len);
        *out.add(len) = 0;
    }
    PLUGIN_SUCCESS
}

/// Helper for plugin authors to parse JSON args from the host.
///
/// # Safety
/// `args_json` must be null or point to a valid null-terminated UTF-8 string.
pub unsafe fn parse_plugin_args(args_json: *const c_char) -> Vec<String> {
    if args_json.is_null() {
        return Vec::new();
    }

    let Ok(json) = unsafe { CStr::from_ptr(args_json) }.to_str() else {
        return Vec::new();
    };

    if json.is_empty() {
        return Vec::new();
    }

    serde_json::from_str::<Vec<String>>(json).unwrap_or_default()
}

/// Helper for plugin authors to read a command name from the host.
///
/// # Safety
/// `command` must be null or point to a valid null-terminated UTF-8 string.
pub unsafe fn read_plugin_command(command: *const c_char) -> Option<String> {
    if command.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(command) }
        .to_str()
        .ok()
        .map(str::to_string)
}
