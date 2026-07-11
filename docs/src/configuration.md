# Configuration

TerminalOS stores configuration in `~/.config/terminalos/`.

## Files

| File | Purpose |
|------|---------|
| `config.toml` | Providers, workspace, active profile, keybindings |
| `keybindings.toml` | Optional global shortcut overrides |
| `profiles/*.toml` | Named profiles with partial UI/layout overrides |

## Profiles

Bundled profiles:

- **default** — balanced layout for everyday development
- **minimal** — distraction-free terminal with hidden side panels
- **coding** — wider terminal area with adjusted pane percentages

Switch profiles from the CLI:

```bash
cargo run -p terminalos-cli -- config profile use minimal
```

Or launch the terminal with a one-off profile:

```bash
cargo run -p terminalos -- --profile coding
```

## Themes

Set the base mode under `[ui]`:

```toml
[ui]
theme = "dark"
```

Apply a named color preset:

```toml
[ui]
theme = "dark"
theme_preset = "dracula"
```

Built-in presets: `dark`, `light`, `dracula`, `nord`, `solarized-dark`.

## Providers

Each provider entry in `config.toml` defines a backend, API key environment variable, model, and enabled flag. Set `default_provider` to choose the active AI backend.

## Workspace

The `[workspace]` section controls session restore and autosave intervals. Workspace snapshots are stored in SQLite alongside chat memory.
