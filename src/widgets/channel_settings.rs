use std::borrow::Cow;

use egui::{Align, Frame, Key, Label, Layout, RichText, ScrollArea, TextEdit, TopBottomPanel};

use crate::{
    font_icon::{AUTOJOIN, REMOVE, TIME, USER_LIST},
    Channel,
};

#[derive(Default)]
pub struct TwitchChannelsState {
    showing_error: bool,
    reason: Cow<'static, str>,
    duplicate: Option<String>,
    remove: Option<String>,
    buffer: String,
}

pub struct ChannelSettings<'a> {
    state: &'a mut TwitchChannelsState,
    channels: &'a mut Vec<Channel>,
}

impl<'a> ChannelSettings<'a> {
    pub fn new(state: &'a mut TwitchChannelsState, channels: &'a mut Vec<Channel>) -> Self {
        Self { state, channels }
    }

    pub fn display(mut self, ui: &mut egui::Ui) {
        TopBottomPanel::bottom("add_channel")
            .resizable(false)
            .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
            .show_inside(ui, |ui| {
                let mut resp = ui.add(
                    TextEdit::singleline(&mut self.state.buffer)
                        .text_color(
                            self.state
                                .showing_error
                                .then(|| ui.style().visuals.error_fg_color)
                                .unwrap_or_else(|| ui.style().visuals.text_color()),
                        )
                        .frame(false)
                        .hint_text(Self::DEFAULT_CHANNEL_HINT)
                        .lock_focus(true),
                );

                self.check_input_validity();

                if self.state.showing_error {
                    resp = resp.on_hover_ui(|ui| {
                        ui.colored_label(ui.style().visuals.warn_fg_color, &*self.state.reason);
                    })
                }

                if resp.lost_focus() && ui.ctx().input().key_pressed(Key::Enter) {
                    self.try_add_channel(ui);
                }
                resp.request_focus();
            });

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.state.duplicate.is_none() || self.state.buffer.is_empty())
            .show(ui, |ui| {
                for channel in self.channels.iter_mut() {
                    let duplicate = self.state.duplicate.as_deref() == Some(&*channel.name);

                    let resp = ui.horizontal(|ui| {
                        let resp = ui.add(Label::new(
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
                                self.state.remove.replace(channel.name.clone());
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

                        if duplicate || self.state.buffer.is_empty() {
                            resp.scroll_to_me(None)
                        }
                    });
                }
            });

        if let Some(remove) = self.state.remove.take() {
            if let Some(pos) = self.channels.iter().position(|c| c.name == remove) {
                self.channels.remove(pos);
            }
        }
    }

    fn try_add_channel(&mut self, ui: &mut egui::Ui) {
        if self.state.showing_error {
            return;
        }

        let channel = &mut self.state.buffer;
        if channel.chars().all(|c| c == '#') {
            return;
        }

        let channel = std::mem::take(channel);
        let name = match channel.starts_with('#') {
            true => channel,
            false => format!("#{channel}"),
        };

        self.channels.push(Channel::new(name));
        std::mem::take(self.state);
    }

    fn check_input_validity(&mut self) {
        let buf = &mut self.state.buffer;
        if buf.is_empty() {
            std::mem::take(self.state);
            return;
        }

        if buf.contains(' ') {
            self.report_spaces();
            return;
        }

        if let Some(channel) = self
            .channels
            .iter()
            .find(|left| Self::is_same_channel(&left.name, buf))
        {
            self.report_duplicate(channel.name.clone());
            return;
        }

        self.reset_error()
    }

    fn reset_error(&mut self) {
        self.state.duplicate.take();
        self.state.showing_error = false;
    }

    fn report_duplicate(&mut self, channel: impl ToString) {
        let channel = channel.to_string();
        self.report_error(format!("duplicate channels aren't allowed: {channel}"));
        self.state.duplicate.replace(channel);
    }

    fn report_spaces(&mut self) {
        self.report_error("cannot contain spaces");
        self.state.duplicate.take();
    }

    fn report_error(&mut self, reason: impl Into<Cow<'static, str>>) {
        self.state.showing_error = true;
        self.state.reason = reason.into();
    }

    fn is_same_channel(left: &str, right: &str) -> bool {
        left.strip_prefix('#').unwrap_or(left) == right.strip_prefix('#').unwrap_or(right)
    }

    fn update_tooltip(bool: bool, text: &str) -> String {
        bool.then(|| format!("Disable {text}"))
            .unwrap_or_else(|| format!("Enable {text}"))
    }

    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

    const DEFAULT_CHANNEL_HINT: &'static str = "#museun";
}
