use std::collections::BTreeMap;

use egui::{vec2, Grid, ScrollArea, Window};
use egui_extras::RetainedImage;

use crate::{
    helix,
    state::{AppState, BorrowedPersistState},
    widgets::{MainView, MainViewView},
    TwitchImage, SETTINGS_KEY,
};

pub struct App {
    context: egui::Context,
    pub app: AppState,
}

impl App {
    pub fn new(context: egui::Context, state: AppState) -> Self {
        Self {
            context,
            app: state,
        }
    }

    const fn is_connected(&self) -> bool {
        self.app.twitch.is_some()
    }

    fn toggle_tab_bar(&mut self) {
        self.app.state.chat_view_state.toggle_tab_bar();
    }

    fn toggle_user_list(&mut self) {
        let cvs = &mut self.app.state.chat_view_state;
        let active = cvs.active();
        if let Some((visible, _)) = cvs.chatters_mut(active).as_mut() {
            *visible = !*visible;
        }
    }

    fn toggle_line_mode(&mut self) {
        // self.app.tabs.active_mut().next_line_mode()
    }

    fn toggle_timestamps(&mut self) {
        // self.app.tabs.active_mut().toggle_timestamps()
    }

    fn next_tab(&mut self) {
        self.app.state.chat_view_state.next();
    }

    fn previous_tab(&mut self) {
        self.app.state.chat_view_state.previous();
    }

    fn try_set_active_tab(&mut self, index: usize) {
        self.app.state.chat_view_state.set_active(index)
    }

    fn switch_to_settings(&mut self) {
        self.app
            .state
            .view_state
            .switch_to_view(MainViewView::Settings)
    }

    fn switch_to_main(&mut self) {
        let vs = &mut self.app.state.view_state;
        vs.switch_to_view(vs.previous_view)
    }

    fn try_fetch_chatters(&mut self) {
        // for (room_id, name) in self.app.state.chat_view_state.channels() {
        //     if let Ok(chatters) = helix::Client::get_chatters_for(name) {
        //         if let Some((_, ch)) = self.app.state.chat_view_state.chatters_mut(room_id) {
        //             *ch = chatters
        //         }
        //     }
        // }
    }

    fn try_fetch_badges(&mut self) {
        const DESIRED_USER_LIST_BADGES: [&str; 8] = [
            "broadcaster",
            "vip",
            "moderator",
            "staff",
            "admin",
            "global_mod",
            "no_audio",
            "no_video",
        ];

        // if we have all of the desired badges already, do nothing
        if DESIRED_USER_LIST_BADGES
            .into_iter()
            .fold(true, |ok, key| ok & self.app.state.images.has(key))
        {
            return;
        }

        let badges = match self.app.runtime.global_badges.ready() {
            Some(badges) => badges,
            None => {
                let helix = match self.app.runtime.helix.ready() {
                    Some(helix) => helix,
                    None => return,
                };

                let _ = self.app.runtime.helix_ready.send(helix.clone());
                return;
            }
        };

        for (set_id, (id, url)) in badges.iter().flat_map(|badge| {
            std::iter::repeat(&badge.set_id)
                .zip(badge.versions.iter().map(|v| (&v.id, &v.image_url_4x)))
        }) {
            if !DESIRED_USER_LIST_BADGES.contains(&&**set_id) {
                continue;
            }

            let badge = TwitchImage::badge(id, &set_id, url);
            if self.app.state.images.has_id(badge.id()) {
                continue;
            }

            if !self.app.state.requested_images.insert(badge.id()) {
                continue;
            }

            self.app.runtime.fetch.fetch(badge);
        }
    }

    fn try_fetch_image(&mut self) {
        let (image, data) = match self.app.runtime.fetch.try_next() {
            Some((image, data)) => (image, data),
            _ => return,
        };

        let images = &mut self.app.state.images;
        if images.has_id(image.id()) {
            return;
        }

        match RetainedImage::from_image_bytes(image.name(), &data) {
            Ok(data) => {
                images.add(image.name(), image.id(), data);
                let _ = self.app.state.requested_images.remove(&image.id());
            }
            Err(err) => {
                eprintln!("cannot create ({}) {} : {err}", image.id(), image.name())
            }
        }
    }

    fn try_poll_twitch(&mut self) {
        let twitch = match &self.app.twitch {
            Some(twitch) => twitch,
            _ => return,
        };

        self.app
            .interaction
            .poll(twitch)
            .expect("FIXME: this should reset the state"); // XXX: what does this mean?
    }

