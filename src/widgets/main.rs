use std::collections::{HashMap, HashSet};

use eframe::epaint::Pos2;
use egui::{
    pos2, vec2, CentralPanel, Color32, CursorIcon, Frame, Id, Label, PointerButton, Rect, Response,
    RichText, Rounding, ScrollArea, Sense, SidePanel, Stroke, TextEdit, TextStyle, TopBottomPanel,
    Vec2,
};
use egui_extras::RetainedImage;
use time::OffsetDateTime;

use crate::{
    fetch::ImageKind,
    helix::{Chatters, Kind},
    state::AppState,
    store::Image,
    twitch::{self, EmoteSpan},
    Channel, FetchQueue, ImageCache, Queue,
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

    pub fn display(mut self, ctx: &egui::Context) {
        match self.state.state.view_state.current_view {
            MainView::Start => {
                self.display_main(ctx)

                // CentralPanel::default().show(ctx, |ui| {
                //     self.display_start(ui);
                // });
            }
            MainView::Settings => {
                CentralPanel::default().show(ctx, |ui| {
                    self.display_settings(ui);
                });
            }
            MainView::Main => {
                // CentralPanel::default().show(ctx, |ui| {
                //     self.display_start(ui);
                // });
                self.display_main(ctx)
            }
        }
    }

    fn display_main(&mut self, ctx: &egui::Context) {
        let writer = self.state.writer.clone();
        ChatView::new(self.state, writer).display(ctx);
    }

    fn display_settings(&mut self, ui: &mut egui::Ui) {
        if SettingsView::new(self.state).display(ui) {
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
    const fn new(
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
                        ui.small(format!(
                            "{} ago",
                            crate::format_seconds(s.whole_seconds() as _)
                        ));
                    });
            }

            ui.scope(|ui| {
                let width = ui
                    .fonts()
                    .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
                ui.spacing_mut().item_spacing.x = width;

                // if let Some((badge, version)) = pm.badges().next() {
                //     if let Some(img) = self.cache.get(badge) {
                //         img.show_size(ui, vec2(8.0, 8.0));
                //         // .on_hover_text_at_pointer(self.emote_map.get(badge).unwrap());
                //     }
                // }

                ui.colored_label(pm.color(), pm.sender);

                for spans in &self.line.spans {
                    match spans {
                        EmoteSpan::Emote(s) =>
                        // match self.cache.get(s) {
                            // Some(img) => {
                            //     img.show_size(ui, vec2(16.0, 16.0))
                            //         .on_hover_text_at_pointer(self.emote_map.get(s).unwrap());
                            // }
                            // None => {
                               { ui.add(Label::new(s));}
                            // }
                        // },
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

#[derive(Clone)]
pub struct Timestamp {
    pub(crate) date_time: OffsetDateTime, // why?
    pub(crate) repr: String,
}

impl Timestamp {
    pub fn now_local() -> Self {
        let date_time = OffsetDateTime::now_local().expect("valid time");
        let repr = date_time.format(&crate::FORMAT).expect("valid time");
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

#[derive(Default)]
pub struct EditBuffer {
    pub buffer: String,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Position {
    Top,
    Right,
    Bottom,
    #[default]
    Left,
}

impl Position {
    const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    const fn is_vertical(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    fn rect(&self, size: f32, max: Vec2) -> Rect {
        match self {
            Self::Top => Rect::from_min_size(pos2(0.0, 0.0), max),
            Self::Bottom => Rect::from_min_size(pos2(0.0, max.y), max),
            Self::Left => Rect::from_min_size(pos2(0.0, 0.0), max),
            Self::Right => Rect::from_min_size(pos2(max.x, 0.0), max),
        }
    }

    // BUG why is this different from `rect`?
    fn rects(size: f32, max: Vec2) -> [(Self, Rect); 4] {
        use Position::*;

        let right = vec2(max.x, size);
        let bottom = vec2(size, max.y);

        [
            (Top, Rect::from_min_size(Pos2::ZERO, right)),
            (Right, Rect::from_min_size(pos2(max.x - size, 0.0), bottom)),
            (Bottom, Rect::from_min_size(pos2(0.0, max.y - size), right)),
            (Left, Rect::from_min_size(Pos2::ZERO, bottom)),
        ]
    }

    const fn as_side(&self) -> Option<egui::panel::Side> {
        Some(match self {
            Self::Right => egui::panel::Side::Right,
            Self::Left => egui::panel::Side::Left,
            _ => return None,
        })
    }

    const fn as_top_bottom(&self) -> Option<egui::panel::TopBottomSide> {
        Some(match self {
            Self::Bottom => egui::panel::TopBottomSide::Bottom,
            Self::Top => egui::panel::TopBottomSide::Top,
            _ => return None,
        })
    }

    const fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Right => "Right",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
        }
    }
}

struct TabView<'a> {
    images: &'a mut ImageCache,
    state: &'a mut ChatViewState,
    fetch: &'a mut FetchQueue<Image>,
    channels: &'a mut Vec<Channel>,
    dark_mask: &'a RetainedImage,
    bottom_right: Vec2,
}

impl<'a> TabView<'a> {
    fn new(
        images: &'a mut ImageCache,
        state: &'a mut ChatViewState,
        fetch: &'a mut FetchQueue<Image>,
        channels: &'a mut Vec<Channel>,
        dark_mask: &'a RetainedImage,
        bottom_right: Vec2,
    ) -> Self {
        Self {
            images,
            state,
            fetch,
            bottom_right,
            dark_mask,
            channels,
        }
    }

    fn display(self, ui: &mut egui::Ui) {
        for (i, channel) in self.channels.iter().enumerate() {
            let img = match self.images.get_id(channel.image_id) {
                Some(img) => img,
                None => {
                    self.fetch.fetch(Image {
                        id: channel.image_id,
                        kind: ImageKind::Display,
                        url: channel.profile_image_url.clone(),
                        meta: (),
                    });
                    continue;
                }
            };

            let resp = img.show_max_size(ui, vec2(self.state.image_size, self.state.image_size));
            let resp = ui.interact(resp.rect, resp.id, Sense::click_and_drag());

            if Some(i) != self.state.active {
                ui.painter().rect_filled(
                    resp.rect,
                    Rounding::none(),
                    Color32::from_black_alpha(0xF0),
                );
            }

            if self.state.show_mask {
                ui.put(resp.rect, |ui: &mut egui::Ui| {
                    self.dark_mask
                        .show_max_size(ui, vec2(self.state.image_size, self.state.image_size))
                });
            }

            if resp.hovered() && !resp.dragged() {
                ui.painter().rect(
                    resp.rect,
                    Rounding::none(),
                    Color32::TRANSPARENT,
                    ui.style().visuals.selection.stroke,
                );
            }

            let resp = resp.on_hover_ui_at_pointer(|ui| {
                ui.vertical(|ui| {
                    ui.monospace(&channel.display_name);

                    if ui.ctx().input().modifiers.shift {
                        if !channel.description.is_empty() {
                            ui.set_max_width(self.bottom_right.x * 0.75);
                            ui.separator();
                            ui.add(Label::new(&channel.description).wrap(true));
                        }
                        if ui.ctx().input().modifiers.ctrl {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("user id:");
                                ui.monospace(channel.id.to_string());
                            });
                        }
                    }
                });
            });

            if resp.dragged_by(PointerButton::Primary) && ui.input().modifiers.command_only() {
                ui.output().cursor_icon = CursorIcon::Grab;
                ui.data()
                    .insert_temp(Id::new("tab_being_dragged"), (i, resp.clone()));
            }

            if resp.clicked() {
                self.state.active.replace(i);
            }
        }

        if !ui.input().modifiers.command_only() {
            return;
        }

        let (id, resp) = match ui
            .data()
            .get_temp::<(usize, Response)>(Id::new("tab_being_dragged"))
        {
            Some((id, resp)) => (id, resp),
            None => return,
        };

        if !resp.dragged_by(PointerButton::Primary) {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                if resp.rect.contains(mouse_pos) {
                    let mut data = ui.data();
                    data.remove::<(usize, Response)>(Id::new("tab_being_dragged"));
                    data.remove::<usize>(Id::new("drag_tab_target"));
                    return;
                }
            }

            if let Some(next) = ui.data().get_temp::<usize>(Id::new("drag_tab_target")) {
                if next != id {
                    let channel = self.channels.remove(id);
                    self.channels.insert(next, channel);
                    self.state.active.replace(next);
                }
            }

            let mut data = ui.data();
            data.remove::<(usize, Response)>(Id::new("tab_being_dragged"));
            data.remove::<usize>(Id::new("drag_tab_target"));

            return;
        }

        let mouse_pos = match resp.interact_pointer_pos() {
            Some(mouse_pos) => mouse_pos,
            _ => return,
        };

        let line = self
            .state
            .tab_bar_position
            .rect(self.state.image_size, self.bottom_right);

        let Vec2 { x, y } = ui.spacing().item_spacing;
        let spacing = self.state.image_size
            + self
                .state
                .tab_bar_position
                .is_horizontal()
                .then_some(x)
                .unwrap_or(y);

        let x = line.left()
            - matches!(self.state.tab_bar_position, Position::Right)
                .then_some(self.state.image_size)
                .unwrap_or(0.0);

        let y = line.top()
            - matches!(self.state.tab_bar_position, Position::Bottom)
                .then_some(self.state.image_size)
                .unwrap_or(0.0);

        for i in 0..self.channels.len() {
            if i == id {
                ui.ctx().debug_painter().rect_stroke(
                    resp.rect,
                    Rounding::default(),
                    Stroke::new(1.0, Color32::RED),
                );
                continue;
            }

            let spacing = if i == 0 { 0.0 } else { spacing } * (i as f32);
            let offset = if self.state.tab_bar_position.is_horizontal() {
                pos2(spacing, y)
            } else {
                pos2(x, spacing)
            };

            let bb = Rect::from_min_size(
                offset,
                vec2(
                    self.state.image_size, //
                    self.state.image_size,
                ),
            );
            if !ui.rect_contains_pointer(bb) {
                continue;
            }

            let mut offset = spacing + if i > id { self.state.image_size } else { 0.0 };

            if self.state.tab_bar_position.is_horizontal() {
                if mouse_pos.x < (bb.left() / 2.0) {
                    offset -= spacing;
                }
                ui.ctx().debug_painter().vline(
                    offset,
                    y + 0.0..=y + self.state.image_size,
                    Stroke::new(3.0, Color32::GREEN),
                );
            } else {
                if mouse_pos.y < (bb.top() / 2.0) {
                    offset -= spacing;
                }
                ui.ctx().debug_painter().hline(
                    x + 0.0..=x + self.state.image_size,
                    offset,
                    Stroke::new(3.0, Color32::GREEN),
                );
            }

            ui.data().insert_temp(Id::new("drag_tab_target"), i);
        }
    }
}

struct TabBar {
    side: Position,
    width: f32,
}

impl TabBar {
    const fn new(side: Position, width: f32) -> Self {
        Self { side, width }
    }

    fn display(
        self,
        ctx: &egui::Context,
        hash_source: impl std::hash::Hash,
        body: impl FnOnce(&mut egui::Ui),
    ) -> (Id, egui::Rect) {
        let frame = Frame::none().fill(ctx.style().visuals.faint_bg_color);
        let range = self.width..=self.width;

        let id = Id::new(hash_source);

        // TODO enable scroll but disable scroll bars
        let resp = match (self.side.as_side(), self.side.as_top_bottom()) {
            (None, Some(top_bottom)) => {
                TopBottomPanel::new(top_bottom, id.with("tab_bar"))
                    .resizable(false)
                    .frame(frame)
                    .height_range(range)
                    // TODO this style exists: pub scroll_bar_width: f32,
                    .show(ctx, |ui| {
                        ui.horizontal(body);
                    })
                    .response
            }
            (Some(side), None) => {
                SidePanel::new(side, id.with("tab_bar"))
                    .resizable(false)
                    .frame(frame)
                    .width_range(range)
                    // TODO this style exists: pub scroll_bar_width: f32,
                    .show(ctx, |ui| {
                        ui.vertical(body);
                    })
                    .response
            }
            _ => unreachable!(),
        };

        (id, resp.rect)
    }
}

struct ChatView<'a> {
    state: &'a mut AppState,
    writer: flume::Sender<String>,
}

impl<'a> ChatView<'a> {
    fn new(state: &'a mut AppState, writer: flume::Sender<String>) -> Self {
        Self { state, writer }
    }

    fn display(self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            let cvs = &mut self.state.state.chat_view_state;

            let (id, rect) = TabBar::new(cvs.tab_bar_position, cvs.image_size).display(
                ctx,
                "main_tab_bar",
                |ui: &mut egui::Ui| {
                    TabView::new(
                        &mut self.state.state.images,
                        cvs,
                        &mut self.state.runtime.fetch,
                        &mut self.state.state.channels,
                        &self.state.dark_image_mask,
                        self.state.state.window_size,
                    )
                    .display(ui);
                },
            );

            let resp = ui.interact(rect, id, Sense::click_and_drag());

            if resp.dragged_by(PointerButton::Primary) && ui.input().modifiers.shift_only() {
                ui.output().cursor_icon = CursorIcon::Grab;

                let mouse_pos = ui.input().pointer.hover_pos().unwrap();

                let landing = Position::rects(cvs.image_size, self.state.state.window_size);
                let distance = landing.map(|(_, rect)| rect.signed_distance_to_pos(mouse_pos));

                if let Some((pos, rect)) = distance
                    .into_iter()
                    .enumerate()
                    .find_map(|(i, c)| (c < cvs.image_size * 0.5).then_some(i))
                    .map(|index| landing[index])
                {
                    ui.data().insert_temp(Id::new("tab_bar_drag_pos"), pos);

                    // don't show the ghost if we're already showing the real thing
                    if pos == cvs.tab_bar_position {
                        return;
                    }

                    ctx.move_to_top(ui.layer_id());

                    let (id, rect) = (TabBar {
                        side: pos,
                        width: cvs.image_size,
                    })
                    .display(ctx, "temp_tab_bar", |ui: &mut egui::Ui| {
                        TabView::new(
                            &mut self.state.state.images,
                            cvs,
                            &mut self.state.runtime.fetch,
                            &mut self.state.state.channels,
                            &self.state.dark_image_mask,
                            self.state.state.window_size,
                        )
                        .display(ui);
                    });

                    // this is the ghost
                    ui.painter().rect(
                        rect,
                        Rounding::none(),
                        Color32::from_black_alpha(0x99),
                        ui.style().visuals.selection.stroke,
                    );
                }
            }

            if resp.drag_released() {
                let mut data = ui.data();
                cvs.tab_bar_position = data
                    .get_temp(Id::new("tab_bar_drag_pos"))
                    .unwrap_or(cvs.tab_bar_position);
                data.remove::<Position>(Id::new("tab_bar_drag_pos"));
            }
        });

        // let state = match cvs.active_mut() {
        //     Some(state) => state,
        //     None => return,
        // };

        // let buf = &mut state.buffer;

        // TopBottomPanel::bottom("input")
        //     .resizable(false)
        //     .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
        //     .show_inside(ui, |ui| {
        //         ui.with_layout(
        //             Layout::centered_and_justified(Direction::LeftToRight),
        //             |ui| {
        //                 EditBox::new(&mut buf.buffer, &self.writer).display(ui);
        //             },
        //         );
        //     });

        // if let Some(channel) = self
        //     .state
        //     .state
        //     .channels
        //     .iter_mut()
        //     .find(|c| ChatViewState::is_same_channel(&c.login, &state.channel))
        // {
        //     if channel.show_user_list {
        //         SidePanel::right("user_list")
        //             .frame(Frame::none().fill(ui.style().visuals.faint_bg_color))
        //             .show_inside(ui, |ui| {
        //                 UserList::new(&state.chatters, &self.state.state.images).display(ui);
        //             });
        //     }
        // }

        //     let show_timestamp = self
        //     .state
        //     .state
        //     .channels
        //     .iter()
        //     .find_map(|c| (c.login == state.name()).then_some(c.show_timestamps))
        //     .unwrap_or(true);

        // ScrollArea::vertical()
        //     .auto_shrink([false, false])
        //     .stick_to_bottom(true) // TODO if we're scrolled up don't do this
        //     .show(ui, |ui| {
        //         for line in state.lines.iter() {
        //             match line {
        //                 Line::Chat(line) => {
        //                     ChatLineView::new(
        //                         line,
        //                         &self.state.state.images,
        //                         &self.state.state.emote_map,
        //                         show_timestamp,
        //                     )
        //                     .display(ui);
        //                 }
        //             }
        //         }
        //     });
    }
}

struct UserList<'a> {
    chatters: &'a Chatters,
    images: &'a ImageCache,
}

impl<'a> UserList<'a> {
    const fn new(chatters: &'a Chatters, images: &'a ImageCache) -> Self {
        Self { chatters, images }
    }

    fn get_image(&self, kind: Kind) -> Option<&RetainedImage> {
        None
        // self.images.get(&kind.as_str()[..kind.as_str().len() - 1])
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
