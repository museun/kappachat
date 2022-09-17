use egui::TextEdit;

// TODO spell check
// TODO kappa completion
// TODO name completion
pub struct EditBox<'a> {
    buffer: &'a mut String,
    write: &'a flume::Sender<String>,
}

impl<'a> EditBox<'a> {
    pub fn new(buffer: &'a mut String, write: &'a flume::Sender<String>) -> Self {
        Self { buffer, write }
    }

    pub fn display(self, ui: &mut egui::Ui) {
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
