use std::borrow::Cow;

use egui::{
    style::Margin, Align, CentralPanel, Frame, Label, Layout, Modifiers, RichText, ScrollArea,
    Sense, TextEdit,
};

use crate::{
    font_icon::{ADD, REMOVE},
    Chord, KeyAction, KeyHelper, KeyMapping,
};

#[derive(Default)]
pub struct KeybindingsState {
    buffer: String,
    view: KeyBindingsView,
    changed: bool,

    key_input: KeyInput,
    editing: EditingAction,
}

impl KeybindingsState {
    pub const fn is_capturing(&self) -> bool {
        matches!(
            self.editing,
            EditingAction::Add { .. } | EditingAction::Edit { .. }
        )
    }
}

#[derive(Default)]
struct KeyInput {
    maybe_chord: MaybeChord,
    chord: Option<Chord>,
}

impl KeyInput {
    fn update_modifiers(&mut self, modifiers: egui::Modifiers) -> bool {
        self.maybe_chord = MaybeChord {
            modifiers,
            ..Default::default()
        };

        true
    }
}

#[derive(Default)]
struct MaybeChord {
    modifiers: egui::Modifiers,
    key: Option<egui::Key>,
}

impl MaybeChord {
    fn as_chord(&self) -> Option<Chord> {
        let key = self.key?;
        Some((self.modifiers, key).into())
    }

    fn display(&self) -> String {
        let mut buf = String::new();
        Chord::display_modifiers(&mut buf, &self.modifiers);
        if let Some(key) = self.key {
            buf.push_str(KeyHelper::stringify_key(&key));
        }
        buf
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum KeyBindingsView {
    #[default]
    Input,
    Raw,
}

#[derive(Default)]
enum EditingAction {
    Add(KeyAction),
    Edit(KeyAction, usize),
    Remove(KeyAction, usize),
    Reset(KeyAction),
    #[default]
    None,
}

impl EditingAction {
    fn reset(&mut self) {
        std::mem::take(self);
    }

    const fn display(&self) -> Option<&'static str> {
        match self {
            Self::Add(action)
            | Self::Edit(action, _)
            | Self::Remove(action, _)
            | Self::Reset(action) => Some(action.display()),
            Self::None => None,
        }
    }
}

pub struct KeybindSettings<'a> {
    state: &'a mut KeybindingsState,
    mapping: &'a mut KeyMapping,
}

impl<'a> KeybindSettings<'a> {
    pub fn new(state: &'a mut KeybindingsState, mapping: &'a mut KeyMapping) -> Self {
        Self { state, mapping }
    }

    pub fn display(mut self, ui: &mut egui::Ui) {
        use KeyBindingsView::*;

        if self.state.changed {
            self.state.changed = false;
            self.mapping.update_from_reverse();
        }

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.view, Input, "Input");

