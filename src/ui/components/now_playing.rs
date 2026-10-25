use gpui_icons::GpuiIcons;
use std::{
    sync::{Arc, OnceLock},
    time::Instant,
};

use gpui::{
    AnyElement, ClipboardItem, Div, FontWeight, Image, ImageFormat, ImageSource, MouseButton,
    ObjectFit, SharedString, Stateful, Window, div, img, prelude::*, px, rgb, rgba, svg,
};
use gpui_effects::{ArtworkFlowOptions, artwork_flow};
use uic::components::context_menu::{self, ContextMenu};

use crate::ui::components::feedback as toast;
use crate::{
    app::music_shell::{EffectsViewer, MusicShell, pages::songs},
    domain::Page,
    services::api::TrackResponse,
    ui::{
        components::{lyrics_panel, player_bar, queue_panel},
        strings, theme,
    },
};

const FALLBACK_COVER: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="800" viewBox="0 0 800 800"><rect width="800" height="800" fill="#303840"/><path d="M510 175v360c0 74-60 134-134 134s-134-60-134-134 60-134 134-134c28 0 54 9 75 23V232l59-57z" fill="#91C8D5" fill-opacity=".8"/></svg>"##;

/// Fade out the visible panel before replacing it, then fade in the new panel.
/// Keep the current visibility when reversing direction during a rapid toggle.
#[derive(Default)]
pub struct PanelTransition {
    frame: Option<PanelTransitionFrame>,
}

struct PanelTransitionFrame {
    lyrics: bool,
    visibility: f32,
    at: Instant,
}

impl PanelTransition {
    fn sample(&mut self, target_lyrics: bool, now: Instant) -> (bool, f32) {
        const FADE_SECONDS: f32 = 0.16;
        let frame = self.frame.get_or_insert(PanelTransitionFrame {
            lyrics: target_lyrics,
            visibility: 1.,
            at: now,
        });
        let step = now.saturating_duration_since(frame.at).as_secs_f32() / FADE_SECONDS;
        frame.at = now;
        if frame.lyrics == target_lyrics {
            frame.visibility = (frame.visibility + step).min(1.);
        } else if step < frame.visibility {
            frame.visibility -= step;
        } else {
            // Change the layout only at zero opacity; apply any remaining frame
            // time to the incoming panel so slow frames do not extend the fade.
            frame.lyrics = target_lyrics;
            frame.visibility = (step - frame.visibility).min(1.);
        }
        let t = frame.visibility;
        (frame.lyrics, t * t * (3. - 2. * t))
    }
}

pub(crate) fn render_background_rounded(
    artwork: ImageSource,
    elapsed: f32,
    radius: gpui::Pixels,
) -> AnyElement {
    artwork_flow("player-artwork-background", artwork)
        .options(ArtworkFlowOptions::default().motion(1.5).dither(2.))
        .time(elapsed)
        .size_full()
        .rounded(radius)
        .into_any_element()
}

