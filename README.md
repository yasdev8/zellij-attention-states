# zellij-attention

Know which Zellij tab needs your attention — without checking each one.

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

A standalone Zellij WASM plugin that adds notification icons directly to tab names. Works with both the default Zellij tab bar and [zjstatus](https://github.com/dj95/zjstatus). When an external process (like Claude Code) reports its state, the tab is renamed with an indicator — e.g., `terminal` becomes `terminal ⏳` while it works, `terminal 🟠` when it needs input and `terminal ✅` when it is done. The icon follows the state sent by the process, and a `clear` message removes it.

https://github.com/user-attachments/assets/646effc0-1c24-413d-bef3-3d85591cd89b

## Features

- **Tab-level notifications** — icons appended to tab names, visible at a glance
- **Three notification states** — 🟠 waiting (needs input), ⏳ executing (working) and ✅ completed (task done)
- **Explicit clear** — a `clear` message removes the icon; auto-clear on focus is available as an option (`clear_on_focus`)
- **Memory-only state** — lightweight, no disk I/O; stale icons cleaned up automatically on restart
- **Configurable icons** — use any character or emoji as notification indicator
- **Standalone plugin** — works independently, no zjstatus or other status bar plugins needed

## Installation

Download the plugin and add it to your Zellij config:

```bash
mkdir -p ~/.config/zellij/plugins
curl -L https://github.com/KiryuuLight/zellij-attention/releases/latest/download/zellij-attention.wasm \
  -o ~/.config/zellij/plugins/zellij-attention.wasm
```

Add to `~/.config/zellij/config.kdl`:

```kdl
load_plugins {
    "file:~/.config/zellij/plugins/zellij-attention.wasm" {
        // All options are optional — defaults shown
        enabled "true"
        waiting_icon "🟠"
        executing_icon "⏳"
        completed_icon "✅"
        clear_on_focus "false"
    }
}
```

The plugin loads in the background with no visible pane — it won't consume any screen space.

## Quick Start

After installing, restart Zellij and test with a pipe command:

```bash
# Send a waiting notification to the current pane
zellij pipe --name "zellij-attention::waiting::$ZELLIJ_PANE_ID"

# Send an executing notification
zellij pipe --name "zellij-attention::executing::$ZELLIJ_PANE_ID"

# Send a completed notification
zellij pipe --name "zellij-attention::completed::$ZELLIJ_PANE_ID"

# Remove the notification
zellij pipe --name "zellij-attention::clear::$ZELLIJ_PANE_ID"
```

Switch to the tab — the icon should appear. Send `clear` to remove it.

## Claude Code Integration

Automate notifications with [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code). Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::clear::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::executing::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::executing::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::waiting::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::completed::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "[ -n \"$ZELLIJ\" ] && zellij pipe --name \"zellij-attention::clear::$ZELLIJ_PANE_ID\" 2>/dev/null; true"
          }
        ]
      }
    ]
  }
}
```

| Hook                                    | Notification | Meaning                                   |
| --------------------------------------- | ------------ | ----------------------------------------- |
| `UserPromptSubmit`, `PostToolUse`       | ⏳ executing | Claude is working                         |
| `Notification` (`permission_prompt`)    | 🟠 waiting   | Claude needs a permission answer          |
| `Stop`                                  | ✅ completed | Claude finished the task                  |
| `SessionStart`, `SessionEnd`            | clear        | No icon when a session starts or ends     |

`PostToolUse` switches 🟠 back to ⏳ once the permission is answered. The `[ -n "$ZELLIJ" ]` guard and the trailing `true` keep the hooks silent outside Zellij.

If you already have hooks for these events, add these entries next to them instead of replacing them.

## Configuration

All configuration is optional — the plugin works out of the box.

| Option           | Default   | Description                                     |
| ---------------- | --------- | ----------------------------------------------- |
| `enabled`        | `"true"`  | Enable or disable notifications                 |
| `waiting_icon`   | `"🟠"`    | Icon for waiting state (needs input)            |
| `executing_icon` | `"⏳"`    | Icon for executing state (working)              |
| `completed_icon` | `"✅"`    | Icon for completed state                        |
| `clear_on_focus` | `"false"` | Clear a pane's notification when it gets focus  |

When several panes of a tab have a notification, the tab shows 🟠 first, then ⏳, then ✅.

Icons are appended to the end of tab names (e.g., `terminal ⏳`).

## Pipe Message Format

```
zellij-attention::EVENT_TYPE::PANE_ID
```

- `EVENT_TYPE` — `waiting`, `executing` (alias `running`), `completed` or `clear` (case-insensitive)
- `PANE_ID` — numeric pane ID from `$ZELLIJ_PANE_ID`

> **Important:** Always use `--name` (broadcast pipe), never `--plugin` (targeted). Targeted pipes create new plugin instances instead of reaching existing ones.

## Shell Functions

For manual testing or integration with other tools:

```bash
notify-waiting() {
    [ -z "$ZELLIJ_PANE_ID" ] && echo "Not in Zellij" && return 1
    zellij pipe --name "zellij-attention::waiting::$ZELLIJ_PANE_ID"
}

notify-completed() {
    [ -z "$ZELLIJ_PANE_ID" ] && echo "Not in Zellij" && return 1
    zellij pipe --name "zellij-attention::completed::$ZELLIJ_PANE_ID"
}
```

## Development

```bash
# Build
cargo build --target wasm32-wasip1 --release

# Install
cp target/wasm32-wasip1/release/zellij-attention.wasm ~/.config/zellij/plugins/

# Debug build (enables verbose logging)
cargo build --target wasm32-wasip1
tail -f /tmp/zellij-*/zellij-log-*/zellij.log | grep "zellij-attention"
```

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.

## License

MIT
