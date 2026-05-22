use eframe::egui;

#[derive(Default)]
pub(super) struct Status {
    message: String,
    kind: StatusKind,
}

impl Status {
    pub(super) fn ok(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: StatusKind::Ok,
        }
    }

    pub(super) fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: StatusKind::Error,
        }
    }

    pub(super) fn show(&self, ui: &mut egui::Ui) {
        if self.message.is_empty() {
            return;
        }

        ui.colored_label(self.kind.color(), &self.message);
    }
}

#[derive(Default)]
enum StatusKind {
    #[default]
    Ok,
    Error,
}

impl StatusKind {
    fn color(&self) -> egui::Color32 {
        match self {
            Self::Ok => egui::Color32::from_rgb(28, 128, 80),
            Self::Error => egui::Color32::from_rgb(170, 42, 42),
        }
    }
}
