use egui::{Key, TextEdit};

use crate::tabs::Tabs;

pub struct EditBox<'a> {
    tabs: &'a mut Tabs,
    line: &'a mut Option<String>,
}

impl<'a> EditBox<'a> {
    pub fn new(tabs: &'a mut Tabs, line: &'a mut Option<String>) -> Self {
        Self { tabs, line }
    }
}

impl<'a> egui::Widget for EditBox<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // TODO multi-line edit box
        let resp = ui.add(
            TextEdit::singleline(self.tabs.active_mut().buffer_mut())
                .frame(false)
                .lock_focus(true),
        );

        let id = resp.id;
        if resp.lost_focus() && ui.ctx().input().key_pressed(Key::Enter) {
            let input = std::mem::take(self.tabs.active_mut().buffer_mut());
            self.line.replace(input);
        }

        ui.ctx().memory().request_focus(id);
        resp
    }
}
