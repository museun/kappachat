use std::collections::{HashMap, HashSet};

use egui::{
    vec2, Direction, Frame, Label, Layout, RichText, ScrollArea, Sense, SidePanel, TextEdit,
    TextStyle, TopBottomPanel,
};
use egui_extras::RetainedImage;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

use crate::{
    helix::{Chatters, Kind},
    state::{AppState, State},
    twitch::{self, EmoteSpan},
    ImageCache, Queue,
};

use super::{ActiveSettingsView, SettingsView, StartView};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainView {
    #[default]
    Start,
    Main,
    Settings,
}

#[derive(Default)]
pub struct MainViewState {
    channels_to_join: Vec<String>,
    channels_not_joined: HashSet<String>,
}

pub struct Main<'a> {
    state: &'a mut AppState,
}

impl<'a> Main<'a> {
    pub fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }

    pub fn display(mut self, ui: &mut egui::Ui) {
        match self.state.state.view_state.current_view {
            MainView::Start => self.display_start(ui),
            MainView::Settings => self.display_settings(ui),
            MainView::Main => self.display_main(ui),
        }
    }

    fn display_main(&mut self, ui: &mut egui::Ui) {
        ChatView::new(&mut self.state.state, &self.state.writer).display(ui)
    }

    fn display_settings(&mut self, ui: &mut egui::Ui) {
        if SettingsView::new(&mut self.state.state).display(ui) {
            self.state.state.view_state.current_view = self.state.state.view_state.previous_view;
            std::mem::take(&mut self.state.state.keybind_state);
        }
    }

    fn display_start(&mut self, ui: &mut egui::Ui) {
        if StartView::new(
            &mut self.state.state.start_state,
            &mut self.state.state.key_mapping,
            &mut self.state.state.view_state,
        )
        .display(ui)
        {
            if self.state.state.twitch_settings.seems_good() {
                // BUG this blocks, so it should be using a promise
                if let Err(err) = self.state.connect(ui.ctx().clone()) {
                    eprintln!("cannot connect: {err}");
                    todo!("recover from this")
                }

                self.state.state.view_state.current_view = MainView::Main;
                return;
            }
            self.state
                .state
                .view_state
                .switch_to_view(MainView::Settings);
            SettingsView::activate(&mut self.state.state, ActiveSettingsView::Twitch);
        }
    }
}

pub struct ChatLine {
    ts: Timestamp,
    id: uuid::Uuid,
    spans: Vec<EmoteSpan>,
    msg: twitch::Message,
}

struct ChatLineView<'a> {
    line: &'a ChatLine,
    cache: &'a ImageCache,
    emote_map: &'a HashMap<String, String>,
    show_timestamp: bool,
}

impl<'a> ChatLineView<'a> {
    fn new(
        line: &'a ChatLine,
        cache: &'a ImageCache,
        emote_map: &'a HashMap<String, String>,
        show_timestamp: bool,
    ) -> Self {
        Self {
            line,
            cache,
            emote_map,
            show_timestamp,
        }
    }

    fn display(&self, ui: &mut egui::Ui) {
        let pm = self.line.msg.as_privmsg().expect("this must be a privmsg");

        ui.horizontal_wrapped(|ui| {
            if self.show_timestamp {
                ui.small(self.line.ts.as_str())
                    .on_hover_ui_at_pointer(|ui| {
                        let s = OffsetDateTime::now_local().unwrap() - self.line.ts.date_time;
                        ui.small(format!("{} ago", format_seconds(s.whole_seconds() as _)));
                    });
            }

            ui.scope(|ui| {
                let width = ui
                    .fonts()
                    .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
                ui.spacing_mut().item_spacing.x = width;

                if let Some((badge, version)) = pm.badges().next() {
                    if let Some(img) = self.cache.get(badge) {
                        img.show_size(ui, vec2(8.0, 8.0));
                        // .on_hover_text_at_pointer(self.emote_map.get(badge).unwrap());
                    }
                }

                ui.colored_label(pm.color(), pm.sender);

                for spans in &self.line.spans {
                    match spans {
                        EmoteSpan::Emote(s) => match self.cache.get(s) {
                            Some(img) => {
                                img.show_size(ui, vec2(16.0, 16.0))
                                    .on_hover_text_at_pointer(self.emote_map.get(s).unwrap());
                            }
                            None => {
                                ui.add(Label::new(s));
                            }
                        },
                        EmoteSpan::Text(s) => {
                            ui.add(Label::new(s));
                        }
                    }
                }
            });
        });
    }
}

