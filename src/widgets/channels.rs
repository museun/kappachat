use egui::{Align, Grid, Key, Label, Layout, RichText, TextEdit};

use crate::{
    font_icon::{ADD, AUTOJOIN, REMOVE, TIME, USER_LIST},
    Channel,
};

#[derive(Default)]
pub struct ChannelState {
    showing_error: bool,
    reason: &'static str, // TODO enum

    duplicate: Option<usize>,
    remove: Vec<String>,
    channel: Option<String>,
}

pub struct TwitchChannels<'a> {
    state: &'a mut ChannelState,
    channels: &'a mut Vec<Channel>,
}

impl<'a> TwitchChannels<'a> {
    pub fn new(state: &'a mut ChannelState, channels: &'a mut Vec<Channel>) -> Self {
        Self { state, channels }
    }

    fn list_channels(&mut self, ui: &mut egui::Ui) {
        Grid::new("twitch_channels")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (i, channel) in self.channels.iter_mut().enumerate() {
                    let duplicate = self.state.duplicate == Some(i);

                    ui.add(Label::new(
                        RichText::new(&channel.name).monospace().color(
                            duplicate
                                .then(|| ui.style().visuals.warn_fg_color)
                                .unwrap_or_else(|| ui.style().visuals.text_color()),
                        ),
                    ));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .small_button(REMOVE)
                            .on_hover_text_at_pointer("Remove channel, leaving it")
                            .clicked()
                        {
                            self.state.remove.push(channel.name.clone());
                        }

                        for (icon, description, state) in [
                            (AUTOJOIN, "auto-join", &mut channel.auto_join),
                            (USER_LIST, "the user list", &mut channel.show_user_list),
                            (TIME, "timestamps", &mut channel.show_timestamps),
                        ] {
                            ui.scope(|ui| {
                                ui.style_mut().interaction.show_tooltips_only_when_still = true;
                                ui.toggle_value(state, icon).on_hover_text_at_pointer(
                                    Self::update_tooltip(*state, description),
                                );
                            });
                        }
                    });

                    ui.end_row()
                }
            });
    }

    fn update_tooltip(bool: bool, text: &str) -> String {
        bool.then(|| format!("Disable {text}"))
            .unwrap_or_else(|| format!("Enable {text}"))
    }

    fn try_show_edit_box(&mut self, ui: &mut egui::Ui) {
        let channel = match &mut self.state.channel {
            Some(channel) => channel,
            None => return,
        };

        let resp = ui.add(
            TextEdit::singleline(channel)
                .text_color(
                    self.state
                        .showing_error
                        .then(|| ui.style().visuals.error_fg_color)
                        .unwrap_or_else(|| ui.style().visuals.text_color()),
                )
                .hint_text("#museun")
                .lock_focus(true),
        );

        self.check_validity();

        let resp = if self.state.showing_error {
            resp.on_hover_ui(|ui| {
                ui.colored_label(ui.style().visuals.warn_fg_color, self.state.reason);
            })
        } else {
            resp
        };

        if resp.lost_focus() && self.state.showing_error {
            resp.request_focus();
            return;
        }

        if resp.lost_focus() && ui.ctx().input().key_pressed(Key::Enter) {
            self.try_add_channel(ui);
        }
    }

    fn try_add_channel(&mut self, ui: &mut egui::Ui) {
        if self.state.showing_error {
            return;
        }

        let channel = match self.state.channel.take() {
            Some(channel) => channel,
            None => return,
        };

        if channel.chars().all(|c| c == '#') {
            return;
        }

        let name = match channel.starts_with('#') {
            true => channel,
            false => format!("#{channel}"),
        };

        self.channels.push(Channel::new(name));

        std::mem::take(self.state);
    }

    fn check_validity(&mut self) {
        let channel = match &self.state.channel {
            Some(channel) => channel,
            None => return,
        };

        if channel.contains(' ') {
            self.report_spaces();
            return;
        }

        if let Some(pos) = self
            .channels
            .iter()
            .position(|left| Self::is_same_channel(&left.name, channel))
        {
            self.report_duplicate(pos);
            return;
        }

        self.state.duplicate.take();
        self.state.showing_error = false;
    }

    fn report_duplicate(&mut self, pos: usize) {
        self.report_error("duplicate channels aren't allowed");
        self.state.duplicate.replace(pos);
    }

    fn report_spaces(&mut self) {
        self.report_error("cannot contain spaces");
        self.state.duplicate.take();
    }

    fn report_error(&mut self, reason: &'static str) {
        self.state.showing_error = true;
        self.state.reason = reason;
    }

    fn is_same_channel(left: &str, right: &str) -> bool {
        left.strip_prefix('#').unwrap_or(left) == right.strip_prefix('#').unwrap_or(right)
    }
}