pub fn render(
    app: &mut MusicShell,
    window: &mut Window,
    window_width: f32,
    window_height: f32,
    cx: &mut gpui::Context<MusicShell>,
) -> AnyElement {
    let (playback, current_track, artwork_is_loading) = {
        let controller = app.playback.read(cx);
        (
            controller.snapshot.clone(),
            controller.queue.current().cloned(),
            controller.artwork_is_loading(),
        )
    };
    let track_id = current_track.as_ref().map(|track| track.id.clone());
    // None means still loading: retain the same image for both the cover and background.
    // Only a completed missing/failed artwork request or decode selects the placeholder.
    let next_artwork = match playback.track.artwork.as_ref() {
        Some(cover) => match ImageSource::from(cover.image.clone()).use_data(None, window, cx) {
            Some(Ok(image)) if image.frame_count() > 0 => Some(cover.image.clone()),
            Some(_) => Some(fallback_cover()),
            None => None,
        },
        None if artwork_is_loading => None,
        None => Some(fallback_cover()),
    };
    if let Some(artwork) = next_artwork {
        if let Some(effect) = app.effects_viewer.as_mut() {
            effect.track_id = track_id;
            effect.album_cover = artwork;
        } else {
            app.effects_viewer = Some(EffectsViewer {
                track_id,
                album_cover: artwork,
                clock: Default::default(),
            });
        }
    }
    let artwork = app
        .effects_viewer
        .as_ref()
        .map(|effect| effect.album_cover.clone());
    let phase = app.effects_viewer.as_mut().map_or(0., |effect| {
        effect.clock.sample(Instant::now(), playback.is_playing())
    });
    let scale = (window_width / 698.).min(window_height / 435.);
    let left = (window_width - 698. * scale) / 2.;
    let top = (window_height - 435. * scale) / 2.;
    let favorite = current_track.as_ref().is_some_and(|track| track.favorite);
    let (show_lyrics, panel_opacity) = app
        .now_playing_panel_transition
        .sample(app.now_playing_lyrics, Instant::now());
    let side_panel = if show_lyrics {
        lyrics_panel::render_reference(
            &playback,
            &mut app.lyrics_panel,
            app.playback.clone(),
            scale,
            345. * scale,
        )
    } else {
        queue_panel::render_now_playing(app, window, cx).into_any_element()
    };
    if app.mini_window.is_none()
        && (playback.is_playing()
            || panel_opacity < 1.
            || (show_lyrics && app.lyrics_panel.is_animating(&playback, true)))
    {
        window.request_animation_frame();
    }

    div()
        .relative()
        .size_full()
        .overflow_hidden()
        .rounded(px(if window.is_maximized() {
            0.
        } else {
            12. * scale
        }))
        .bg(crate::window::panel_color(cx))
        .text_color(rgba(0xf2f4f7ff))
        .child(
            div()
                .absolute()
                .left(px(left + 36. * scale))
                .top(px(top + 52. * scale))
                .w(px(250. * scale))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .w_full()
                        .text_center()
                        .child(
                            div()
                                .truncate()
                                .text_size(px(12. * scale))
                                .line_height(gpui::relative(1.3))
                                .font_weight(FontWeight::MEDIUM)
                                .child(playback.track.title.clone()),
                        )
                        .child(
                            div()
                                .mt(px(2. * scale))
                                .font_weight(FontWeight::NORMAL)
                                .truncate()
                                .text_size(px(8. * scale))
                                .line_height(gpui::relative(1.3))
                                .text_color(rgba(0xaab4bfff))
                                .child(playback.track.artist.clone()),
                        ),
                )
                .child(
                    div()
                        .id("reference-cover")
                        .mt(px(18. * scale))
                        .size(px(210. * scale))
                        .flex_none()
                        .overflow_hidden()
                        .rounded_full()
                        .when(current_track.is_none(), |cover| {
                            cover
                                .aria_label("Choose local music")
                                .cursor_pointer()
                                .on_click(cx.listener(|app, _, _, cx| {
                                    app.now_playing = false;
                                    cx.notify();
                                }))
                        })
                        .when_some(artwork, |cover, artwork| {
                            cover.child(
                                img(artwork)
                                    .size_full()
                                    .rounded_full()
                                    .object_fit(ObjectFit::Cover),
                            )
                        }),
                )
                .child(
                    div()
                        .mt(px(4. * scale))
                        .h(px(24. * scale))
                        .flex()
                        .items_center()
                        .gap(px(3. * scale))
                        .child(
                            round_icon_button(
                                "favorite-track",
                                GpuiIcons::LucideHeart,
                                12. * scale,
                                favorite,
                                current_track.is_some(),
                            )
                            .size(px(21. * scale))
                            .aria_label("Favorite")
                            .when(current_track.is_some(), |button| {
                                button.on_click(
                                    cx.listener(|app, _, _, cx| app.toggle_current_favorite(cx)),
                                )
                            }),
                        )
                        .child(
                            round_icon_button(
                                "track-actions",
                                GpuiIcons::LucideEllipsis,
                                12. * scale,
                                false,
                                current_track.is_some(),
                            )
                            .size(px(21. * scale))
                            .aria_label("Track actions")
                            .when_some(
                                current_track.clone(),
                                |button, track| {
                                    let entity = cx.entity();
                                    button.on_mouse_down(
                                        MouseButton::Left,
                                        move |event, window, cx| {
                                            let menu = now_playing_track_menu(
                                                entity.clone(),
                                                track.clone(),
                                            );
                                            let _ = context_menu::show(
                                                menu,
                                                event.position,
                                                window,
                                                cx,
                                            );
                                            cx.stop_propagation();
                                        },
                                    )
                                },
                            ),
                        ),
                )
                .child(
                    div()
                        .relative()
                        .w_full()
                        .mt(px(8. * scale))
                        .child(player_bar::render_now_playing(app, cx, phase, scale))
                        .child(
                            toast::cover_layer(cx)
                                .bottom(gpui::Length::Auto)
                                .top(gpui::relative(1.))
                                .mb(px(0.))
                                .mt(px(14.)),
                        ),
                ),
        )
        .child(
            div()
                .absolute()
                .left(px(left + 330. * scale))
                .top(px(top + 52. * scale))
                .w(px(336. * scale))
                .h(px(345. * scale))
                .opacity(panel_opacity)
                .child(side_panel),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(56.))
                .child(render_window_controls(app, window, cx)),
        )
        .into_any_element()
}

// Native traffic lights and the adjacent controls share this top-row anchor.
#[cfg(target_os = "macos")]
pub(crate) const WINDOW_CONTROLS_TOP: f32 = 8.;

fn render_window_controls(
    _app: &mut MusicShell,
    window: &Window,
    cx: &mut gpui::Context<MusicShell>,
) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .h(px(48.))
        .flex()
        .items_center()
        .child(
            div()
                .ml(px(14.))
                .flex()
                .gap(px(6.))
                .when(cfg!(target_os = "macos"), |bar| bar.ml(px(84.)))
                .child(
                    top_icon_button("open-library", GpuiIcons::LucideLibrary)
                        .aria_label("Open music library")
                        .on_click(cx.listener(|app, _, _, cx| {
                            app.return_to_library(cx);
                        })),
                )
                .child(
                    top_icon_button("mini-now-playing", GpuiIcons::LucidePictureInPicture2)
                        .aria_label("Mini player")
                        .on_click(
                            cx.listener(|app, _, window, cx| app.enter_mini_player(window, cx)),
                        ),
                ),
        )
        .child(super::window_controls::drag_region().h_full().flex_1())
        .when(super::window_controls::visible(cx), |bar| {
            bar.child(super::window_controls::render(window))
        })
        .into_any_element()
}

