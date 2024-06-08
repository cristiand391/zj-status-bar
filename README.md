# ⚡ zj-status-bar ⚡

An opinionated status bar plugin for [`zellij`](https://zellij.dev/) based on the built-in zellij's compact-bar plugin.

![image](https://github.com/cristiand391/zj-status-bar/assets/6853656/4b9f6c60-820d-432e-9d9e-6a88f46c5824)

![image](https://github.com/cristiand391/zj-status-bar/assets/6853656/bca06f58-c367-4a51-bf8a-8d6c5dd0f928)

(mostly) no config, compact-bar experience + some goodies :)

### Features
* Pane fullscreen indicator
* Tab indexes
* [Tab alerts](#tab-alerts)

### Others
* No alternate tab colors
* Inactive tabs are rendered in italics
* `Zellij` string removed from the top-left corner


> [!NOTE]
> The screenshots above are with `simplified_ui`  enabled/disabled, this config option is part of the original compact-bar plugin and is still supported, other than that there's no configuration options.
> 
> https://zellij.dev/documentation/options#simplified_ui

## Usage

Download the last release available at https://github.com/cristiand391/zj-status-bar/releases/ and load it from a layout.
**Example:**

```
layout {
  pane size=1 {
    plugin location="file:/path/to/zj-status-bar.wasm"
  }
  pane
}
```

On the first load you need to navigate to the plugin pane and press `y` to accept the perms required for the plugin to work.
![image](https://github.com/cristiand391/zj-status-bar/assets/6853656/7edc6b33-a0ed-434c-a9b2-84881dd1d503)

> [!NOTE]  
> If you start the plugin pane with [`borderless`](https://zellij.dev/documentation/creating-a-layout#borderless) set to true you won't be able to view it and accept the perms.
> After accepting permissions you can disable borders again.

## Tab alerts

Always keep an eye on long-running processes, even if they are on different tabs!

[tab-alerts-demo.webm](https://github.com/cristiand391/zj-status-bar/assets/6853656/953d7abf-3011-48d4-ad38-f45c96c3583a)

When running commands via `zw` you'll get a green/red alert (based on the exit code > 0) on the tab section when you are on a different tab.
The alerts are rendered every 1s and are cleared once you focus on that tab.

Add the `zw` helper to your shell setup:

zsh/bash:
```zsh
zw() {
  eval "$*"
  zellij pipe --name zj-status-bar:cli:tab_alert --args "pane_id=$ZELLIJ_PANE_ID,exit_code=$?"
}
```

then pass it the command you want to watch
`zw cargo build`

> [!NOTE]  
> If you want to chain multiple commands make sure to wrap them in quotes (e.g `zw 'sleep 3 && cargo build'`), otherwise your shell will interpret it as 2 different commands and you'll only get an alert about the first one.

## Development

1. Clone repo: `gh repo clone cristiand391/zj-status-bar`
2. Build (debug/dev build): `cargo build` should get you a wasm build at `target/wasm32-wasi/debug/zj-status-bar.wasm`
3. Load it from a layout
