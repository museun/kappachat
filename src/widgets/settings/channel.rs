use std::borrow::Cow;

use egui::{
    vec2, Align, Frame, Key, Label, Layout, RichText, ScrollArea, TextEdit, TopBottomPanel,
};
use poll_promise::Promise;

use crate::{
    fetch::ImageKind,
    font_icon::{AUTOJOIN, REMOVE, TIME, USER_LIST},
    helix::{self, IdOrLogin},
    store::Image,
    Channel, FetchQueue, ImageCache,
};

#[derive(Default)]
pub struct TwitchChannelsState {
    showing_error: bool,
    reason: Cow<'static, str>,
    duplicate: Option<String>,
    invalid: Option<String>,
    remove: Option<String>,
    buffer: String,
}

pub struct ChannelSettings<'a> {
    state: &'a mut TwitchChannelsState,
    helix: &'a Promise<helix::Client>,
    channels: &'a mut Vec<Channel>,
    images: &'a ImageCache,
    fetch: &'a mut FetchQueue<Image>,
}

impl<'a> ChannelSettings<'a> {
    pub fn new(
        state: &'a mut TwitchChannelsState,
        channels: &'a mut Vec<Channel>,
        helix: &'a Promise<helix::Client>,
        images: &'a ImageCache,
        fetch: &'a mut FetchQueue<Image>,
    ) -> Self {
        Self {
            state,
            channels,
            helix,
            images,
            fetch,
        }
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

                if resp.changed() {
                    self.state.invalid.take();
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
                    let duplicate = self.state.duplicate.as_deref() == Some(&*channel.login);

                    let resp = ui.horizontal(|ui| {
                        let resp = ui
                            .horizontal(|ui| {
                                match self.images.get_id(channel.image_id) {
                                    Some(img) => {
                                        // TODO scale with ui
                                        img.show_max_size(ui, vec2(16.0, 16.0));
                                    }
                                    None => {
                                        self.fetch.fetch(Image {
                                            id: channel.image_id,
                                            url: channel.profile_image_url.clone(),
                                            kind: ImageKind::Display,
                                            meta: (),
                                        });
                                    }
                                }

                                ui.add(Label::new(
                                    RichText::new(&channel.login).monospace().color(
                                        duplicate
                                            .then(|| ui.style().visuals.warn_fg_color)
                                            .unwrap_or_else(|| ui.style().visuals.text_color()),
                                    ),
                                ))
                            })
                            .inner;

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(REMOVE)
                                .on_hover_text_at_pointer("Remove channel, leaving it")
                                .clicked()
                            {
                                self.state.remove.replace(channel.login.clone());
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
            if let Some(pos) = self.channels.iter().position(|c| c.login == remove) {
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

        let helix = match self.helix.ready() {
            Some(helix) => helix,
            None => return,
        };

        let mut users = helix
            .get_users([IdOrLogin::Login(&*channel)])
            .expect("get users from twitch");

        if users.is_empty() {
            let problem = channel.clone();
            self.report_invalid_channel(problem);
            return;
        }

        let user = users.remove(0);
        self.channels.push(Channel::new(user));
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

        if self.state.invalid.is_some() {
            return;
        }

        if let Some(channel) = self
            .channels
            .iter()
            .find(|left| Self::is_same_channel(&left.login, buf))
        {
            self.report_duplicate(channel.login.clone());
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
        self.report_error(format!("Duplicate channels aren't allowed: {channel}"));
        self.state.duplicate.replace(channel);
    }

    fn report_invalid_channel(&mut self, channel: impl ToString) {
        let channel = channel.to_string();
        self.report_error(format!("Twitch doesn't have a channel for: {channel}"));
        self.state.invalid.replace(channel);
        self.state.duplicate.take();
    }

    fn report_spaces(&mut self) {
        self.report_error("Channels cannot contain spaces");
        self.state.duplicate.take();
        self.state.invalid.take();
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
