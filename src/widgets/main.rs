use std::collections::{hash_map::Entry, HashMap, HashSet};

use egui::{
    collapsing_header::CollapsingState, vec2, Label, RichText, ScrollArea, Sense, SidePanel,
    TextEdit, TextStyle, TopBottomPanel,
};
use egui_extras::RetainedImage;

use crate::{
    helix::{Chatters, Kind},
    state::{AppState, State},
    twitch::EmoteSpan,
    Queue,
};

use super::{SettingsView, StartView};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainViewView {
    #[default]
    Start,
    Main,
    Settings,
    Connecting,
    Connected,
    Foo,
    // Image Cache
    // Links for channel
    // Mentions
}

#[derive(Default)]
pub struct MainViewState {
    channels_to_join: Vec<String>,
    channels_not_joined: HashSet<String>,
}

pub struct MainView<'a> {
    app: &'a mut AppState,
}

impl<'a> MainView<'a> {
    pub fn new(app: &'a mut AppState) -> Self {
        Self { app }
    }

    // TODO redo this
    pub fn display(mut self, ui: &mut egui::Ui) {
        match self.app.state.view_state.current_view {
            MainViewView::Start => {
                if StartView::new(
                    &mut self.app.state.start_state,
                    &mut self.app.state.key_mapping,
                    &mut self.app.state.view_state,
                )
                .display(ui)
                {
                    if !self.app.state.twitch_settings.seems_good() {
                        self.app.state.view_state.previous_view = std::mem::replace(
                            &mut self.app.state.view_state.current_view, //
                            MainViewView::Settings,
                        );
                        SettingsView::activate(
                            &mut self.app.state,
                            super::ActiveSettingsView::Twitch,
                        );
                        return;
                    }

                    self.app.state.view_state.current_view = MainViewView::Connecting;
                }
            }

            MainViewView::Settings => {
                if SettingsView::new(&mut self.app.state).display(ui) {
                    self.app.state.view_state.current_view =
                        self.app.state.view_state.previous_view;
                    std::mem::take(&mut self.app.state.keybind_state);
                }
            }

            MainViewView::Connecting => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("connecting to Twitch");
                        ui.spinner();
                    });

                    ui.label(format!("our name: {}", self.app.state.config.twitch_name));
                });

                if let Err(err) = self.app.connect(ui.ctx().clone()) {
                    eprintln!("error: {err}");
                    self.app.state.view_state.current_view = MainViewView::Start;
                }

                self.app.state.main_view.channels_to_join.extend(
                    self.app
                        .state
                        .channels
                        .iter()
                        .filter_map(|c| c.auto_join.then(|| c.name.clone())),
                );

                for channel in &self.app.state.main_view.channels_to_join {
                    self.app.join_channel(channel);
                }

                self.app.state.view_state.current_view = MainViewView::Connected;
            }

            MainViewView::Connected => {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "connected as: {} ({})",
                        self.app.identity().user_name,
                        self.app.identity().user_id
                    ));

                    ui.label("joining channels:");
                    for channel in &self.app.state.main_view.channels_to_join {
                        ui.label(channel);
                    }

                    for join in self.app.state.messages.iter().filter_map(|c| c.as_join()) {
                        if self.app.is_our_name(join.user) {
                            if self
                                .app
                                .state
                                .main_view
                                .channels_not_joined
                                .remove(join.channel)
                            {
                                ui.label(format!("joined: {}", join.channel));
                            }
                        }
                    }
                });

                if self.app.state.main_view.channels_not_joined.is_empty() {
                    self.app.state.view_state.current_view = MainViewView::Main;
                }
            }

            MainViewView::Foo => {
                ChatView::new(&mut self.app).display(ui);
            }

            MainViewView::Main => {
                ui.scope(|ui| {
                    let width = ui
                        .fonts()
                        .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
                    ui.spacing_mut().item_spacing.x = width;

                    ui.vertical(|ui| {
                        for pm in self
                            .app
                            .state
                            .messages
                            .iter()
                            .flat_map(|msg| msg.as_privmsg())
                        {
                            let spanned = match self.app.state.spanned_lines.get(&pm.id()) {
                                Some(spanned) => spanned,
                                None => continue,
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(pm.color(), pm.sender);
                                for spans in spanned {
                                    match spans {
                                        EmoteSpan::Emote(s) => {
                                            match self.app.state.images.get(&*s) {
                                                Some(img) => {
                                                    img.show_size(ui, vec2(16.0, 16.0))
                                                        .on_hover_text_at_pointer(
                                                            self.app
                                                                .state
                                                                .emote_map
                                                                .get(&*s)
                                                                .unwrap(),
                                                        );
                                                }
                                                None => {
                                                    ui.add(Label::new(s).wrap(true));
                                                }
                                            }
                                        }
                                        EmoteSpan::Text(s) => {
                                            ui.add(Label::new(s).wrap(true));
                                        }
                                    }
                                }
                            });
                        }
                    });
                });
            }
        }
    }
}

