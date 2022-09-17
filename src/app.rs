use egui_extras::RetainedImage;

use crate::{
    state::{AppState, BorrowedPersistState},
    store::{Image, ImageStore},
    widgets::{Main, MainView},
    Channel, FetchImage, SETTINGS_KEY,
};

pub struct App {
    context: egui::Context,
    pub app: AppState,
}

impl App {
    pub const fn new(context: egui::Context, app: AppState) -> Self {
        Self { context, app }
    }

    const fn is_connected(&self) -> bool {
        self.app.twitch.is_some()
    }

    fn toggle_tab_bar(&mut self) {
        self.app.state.chat_view_state.toggle_tab_bar();
    }

    fn toggle_user_list(&mut self) {
        let name = match self.find_active_channel() {
            Some(channel) => {
                channel.show_user_list = !channel.show_user_list;
                channel.login.clone()
            }
            _ => return,
        };

        self.app.runtime.chatters_update.request_update(name);
    }

    fn toggle_line_mode(&mut self) {
        // self.app.tabs.active_mut().next_line_mode()
    }

    fn toggle_timestamps(&mut self) {
        if let Some(channel) = self.find_active_channel() {
            channel.show_timestamps = !channel.show_timestamps;
        }
    }

    fn find_active_channel(&mut self) -> Option<&mut Channel> {
        let active = self.app.state.chat_view_state.active()?;
        self.app
            .state
            .channels
            .iter_mut()
            .find(|c| c.login == active.name())
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
        self.app.state.view_state.switch_to_view(MainView::Settings)
    }

    fn switch_to_main(&mut self) {
        let vs = &mut self.app.state.view_state;
        vs.switch_to_view(vs.previous_view)
    }

    fn try_fetch_chatters(&mut self) {
        for (channel, chatters) in self.app.runtime.chatters_update.poll() {
            if let Some(channel) = self.app.state.chat_view_state.get_mut_by_name(&channel) {
                *channel.chatters_mut() = chatters;
            }
        }
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
        if true
        //  DESIRED_USER_LIST_BADGES
        // .into_iter()
        // .fold(true, |ok, key| ok & self.app.state.images.has(key))
        {
            return;
        }

        // let badges = match self.app.runtime.global_badges.ready() {
        //     Some(badges) => badges,
        //     None => {
        //         let helix = match self.app.runtime.helix.ready() {
        //             Some(helix) => helix,
        //             None => return,
        //         };

        //         let _ = self.app.runtime.helix_ready.send(helix.clone());
        //         return;
        //     }
        // };

        // for (set_id, (id, url)) in badges.iter().flat_map(|badge| {
        //     std::iter::repeat(&badge.set_id)
        //         .zip(badge.versions.iter().map(|v| (&v.id, &v.image_url_4x)))
        // }) {
        //     if !DESIRED_USER_LIST_BADGES.contains(&&**set_id) {
        //         continue;
        //     }

        //     let badge = TwitchImage::badge(id, &set_id, url);
        //     if self.app.state.images.has_id(badge.id()) {
        //         continue;
        //     }

        //     if !self.app.state.requested_images.insert(badge.id()) {
        //         continue;
        //     }

        //     self.app.runtime.fetch.fetch(badge);
        // }
    }

    fn try_fetch_image(&mut self) {
        let (image, data) = match self.app.runtime.fetch.try_next() {
            Some((image, data)) => (image, data),
            _ => return,
        };

        let images = &mut self.app.state.images;
        if images.has_id(image.id) {
            return;
        }

        match RetainedImage::from_image_bytes(image.url(), &data) {
            Ok(img) => {
                images.add(image.id, img);
                let _ = self.app.state.requested_images.remove(&image.id);
                ImageStore::<Image>::add(&image, &(), &data);
            }
            Err(err) => {
                eprintln!("cannot create ({}) {} : {err}", image.id, image.url())
            }
        }
    }

    fn try_update_images(&mut self) {}

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
        self.try_privmsg(&msg);

        if let Some(join) = msg.as_join() {
            if self.app.is_our_name(join.user) {
                self.app.state.chat_view_state.add_channel(join.channel);
                self.app.runtime.chatters_update.subscribe(join.channel);
            }
        }

        if let Some(part) = msg.as_part() {
            if self.app.is_our_name(part.user) {
                self.app.state.chat_view_state.remove_channel(part.channel);
                self.app.runtime.chatters_update.unsubscribe(part.channel);
            }
        }

