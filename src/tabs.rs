use egui::{
    containers::ScrollArea, text::LayoutJob, vec2, Color32, Frame, Label, Layout, Response,
    RichText, Sense, Widget,
};

use crate::{
    helix::{CachedImages, Chatters, Kind},
    twitch::TextKind,
    ChatLayout,
};

pub enum Line {
    Twitch { line: crate::TwitchLine },
    Status { msg: LayoutJob },
}

pub struct Tabs {
    tabs: Vec<Tab>,
    active: usize,
}

impl Tabs {
    pub fn create() -> Self {
        Self {
            tabs: vec![Tab::new("*status")],
            active: 0,
        }
    }

    pub fn tabs(&self) -> impl Iterator<Item = &Tab> + ExactSizeIterator {
        self.tabs.iter().skip(1)
    }

    pub fn tabs_mut(&mut self) -> impl Iterator<Item = &mut Tab> + ExactSizeIterator {
        self.tabs.iter_mut().skip(0)
    }

    pub fn next_tab(&mut self) {
        self.active = (self.active + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        self.active = self.active.checked_sub(1).unwrap_or(self.tabs.len() - 1)
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }

        self.active = index;
    }

    pub fn set_active_by_name(&mut self, key: &str) {
        if let Some(pos) = self.tabs.iter().position(|tab| tab.title == key) {
            self.active = pos;
        }
    }

    pub fn active(&self) -> &Tab {
        &self.tabs[self.active]
    }

    pub fn remove_tab(&mut self, key: &str) {
        if let Some(pos) = self.tabs.iter().position(|k| k.title == key) {
            self.tabs.remove(pos);
            self.active -= 1;
            self.next_tab();
        }
    }

    pub fn get_mut(&mut self, key: &str) -> &mut Tab {
        if !self.tabs.iter().any(|tab| tab.title == key) {
            self.tabs.push(Tab::new(key));
        }

        self.tabs
            .iter_mut()
            .find(|tab| tab.title == key)
            .expect("tab to exist")
    }

    pub fn active_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active]
    }

    pub fn tab_color(&self, index: usize) -> Color32 {
        if self.active == index {
            Color32::LIGHT_RED
        } else {
            Color32::WHITE
        }
    }

    pub fn is_active(&self, index: usize) -> bool {
        self.active == index
    }
}

impl egui::Widget for &mut Tabs {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let active = self.active;
            for (index, tab) in self.tabs.iter().enumerate() {
                let text = RichText::new(&tab.title).color(self.tab_color(index));
                if ui.selectable_label(self.is_active(index), text).clicked() {
                    self.active = index;
                }

                // TODO close button
            }
        })
        .response
    }
}

pub struct Tab {
    title: String,
    buffer: String,
    show_user_list: bool,
    timestamp: bool,
    line_mode: ChatLayout,
    queue: crate::Queue<Line>,
    chatters: Chatters,
}

impl Tab {
    pub fn new(title: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            buffer: String::with_capacity(100),
            queue: crate::Queue::with_capacity(100),
            show_user_list: true,
            timestamp: true,
            line_mode: ChatLayout::Traditional,
            chatters: Chatters::default(),
        }
    }

    pub fn buffer_mut(&mut self) -> &mut String {
        &mut self.buffer
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn showing_user_list(&self) -> bool {
        self.show_user_list
    }

    pub fn showing_timestamp(&self) -> bool {
        self.timestamp
    }

    pub fn showing_user_list_mut(&mut self) -> &mut bool {
        &mut self.show_user_list
    }

    pub fn showing_timestamp_mut(&mut self) -> &mut bool {
        &mut self.timestamp
    }

    pub fn line_mode(&self) -> ChatLayout {
        self.line_mode
    }

    pub fn next_line_mode(&mut self) {
        self.line_mode.cycle()
    }

    pub fn toggle_user_list(&mut self) {
        self.show_user_list = !self.show_user_list
    }

    pub fn toggle_timestamps(&mut self) {
        self.timestamp = !self.timestamp
    }

    pub fn append(&mut self, line: Line) {
        self.queue.push(line)
    }

    pub fn update_chatters(&mut self, chatters: Chatters) {
        self.chatters = chatters;
    }

    pub fn entries(&self) -> impl Iterator<Item = &Line> + ExactSizeIterator + DoubleEndedIterator {
        self.queue.iter()
    }

    pub fn as_chatters<'a>(&'a self, cached: &'a CachedImages) -> impl Widget + 'a {
        ChatterList {
            chatters: &self.chatters,
            cached_images: cached,
        }
    }

    pub fn as_widget<'a>(&'a self, line: &'a Line) -> impl Widget + 'a {
        LineWidget {
            line,
            tab: self,
            // cached_images: cached,
        }
    }
}

