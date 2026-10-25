#[cfg(target_os = "linux")]
use gpui::MouseButton;
use gpui::{AnyElement, App, Div, Window, WindowControlArea, div, prelude::*, px, rgb, rgba, svg};
use gpui_icons::GpuiIcons;

use crate::{
    resources::RomusIcons,
    services::config::{LinuxTitlebar, LoadedClientConfig},
};

pub const HEIGHT: f32 = 56.;

pub fn visible(cx: &App) -> bool {
    cfg!(target_os = "windows")
        || (cfg!(target_os = "linux")
            && cx.global::<LoadedClientConfig>().config.linux.titlebar == LinuxTitlebar::Inline)
}

pub fn width() -> f32 {
    138.
}

pub fn below_titlebar(panel: impl IntoElement, enabled: bool) -> AnyElement {
    if !enabled {
        return panel.into_any_element();
    }
    div()
        .relative()
        .h_full()
        .flex_none()
        .pt(px(HEIGHT))
        .bg(rgb(0x303840))
        .child(
            drag_region()
                .absolute()
                .top_0()
                .left_0()
                .right(px(width()))
                .h(px(HEIGHT)),
        )
        .child(panel)
        .into_any_element()
}

/// Windows delegates caption gestures to native hit testing. Linux needs explicit handlers.
pub fn drag_region() -> Div {
    let region = div().window_control_area(WindowControlArea::Drag);
    #[cfg(target_os = "linux")]
    let region = region
        .on_mouse_down(MouseButton::Left, |event, window, _| {
            if event.click_count == 2 {
                window.zoom_window();
            } else {
                window.start_window_move();
            }
        })
        .on_mouse_down(MouseButton::Right, |event, window, _| {
            window.show_window_menu(event.position);
        });
    region
}

pub fn render(window: &Window) -> Div {
    let maximized = window.is_maximized();
    div().h(px(40.)).w(px(width())).flex_none().flex().children(
        [
            (
                "window-minimize",
                WindowControlArea::Min,
                GpuiIcons::LucideMinus.into(),
                "Minimize",
            ),
            (
                "window-maximize",
                WindowControlArea::Max,
                if maximized {
                    RomusIcons::WindowRestore.into()
                } else {
                    RomusIcons::WindowMaximize.into()
                },
                if maximized { "Restore" } else { "Maximize" },
            ),
            (
                "window-close",
                WindowControlArea::Close,
                GpuiIcons::LucideX.into(),
                "Close",
            ),
        ]
        .map(|(id, area, icon, label): (_, _, gpui::SharedString, _)| {
            let button = div()
                .id(id)
                .role(gpui::Role::Button)
                .aria_label(label)
                .w(px(46.))
                .h(px(36.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgba(0xaab4bfff))
                .hover(move |style| {
                    style
                        .bg(if area == WindowControlArea::Close {
                            rgba(0xe58b9338)
                        } else {
                            rgba(0xffffff12)
                        })
                        .text_color(rgba(0xf2f4f7ff))
                });
            #[cfg(target_os = "windows")]
            let button = button.window_control_area(area).on_a11y_action(
                gpui::AccessibleAction::Click,
                move |_, window, cx| {
                    cx.stop_propagation();
                    match area {
                        WindowControlArea::Min => window.minimize_window(),
                        WindowControlArea::Max => window.zoom_window(),
                        WindowControlArea::Close => window.remove_window(),
                        WindowControlArea::Drag => unreachable!(),
                    }
                },
            );
            #[cfg(not(target_os = "windows"))]
            let button = button.on_click(move |_, window, cx| {
                cx.stop_propagation();
                match area {
                    WindowControlArea::Min => window.minimize_window(),
                    WindowControlArea::Max => window.zoom_window(),
                    WindowControlArea::Close => window.remove_window(),
                    WindowControlArea::Drag => unreachable!(),
                }
            });
            button.child(svg().path(icon).size(px(12.)).text_color(rgba(0xaab4bfff)))
        }),
    )
}