        self.app.state.messages.push(msg);
    }

    fn try_privmsg(&mut self, msg: &crate::twitch::Message) {
        let pm = match msg.as_privmsg() {
            Some(item) => item,
            _ => return,
        };

        pm.update_emote_map(&mut self.app.state.emote_map);

        let (id, spans) = pm.make_spans();

        let active = match self.app.state.chat_view_state.get_mut_by_name(pm.target) {
            Some(active) => active,
            None => {
                eprintln!(
                    "!! we're not on {} but we got a msg: {}",
                    pm.target,
                    msg.raw.escape_debug()
                );
                return;
            }
        };

        active.push_privmsg(id, spans, msg.clone());

        // for (emote, _) in pm.emotes() {
        //     if self.app.state.images.has(emote) {
        //         continue;
        //     }

        //     let name = match self.app.state.emote_map.get(emote) {
        //         Some(name) => name,
        //         None => {
        //             eprintln!("emote missing: {emote}");
        //             continue;
        //         }
        //     };

        //     // TODO also fetch the light one
        //     // TODO this returns whether its fetching it or not
        //     self.app.runtime.fetch.fetch(TwitchImage::emote(
        //         &emote,
        //         &name,
        //         format!("https://static-cdn.jtvnw.net/emoticons/v2/{emote}/static/dark/3.0"),
        //     ));
        // }
    }

    fn try_handle_user_input(&mut self) {
        if let Ok(msg) = self.app.reader.try_recv() {
            if let Some(ch) = self.app.state.chat_view_state.active() {
                self.app
                    .interaction
                    .send_raw(format!("PRIVMSG {} :{}\r\n", ch.name(), msg.trim()));
            }
        }
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // TODO diff this?
        self.app.state.window_size = frame.info().window_info.size;

        self.try_poll_twitch();
        self.try_fetch_badges();
        self.try_fetch_chatters();
        self.try_fetch_image();
        self.try_update_images();
        self.try_read_message();
        self.try_handle_user_input();
        self.try_handle_key_press();

        // Window::new("image cache")
        //     .open(&mut self.app.state.show_image_map)
        //     .show(&self.context, |ui| {
        //         ui.vertical(|ui| {
        //             CollapsingHeader::new("image cache")
        //                 .default_open(false)
        //                 .show(ui, |ui| {
        //                     ScrollArea::vertical().show(ui, |ui| {
        //                         if !self.app.state.requested_images.is_empty() {
        //                             ui.group(|ui| {
        //                                 ui.label("requested images");
        //                                 ui.vertical(|ui| {
        //                                     let mut ids = self
        //                                         .app
        //                                         .state
        //                                         .requested_images
        //                                         .iter()
        //                                         .collect::<Vec<_>>();
        //                                     ids.sort();

        //                                     for id in ids {
        //                                         ui.monospace(id.to_string());
        //                                     }
        //                                 })
        //                             });
        //                         }

        //                         Grid::new("image_map").num_columns(3).show(ui, |ui| {
        //                             for (id, img) in
        //                                 self.app.state.images.map.iter().collect::<BTreeMap<_, _>>()
        //                             {
        //                                 img.show_size(ui, vec2(16.0, 16.0));
        //                                 ui.label(img.debug_name());
        //                                 ui.monospace(id.to_string());
        //                                 ui.end_row()
        //                             }
        //                         });
        //                     });
        //                 });

        //             CollapsingHeader::new("disk cache")
        //                 .default_open(false)
        //                 .show(ui, |ui| {
        //                     // TODO cache this
        //                     ScrollArea::vertical().show(ui, |ui| {
        //                         for image in ImageStore::<Image>::get_all_debug() {
        //                             Grid::new(image.image.id).num_columns(2).show(ui, |ui| {
        //                                 ui.monospace(image.image.id.to_string());
        //                                 if let Some(img) =
        //                                     self.app.state.images.get_id(image.image.id)
        //                                 {
        //                                     img.show_max_size(ui, vec2(32.0, 32.0));
        //                                 }

        //                                 ui.end_row()
        //                             });
        //                         }
        //                     });
        //                 });
        //         });
        //     });

        Main::new(&mut self.app).display(ctx)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let data = BorrowedPersistState {
            env_config: &self.app.state.config,
            key_mapping: &self.app.state.key_mapping,
            channels: &self.app.state.channels,
            pixels_per_point: &self.app.state.pixels_per_point,
            tab_bar_position: self.app.state.chat_view_state.tab_bar_position,
            tab_bar_image_size: self.app.state.chat_view_state.image_size,
            show_image_mask: self.app.state.chat_view_state.show_mask,
        };

        let json = serde_json::to_string(&data).expect("valid json");
        storage.set_string(SETTINGS_KEY, json);
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
