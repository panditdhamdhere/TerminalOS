use terminalos_config::{AppConfig, Keybindings, binding_map, builtin_preset_names, resolve_theme};
use terminalos_shared::ThemeMode;

#[test]
fn snapshot_default_config_toml() {
    let config = AppConfig::default();
    let toml = toml::to_string_pretty(&config).expect("serialize config");
    insta::assert_snapshot!("default_config_toml", toml);
}

#[test]
fn snapshot_default_keybindings_toml() {
    let bindings = Keybindings::default();
    let toml = toml::to_string_pretty(&bindings).expect("serialize keybindings");
    insta::assert_snapshot!("default_keybindings_toml", toml);
}

#[test]
fn snapshot_dracula_theme() {
    let theme = resolve_theme(ThemeMode::Dark, Some("dracula"));
    insta::assert_snapshot!(
        "dracula_theme",
        format!(
            "name={}\nmode={:?}\nbackground={}\nforeground={}\naccent={}",
            theme.name, theme.mode, theme.background, theme.foreground, theme.accent
        )
    );
}

#[test]
fn snapshot_binding_map() {
    let bindings = Keybindings::default();
    let mut lines: Vec<String> = binding_map(&bindings)
        .into_iter()
        .map(|(action, combo)| format!("{action}={combo}"))
        .collect();
    lines.sort();
    insta::assert_snapshot!("binding_map", lines.join("\n"));
}

#[test]
fn snapshot_builtin_theme_names() {
    insta::assert_snapshot!("builtin_theme_names", builtin_preset_names().join("\n"));
}
