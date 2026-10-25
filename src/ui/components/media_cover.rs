use std::{path::PathBuf, sync::Arc};

use gpui::{Background, Div, Image, ObjectFit, Pixels, div, img, prelude::*, px, rgba};

#[derive(Clone)]
pub enum CoverSource {
    Path(PathBuf),
    Image(Arc<Image>),
}

/// Keep missing-artwork placeholders neutral; album imagery supplies its own color.
pub fn named_gradient(_name: &str) -> Background {
    gpui::rgb(0x303840).into()
}

pub fn media_cover(
    name: impl Into<String>,
    source: Option<CoverSource>,
    size: Pixels,
    radius: Pixels,
    show_fallback_name: bool,
) -> Div {
    let name = name.into();
    let has_source = source.is_some();
    div()
        .relative()
        .size(size)
        .flex_none()
        .overflow_hidden()
        .rounded(radius)
        .bg(named_gradient(&name))
        .when_some(source, |cover, source| match source {
            CoverSource::Path(path) => cover.child(
                img(path)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded(radius),
            ),
            CoverSource::Image(image) => cover.child(
                img(image)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded(radius),
            ),
        })
        .when(!has_source && show_fallback_name, |cover| {
            cover.child(
                div()
                    .absolute()
                    .top(px(13.))
                    .left(px(14.))
                    .right(px(12.))
                    .text_size(px((size.as_f32() * 0.105).clamp(11., 22.)))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgba(0xf2f4f7e8))
                    .line_height(px((size.as_f32() * 0.13).clamp(14., 27.)))
                    .child(name),
            )
        })
}

pub fn liked_playlist_cover(size: Pixels, radius: Pixels) -> Div {
    div()
        .size(size)
        .flex_none()
        .overflow_hidden()
        .rounded(radius)
        .bg(gpui::rgb(0x303840))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px((size.as_f32() * 0.37).clamp(34., 68.)))
                .line_height(size)
                .text_color(rgba(0x91c8d5ff))
                .child("♥"),
        )
}
