use gpui_icons::GpuiIcons;
use std::time::Duration;

use gpui::{
    AnyElement, CursorStyle, Div, FontWeight, ObjectFit, Rgba, SharedString, Stateful, div, img,
    prelude::*, px, rgb, rgba, svg,
};

use uic::components::slider::{
    Slider, SliderAppearance, SliderThumbVisibility, SliderTooltipPlacement,
};

use crate::{
    app::music_shell::{MusicShell, PlaybackPanel},
    resources::RomusIcons,
    state::RepeatMode,
    ui::{
        theme::{self, ACCENT},
        timeline_tooltip,
    },
    utils::format_time,
};

fn mode_button(id: &'static str, icon: GpuiIcons, active: bool) -> Stateful<Div> {
    div()
        .id(id)
        .relative()
        .size(px(27.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .when(active, |button| button.bg(rgba(0x91c8d514)))
        .hover(|style| style.bg(rgba(0xffffff0f)))
        .child(svg().path(icon).size_5().text_color(if active {
            rgb(ACCENT)
        } else {
            rgba(0xd5d5ddeb)
        }))
}

fn icon_button(id: &'static str, icon: impl Into<SharedString>, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(27.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .hover(|style| style.bg(rgba(0xffffff0f)))
        .child(svg().path(icon).size_6().text_color(if enabled {
            rgba(0xd5d5ddeb)
        } else {
            rgba(0xa8a9b34d)
        }))
}

pub fn render(
    app: &mut MusicShell,
    cx: &mut gpui::Context<MusicShell>,
    compact: bool,
) -> impl IntoElement {
    let playback = app.playback.read(cx);
    let playing = playback.snapshot.is_playing();
    let timeline = playback.snapshot.timeline;
    let position = timeline.duration().map_or_else(
        || timeline.position(),
        |_| Duration::from_secs_f64(app.timeline_slider.read(cx).value()),
    );
    let elapsed = format_time(position);
    let remaining = playback
        .snapshot
        .timeline
        .duration()
        .map(|duration| format_time(duration.saturating_sub(position)))
        .unwrap_or_else(|| "--".to_owned());
    let title = playback.snapshot.track.title.clone();
    let artist = playback.snapshot.track.artist.clone();
    let error = playback.snapshot.error.clone();
    let muted = playback.snapshot.muted;
    let shuffle = playback.queue.shuffle;
    let repeat = playback.queue.repeat;
    let can_next = playback.can_skip_next();
    let can_previous = playback.can_skip_previous();
    let artwork = playback
        .snapshot
        .track
        .artwork
        .as_ref()
        .map(|artwork| artwork.image.clone());
    let _ = playback;

    theme::frosted()
        .appearance(
            theme::frosted_appearance()
                .blur_radius(px(22.))
                .tint(rgba(0x181a23c4).into()),
        )
        .relative()
        .w(px(if compact { 500. } else { 600. }))
        .px(px(10.))
        .pt_2()
        .pb_2()
        .m_10()
        .flex()
        .items_center()
        .justify_between()
        .rounded(px(32.))
        .shadow(vec![
            gpui::BoxShadow::new(px(0.), px(9.), rgba(0x00000051).into())
                .blur_radius(px(22.))
                .spread_radius(px(-7.)),
        ])
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .when(!compact, |this| {
                    this.child(
                        mode_button("shuffle", GpuiIcons::LucideShuffle, shuffle).on_click(
                            cx.listener(|app, _, _, cx| {
                                app.playback
                                    .update(cx, |playback, cx| playback.toggle_shuffle(cx));
                            }),
                        ),
                    )
                })
                .child(
                    icon_button("skip-back", RomusIcons::MusicPrevious, can_previous).on_click(
                        cx.listener(|app, _, _, cx| {
                            app.playback
                                .update(cx, |playback, cx| playback.skip_previous(cx));
                        }),
                    ),
                )
                .child(
                    div()
                        .id("play-pause")
                        .size(px(31.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|style| style.bg(rgba(0xffffff13)))
                        .on_click(cx.listener(|app, _, _, cx| {
                            app.toggle_playback(cx);
                        }))
                        .child(
                            svg()
                                .path(if playing {
                                    RomusIcons::MusicPause
                                } else {
                                    RomusIcons::MusicPlay
                                })
                                .size_7()
                                .text_color(rgba(0xffffffff)),
                        ),
                )
                .child(
                    icon_button("skip-forward", RomusIcons::MusicNext, can_next).on_click(
                        cx.listener(|app, _, _, cx| {
                            app.playback
                                .update(cx, |playback, cx| playback.skip_next(cx));
                        }),
                    ),
                )
                .child(
                    mode_button("repeat", GpuiIcons::LucideRepeat, repeat != RepeatMode::Off)
                        .on_click(cx.listener(|app, _, _, cx| {
                            app.playback
                                .update(cx, |playback, cx| playback.cycle_repeat(cx));
                        }))
                        .when(repeat == RepeatMode::One, |button| {
                            button.child(
                                div()
                                    .absolute()
                                    .bottom(px(0.))
                                    .right(px(2.))
                                    .text_size(px(8.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(ACCENT))
                                    .child("1"),
                            )
                        }),
                ),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .m_2()
                        .items_center()
                        .justify_start()
                        .rounded(px(12.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(11.))
                                .group_hover("player-progress-hover", |style| style.opacity(0.15))
                                .when_some(artwork, |metadata, artwork| {
                                    metadata.child(
                                        div()
                                            .m_2()
                                            .id("open-now-playing")
                                            .relative()
                                            .size(px(32.))
                                            .flex_none()
                                            .overflow_hidden()
                                            .rounded(px(9.))
                                            .group("player-cover-hover")
                                            .cursor_pointer()
                                            .border_1()
                                            .border_color(rgba(0xffffff18))
                                            .shadow(vec![
                                                gpui::BoxShadow::new(
                                                    px(0.),
                                                    px(4.),
                                                    rgba(0x00000048).into(),
                                                )
                                                .blur_radius(px(10.))
                                                .spread_radius(px(-3.)),
                                            ])
                                            .child(
                                                img(artwork)
                                                    .size_full()
                                                    .rounded(px(9.))
                                                    .object_fit(ObjectFit::Cover),
                                            )
                                            .child(
                                                div()
                                                    .absolute()
                                                    .top_0()
                                                    .right_0()
                                                    .bottom_0()
                                                    .left_0()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(9.))
                                                    .opacity(0.)
                                                    .group_hover("player-cover-hover", |style| {
                                                        style.opacity(1.).bg(rgba(0x090a0fc2))
                                                    })
                                                    .child(
                                                        svg()
                                                            .path(GpuiIcons::LucideMaximize2)
                                                            .size(px(18.))
                                                            .text_color(rgba(0xffffffff)),
                                                    ),
                                            )
                                            .on_click(cx.listener(|app, _, _, cx| {
                                                app.now_playing = true;
                                                cx.notify();
                                            })),
                                    )
                                })
                                .child(
                                    div()
                                        .max_w(px(if compact { 125. } else { 230. }))
                                        .flex()
                                        .flex_col()
                                        .items_start()
                                        .child(
                                            div()
                                                .w_full()
                                                .truncate()
                                                .text_size(px(12.))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgba(0xf1f1f4e8))
                                                .child(error.unwrap_or(title)),
                                        )
                                        .child(
                                            div()
                                                .w_full()
                                                .mt(px(2.))
                                                .truncate()
                                                .font_weight(FontWeight::BOLD)
                                                .text_size(px(9.5))
                                                .text_color(rgba(0xb7b8c288))
                                                .child(artist),
                                        ),
                                ),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .h(px(16.))
                        .w_full()
                        .flex()
                        .group("player-progress-hover-zone")
                        .bottom(px(-5.))
                        .left_0()
                        .items_end()
                        .justify_center()
                        .right_0()
                        .text_color(gpui::white())
                        .child(
                            div()
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .w(px(36.))
                                        .flex_none()
                                        .text_right()
                                        .opacity(0.)
                                        .group_hover("player-progress-hover-zone", |style| {
                                            style.opacity(1.)
                                        })
                                        .text_size(px(10.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgba(0xffffffe5))
                                        .child(elapsed),
                                )
                                .child(
                                    Slider::new(&app.timeline_slider)
                                        .label("Playback position")
                                        .cursor(CursorStyle::PointingHand)
                                        .appearance(
                                            SliderAppearance::default()
                                                .active_track(rgba(0xffffffff).into())
                                                .h(px(1.))
                                                .rounded_full()
                                                .bg(rgba(0xffffff00)),
                                        )
                                        .thumb_visibility(SliderThumbVisibility::Never)
                                        .hover_track_height(px(4.))
                                        .value_tooltip(
                                            SliderTooltipPlacement::Top,
                                            |value, _, _| timeline_tooltip(value),
                                        )
                                        .h(px(16.))
                                        .mx(px(4.))
                                        .flex_1(),
                                )
                                .child(
                                    div()
                                        .w(px(36.))
                                        .flex_none()
                                        .opacity(0.)
                                        .group_hover("player-progress-hover-zone", |style| {
                                            style.opacity(1.)
                                        })
                                        .text_size(px(10.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgba(0xffffffe5))
                                        .child(remaining),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .when(!compact, |this| {
                    this.child(
                        div()
                            .id("toggle-mini-player")
                            .size(px(27.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .cursor_pointer()
                            .hover(|button| button.bg(rgba(0xffffff0f)))
                            .on_click(cx.listener(|app, _, w, cx| {
                                app.enter_mini_player(w, cx);
                            }))
                            .child(
                                svg()
                                    .path(GpuiIcons::LucidePictureInPicture2)
                                    .size_5()
                                    .text_color(rgba(0xd5d5ddeb)),
                            ),
                    )
                })
                .child(
                    div()
                        .id("toggle-inline-lyrics")
                        .size(px(27.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .cursor_pointer()
                        .when(
                            app.playback_panel == PlaybackPanel::InlineLyrics,
                            |button| button.bg(rgba(0x91c8d514)),
                        )
                        .hover(|button| button.bg(rgba(0xffffff0f)))
                        .on_click(cx.listener(|app, _, _, cx| {
                            app.toggle_playback_panel(PlaybackPanel::InlineLyrics, cx);
                        }))
                        .child(svg().path(GpuiIcons::LucideMicVocal).size_5().text_color(
                            if app.playback_panel == PlaybackPanel::InlineLyrics {
                                rgb(ACCENT)
                            } else {
                                rgba(0xd5d5ddeb)
                            },
                        )),
                )
                .child(
                    mode_button(
                        "queue",
                        GpuiIcons::LucideListMusic,
                        app.playback_panel == PlaybackPanel::Queue,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.toggle_playback_panel(PlaybackPanel::Queue, cx);
                    })),
                )
                .when(!compact, |this| {
                    this.child(
                        div()
                            .relative()
                            .child(
                                icon_button(
                                    "volume",
                                    if muted {
                                        GpuiIcons::LucideVolumeX
                                    } else {
                                        GpuiIcons::LucideVolume2
                                    },
                                    true,
                                )
                                .on_click(cx.listener(
                                    |app, _, _, cx| {
                                        app.volume_open = !app.volume_open;
                                        cx.notify();
                                    },
                                )),
                            )
                            .when(app.volume_open, |popup| popup.child(volume_popup(app))),
                    )
                }),
        )
        .child(super::feedback::layer(cx))
}

pub fn render_now_playing(
    app: &mut MusicShell,
    cx: &mut gpui::Context<MusicShell>,
    phase: f32,
    scale: f32,
) -> AnyElement {
    let playback = app.playback.read(cx);
    let playing = playback.snapshot.is_playing();
    let shuffle = playback.queue.shuffle;
    let repeat = playback.queue.repeat;
    let muted = playback.snapshot.muted;
    let can_previous = playback.can_skip_previous();
    let can_next = playback.can_skip_next();
    let timeline = playback.snapshot.timeline;
    let duration = timeline.duration();
    let position = duration.map_or_else(
        || timeline.position(),
        |_| Duration::from_secs_f64(app.timeline_slider.read(cx).value()),
    );
    let progress = duration
        .filter(|d| !d.is_zero())
        .map_or(0., |d| (position.as_secs_f64() / d.as_secs_f64()) as f32);
    let _ = playback;
    let button = |id, icon: SharedString, label, enabled, active| {
        reference_button(id, icon, label, enabled, active, scale)
    };
    div()
        .w_full()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(24. * scale))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    button(
                        "now-playing-shuffle",
                        GpuiIcons::LucideShuffle.into(),
                        "Shuffle",
                        true,
                        shuffle,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.playback
                            .update(cx, |player, cx| player.toggle_shuffle(cx));
                    })),
                )
                .child(
                    button(
                        "now-playing-skip-back",
                        RomusIcons::MusicPrevious.into(),
                        "Previous track",
                        can_previous,
                        false,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.playback
                            .update(cx, |player, cx| player.skip_previous(cx));
                    })),
                )
                .child(
                    button(
                        "now-playing-play-pause",
                        if playing {
                            RomusIcons::MusicPause.into()
                        } else {
                            RomusIcons::MusicPlay.into()
                        },
                        if playing { "Pause" } else { "Play" },
                        can_previous,
                        playing,
                    )
                    .on_click(cx.listener(|app, _, _, cx| app.toggle_playback(cx))),
                )
                .child(
                    button(
                        "now-playing-skip-forward",
                        RomusIcons::MusicNext.into(),
                        "Next track",
                        can_next,
                        false,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.playback.update(cx, |player, cx| player.skip_next(cx));
                    })),
                )
                .child(
                    button(
                        "now-playing-repeat",
                        GpuiIcons::LucideRepeat.into(),
                        "Repeat",
                        true,
                        repeat != RepeatMode::Off,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.playback
                            .update(cx, |player, cx| player.cycle_repeat(cx));
                    }))
                    .when(repeat == RepeatMode::One, |button| {
                        button.child(
                            div()
                                .absolute()
                                .bottom_0()
                                .right_0()
                                .text_size(px(6. * scale))
                                .child("1"),
                        )
                    }),
                )
                .child(div().w(px(8. * scale)))
                .child(
                    button(
                        "reference-mini",
                        GpuiIcons::LucidePictureInPicture2.into(),
                        "Mini player",
                        true,
                        false,
                    )
                    .on_click(cx.listener(|app, _, window, cx| app.enter_mini_player(window, cx))),
                )
                .child(
                    button(
                        "reference-lyrics",
                        GpuiIcons::LucideMicVocal.into(),
                        "Lyrics",
                        true,
                        app.now_playing_lyrics,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.now_playing_lyrics = true;
                        cx.notify();
                    })),
                )
                .child(
                    button(
                        "reference-queue",
                        GpuiIcons::LucideListMusic.into(),
                        "Queue",
                        true,
                        !app.now_playing_lyrics,
                    )
                    .on_click(cx.listener(|app, _, _, cx| {
                        app.now_playing_lyrics = false;
                        cx.notify();
                    })),
                )
                .child(
                    div()
                        .relative()
                        .group("reference-volume")
                        .child(
                            button(
                                "reference-volume",
                                if muted {
                                    GpuiIcons::LucideVolumeX.into()
                                } else {
                                    GpuiIcons::LucideVolume2.into()
                                },
                                "Adjust volume",
                                true,
                                false,
                            )
                            .on_click(cx.listener(|app, _, _, cx| {
                                app.volume_open = !app.volume_open;
                                cx.notify();
                            })),
                        )
                        .child(
                            div()
                                .absolute()
                                .bottom_full()
                                .right_0()
                                .w(px(100. * scale))
                                .p(px(8. * scale))
                                .rounded(px(6. * scale))
                                .bg(rgba(0x303840fa))
                                .when(!app.volume_open, |popup| popup.invisible())
                                .child(
                                    Slider::new(&app.volume_slider)
                                        .label("Volume")
                                        .appearance(
                                            SliderAppearance::default()
                                                .active_track(rgba(0x91c8d5ff).into())
                                                .h(px(3. * scale))
                                                .bg(rgba(0xffffff26)),
                                        )
                                        .thumb_visibility(SliderThumbVisibility::Hover)
                                        .h(px(18. * scale)),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .mt(px(4. * scale))
                .flex()
                .items_center()
                .gap(px(7. * scale))
                .text_size(px(8. * scale))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba(0xaab4bfff))
                .child(div().w(px(22. * scale)).child(format_time(position)))
                .child(
                    div().flex_1().min_w_0().child(
                        Slider::new(&app.timeline_slider)
                            .label("Playback position")
                            .cursor(CursorStyle::PointingHand)
                            .appearance(
                                SliderAppearance::default()
                                    .active_track(rgba(0x00000000).into())
                                    .h(px(12. * scale))
                                    .bg(rgba(0x00000000)),
                            )
                            .track_content(super::wave_progress::track(progress, phase, scale))
                            .thumb_visibility(SliderThumbVisibility::Never)
                            .value_tooltip(SliderTooltipPlacement::Top, |value, _, _| {
                                timeline_tooltip(value)
                            })
                            .border_0()
                            .h(px(20. * scale)),
                    ),
                )
                .child(
                    div().w(px(22. * scale)).text_right().child(
                        duration
                            .map(format_time)
                            .unwrap_or_else(|| "--:--".to_owned()),
                    ),
                ),
        )
        .into_any_element()
}