fn round_icon_button(
    id: &'static str,
    icon: impl Into<SharedString>,
    icon_size: f32,
    active: bool,
    enabled: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .role(gpui::Role::Button)
        .aria_disabled(!enabled)
        .size(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgba(0xffffff22)))
        })
        .when(!enabled, |button| button.opacity(0.42))
        .when(active, |button| button.bg(rgba(0xffffff22)))
        .child(svg().path(icon).size(px(icon_size)).text_color(if active {
            rgb(0xffffffff)
        } else {
            rgba(0xffffffe5)
        }))
}

fn top_icon_button(id: &'static str, icon: GpuiIcons) -> Stateful<Div> {
    div()
        .id(id)
        .role(gpui::Role::Button)
        .size(px(27.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .child(svg().path(icon).size(px(20.)).text_color(rgba(0xffffffff)))
}

fn now_playing_track_menu(entity: gpui::Entity<MusicShell>, track: TrackResponse) -> ContextMenu {
    let info_track = track.clone();
    let next_track = track.clone();
    let enqueue_track = track.clone();
    let copy_track = track.clone();
    let artist_id = track.artist_id.clone();
    let album_id = track.album_id.clone();
    let like_label = if track.favorite { "Unlike" } else { "Like" };
    let favorite_entity = entity.clone();
    let next_entity = entity.clone();
    let enqueue_entity = entity.clone();

    let mut menu = ContextMenu::new()
        .action_with(
            |_, _| menu_label(GpuiIcons::LucideBadgeInfo, "Get Info"),
            move |window, cx| songs::show_track_info(info_track.clone(), window, cx),
        )
        .action_with(
            move |_, _| menu_label(GpuiIcons::LucideHeart, like_label),
            move |_, cx| {
                favorite_entity.update(cx, |app, cx| app.toggle_current_favorite(cx));
            },
        )
        .separator()
        .action_with(
            |_, _| menu_label(GpuiIcons::LucideListPlus, strings::play_next()),
            move |_, cx| {
                next_entity.update(cx, |app, cx| {
                    app.playback.update(cx, |playback, cx| {
                        playback.play_next(next_track.clone(), cx)
                    });
                });
                toast::success(strings::play_next(), cx);
            },
        )
        .action_with(
            |_, _| menu_label(GpuiIcons::LucideListEnd, strings::add_to_queue()),
            move |_, cx| {
                enqueue_entity.update(cx, |app, cx| {
                    app.playback.update(cx, |playback, cx| {
                        playback.enqueue(enqueue_track.clone(), cx)
                    });
                });
                toast::success(strings::add_to_queue(), cx);
            },
        )
        .separator()
        .action_with(
            |_, _| menu_label(GpuiIcons::LucideCopy, "Copy"),
            move |_, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(songs::song_clipboard_text(
                    &copy_track,
                )));
                toast::success("Song info copied", cx);
            },
        );
    if let Some(artist_id) = artist_id {
        let artist_entity = entity.clone();
        menu = menu.action_with(
            |_, _| menu_label(GpuiIcons::LucideMicVocal, "Go to Artist"),
            move |_, cx| {
                artist_entity.update(cx, |app, cx| {
                    app.now_playing = false;
                    app.navigate(Page::Artist(artist_id.clone().into()), cx);
                });
            },
        );
    }
    if let Some(album_id) = album_id {
        menu = menu.action_with(
            |_, _| menu_label(GpuiIcons::LucideDiscAlbum, "Go to Album"),
            move |_, cx| {
                entity.update(cx, |app, cx| {
                    app.now_playing = false;
                    app.navigate(Page::Album(album_id.clone().into()), cx);
                });
            },
        );
    }
    theme::style_fullscreen_context_menu(menu)
}

fn menu_label(icon: GpuiIcons, label: &'static str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(svg().path(icon).size(px(16.)).text_color(rgba(0xf2f0f6f2)))
        .child(label)
}

pub(crate) fn playback_artwork(playback: &crate::domain::PlaybackState) -> Arc<Image> {
    playback
        .track
        .artwork
        .as_ref()
        .map(|artwork| artwork.image.clone())
        .unwrap_or_else(fallback_cover)
}

pub(crate) fn fallback_cover() -> Arc<Image> {
    static COVER: OnceLock<Arc<Image>> = OnceLock::new();
    COVER
        .get_or_init(|| {
            Arc::new(Image::from_bytes(
                ImageFormat::Svg,
                FALLBACK_COVER.as_bytes().to_vec(),
            ))
        })
        .clone()
}
