use egui::{Align, ComboBox, Frame, Grid, Key, Label, Layout, RichText, Sense, TextEdit};

use crate::{font_icon::ADD, Chord, KeyAction, KeyHelper, KeyMapping};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum KeyBindingsView {
    #[default]
    Input,
    Grid,
    Raw,
}

#[derive(Default)]
pub struct KeyBindingsState {
    buffer: String,
    view: KeyBindingsView,
    changed: bool,
    selected: usize,

    key_input: KeyInput,
    editing: Option<(KeyBindingsView, KeyAction, usize)>,
}

#[derive(Default)]
struct KeyInput {
    buffer: String,
    modifiers: egui::Modifiers,
    key: Option<egui::Key>,
    chord: Option<Chord>,

    recording: bool,
    changed: bool,
}

impl KeyInput {
    fn update_modifiers(&mut self, modifiers: egui::Modifiers) {
        if !self.recording || self.modifiers == modifiers {
            return;
        }

        self.changed = true;
        self.modifiers = modifiers;
        self.key.take();
    }

    fn update_key(&mut self, key: egui::Key) {
        const EXCLUDED_KEYS: &[egui::Key] = &[egui::Key::Enter];
        if !self.recording || EXCLUDED_KEYS.contains(&key) {
            return;
        }

        if self.key == Some(key) {
            return;
        }

        self.changed = true;
        self.key.replace(key);
    }

    fn display(&mut self) {
        if !self.changed {
            return;
        }

        if let Some(key) = self.key {
            let chord = Chord::from((self.modifiers, key));
            self.chord.replace(chord);
            self.buffer = chord.display();
            self.changed = true;
        } else {
            self.buffer.clear();
            Chord::display_modifiers(&mut self.buffer, &self.modifiers);
            self.changed = true;
        }
    }
}

pub struct KeyBindings<'a> {
    mapping: &'a mut KeyMapping,
    state: &'a mut KeyBindingsState,
}

impl<'a> KeyBindings<'a> {
    pub fn new(mapping: &'a mut KeyMapping, state: &'a mut KeyBindingsState) -> Self {
        Self { mapping, state }
    }

