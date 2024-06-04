mod line;
mod tab;

use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryInto;

use tab::get_tab_to_focus;
use zellij_tile::prelude::*;

use crate::line::tab_line;
use crate::tab::tab_style;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct LinePart {
    part: String,
    len: usize,
    tab_index: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct TabAlert {
    success: bool,
    alternate_color: bool,
}

#[derive(Default)]
struct State {
    pane_info: PaneManifest,
    tab_alerts: HashMap<usize, TabAlert>,
    tabs: Vec<TabInfo>,
    active_tab_idx: usize,
    mode_info: ModeInfo,
    tab_line: Vec<LinePart>,
}

static ARROW_SEPARATOR: &str = "";

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::ModeUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
            EventType::Timer,
        ]);
        // Set as selectable on load so user can accept/deny perms.
        // After the first load, if the user allowed access, the perm event handler
        // in `update` will always set it as unselectable.
        set_selectable(true);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::PaneUpdate(pane_info) => {
                self.pane_info = pane_info;
            }
            Event::ModeUpdate(mode_info) => {
                if self.mode_info != mode_info {
                    should_render = true;
                }
                self.mode_info = mode_info
            }
            Event::Timer(_) => {
                // This timer is fired in the `pipe` lifecycle method and it's guaranteed there
                // will be at least one alert.
                let tab_alert_indexes = &self.tab_alerts.clone();

                for (tab_idx, tab_alert) in tab_alert_indexes {
                    self.tab_alerts.insert(
                        *tab_idx,
                        TabAlert {
                            success: tab_alert.success,
                            alternate_color: !tab_alert.alternate_color,
                        },
                    );
                }

                set_timeout(1.0);
                should_render = true;

                // Broadcast the state of tab alerts to all instances of `zj-status-bar` for new
                // instances to "catch up" on previous alerts.
                //
                // Only broadcast state if there's something to share.
                // There's a scenario where after visiting the last tab with an alert you end up
                // with an empty state before the last `Timer` event is fired, broadcasting it
                // would cause an infinite loop.
                if !self.tab_alerts.is_empty() {
                    pipe_message_to_plugin(
                        MessageToPlugin::new("zj-status-bar:process_status:broadcast")
                            .with_plugin_url("zellij:OWN_URL")
                            .with_payload(serde_json::to_string(&self.tab_alerts).unwrap()),
                    )
                }
            }
            Event::TabUpdate(tabs) => {
                if let Some(active_tab_index) = tabs.iter().position(|t| t.active) {
                    // tabs are indexed starting from 1 so we need to add 1
                    let active_tab_idx = active_tab_index + 1;
                    if self.active_tab_idx != active_tab_idx || self.tabs != tabs {
                        should_render = true;
                    }
                    self.active_tab_idx = active_tab_idx;
                    self.tabs = tabs;
                } else {
                    eprintln!("Could not find active tab.");
                }
            }
            Event::Mouse(me) => match me {
                Mouse::LeftClick(_, col) => {
                    let tab_to_focus = get_tab_to_focus(&self.tab_line, self.active_tab_idx, col);
                    if let Some(idx) = tab_to_focus {
                        switch_tab_to(idx.try_into().unwrap());
                    }
                }
                Mouse::ScrollUp(_) => {
                    switch_tab_to(min(self.active_tab_idx + 1, self.tabs.len()) as u32);
                }
                Mouse::ScrollDown(_) => {
                    switch_tab_to(max(self.active_tab_idx.saturating_sub(1), 1) as u32);
                }
                _ => {}
            },
            Event::PermissionRequestResult(result) => match result {
                PermissionStatus::Granted => set_selectable(false),
                PermissionStatus::Denied => eprintln!("Permission denied by user."),
            },
            _ => {
                eprintln!("Got unrecognized event: {:?}", event);
            }
        };
        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        match pipe_message.source {
            PipeSource::Cli(_) => {
                if pipe_message.name == "zj-status-bar:process_status" {
                    if let (Some(pane_id_str), Some(exit_code_str)) = (
                        pipe_message.args.get("pane_id"),
                        pipe_message.args.get("exit_code"),
                    ) {
                        let pane_id: u32 = pane_id_str.parse().unwrap();
                        let exit_code: u32 = exit_code_str.parse().unwrap();

                        for (tab_idx, pane_vec) in &mut self.pane_info.panes {
                            // skip panes in current tab
                            if *tab_idx == self.active_tab_idx - 1 {
                                continue;
                            }

                            // find tab position containing the pane by its id.
                            if pane_vec.iter().find(|p| p.id == pane_id).is_some() {
                                let first_alert = self.tab_alerts.is_empty();

                                self.tab_alerts.insert(
                                    *tab_idx,
                                    TabAlert {
                                        success: exit_code == 0,
                                        alternate_color: true,
                                    },
                                );

                                // Only fire timer/re-render on the first alert, when the 1st timer
                                // expires the state is updated there and new timer is set.
                                if first_alert {
                                    set_timeout(1.0);
                                    should_render = true;
                                }
                            }
                        }
                    }
                }
            }
            PipeSource::Plugin(_source_plugin_id) => {
                // This message is sent by other plugin instances on each `Timer` event and
                // contains the state of tabs alerts.
                //
                // Only read it if the current instance doesn't contain any info (new tab created
                // after alerts were piped from a pane) to "catch up" and render them.
                if pipe_message.is_private
                    && pipe_message.name == "zj-status-bar:process_status:broadcast"
                    && self.tab_alerts.is_empty()
                {
                    self.tab_alerts = serde_json::from_str(&pipe_message.payload.unwrap()).unwrap();

                    // fire 1st timer/re-render
                    set_timeout(1.0);
                    should_render = true;
                }
            }
            _ => {
                should_render = false;
            }
        }
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if self.tabs.is_empty() {
            return;
        }
        let mut all_tabs: Vec<LinePart> = vec![];
        let mut active_tab_index = 0;
        let mut active_swap_layout_name = None;
        let mut is_swap_layout_dirty = false;
        for t in &mut self.tabs {
            let mut tabname = t.name.clone();
            if t.active && self.mode_info.mode == InputMode::RenameTab {
                if tabname.is_empty() {
                    tabname = String::from("Enter name...");
                }
                active_tab_index = t.position;
            } else if t.active {
                active_tab_index = t.position;
                is_swap_layout_dirty = t.is_swap_layout_dirty;
                active_swap_layout_name = t.active_swap_layout_name.clone();
            }

            // insert tab index
            tabname.insert_str(0, &format!("{} ", t.position + 1));

            let mut alternate_color = false;
            let mut success = false;

            if let Some(i) = self.tab_alerts.get(&t.position) {
                alternate_color = i.alternate_color;
                success = i.success;

                if t.active {
                    self.tab_alerts.remove(&t.position);
                }
            }

            let tab = tab_style(
                tabname,
                t,
                self.mode_info.style.colors,
                self.mode_info.capabilities,
                alternate_color,
                success,
            );
            all_tabs.push(tab);
        }
        self.tab_line = tab_line(
            self.mode_info.session_name.as_deref(),
            all_tabs,
            active_tab_index,
            cols.saturating_sub(1),
            self.mode_info.style.colors,
            self.mode_info.capabilities,
            self.mode_info.style.hide_session_name,
            self.mode_info.mode,
            &active_swap_layout_name,
            is_swap_layout_dirty,
        );
        let output = self
            .tab_line
            .iter()
            .fold(String::new(), |output, part| output + &part.part);
        let background = match self.mode_info.style.colors.theme_hue {
            ThemeHue::Dark => self.mode_info.style.colors.black,
            ThemeHue::Light => self.mode_info.style.colors.white,
        };
        match background {
            PaletteColor::Rgb((r, g, b)) => {
                print!("{}\u{1b}[48;2;{};{};{}m\u{1b}[0K", output, r, g, b);
            }
            PaletteColor::EightBit(color) => {
                print!("{}\u{1b}[48;5;{}m\u{1b}[0K", output, color);
            }
        }
    }
}