            if ui
                .selectable_value(&mut self.state.view, Raw, "Raw")
                .clicked()
            {
                self.serialize_reverse_mapping();
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.state.view {
                Input => self.display_input(ui),
                Raw => self.display_raw(ui),
            });
    }

    fn display_raw(self, ui: &mut egui::Ui) {
        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            ui.add(TextEdit::multiline(&mut &*self.state.buffer).code_editor());
        });
    }

    fn display_input(&mut self, ui: &mut egui::Ui) {
        if self.handle_action(ui) {
            return;
        }

        for (i, (action, chords)) in self.mapping.reverse_mapping_mut().iter().enumerate() {
            if i > 0 {
                ui.separator();
            }

            ui.horizontal(|ui| {
                Self::display_label(self.state, action, ui);

                ui.vertical(|ui| {
                    if chords.is_empty() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(ADD)
                                .on_hover_text_at_pointer("Add a new keybinding")
                                .clicked()
                            {
                                self.state.editing = EditingAction::Add(*action);
                            }
                        });
                    }

                    for (i, chord) in chords.iter().enumerate() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(REMOVE)
                                .on_hover_text_at_pointer("Remove this keybinding")
                                .clicked()
                            {
                                self.state.editing = EditingAction::Remove(*action, i);
                            }

                            if ui
                                .add(
                                    Label::new(RichText::new(chord.display()))
                                        .sense(Sense::click()),
                                )
                                .on_hover_text_at_pointer("Double click to edit")
                                .double_clicked()
                            {
                                self.state.editing = EditingAction::Edit(*action, i);
                                ui.close_menu()
                            }
                        });
                    }
                });
            });
        }
    }

    fn handle_remove(&mut self, action: KeyAction, index: usize) {
        if let Some(chords) = self
            .mapping
            .reverse_mapping_mut()
            .iter_mut()
            .find_map(|(left, chords)| (*left == action).then_some(chords))
        {
            chords.remove(index);
        }

        self.state.editing.reset();
        self.mapping.update_from_reverse();
    }

    fn handle_reset(&mut self, action: KeyAction) {
        let mut default = KeyMapping::default();

        if let Some((left, right)) =
            self.mapping
                .find_chords_reverse_mut(&action)
                .and_then(|left| {
                    default
                        .find_chords_reverse_mut(&action)
                        .map(|right| (left, right))
                })
        {
            std::mem::swap(left, right);
            self.state.editing.reset();
            self.mapping.update_from_reverse();
        }
    }

    fn handle_action(&mut self, ui: &mut egui::Ui) -> bool {
        use EditingAction::*;

        if matches!(self.state.editing, None) {
            return false;
        }

        match self.state.editing {
            Remove(action, index) => {
                self.handle_remove(action, index);
                return true;
            }
            Reset(action) => {
                self.handle_reset(action);
                return true;
            }
            _ => {}
        }

        let chord = match self.capture_input(ui) {
            Some(chord) => chord,
            _ => {
                self.display_key_input(ui);
                return true;
            }
        };

        match std::mem::take(&mut self.state.editing) {
            Add(action) => {
                if let Some(chords) = self.mapping.find_chords_reverse_mut(&action) {
                    chords.push(chord);
                }
            }

            Edit(action, index) => {
                if let Some(c) = self.find_chord_reverse(&action, index) {
                    *c = chord;
                }
            }

            _ => {}
        }

        self.mapping.update_from_reverse();
        true
    }

    fn find_chord_reverse<'b>(
        &'b mut self,
        action: &KeyAction,
        index: usize,
    ) -> Option<&'b mut Chord> {
        self.mapping
            .find_chords_reverse_mut(action)
            .and_then(|s| s.get_mut(index))
    }

    fn display_key_input(&mut self, ui: &mut egui::Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            let description = match self.state.editing {
                EditingAction::Add(_) => Cow::from("Adding keybind to"),
                EditingAction::Edit(action, index) => {
                    let chord = self
                        .find_chord_reverse(&action, index)
                        .expect("valid action");
                    Cow::from(format!("Editing '{}' for", chord.display()))
                }

                _ => return,
            };

            let action_name = self.state.editing.display().unwrap();
            ui.heading(format!("{description} {action_name}",));
            ui.separator();

            ui.horizontal(|ui| {
                ui.small("press enter to accept");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.small("press esc to reject");
                })
            });

            Frame::none()
                .fill(ui.style().visuals.extreme_bg_color)
                .inner_margin(Margin::symmetric(50.0, 50.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.monospace(
                            self.state
                                .key_input
                                .chord
                                .map(|c| c.display())
                                .unwrap_or_else(|| self.state.key_input.maybe_chord.display()),
                        );
                    });
                });
        });
    }

    fn display_label(state: &mut KeybindingsState, action: &KeyAction, ui: &mut egui::Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.add(Label::new(RichText::new(action.display()).monospace()).sense(Sense::click()))
                .on_hover_text_at_pointer(action.help())
                .context_menu(|ui| {
                    if ui.small_button("➕ add keybinding").clicked() {
                        state.editing = EditingAction::Add(*action);
                        ui.close_menu()
                    }
                    if ui.small_button("🔄 reset to default").clicked() {
                        state.editing = EditingAction::Reset(*action);
                        ui.close_menu()
                    }
                });
        });
    }

    fn capture_input(&mut self, ui: &mut egui::Ui) -> Option<Chord> {
        if matches!(self.state.editing, EditingAction::None) {
            return None;
        }

        if ui
            .ctx()
            .input_mut()
            .consume_key(Modifiers::NONE, egui::Key::Escape)
        {
            std::mem::take(&mut self.state.key_input);
            std::mem::take(&mut self.state.editing);
            return None; // failure
        }

        if ui
            .ctx()
            .input_mut()
            .consume_key(Modifiers::NONE, egui::Key::Enter)
        {
            let input = std::mem::take(&mut self.state.key_input);
            return input.chord; // success
        }

        if self.state.key_input.chord.is_none() {
            std::mem::take(&mut self.state.key_input.maybe_chord);
        }

        let mut initial = Some(ui.ctx().input().modifiers).filter(|c| !c.is_none());

        for (key, modifiers) in ui
            .ctx()
            .input()
            .events
            .iter()
            .flat_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                } if *pressed => Some((*key, *modifiers)),
                _ => None,
            })
        {
            if initial == Some(modifiers) {
                initial.take();
            }

            self.state.key_input.update_modifiers(modifiers);
            self.state.key_input.maybe_chord.key.replace(key);
            self.state.key_input.chord.take();
        }

        if let Some(modifiers) = initial.take() {
            self.state.key_input.update_modifiers(modifiers);
            std::mem::take(&mut self.state.key_input.maybe_chord);
        }

        if let Some(chord) = self.state.key_input.maybe_chord.as_chord() {
            self.state.key_input.chord.replace(chord);
            std::mem::take(&mut self.state.key_input.maybe_chord);
        }

        None
    }

    fn serialize_reverse_mapping(&mut self) {
        use serde::ser::SerializeMap as _;
        use std::fmt::Write;

        self.state.buffer.clear();

        struct Binding<'a> {
            action: &'a KeyAction,
            chords: &'a [Chord],
        }

        impl<'a> serde::Serialize for Binding<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    &self.action.display(),
                    &self.chords.iter().map(|c| c.display()).collect::<Vec<_>>(),
                )?;
                map.end()
            }
        }

        for (i, (action, chords)) in self.mapping.reverse_mapping_mut().iter().enumerate() {
            if i > 0 {
                let _ = writeln!(&mut self.state.buffer);
            }

            let _ = write!(
                &mut self.state.buffer,
                "{}",
                serde_yaml::to_string(&Binding { action, chords }).unwrap()
            );
        }
    }
}
