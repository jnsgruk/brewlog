use anyhow::{Context, bail};
use base64::Engine;
use image::ImageReader;
use std::io::Cursor;

/// Maximum dimension (width or height) for the full-size image.
const MAX_FULL_SIZE: u32 = 1200;

/// Maximum dimension (width or height) for the thumbnail.
const MAX_THUMBNAIL_SIZE: u32 = 200;

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
    let img = ImageReader::new(Cursor::new(raw_bytes))
        .with_guessed_format()
        .context("failed to guess image format")?
        .decode()
        .context("failed to decode image")?;

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

/// Decode a `data:image/...;base64,...` URL into raw bytes.
fn decode_data_url(data_url: &str) -> anyhow::Result<Vec<u8>> {
    let Some(rest) = data_url.strip_prefix("data:") else {
        bail!("invalid data URL: missing data: prefix");
    };

    let Some((_mime, encoded)) = rest.split_once(',') else {
        bail!("invalid data URL: missing comma separator");
    };

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
}
