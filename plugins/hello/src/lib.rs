//! Example dynamic plugin for TerminalOS.
#![allow(unsafe_code)]

use std::os::raw::c_char;

use terminalos_plugin::{
    PLUGIN_ERROR, PLUGIN_SUCCESS, export_plugin, parse_plugin_args, read_plugin_command,
    write_plugin_output,
};

extern "C" fn init() -> i32 {
    PLUGIN_SUCCESS
}

extern "C" fn shutdown() {}

extern "C" fn execute(
    command: *const c_char,
    args_json: *const c_char,
    out: *mut u8,
    out_cap: usize,
) -> i32 {
    let Some(command) = (unsafe { read_plugin_command(command) }) else {
        return PLUGIN_ERROR;
    };
    let args = unsafe { parse_plugin_args(args_json) };

    let response = match command.as_str() {
        "greet" => {
            let name = args.first().map(String::as_str).unwrap_or("developer");
            format!("Hello, {name}! — from TerminalOS hello plugin")
        }
        "version" => "hello plugin v0.1.0".to_string(),
        _ => return PLUGIN_ERROR,
    };

    unsafe { write_plugin_output(out, out_cap, &response) }
}

export_plugin!(
    "hello",
    "0.1.0",
    "Greets the user with a customizable message",
    "TerminalOS",
    init,
    execute,
    shutdown
);