pub enum Line {
    Chat(ChatLine),
}

pub struct ChannelState {
    chatters: Chatters,
    buffer: EditBuffer,
    lines: Queue<Line>,
    channel: String,
}

impl ChannelState {
    pub fn chatters_mut(&mut self) -> &mut Chatters {
        &mut self.chatters
    }

    pub fn name(&self) -> &str {
        &self.channel
    }

    // TODO do we really need the full message?
    // if we make an owned variant of Privmsg we can just store that
    pub fn push_privmsg(&mut self, id: uuid::Uuid, spans: Vec<EmoteSpan>, msg: twitch::Message) {
        let ts = Timestamp::now_local();
        self.lines.push(Line::Chat(ChatLine { ts, id, spans, msg }))
    }
}

pub fn format_seconds(mut secs: u64) -> String {
    const TABLE: [(&str, u64); 4] = [
        ("days", 86400),
        ("hours", 3600),
        ("minutes", 60),
        ("seconds", 1),
    ];

    fn pluralize(s: &str, n: u64) -> String {
        format!("{} {}", n, if n > 1 { s } else { &s[..s.len() - 1] })
    }

    let mut time = vec![];
    for (name, d) in &TABLE {
        let div = secs / d;
        if div > 0 {
            time.push(pluralize(name, div));
            secs -= d * div;
        }
    }

    let len = time.len();
    if len > 1 {
        if len > 2 {
            for segment in time.iter_mut().take(len - 2) {
                segment.push(',')
            }
        }
        time.insert(len - 1, "and".into())
    }
    time.join(" ")
}

static FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

#[derive(Clone)]
pub struct Timestamp {
    pub(crate) date_time: OffsetDateTime, // why?
    pub(crate) repr: String,
}

impl Timestamp {
    pub fn now_local() -> Self {
        static FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");
        let date_time = OffsetDateTime::now_local().expect("valid time");
        let repr = date_time.format(&FORMAT).expect("valid time");
        Self { date_time, repr }
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Default)]
pub struct ChatViewState {
    channels: Vec<ChannelState>,
    active: Option<usize>,
    tab_bar_hidden: bool,
}

impl ChatViewState {
    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.channels.len() || self.channels.is_empty() {
            return;
        }
        self.active.replace(index);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut ChannelState> {
        self.channels.get_mut(index)
    }

    pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut ChannelState> {
        self.channels
            .iter_mut()
            .find(|ch| Self::is_same_channel(&*ch.channel, name))
    }

    fn is_same_channel(left: &str, right: &str) -> bool {
        left.strip_prefix('#').unwrap_or(left) == right.strip_prefix('#').unwrap_or(right)
    }

    pub fn active(&self) -> Option<&ChannelState> {
        Some(&self.channels[self.active?]) // XXX force the panic -- it shouldn't happen
    }

    pub fn active_mut(&mut self) -> Option<&mut ChannelState> {
        Some(&mut self.channels[self.active?]) // XXX force the panic -- it shouldn't happen
    }

    pub fn next(&mut self) {
        let active = match &mut self.active {
            Some(active) => active,
            None => return,
        };
        *active = (*active + 1) % self.channels.len();
    }

    pub fn previous(&mut self) {
        let active = match &mut self.active {
            Some(active) => active,
            None => return,
        };

        *active = (*active == 0)
            .then_some(self.channels.len())
            .unwrap_or(*active)
            - 1;
    }

    pub fn add_channel(&mut self, channel: impl ToString) {
        self.channels.push(ChannelState {
            chatters: Chatters::default(),
            buffer: EditBuffer::default(),
            lines: Queue::default(),
            channel: channel.to_string(),
        });
        self.set_active(self.channels.len() - 1);
    }

    pub fn remove_channel(&mut self, name: &str) {
        if let Some(pos) = self.channels.iter().position(|c| c.channel == name) {
            self.set_active(pos.saturating_sub(1));
            self.channels.remove(pos);
        }
    }

    pub fn toggle_tab_bar(&mut self) {
        self.tab_bar_hidden = !self.tab_bar_hidden;
    }
}

#[derive(Default)]
pub struct EditBuffer {
    pub buffer: String,
}

struct TabView<'a> {
    state: &'a mut ChatViewState,
}