    fn display_label(action: &KeyAction, ui: &mut egui::Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.add(Label::new(RichText::new(action.display()).monospace()).sense(Sense::click()))
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
        });
    }

    fn capture_input(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.state.key_input.recording {
            return false;
        }

        self.state
            .key_input
            .update_modifiers(ui.ctx().input().modifiers);

        let mut done = false;
        for (key, modifiers) in ui
            .ctx()
            .input()
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                } if !pressed => Some((key, modifiers)),

                _ => None,
            })
        {
            done ^= *key == Key::Enter;

            self.state.key_input.update_modifiers(*modifiers);
            self.state.key_input.update_key(*key);
        }

        self.state.key_input.display();

        done
    }

    fn display_input(&mut self, ui: &mut egui::Ui) {
        // TODO: consume key
        // TODO: if capturing, consume esc (and cancel edit)
        // TODO: clear input if we activate a different edit
        // TODO: center align the edit
        // TODO: also right-click menu to enable edit
        // TODO: more excluded keys (non-prefixed: enter, esc, backspace)
        if self.capture_input(ui) {
            if let Some((_, action, index)) = self.state.editing.take() {
                self.state.buffer.clear();
                let input = std::mem::take(&mut self.state.key_input);
                if let Some(c) = self
                    .mapping
                    .reverse_mapping()
                    .iter_mut()
                    .flat_map(|(act, chords)| (*act == action).then_some(chords))
                    .flatten()
                    .nth(index)
                {
                    *c = input.chord.expect("chord should be set");
                    self.mapping.update_from_reverse();
                }
            }
        }

        Grid::new("key_bindings_input")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (action, chords) in self.mapping.reverse_mapping() {
                    Self::display_label(action, ui);

                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            for (i, chord) in chords.iter().enumerate() {
                                if i > 0 {
                                    ui.separator();
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if let Some((KeyBindingsView::Input, act, index)) =
                                        self.state.editing
                                    {
                                        if *action == act && index == i {
                                            ui.add(
                                                TextEdit::singleline(
                                                    &mut self.state.key_input.buffer,
                                                )
                                                .interactive(false),
                                            );
                                            return;
                                        }
                                    }

                                    if ui
                                        .add(
                                            Label::new(RichText::new(chord.display()).monospace())
                                                .wrap(false)
                                                .sense(Sense::click()),
                                        )
                                        .double_clicked()
                                    {
                                        self.state.key_input.recording = true;
                                        self.state.editing.replace((
                                            KeyBindingsView::Input,
                                            *action,
                                            i,
                                        ));
                                    }
                                });
                            }
                        });
                    });

                    ui.end_row();
                }
            });
    }

    fn display_grid(&mut self, ui: &mut egui::Ui) {
        Grid::new("key_bindings_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (action, chords) in self.mapping.reverse_mapping() {
                    Self::display_label(action, ui);

                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            for (i, chord) in chords.iter_mut().enumerate() {
                                if i > 0 {
                                    ui.separator();
                                }

                                let rect = ui
                                    .with_layout(Layout::right_to_left(Align::Max), |ui| {
                                        ui.horizontal(|ui| {
                                            match KeyHelper::keys()
                                                .iter()
                                                .position(|&(_, k)| k == chord.key())
                                            {
                                                Some(pos) => self.state.selected = pos,
                                                None => self.state.selected = 0,
                                            };

                                            let resp = ComboBox::from_id_source(chord.display())
                                                .selected_text(chord.display_key())
                                                .show_index(
                                                    ui,
                                                    &mut self.state.selected,
                                                    KeyHelper::keys().len(),
                                                    |i| KeyHelper::keys()[i].0.to_string(),
                                                );

                                            if resp.changed()
                                                || ui.toggle_value(chord.ctrl(), "Ctrl").changed()
                                                || ui.toggle_value(chord.alt(), "Alt").changed()
                                                || ui.toggle_value(chord.shift(), "Shift").changed()
                                            {
                                                chord.set_key(
                                                    KeyHelper::keys()[self.state.selected].1,
                                                );
                                                self.state.changed = true;
                                            }
                                        })
                                    })
                                    .response
                                    .rect;

                                if ui
                                    .ctx()
                                    .input()
                                    .pointer
                                    .button_triple_clicked(egui::PointerButton::Secondary)
                                    && ui
                                        .ctx()
                                        .pointer_interact_pos()
                                        .filter(|pos| rect.contains(*pos))
                                        .is_some()
                                {
                                    eprintln!(
                                        "delete: {} -> {}",
                                        action.display(),
                                        chord.display()
                                    );
                                }
                            }
                        });
                    });

                    ui.end_row()
                }
            });
    }

    fn display_raw(&mut self, ui: &mut egui::Ui) {
        ui.code_editor(&mut self.state.buffer);
    }
}

impl<'a> egui::Widget for KeyBindings<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        use KeyBindingsView::*;

        if self.state.changed {
            self.state.changed = false;
            self.mapping.update_from_reverse();
        }

        Frame::none()
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.state.view, Input, "Input");
                    ui.selectable_value(&mut self.state.view, Grid, "Grid");

                    if ui
                        .selectable_value(&mut self.state.view, Raw, "Raw")
                        .clicked()
                    {
                        self.state.buffer.clear();
                        let bindings = serde_yaml::to_string(&self.mapping).unwrap();
                        self.state.buffer = bindings;
                    }
                });
                ui.separator();

                match self.state.view {
                    Input => self.display_input(ui),
                    Grid => self.display_grid(ui),
                    Raw => self.display_raw(ui),
                }
            })
            .response
    }
}
