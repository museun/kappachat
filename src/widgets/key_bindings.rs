use egui::{Align, ComboBox, Frame, Grid, Label, Layout, RichText, Sense};

use crate::{KeyHelper, KeyMapping};

pub struct KeyBindings<'a> {
    mapping: &'a mut KeyMapping,
}

impl<'a> KeyBindings<'a> {
    pub fn new(mapping: &'a mut KeyMapping) -> Self {
        Self { mapping }
    }
}

impl<'a> egui::Widget for KeyBindings<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Frame::none()
            .show(ui, |ui| {
                Grid::new("key_bindings")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (action, chords) in self.mapping.reverse_mapping() {
                            ui.add(
                                Label::new(RichText::new(action.display()).monospace())
                                    .sense(Sense::click()),
                            )
                            .on_hover_text_at_pointer(action.help())
                            .context_menu(|ui| {
                                if ui.small_button("➕ add keybinding").clicked() {
                                    // TODO add keybinding
                                    ui.close_menu()
                                }
                                if ui.small_button("🔄 reset to default").clicked() {
                                    // TODO reset to default
                                    ui.close_menu()
                                }
                            });

                            ui.vertical(|ui| {
                                for chord in chords {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.horizontal(|ui| {
                                            ComboBox::from_id_source(chord.display())
                                                .selected_text(chord.display_key())
                                                .show_index(
                                                    ui,
                                                    &mut KeyHelper::keys()
                                                        .iter()
                                                        .position(|(name, key)| *key == chord.key())
                                                        .unwrap_or(0),
                                                    KeyHelper::keys().len(),
                                                    |i| KeyHelper::keys()[i].0.to_string(),
                                                )
                                                .context_menu(|ui| {
                                                    if ui
                                                        .small_button("❌ remove keybinding")
                                                        .clicked()
                                                    {
                                                        // TODO remove keybinding
                                                        ui.close_menu()
                                                    }
                                                    if ui
                                                        .small_button("🔄 reset to default")
                                                        .clicked()
                                                    {
                                                        // TODO reset to default
                                                        ui.close_menu()
                                                    }
                                                });
                                            // TODO actually update these
                                            ui.toggle_value(chord.ctrl(), "Ctrl");
                                            ui.toggle_value(chord.alt(), "Alt");
                                            ui.toggle_value(chord.shift(), "Shift");
                                        });
                                    });
                                }
                            });

                            ui.end_row()
                        }
                    })
            })
            .response
    }
}
