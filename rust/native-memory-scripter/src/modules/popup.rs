use rustpython_vm::pymodule;

#[pymodule]
pub mod popup {
    use crate::popup::{display_popup, MessageBoxIcon};

    #[pyfunction]
    pub fn exclamation(title: String, message: String) {
        display_popup(title, message, MessageBoxIcon::Exclamation);
    }

    #[pyfunction]
    pub fn question(title: String, message: String) {
        display_popup(title, message, MessageBoxIcon::Question);
    }

    #[pyfunction]
    pub fn info(title: String, message: String) {
        display_popup(title, message, MessageBoxIcon::Info);
    }

    #[pyfunction]
    pub fn warn(title: String, message: String) {
        display_popup(title, message, MessageBoxIcon::Warn);
    }

    #[pyfunction]
    pub fn error(title: String, message: String) {
        display_popup(title, message, MessageBoxIcon::Error);
    }
}
