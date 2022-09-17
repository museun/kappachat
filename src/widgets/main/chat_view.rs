use egui::{CentralPanel, Color32, CursorIcon, Id, PointerButton, Rounding, Sense};

use crate::state::AppState;

use super::{Position, TabBar, TabView};

pub struct ChatView<'a> {
    state: &'a mut AppState,
    writer: flume::Sender<String>,
}

impl<'a> ChatView<'a> {
    pub fn new(state: &'a mut AppState, writer: flume::Sender<String>) -> Self {
        Self { state, writer }
    }

    pub fn display(self, ctx: &egui::Context) {
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

                    let (id, rect) = TabBar::new(pos, cvs.image_size).display(
                        ctx,
                        "temp_tab_bar",
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
