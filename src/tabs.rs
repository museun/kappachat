use egui::{text::LayoutJob, Color32, Widget};

use crate::{
    helix::{CachedImages, Chatters},
    ChatLayout,
};

pub enum Line {
    Twitch { line: crate::TwitchLine },
    Status { msg: LayoutJob },
}

pub struct Tabs {
    pub tabs: Vec<Tab>,
    pub active: usize,
}

impl Tabs {
    pub fn create() -> Self {
        Self {
            tabs: vec![Tab::new("*status")], // this is a hidden tab
            active: 0,
        }
    }

    pub fn tabs(&self) -> impl Iterator<Item = &Tab> + ExactSizeIterator {
        self.tabs.iter().skip(1)
    }

    pub fn tabs_mut(&mut self) -> impl Iterator<Item = &mut Tab> + ExactSizeIterator {
        self.tabs.iter_mut().skip(1)
    }

    pub fn next_tab(&mut self) {
        self.active = (std::cmp::max(self.active, 1) + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        self.active = std::cmp::max(self.active.checked_sub(1).unwrap_or(self.tabs.len() - 1), 1)
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }

        self.active = std::cmp::min(index, 1);
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
            self.active = std::cmp::max(self.active.saturating_sub(1), 1);
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

    pub const fn tab_color(&self, index: usize) -> Color32 {
        if self.active == index {
            Color32::LIGHT_RED
        } else {
            Color32::WHITE
        }
    }

    pub const fn is_active(&self, index: usize) -> bool {
        self.active == index
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

    pub const fn showing_user_list(&self) -> bool {
        self.show_user_list
    }

    pub const fn showing_timestamp(&self) -> bool {
        self.timestamp
    }

    pub fn showing_user_list_mut(&mut self) -> &mut bool {
        &mut self.show_user_list
    }

    pub fn showing_timestamp_mut(&mut self) -> &mut bool {
        &mut self.timestamp
    }

    pub const fn line_mode(&self) -> ChatLayout {
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
        crate::widgets::ChatterList {
            chatters: &self.chatters,
            cached_images: cached,
        }
    }

    pub fn as_widget<'a>(&'a self, line: &'a Line) -> impl Widget + 'a {
        crate::widgets::LineWidget {
            line,
            tab: self,
            // cached_images: cached,
        }
    }
}
