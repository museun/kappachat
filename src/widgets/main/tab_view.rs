use egui::{
    pos2, vec2, Color32, CursorIcon, Id, Label, PointerButton, Rect, Response, Rounding, Sense,
    Stroke, Vec2,
};
use egui_extras::RetainedImage;

use crate::{fetch::ImageKind, store::Image, Channel, FetchQueue, ImageCache};

use super::{ChatViewState, Position};

pub struct TabView<'a> {
    images: &'a mut ImageCache,
    state: &'a mut ChatViewState,
    fetch: &'a mut FetchQueue<Image>,
    channels: &'a mut Vec<Channel>,
    dark_mask: &'a RetainedImage,
    bottom_right: Vec2,
}

impl<'a> TabView<'a> {
    pub fn new(
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

    pub fn display(self, ui: &mut egui::Ui) {
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
