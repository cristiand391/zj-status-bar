# ⚡ zj-status-bar ⚡

An opinionated status bar plugin for [`zellij`](https://zellij.dev/) based on the built-in zellij's compact-bar plugin.

![image](https://github.com/cristiand391/zj-status-bar/assets/6853656/4b9f6c60-820d-432e-9d9e-6a88f46c5824)

![image](https://github.com/cristiand391/zj-status-bar/assets/6853656/bca06f58-c367-4a51-bf8a-8d6c5dd0f928)

(mostly) no config, compact-bar experience + some goodies :)

### Features
* Pane fullscreen indicator
* Tab indexes

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

## Development

1. Clone repo: `gh repo clone cristiand391/zj-status-bar`
2. Build (debug/dev build): `cargo build` should get you a wasm build at `target/wasm32-wasi/debug/zj-status-bar.wasm`
3. Load it from a layout