    fn try_read_message(&mut self) {
        let msg = match self.app.interaction.try_read() {
            Some(item) => item,
            _ => return,
        };

        if let Some(pm) = msg.as_privmsg() {
            pm.update_emote_map(&mut self.app.state.emote_map);

            let (id, spans) = pm.make_spans();
            self.app.state.spanned_lines.insert(id, spans); // TODO this should be bounded

            for (emote, _) in pm.emotes() {
                if self.app.state.images.has(emote) {
                    continue;
                }

                let name = match self.app.state.emote_map.get(emote) {
                    Some(name) => name,
                    None => {
                        eprintln!("emote missing: {emote}");
                        continue;
                    }
                };

                // TODO also fetch the light one
                let url =
                    format!("https://static-cdn.jtvnw.net/emoticons/v2/{emote}/static/dark/3.0");

                let emote = TwitchImage::emote(&emote, &name, url);
                self.app.runtime.fetch.fetch(emote)
            }
        }

        self.app.state.messages.push(msg);
    }

    fn try_handle_key_press(&mut self) {
        if self.context.input().events.is_empty() {
            return;
        }

        if self.context.input().key_pressed(egui::Key::F12) {
            self.context.set_debug_on_hover(
                !self.context.debug_on_hover(), //
            )
        }

        if self.context.input().key_pressed(egui::Key::F10) {
            self.app.state.show_image_map = !self.app.state.show_image_map;
        }

        if self.app.state.keybind_state.is_capturing() {
            return;
        }

        let ctx = self.context.clone();
        for (key, modifiers) in ctx.input().events.iter().filter_map(|c| match c {
            &egui::Event::Key {
                key,
                pressed,
                modifiers,
            } if !pressed => Some((key, modifiers)),
            _ => None,
        }) {
            if let Some(action) = self.app.state.key_mapping.find(key, modifiers) {
                use crate::KeyAction::*;

                eprintln!("action: {action:?}");

                match action {
                    SwitchToSettings => self.switch_to_settings(),
                    SwitchToMain => self.switch_to_main(),

                    ToggleLineMode => self.toggle_line_mode(),
                    ToggleTabBar => self.toggle_tab_bar(),
                    ToggleTimestamps => self.toggle_timestamps(),
                    ToggleUserList => self.toggle_user_list(),

                    SwitchTab0 => self.try_set_active_tab(0),
                    SwitchTab1 => self.try_set_active_tab(1),
                    SwitchTab2 => self.try_set_active_tab(2),
                    SwitchTab3 => self.try_set_active_tab(3),
                    SwitchTab4 => self.try_set_active_tab(4),
                    SwitchTab5 => self.try_set_active_tab(5),
                    SwitchTab6 => self.try_set_active_tab(6),
                    SwitchTab7 => self.try_set_active_tab(7),
                    SwitchTab8 => self.try_set_active_tab(8),
                    SwitchTab9 => self.try_set_active_tab(9),

                    NextTab => self.next_tab(),
                    PreviousTab => self.previous_tab(),
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.try_poll_twitch();
        self.try_fetch_badges();
        self.try_fetch_image();
        self.try_read_message();
        self.try_handle_key_press();

        Window::new("image cache")
            .open(&mut self.app.state.show_image_map)
            .show(&self.context, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if !self.app.state.requested_images.is_empty() {
                        ui.group(|ui| {
                            ui.label("requested images");
                            ui.vertical(|ui| {
                                let mut ids =
                                    self.app.state.requested_images.iter().collect::<Vec<_>>();
                                ids.sort();

                                for id in ids {
                                    ui.monospace(id.to_string());
                                }
                            })
                        });
                    }

                    Grid::new("image_map").num_columns(3).show(ui, |ui| {
                        for (id, img) in
                            self.app.state.images.map.iter().collect::<BTreeMap<_, _>>()
                        {
                            img.show_size(ui, vec2(16.0, 16.0));
                            ui.label(img.debug_name());
                            ui.monospace(id.to_string());
                            ui.end_row()
                        }
                    });
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| MainView::new(&mut self.app).display(ui));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let data = BorrowedPersistState {
            env_config: &self.app.state.config,
            key_mapping: &self.app.state.key_mapping,
            channels: &self.app.state.channels,
            pixels_per_point: &self.app.state.pixels_per_point,
        };

        let json = serde_json::to_string(&data).expect("valid json");
        storage.set_string(SETTINGS_KEY, json);
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
