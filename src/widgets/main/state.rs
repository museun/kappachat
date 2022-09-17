use crate::{
    helix::Chatters,
    twitch::{self, EmoteSpan},
    Queue,
};

use super::{ChatLine, Position, Timestamp};

#[derive(Default)]
pub struct EditBuffer {
    pub buffer: String,
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

pub struct ChatViewState {
    pub channels: Vec<ChannelState>, // BUG what is this
    pub active: Option<usize>,
    pub tab_bar_hidden: bool,

    pub tab_bar_position: Position,
    pub image_size: f32,
    pub show_mask: bool,
}

impl Default for ChatViewState {
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            active: None,
            tab_bar_hidden: false,
            image_size: 32.0,
            tab_bar_position: Position::Top,
            show_mask: false,
        }
    }
}

impl ChatViewState {
    pub const fn active_index(&self) -> Option<usize> {
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