pub enum Line {
    Chat { id: uuid::Uuid },
}

pub(super) type Index = usize;
pub(super) type RoomId = usize;

#[derive(Default)]
pub struct ChannelState {
    edit_buffers: HashMap<RoomId, EditBuffer>,
    history: HashMap<RoomId, Queue<Line>>,
    chatters: HashMap<RoomId, (bool, Chatters)>,
}

impl ChannelState {
    fn add(&mut self, id: RoomId) -> bool {
        match self.edit_buffers.entry(id) {
            Entry::Occupied(..) => return false,
            Entry::Vacant(e) => {
                e.insert(Default::default());
            }
        }
        self.history.insert(id, Queue::default());
        self.chatters.insert(id, (true, Chatters::default()));
        true
    }

    fn remove(&mut self, id: RoomId) {
        self.edit_buffers.remove(&id);
        self.history.remove(&id);
        self.chatters.remove(&id);
    }
}

#[derive(Default)]
pub struct ChatViewState {
    map: HashMap<RoomId, String>,
    state: ChannelState,
    channels: Vec<Index>,
    active: Index,
    tab_bar_hidden: bool,
}

impl ChatViewState {
    pub fn active(&self) -> Index {
        self.active
    }

    pub fn set_active(&mut self, index: Index) {
        if index >= self.channels.len() {
            return;
        }
        self.active = index;
    }

    pub fn next(&mut self) {
        if self.channels.is_empty() {
            return;
        }
        self.active = (self.active + 1) % self.channels.len();
    }

    pub fn previous(&mut self) {
        if self.channels.is_empty() {
            return;
        }

        self.active = (self.active == 0)
            .then_some(self.channels.len())
            .unwrap_or(self.active)
            - 1;
    }

    pub fn toggle_tab_bar(&mut self) {
        self.tab_bar_hidden = !self.tab_bar_hidden;
    }

    pub fn name(&self, id: RoomId) -> Option<&str> {
        self.map.get(&id).map(|c| &**c)
    }

    pub fn chatters_mut(&mut self, id: RoomId) -> Option<&mut (bool, Chatters)> {
        self.state.chatters.get_mut(&id)
    }

    pub fn add_channel(&mut self, id: RoomId, name: impl ToString) {
        if !self.state.add(id) {
            return;
        }

        self.map.insert(id, name.to_string());
        self.channels.push(id);
    }

    pub fn remove_channel_id(&mut self, id: RoomId) -> bool {
        if self.map.remove(&id).is_none() {
            return false;
        }
        self.map.retain(|k, v| *k != id);
        self.state.remove(id);
        self.channels.retain(|&c| c != id);

        true
    }

    pub fn channels(&self) -> impl Iterator<Item = (RoomId, &str)> {
        self.map.iter().map(|(k, v)| (*k, &**v))
    }
}

