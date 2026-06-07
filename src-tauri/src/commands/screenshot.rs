use std::borrow::Cow;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tauri::{AppHandle, Emitter, Manager};
use tracing::info;

use crate::error::{AppError, ErrorResponse};

fn err(msg: impl ToString) -> ErrorResponse {
    ErrorResponse::from(AppError::Internal(msg.to_string()))
}

/// 截取全屏并发送到前端，前端显示区域选择器
#[tauri::command]
pub async fn capture_full_screen(
    app: AppHandle,
) -> Result<(), ErrorResponse> {
    info!("Capturing full screen");

    let monitor = xcap::Monitor::all()
        .map_err(|e| err(format!("Failed to get monitors: {}", e)))?
        .into_iter()
        .next()
        .ok_or_else(|| err("No monitor found"))?;

    // 截取全屏
    let buffer = monitor
        .capture_image()
        .map_err(|e| err(format!("Failed to capture screen: {}", e)))?;

    let dynamic_img = image::DynamicImage::ImageRgba8(buffer);
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    dynamic_img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| err(format!("Failed to encode PNG: {}", e)))?;

    let base64_data = BASE64.encode(&png_bytes);

    #[derive(Debug, Clone, serde::Serialize)]
    struct ScreenshotEvent {
        image: String,
        width: u32,
        height: u32,
    }

    let _ = app.emit(
        "screenshot-fullscreen-captured",
        ScreenshotEvent {
            image: base64_data,
            width: dynamic_img.width(),
            height: dynamic_img.height(),
        },
    );

    info!("Fullscreen screenshot event emitted to frontend");
    Ok(())
}

/// 从 base64 全屏截图中裁剪指定区域
#[tauri::command]
pub async fn capture_region_from_base64(
    app: AppHandle,
    base64_data: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    screen_width: u32,
    screen_height: u32,
) -> Result<(), ErrorResponse> {
    info!("Cropping region: {}x{} at ({}, {}) from {}x{}", width, height, x, y, screen_width, screen_height);

    // 解码 base64 图片
    let bytes = BASE64
        .decode(&base64_data)
        .map_err(|e| err(format!("Invalid base64 data: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| err(format!("Failed to decode image: {}", e)))?
        .to_rgba8();

    // 裁剪指定区域（确保不越界）
    let crop_x = x.max(0).min(screen_width as i32) as u32;
    let crop_y = y.max(0).min(screen_height as i32) as u32;
    let crop_w = width.min(screen_width - crop_x);
    let crop_h = height.min(screen_height - crop_y);

    let cropped = image::imageops::crop_imm(&img, crop_x, crop_y, crop_w, crop_h).to_image();

    // 编码为 PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    let dynamic_img = image::DynamicImage::ImageRgba8(cropped);
    dynamic_img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| err(format!("Failed to encode PNG: {}", e)))?;

    let cropped_base64 = BASE64.encode(&png_bytes);

    #[derive(Debug, Clone, serde::Serialize)]
    struct ScreenshotEvent {
        image: String,
        width: u32,
        height: u32,
    }

    let _ = app.emit(
        "screenshot-captured",
        ScreenshotEvent {
            image: cropped_base64,
            width: crop_w,
            height: crop_h,
        },
    );

    info!("Cropped screenshot event emitted to frontend");
    Ok(())
}

/// 将 base64 截图保存到指定路径
#[tauri::command]
pub async fn save_screenshot(base64_data: String, path: String) -> Result<(), ErrorResponse> {
    let bytes = BASE64
        .decode(&base64_data)
        .map_err(|e| err(format!("Invalid base64 data: {}", e)))?;

    std::fs::write(&path, &bytes)
        .map_err(|e| err(format!("Failed to save screenshot: {}", e)))?;

    info!("Screenshot saved to: {}", path);
    Ok(())
}

/// 将 base64 截图复制到剪贴板
#[tauri::command]
pub async fn copy_screenshot_to_clipboard(base64_data: String) -> Result<(), ErrorResponse> {
    let bytes = BASE64
        .decode(&base64_data)
        .map_err(|e| err(format!("Invalid base64 data: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| err(format!("Failed to decode image: {}", e)))?
        .to_rgba8();

    let (width, height) = (img.width() as usize, img.height() as usize);
    let pixels: Vec<u8> = img.into_raw();

    let img_data = arboard::ImageData {
        width,
        height,
        bytes: Cow::Owned(pixels),
    };

    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| err(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .set_image(img_data)
        .map_err(|e| err(format!("Failed to copy image to clipboard: {}", e)))?;

    info!("Screenshot copied to clipboard");
    Ok(())
}