impl<'a> TabView<'a> {
    fn new(state: &'a mut ChatViewState) -> Self {
        Self { state }
    }

    fn display(self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut active = None;
            for (i, state) in self.state.channels.iter().enumerate() {
                let selected = Some(i) == self.state.active;
                if ui.selectable_label(selected, &state.channel).clicked() {
                    active.replace(i);
                }
            }
            if let Some(active) = active {
                self.state.set_active(active);
            }
        });
    }
}

struct ChatView<'a> {
    state: &'a mut State,
    writer: &'a flume::Sender<String>,
}

impl<'a> ChatView<'a> {
    fn new(state: &'a mut State, writer: &'a flume::Sender<String>) -> Self {
        Self { state, writer }
    }

    fn display(self, ui: &mut egui::Ui) {
        let cvs = &mut self.state.chat_view_state;
        if !cvs.tab_bar_hidden {
            // TODO allow moving from top/left/bottom/right
            TopBottomPanel::top("tab_bar")
                .resizable(false)
                .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
                .show_inside(ui, |ui| {
                    TabView::new(cvs).display(ui);
                });
        }

        let state = match cvs.active_mut() {
            Some(state) => state,
            None => return,
        };

        let buf = &mut state.buffer;

        TopBottomPanel::bottom("input")
            .resizable(false)
            .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
            .show_inside(ui, |ui| {
                ui.with_layout(
                    Layout::centered_and_justified(Direction::LeftToRight),
                    |ui| {
                        EditBox::new(&mut buf.buffer, self.writer).display(ui);
                    },
                );
            });

        if let Some(channel) = self
            .state
            .channels
            .iter_mut()
            .find(|c| ChatViewState::is_same_channel(&c.name, &state.channel))
        {
            if channel.show_user_list {
                SidePanel::right("user_list")
                    .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
                    .show_inside(ui, |ui| {
                        UserList::new(&state.chatters, &self.state.images).display(ui);
                    });
            }
        }

        let show_timestamp = self
            .state
            .channels
            .iter()
            .find_map(|c| (c.name == state.name()).then_some(c.show_timestamps))
            .unwrap_or(true);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true) // TODO if we're scrolled up don't do this
            .show(ui, |ui| {
                for line in state.lines.iter() {
                    match line {
                        Line::Chat(line) => {
                            ChatLineView::new(
                                line,
                                &self.state.images,
                                &self.state.emote_map,
                                show_timestamp,
                            )
                            .display(ui);
                        }
                    }
                }
            });
    }
}

struct UserList<'a> {
    chatters: &'a Chatters,
    images: &'a ImageCache,
}

impl<'a> UserList<'a> {
    fn new(chatters: &'a Chatters, images: &'a ImageCache) -> Self {
        Self { chatters, images }
    }

    fn get_image(&self, kind: Kind) -> Option<&RetainedImage> {
        self.images.get(&kind.as_str()[..kind.as_str().len() - 1])
    }

    fn display(self, ui: &mut egui::Ui) {
        let width = ui
            .fonts()
            .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = width;
            ui.style_mut().spacing.interact_size.y = width;
            let image_size = vec2(8.0, 8.0);

            ScrollArea::vertical().show(ui, |ui| {
                for (kind, chatter) in &self.chatters.chatters {
                    ui.horizontal(|ui| {
                        if let Some(img) = self.get_image(*kind) {
                            img.show_size(ui, image_size);
                        } else {
                            ui.allocate_exact_size(image_size, Sense::hover());
                        }
                        ui.add(Label::new(RichText::new(chatter).small()).wrap(false));
                    });
                }
            });
        });
    }
}

// TODO spell check
// TODO kappa completion
// TODO name completion
struct EditBox<'a> {
    buffer: &'a mut String,
    write: &'a flume::Sender<String>,
}

impl<'a> EditBox<'a> {
    fn new(buffer: &'a mut String, write: &'a flume::Sender<String>) -> Self {
        Self { buffer, write }
    }

    fn display(self, ui: &mut egui::Ui) {
        let id = self.buffer.as_ptr();
        let resp = ui.add(
            TextEdit::singleline(self.buffer)
                .id_source(id)
                .frame(false)
                .lock_focus(true),
        );

        if resp.lost_focus() && ui.ctx().input().key_down(egui::Key::Enter) {
            let line = std::mem::take(self.buffer);
            let _ = self.write.send(line);
        }

        resp.request_focus();
    }
}
