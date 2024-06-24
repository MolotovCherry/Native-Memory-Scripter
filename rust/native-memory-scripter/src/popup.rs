use windows::{
    core::{HSTRING, PCWSTR},
    Win32::UI::WindowsAndMessaging::{
        MessageBoxW, MB_ICONERROR, MB_ICONEXCLAMATION, MB_ICONINFORMATION, MB_ICONQUESTION,
        MB_ICONWARNING, MESSAGEBOX_STYLE,
    },
};

pub enum MessageBoxIcon {
    Exclamation,
    Question,
    Info,
    Warn,
    Error,
}

impl From<MessageBoxIcon> for MESSAGEBOX_STYLE {
    fn from(value: MessageBoxIcon) -> Self {
        match value {
            MessageBoxIcon::Info => MB_ICONINFORMATION,
            MessageBoxIcon::Warn => MB_ICONWARNING,
            MessageBoxIcon::Error => MB_ICONERROR,
            MessageBoxIcon::Exclamation => MB_ICONEXCLAMATION,
            MessageBoxIcon::Question => MB_ICONQUESTION,
        }
    }
}

pub fn display_popup<T: AsRef<str>, M: AsRef<str>>(title: T, message: M, icon: MessageBoxIcon) {
    let title = title.as_ref();
    let message = message.as_ref();

    // these must be explicitly assigned, otherwise they will be temporary and drop
    // and create an invalid pointer, causing corruption and UB
    let h_title = HSTRING::from(title);
    let h_message = HSTRING::from(message);

    let title = PCWSTR::from_raw(h_title.as_ptr());
    let message = PCWSTR::from_raw(h_message.as_ptr());

    let icon = icon.into();

    unsafe {
        MessageBoxW(None, message, title, icon);
    }
}
