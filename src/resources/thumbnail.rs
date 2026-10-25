use std::io::Cursor;

/// Decode, resize and encode away from the UI. Never send a full-size album
/// texture to a 38px queue cell. Malformed or oversized artwork is a fallback.
pub(crate) fn queue_cover(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?.thumbnail(96, 96);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png).ok()?;
    Some(output.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_texture_is_bounded_and_invalid_data_uses_fallback() {
        let source = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            640,
            320,
            image::Rgb([80, 110, 130]),
        ));
        let mut encoded = Cursor::new(Vec::new());
        source
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let thumbnail = queue_cover(encoded.get_ref()).unwrap();
        let decoded = image::load_from_memory(&thumbnail).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (96, 48));
        assert!(queue_cover(b"not an image").is_none());
    }
}
