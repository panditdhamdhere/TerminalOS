# Keyboard Shortcuts

Global shortcuts are configurable in `config.toml` or `keybindings.toml`. Defaults:

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+/` | Toggle AI chat |
| `Ctrl+`` | Toggle logs |
| `Ctrl+1/2/3/4` | Focus terminal/chat/sidebar/logs |
| `Tab` | Cycle focus |
| `Ctrl+←/→` | Resize sidebar |
| `Ctrl+↑/↓` | Resize chat panel |
| `Ctrl+Shift+↑/↓` | Resize logs panel |
| `Ctrl+Shift+C` | Copy terminal |
| `Ctrl+Shift+V` | Paste to terminal |
| `Ctrl+Shift+F` | Search in terminal |
| `Ctrl+Q` | Quit |

Pane-specific keys (terminal scroll, chat input, confirmation prompts) are handled by the focused pane and are not part of the global keybinding map.

## Customizing bindings

Override a binding in `config.toml`:

```toml
[keybindings]
quit = "Ctrl+Q"
new_tab = "Ctrl+T"
```

Or create a dedicated `keybindings.toml` file in the config directory for a full override.

Binding strings use modifiers (`Ctrl`, `Shift`, `Alt`) plus key names (`Tab`, `Left`, `Q`, `` ` ``).
