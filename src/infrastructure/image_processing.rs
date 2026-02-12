use anyhow::{Context, bail};
use base64::Engine;
use image::{DynamicImage, ImageReader};
use std::io::Cursor;

/// Maximum dimension (width or height) for the full-size image.
const MAX_FULL_SIZE: u32 = 1200;

/// Maximum dimension (width or height) for the thumbnail.
const MAX_THUMBNAIL_SIZE: u32 = 200;

/// Maximum allowed input dimension (width or height) to prevent decompression bombs.
const MAX_INPUT_DIMENSION: u32 = 10_000;

/// Maximum memory the decoder may allocate (256 MB).
const MAX_DECODER_ALLOC: u64 = 256 * 1024 * 1024;

/// Allowed MIME types in data URLs.
const ALLOWED_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

/// JPEG quality for the full-size image (0-100).
const JPEG_QUALITY_FULL: u8 = 85;

/// JPEG quality for the thumbnail (0-100).
const JPEG_QUALITY_THUMBNAIL: u8 = 80;

/// Processed image data ready for storage.
pub struct ProcessedImage {
    pub image_data: Vec<u8>,
    pub thumbnail_data: Vec<u8>,
    pub content_type: String,
}

/// Parse a data URL, decode the image, resize to a maximum dimension, and
/// produce both a full-size and thumbnail JPEG.
///
/// Returns `(image_data, thumbnail_data, content_type)`.
pub fn process_data_url(data_url: &str) -> anyhow::Result<ProcessedImage> {
    let raw_bytes = decode_data_url(data_url)?;
    process_image_bytes(&raw_bytes)
}

/// Process raw image bytes (JPEG/PNG/WebP) into resized full + thumbnail JPEGs.
pub fn process_image_bytes(raw_bytes: &[u8]) -> anyhow::Result<ProcessedImage> {
    let orientation = read_exif_orientation(raw_bytes);

    let mut reader = ImageReader::new(Cursor::new(raw_bytes))
        .with_guessed_format()
        .context("failed to guess image format")?;

    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_INPUT_DIMENSION);
    limits.max_image_height = Some(MAX_INPUT_DIMENSION);
    limits.max_alloc = Some(MAX_DECODER_ALLOC);
    reader.limits(limits);

    let img = reader.decode().context("failed to decode image")?;
    let img = apply_exif_orientation(img, orientation);

    let full = img.resize(
        MAX_FULL_SIZE,
        MAX_FULL_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    let thumb = img.resize(
        MAX_THUMBNAIL_SIZE,
        MAX_THUMBNAIL_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    let image_data = encode_jpeg(&full, JPEG_QUALITY_FULL)?;
    let thumbnail_data = encode_jpeg(&thumb, JPEG_QUALITY_THUMBNAIL)?;

    Ok(ProcessedImage {
        image_data,
        thumbnail_data,
        content_type: "image/jpeg".to_string(),
    })
}

/// Read the EXIF orientation tag from raw image bytes.
///
/// Returns the orientation value (1-8), or 1 (normal) if no EXIF data is found.
fn read_exif_orientation(raw_bytes: &[u8]) -> u32 {
    let reader = exif::Reader::new();
    let Ok(exif_data) = reader.read_from_container(&mut Cursor::new(raw_bytes)) else {
        return 1;
    };
    exif_data
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .unwrap_or(1)
}

/// Apply EXIF orientation transforms so the image displays correctly.
///
/// iPhone cameras (and many others) store photos in a fixed sensor orientation
/// and embed an EXIF `Orientation` tag. Without applying this, photos appear
/// rotated or mirrored.
fn apply_exif_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate90().flipv(),
        8 => img.rotate270(),
        _ => img, // 1 (normal) or unknown
    }
}

/// Decode a `data:image/...;base64,...` URL into raw bytes.
fn decode_data_url(data_url: &str) -> anyhow::Result<Vec<u8>> {
    let Some(rest) = data_url.strip_prefix("data:") else {
        bail!("invalid data URL: missing data: prefix");
    };

    let Some((mime, encoded)) = rest.split_once(',') else {
        bail!("invalid data URL: missing comma separator");
    };

    if !ALLOWED_MIMES.iter().any(|m| mime.contains(m)) {
        bail!("unsupported image type: {mime}");
    }

    base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .context("failed to decode base64 image data")
}

/// Encode a `DynamicImage` as JPEG bytes.
fn encode_jpeg(img: &image::DynamicImage, quality: u8) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder)
        .context("failed to encode image as JPEG")?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_data_url_valid() {
        let data = base64::engine::general_purpose::STANDARD.encode(b"hello");
        let url = format!("data:image/png;base64,{data}");
        let result = decode_data_url(&url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"hello");
    }

    #[test]
    fn decode_data_url_missing_prefix() {
        let result = decode_data_url("not-a-data-url");
        assert!(result.is_err());
    }

    #[test]
    fn decode_data_url_missing_comma() {
        let result = decode_data_url("data:image/png;base64");
        assert!(result.is_err());
    }

    #[test]
    fn decode_data_url_rejects_non_image_mime() {
        let data = base64::engine::general_purpose::STANDARD.encode(b"hello");
        let url = format!("data:text/html;base64,{data}");
        let result = decode_data_url(&url);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unsupported image type"),
            "should reject non-image MIME types"
        );
    }

    #[test]
    fn apply_exif_orientation_identity() {
        let img = DynamicImage::new_rgb8(4, 2);
        let result = apply_exif_orientation(img.clone(), 1);
        assert_eq!((result.width(), result.height()), (4, 2));
    }

    #[test]
    fn apply_exif_orientation_rotate90() {
        // Orientation 6 = rotate 90° CW — swaps width and height
        let img = DynamicImage::new_rgb8(4, 2);
        let result = apply_exif_orientation(img, 6);
        assert_eq!((result.width(), result.height()), (2, 4));
    }

    #[test]
    fn apply_exif_orientation_rotate270() {
        // Orientation 8 = rotate 270° CW — swaps width and height
        let img = DynamicImage::new_rgb8(4, 2);
        let result = apply_exif_orientation(img, 8);
        assert_eq!((result.width(), result.height()), (2, 4));
    }

    #[test]
    fn apply_exif_orientation_rotate180() {
        // Orientation 3 = rotate 180° — preserves dimensions
        let img = DynamicImage::new_rgb8(4, 2);
        let result = apply_exif_orientation(img, 3);
        assert_eq!((result.width(), result.height()), (4, 2));
    }

    #[test]
    fn read_exif_orientation_returns_default_for_png() {
        // PNG doesn't have EXIF, should return 1
        let img = DynamicImage::new_rgb8(2, 2);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("encode png");
        assert_eq!(read_exif_orientation(&buf), 1);
    }
}