impl<'a> egui::Widget for TwitchChannels<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.vertical(|ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Channels");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.state.channel.is_some()
                            && ui
                                .small_button(REMOVE)
                                .on_hover_text_at_pointer("Remove input")
                                .clicked()
                        {
                            std::mem::take(self.state);
                        }

                        self.try_show_edit_box(ui);

                        if self.state.channel.is_none()
                            && ui
                                .small_button(ADD)
                                .on_hover_text_at_pointer("Add a new channel")
                                .clicked()
                        {
                            self.state.channel.replace(String::new());
                        }
                    });
                });

                ui.separator();
                self.list_channels(ui);
            });
        });

        for channel in self.state.remove.drain(..) {
            if let Some(pos) = self.channels.iter().position(|c| &c.name == &channel) {
                self.channels.remove(pos);
            }
        }

        resp.response
    }
}

#[cfg(test)]
mod tests {
    use egui::{CentralPanel, Key, Modifiers};

    use crate::{Chord, KeyHelper};

    #[test]
    fn input() {
        #[derive(Default)]
        struct State {
            modifiers: Modifiers,
            key: Option<Key>,
            recording: bool,
        }

        impl State {
            fn update_modifiers(&mut self, modifiers: Modifiers) {
                if !self.recording || self.modifiers == modifiers {
                    return;
                }

                self.modifiers = modifiers;
                self.key.take();
            }

            fn update_key(&mut self, key: Key) {
                if !self.recording {
                    return;
                }

                self.key.replace(key);
            }

            fn chord(&self) -> Option<Chord> {
                Some(Chord::from((self.modifiers, self.key?)))
            }
        }

        #[derive(Default)]
        struct A {
            state: State,
        }

        impl eframe::App for A {
            fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
                CentralPanel::default().show(ctx, |ui| {
                    ui.toggle_value(&mut self.state.recording, "record");

                    self.state.update_modifiers(ctx.input().modifiers);

                    for (key, modifiers) in
                        ctx.input().events.iter().filter_map(|event| match event {
                            egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                            } if !pressed => Some((key, modifiers)),

                            _ => None,
                        })
                    {
                        self.state.update_modifiers(*modifiers);
                        self.state.update_key(*key);
                    }

                    if self.state.recording {
                        ui.horizontal(|ui| {
                            for (s, desc) in [
                                (&mut self.state.modifiers.ctrl, "ctrl"),
                                (&mut self.state.modifiers.shift, "shift"),
                                (&mut self.state.modifiers.alt, "alt"),
                            ] {
                                ui.checkbox(s, desc);
                            }

                            if let Some(key) = self.state.key {
                                ui.label(format!("{}", KeyHelper::stringify_key(&key)));
                            }
                        });

                        if let Some(chord) = self.state.chord() {
                            ui.label(chord.display());
                        }
                    }
                });
            }
        }

        eframe::run_native(
            "input",
            <_>::default(),
            Box::new(|cc| {
                cc.egui_ctx.set_pixels_per_point(2.0);
                Box::new(A::default())
            }),
        )
    }
}
