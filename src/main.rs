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

#[derive(Debug, Default)]
pub struct LinePart {
    part: String,
    len: usize,
    tab_index: Option<usize>,
}

#[derive(Debug, Default, Clone)]
struct WatchStatus {
    viewed: bool,
    alternate_color: bool,
}

#[derive(Default)]
struct State {
    watch_tab_indexes: HashMap<usize, WatchStatus>,
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
        ]);
        subscribe(&[
            EventType::TabUpdate,
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
            Event::ModeUpdate(mode_info) => {
                if self.mode_info != mode_info {
                    should_render = true;
                }
                self.mode_info = mode_info
            }
            Event::Timer(_) => {
                let watch_tab_indexes = &self.watch_tab_indexes.clone();
                
                let mut fired_timer = false;

                for (tab_idx, watch_status) in watch_tab_indexes {
                    if !watch_status.viewed {
                        self.watch_tab_indexes.insert(*tab_idx, WatchStatus {
                            viewed: false,
                            alternate_color: !watch_status.alternate_color
                        });

                        if !fired_timer {
                            set_timeout(1.0);
                            should_render = true;
                            fired_timer = true;
                        }
                    }
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
                if let Some(payload) = pipe_message.payload {
                    let empty_map = self.watch_tab_indexes.is_empty();

                    self.watch_tab_indexes.insert(payload.parse().unwrap(), WatchStatus {
                        viewed: false,
                        alternate_color: true
                    }) ;
                    if empty_map {
                        set_timeout(1.0);
                        should_render = true;
                    }
                   
                }
            }
            // PipeSource::Plugin(source_plugin_id) => {
            //     // pipes can also arrive from other plugins
            // }
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

            if let Some(i) = self.watch_tab_indexes.get(&(&t.position + 1)) {
                alternate_color = i.alternate_color;

                if t.active {
                    self.watch_tab_indexes.remove(&(t.position + 1));
                }
            }

            let tab = tab_style(
                tabname,
                t,
                self.mode_info.style.colors,
                self.mode_info.capabilities,
                alternate_color
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