fn reference_button(
    id: &'static str,
    icon: SharedString,
    label: &'static str,
    enabled: bool,
    active: bool,
    scale: f32,
) -> Stateful<Div> {
    div()
        .id(id)
        .role(gpui::Role::Button)
        .aria_disabled(!enabled)
        .aria_label(label)
        .relative()
        .size(px(21. * scale))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgba(0xffffff14)))
        })
        .when(!enabled, |button| button.opacity(0.3))
        .child(svg().path(icon).size(px(12. * scale)).text_color(
            if id == "now-playing-play-pause" && active {
                rgba(0x91c8d5ff)
            } else if active {
                rgba(0x91c8d5ff)
            } else {
                rgba(0xf2f4f7e0)
            },
        ))
}

pub fn now_playing_icon_button(
    id: &'static str,
    icon: impl Into<SharedString>,
    icon_size: f32,
    enabled: bool,
    active: bool,
    active_color: Option<Rgba>,
) -> Stateful<Div> {
    div()
        .id(id)
        .relative()
        .size(px(50.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgba(0xffffff18)))
        })
        .when(!enabled, |button| button.opacity(0.38))
        .when(!active, |button| button.opacity(0.58))
        .when(active, |button| button.bg(rgba(0xffffff18)))
        .child(svg().path(icon).size(px(icon_size)).text_color(if active {
            active_color.unwrap_or(rgb(ACCENT))
        } else if enabled {
            rgba(0xffffffff)
        } else {
            rgba(0xffffff70)
        }))
}

fn volume_popup(app: &MusicShell) -> impl IntoElement {
    div()
        .absolute()
        .bottom_full()
        .right_0()
        .w(px(150.))
        .p(px(12.))
        .rounded(px(10.))
        .bg(rgba(0x303840ff))
        .child(Slider::new(&app.volume_slider).label("Volume").h(px(24.)))
        .child(
            div()
                .id("volume-mute")
                .cursor_pointer()
                .child("Mute / unmute")
                .on_click({
                    let playback = app.playback.clone();
                    move |_, _, cx| {
                        playback.update(cx, |player, cx| player.toggle_muted(cx));
                    }
                }),
        )
}
