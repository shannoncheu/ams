use crate::app::music_shell::MusicShell;
use gpui::{div, prelude::*, px, rgba};

pub fn render(app: &MusicShell, cx: &mut gpui::Context<MusicShell>) -> impl IntoElement {
    let busy = app.youtube.busy;
    div().id("youtube-content").flex_1().min_h_0().overflow_y_scroll()
        .p(px(24.)).pb(px(130.)).text_color(rgba(0xf2f4f7ff))
        .child(div().text_size(px(24.)).child("YouTube Music"))
        .child(div().mt(px(12.)).text_color(rgba(0xaab4bfff)).child(
            "Google 登录使用独立 WebView2 窗口。会话保存在 Windows Credential Manager。非官方接口可能失效；登录和私人音乐库尚未通过真实账号验收。"))
        .child(div().mt(px(16.)).flex().flex_wrap().gap(px(8.)).children([
            ("login", "Google 登录 / 重试"), ("home", "首页"), ("library", "音乐库与歌单"),
            ("likes", "收藏"), ("history", "历史"), ("logout", "退出并清除会话"),
        ].map(|(action, label)| div().id(action).role(gpui::Role::Button).aria_label(label)
            .p(px(10.)).rounded(px(8.)).bg(rgba(0x303840ff))
            .when(!busy, |button| button.cursor_pointer().hover(|s|s.bg(rgba(0x91c8d530)))
                .on_click(cx.listener(move |app, _, _, cx| app.youtube_request(action,cx))))
            .child(label))))
        .child(div().mt(px(16.)).child(app.youtube.message.clone()))
        .children(app.youtube.rows.iter().map(|row| div().mt(px(12.)).child(row.clone())))
}