#[derive(Default)]
pub struct EditBuffer {
    pub buffer: String,
    pub line: Option<String>,
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
            for channel in self.state.channels.iter().copied() {
                if ui
                    .selectable_label(channel == self.state.active, &self.state.map[&channel])
                    .clicked()
                {
                    self.state.active = channel
                }
            }
        });
    }
}

struct ChatView<'a> {
    state: &'a mut AppState,
}

impl<'a> ChatView<'a> {
    fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }

    fn display(self, ui: &mut egui::Ui) {
        let cvs = &mut self.state.state.chat_view_state;
        let active = cvs.active();

        let buf = match cvs.state.edit_buffers.get_mut(&active) {
            Some(buf) => buf,
            None => panic!("no chat view state for {}", active),
        };

        TopBottomPanel::bottom("input")
            .resizable(false)
            .frame(egui::Frame::none().fill(ui.style().visuals.faint_bg_color))
            .show_inside(ui, |ui| {
                EditBox::new(&mut buf.buffer, &mut buf.line).display(ui);
            });

        if !cvs.tab_bar_hidden {
            // TODO allow moving from top/left/bottom/right
            TopBottomPanel::top("tab_bar")
                .resizable(false)
                .frame(egui::Frame::none().fill(ui.style().visuals.faint_bg_color))
                .show_inside(ui, |ui| {
                    TabView::new(cvs).display(ui);
                });
        }

        let (show, chatters) = &self.state.state.chat_view_state.state.chatters[&active];
        if *show {
            SidePanel::right("user_list")
                // .resizable(false)
                .frame(egui::Frame::none().fill(ui.style().visuals.faint_bg_color))
                .show_inside(ui, |ui| {
                    UserList::new(chatters, &self.state.state).display(ui);
                });
        }
    }
}

struct UserList<'a> {
    chatters: &'a Chatters,
    state: &'a State,
}

impl<'a> UserList<'a> {
    fn new(chatters: &'a Chatters, state: &'a State) -> Self {
        Self { chatters, state }
    }

    fn get_image(&self, kind: Kind) -> Option<&RetainedImage> {
        self.state
            .images
            .get(&kind.as_str()[..kind.as_str().len() - 1])
    }

    fn display(self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            for (kind, chatters) in &self.chatters.chatters {
                let id = ui.make_persistent_id(kind.as_str());

                let mut state = CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    !matches!(kind, Kind::Viewer),
                );

                let header = ui
                    .scope(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;

                        ui.horizontal(|ui| {
                            if let Some(img) = self.get_image(*kind) {
                                img.show_size(ui, vec2(8.0, 8.0));
                            }

                            ui.add(
                                Label::new(
                                    RichText::new(kind.as_str())
                                        .color(ui.style().visuals.strong_text_color())
                                        .small(),
                                )
                                .wrap(false)
                                .sense(Sense::click()),
                            )
                        })
                        .inner
                    })
                    .inner;

                if header.clicked() {
                    state.toggle(ui);
                }

                state.show_body_unindented(ui, |ui| {
                    for chatter in chatters {
                        ui.add(Label::new(RichText::new(chatter).small()).wrap(false));
                    }
                });
            }
        });
    }
}

struct EditBox<'a> {
    buffer: &'a mut String,
    line: &'a mut Option<String>,
}

impl<'a> EditBox<'a> {
    fn new(buffer: &'a mut String, line: &'a mut Option<String>) -> Self {
        Self { buffer, line }
    }

    fn display(self, ui: &mut egui::Ui) {
        let resp = ui.add(
            TextEdit::singleline(self.buffer)
                .frame(false)
                .lock_focus(true),
        );

        if resp.lost_focus() && ui.ctx().input().key_down(egui::Key::Enter) {
            let line = std::mem::take(self.buffer);
            eprintln!("{}", line.escape_debug());
            self.line.replace(line);
        }

        resp.request_focus();
    }
}
