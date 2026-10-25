use std::borrow::Cow;

pub const DEFAULT_FAMILY: &str = "Noto Sans SC";

pub fn register(cx: &gpui::App) -> gpui::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(include_bytes!(
            "../../assets/fonts/NotoSansSC-Variable.ttf"
        ))])
}
