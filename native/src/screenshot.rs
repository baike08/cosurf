//! 截图功能模块
//!
//! 从 src-tauri/src/commands/screenshot.rs 迁移。
//! 提供全屏截图、区域裁剪、保存、复制到剪贴板功能。

use napi::bindgen_prelude::*;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tracing::info;

/// 截取全屏并返回 base64 编码的 PNG
#[napi]
pub fn screenshot_capture_full() -> Result<String> {
    let monitors = xcap::Monitor::all()
        .map_err(|e| Error::from_reason(format!("Failed to get monitors: {}", e)))?;

    let monitor = monitors.into_iter().next()
        .ok_or_else(|| Error::from_reason("No monitor found"))?;

    let buffer = monitor.capture_image()
        .map_err(|e| Error::from_reason(format!("Failed to capture screen: {}", e)))?;

    let dynamic_img = image::DynamicImage::ImageRgba8(buffer);
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    dynamic_img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| Error::from_reason(format!("Failed to encode PNG: {}", e)))?;

    let base64_data = BASE64.encode(&png_bytes);

    info!("Fullscreen screenshot captured: {}x{}", dynamic_img.width(), dynamic_img.height());

    // 返回包含尺寸信息的 JSON
    let result = serde_json::json!({
        "image": base64_data,
        "width": dynamic_img.width(),
        "height": dynamic_img.height(),
    });
    serde_json::to_string(&result)
        .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
}

/// 从 base64 截图中裁剪指定区域
#[napi]
pub fn screenshot_crop(
    base64_data: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    screen_width: u32,
    screen_height: u32,
) -> Result<String> {
    info!("Cropping region: {}x{} at ({}, {}) from {}x{}", width, height, x, y, screen_width, screen_height);

    let bytes = BASE64.decode(&base64_data)
        .map_err(|e| Error::from_reason(format!("Invalid base64 data: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| Error::from_reason(format!("Failed to decode image: {}", e)))?
        .to_rgba8();

    let crop_x = x.max(0).min(screen_width as i32) as u32;
    let crop_y = y.max(0).min(screen_height as i32) as u32;
    let crop_w = width.min(screen_width.saturating_sub(crop_x));
    let crop_h = height.min(screen_height.saturating_sub(crop_y));

    let cropped = image::imageops::crop_imm(&img, crop_x, crop_y, crop_w, crop_h).to_image();

    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    let dynamic_img = image::DynamicImage::ImageRgba8(cropped);
    dynamic_img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| Error::from_reason(format!("Failed to encode PNG: {}", e)))?;

    let cropped_base64 = BASE64.encode(&png_bytes);

    let result = serde_json::json!({
        "image": cropped_base64,
        "width": crop_w,
        "height": crop_h,
    });
    serde_json::to_string(&result)
        .map_err(|e| Error::from_reason(format!("Serialization error: {}", e)))
}

/// 将 base64 截图保存到文件
#[napi]
pub fn screenshot_save(base64_data: String, file_path: String) -> Result<bool> {
    let bytes = BASE64.decode(&base64_data)
        .map_err(|e| Error::from_reason(format!("Invalid base64 data: {}", e)))?;

    std::fs::write(&file_path, &bytes)
        .map_err(|e| Error::from_reason(format!("Failed to save screenshot: {}", e)))?;

    info!("Screenshot saved to: {}", file_path);
    Ok(true)
}

/// 将 base64 截图复制到剪贴板
#[napi]
pub fn screenshot_copy_to_clipboard(base64_data: String) -> Result<bool> {
    use std::borrow::Cow;

    let bytes = BASE64.decode(&base64_data)
        .map_err(|e| Error::from_reason(format!("Invalid base64 data: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| Error::from_reason(format!("Failed to decode image: {}", e)))?
        .to_rgba8();

    let (width, height) = (img.width() as usize, img.height() as usize);
    let pixels: Vec<u8> = img.into_raw();

    let img_data = arboard::ImageData {
        width,
        height,
        bytes: Cow::Owned(pixels),
    };

    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| Error::from_reason(format!("Failed to access clipboard: {}", e)))?;

    clipboard.set_image(img_data)
        .map_err(|e| Error::from_reason(format!("Failed to copy image to clipboard: {}", e)))?;

    info!("Screenshot copied to clipboard");
    Ok(true)
}
