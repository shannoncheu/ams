use gpui::{AnyElement, PathBuilder, canvas, point, prelude::*, px, rgba};

/// A lightweight visual track. UIC's Slider retains pointer capture, keyboard
/// interaction, accessibility and seek commits; this canvas only paints.
pub fn track(progress: f32, seconds: f32, scale: f32) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let width = bounds.size.width.as_f32();
            if width <= 0. {
                return;
            }
            for (fraction, color) in [
                (1., rgba(0x91c8d533)),
                (progress.clamp(0., 1.), rgba(0x91c8d5ff)),
            ] {
                if fraction <= 0. {
                    continue;
                }
                let length = width * fraction;
                let segments = (length / (1.5 * scale)).ceil().clamp(1., 512.) as usize;
                let mut path = PathBuilder::stroke(px(2.2 * scale));
                for index in 0..=segments {
                    let x = length * index as f32 / segments as f32;
                    let y = wave_offset(x, seconds, scale);
                    let position = bounds.origin + point(px(x), bounds.size.height / 2. + px(y));
                    if index == 0 {
                        path.move_to(position);
                    } else {
                        path.line_to(position);
                    }
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, color);
                }
            }
        },
    )
    .size_full()
    .into_any_element()
}

fn wave_offset(x: f32, seconds: f32, scale: f32) -> f32 {
    let wavelength = 24. * scale;
    // Reduce the phase before evaluating sin, preserving precision on long sessions.
    let phase = (seconds / 2.).fract() * std::f32::consts::TAU;
    2.1 * scale * (x / wavelength * std::f32::consts::TAU - phase).sin()
}