struct ChatterList<'a> {
    chatters: &'a Chatters,
    cached_images: &'a CachedImages,
}

impl<'a> egui::Widget for ChatterList<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Frame::none()
            .show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (kind, chatters) in &self.chatters.chatters {
                        let id = ui.make_persistent_id(kind.as_str());

                        let mut state =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                id,
                                !matches!(kind, Kind::Viewer),
                            );

                        let header = ui
                            .scope(|ui| {
                                ui.spacing_mut().item_spacing.x = 2.0;

                                ui.horizontal(|ui| {
                                    if let Some(id) = self.cached_images.id_map.get(kind) {
                                        if let Some(img) = self.cached_images.map.get(id) {
                                            img.show_size(ui, vec2(8.0, 8.0));
                                        }
                                    }

                                    ui.add(
                                        Label::new(
                                            RichText::new(kind.as_str())
                                                .color(Color32::WHITE)
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
                                ui.small(chatter);
                            }
                        });
                    }
                })
            })
            .response
    }
}

struct LineWidget<'a> {
    line: &'a Line,
    tab: &'a Tab,
    // cached_images: &'a CachedImages,
}

impl<'a> egui::Widget for LineWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let line = match self.line {
            Line::Twitch { line } => line,
            Line::Status { msg } => return ui.label(msg.clone()),
        };

        let ts = self
            .tab
            .timestamp
            .then(|| Label::new(RichText::new(line.timestamp.as_str()).weak()));

        let sender = Label::new(RichText::new(&line.sender).color(line.color));

        let data: Box<dyn FnOnce(&mut egui::Ui) -> Response> =
            if !line.spans.iter().any(|c| matches!(c, TextKind::Emote(..))) {
                Box::new(move |ui: &mut egui::Ui| {
                    ui.add(Label::new(RichText::new(&line.data).color(Color32::WHITE)).wrap(true))
                }) as _
            } else {
                Box::new(move |ui: &mut egui::Ui| {
                    ui.add(Label::new(RichText::new(&line.data).color(Color32::WHITE)).wrap(true))
                }) as _

                // let font_id = TextStyle::Body.resolve(&*ui.style());
                // Box::new(move |ui: &mut egui::Ui| ui.small("asdf")) as _
            };

        // let job = line
        //     .spans
        //     .iter()
        //     .fold(LayoutJob::default(), |mut layout, kind| match kind {
        //         TextKind::Emote(id) => {
        //             let id = self.cached_images.emote_map[id];
        //             let img = &self.cached_images.map[&id];

        //             todo!();
        //         }
        //         TextKind::Text(text) => layout.simple(text, font_id.clone(), Color32::WHITE),
        //     });

        match self.tab.line_mode {
            ChatLayout::Traditional => {
                ui.horizontal(|ui| {
                    if let Some(ts) = ts {
                        ui.add(ts);
                    }
                    ui.add(sender);
                    ui.add(data);
                })
                .response
            }
            ChatLayout::Modern => {
                ui.vertical(|ui| {
                    ui.horizontal_top(|ui| {
                        ui.add(sender);
                        if let Some(ts) = ts {
                            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                                ui.add(ts)
                            });
                        }
                    });

                    ui.add(data);
                    ui.separator();
                })
                .response
            }
        }
    }
}
