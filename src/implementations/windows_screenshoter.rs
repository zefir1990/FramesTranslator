use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use crate::frame::{CHANNELS_PER_PIXEL, Frame};
use crate::screenshoter::{ScreenshotError, Screenshoter};

type Bool = i32;
type Handle = *mut c_void;
type WindowHandle = Handle;
type DeviceContext = Handle;
type BitmapHandle = Handle;
type GdiObjectHandle = Handle;
type WindowEnumerationParameter = isize;

const WINDOW_EXTENDED_STYLE_INDEX: i32 = -20;
const WINDOW_EXTENDED_STYLE_TOOL_WINDOW: u32 = 0x0000_0080;
const PRINT_WINDOW_CLIENT_ONLY: u32 = 0x0000_0001;
const PRINT_WINDOW_RENDER_FULL_CONTENT: u32 = 0x0000_0002;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
const SRCCOPY: u32 = 0x00CC_0020;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RgbQuad {
    blue: u8,
    green: u8,
    red: u8,
    reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [RgbQuad; 1],
}

type WindowEnumerationCallback =
    Option<unsafe extern "system" fn(WindowHandle, WindowEnumerationParameter) -> Bool>;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(
        callback: WindowEnumerationCallback,
        parameter: WindowEnumerationParameter,
    ) -> Bool;
    fn GetWindowLongW(window: WindowHandle, index: i32) -> i32;
    fn GetWindowTextLengthW(window: WindowHandle) -> i32;
    fn GetWindowTextW(window: WindowHandle, text: *mut u16, maximum_length: i32) -> i32;
    fn IsWindowVisible(window: WindowHandle) -> Bool;
    fn GetWindowRect(window: WindowHandle, rectangle: *mut Rect) -> Bool;
    fn GetClientRect(window: WindowHandle, rectangle: *mut Rect) -> Bool;
    fn ClientToScreen(window: WindowHandle, point: *mut Point) -> Bool;
    fn GetWindowDC(window: WindowHandle) -> DeviceContext;
    fn ReleaseDC(window: WindowHandle, device_context: DeviceContext) -> i32;
    fn PrintWindow(window: WindowHandle, device_context: DeviceContext, flags: u32) -> Bool;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(device_context: DeviceContext) -> DeviceContext;
    fn CreateDIBSection(
        device_context: DeviceContext,
        bitmap_info: *const BitmapInfo,
        usage: u32,
        pixels: *mut *mut c_void,
        section: Handle,
        offset: u32,
    ) -> BitmapHandle;
    fn SelectObject(device_context: DeviceContext, object: GdiObjectHandle) -> GdiObjectHandle;
    fn DeleteObject(object: GdiObjectHandle) -> Bool;
    fn DeleteDC(device_context: DeviceContext) -> Bool;
    fn BitBlt(
        destination: DeviceContext,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        source: DeviceContext,
        source_x: i32,
        source_y: i32,
        operation: u32,
    ) -> Bool;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsScreenshoter;

impl WindowsScreenshoter {
    pub const fn new() -> Self {
        Self
    }
}

impl Screenshoter for WindowsScreenshoter {
    fn sources(&self) -> Result<Vec<String>, ScreenshotError> {
        let windows = enumerate_windows()?;
        let mut source_names = Vec::new();
        let mut known_titles = HashSet::new();
        for window in windows {
            if known_titles.insert(window.title.clone()) {
                source_names.push(window.title);
            }
        }
        Ok(source_names)
    }

    fn screenshot(&self, source: &str) -> Result<Frame, ScreenshotError> {
        let windows = enumerate_windows()?;
        let window = windows
            .into_iter()
            .find(|window| window.title == source)
            .ok_or_else(|| ScreenshotError::UnknownSource {
                source: source.to_owned(),
            })?;
        capture_window(&window)
    }
}

struct WindowSource {
    title: String,
    window: WindowHandle,
}

#[derive(Clone, Copy)]
struct Bounds {
    width: u32,
    height: u32,
}

impl Bounds {
    fn pixel_byte_length(self, source: &str) -> Result<usize, ScreenshotError> {
        (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|pixel_count| pixel_count.checked_mul(CHANNELS_PER_PIXEL))
            .ok_or_else(|| ScreenshotError::BufferSizeOverflow {
                source: source.to_owned(),
                width: self.width,
                height: self.height,
            })
    }
}

fn enumerate_windows() -> Result<Vec<WindowSource>, ScreenshotError> {
    let mut windows = Vec::new();
    let succeeded = unsafe {
        EnumWindows(
            Some(collect_window_source),
            &mut windows as *mut Vec<WindowSource> as WindowEnumerationParameter,
        )
    };
    if succeeded == 0 {
        return Err(ScreenshotError::Enumeration {
            reason: std::io::Error::last_os_error().to_string(),
        });
    }
    Ok(windows)
}

unsafe extern "system" fn collect_window_source(
    window: WindowHandle,
    parameter: WindowEnumerationParameter,
) -> Bool {
    let windows = unsafe { &mut *(parameter as *mut Vec<WindowSource>) };
    if let Some(window_source) = unsafe { window_source(window) } {
        windows.push(window_source);
    }
    1
}

unsafe fn window_source(window: WindowHandle) -> Option<WindowSource> {
    if unsafe { IsWindowVisible(window) } == 0 {
        return None;
    }
    let extended_style = unsafe { GetWindowLongW(window, WINDOW_EXTENDED_STYLE_INDEX) } as u32;
    if extended_style & WINDOW_EXTENDED_STYLE_TOOL_WINDOW != 0 {
        return None;
    }
    let title = unsafe { window_title(window) }?;
    Some(WindowSource { title, window })
}

unsafe fn window_title(window: WindowHandle) -> Option<String> {
    let title_length = unsafe { GetWindowTextLengthW(window) };
    if title_length <= 0 {
        return None;
    }
    let mut title_buffer = vec![0u16; title_length as usize + 1];
    let copied_length =
        unsafe { GetWindowTextW(window, title_buffer.as_mut_ptr(), title_buffer.len() as i32) };
    if copied_length <= 0 {
        return None;
    }
    title_buffer.truncate(copied_length as usize);
    let title = String::from_utf16_lossy(&title_buffer);
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
        None
    } else {
        Some(trimmed_title.to_owned())
    }
}

fn capture_window(window_source: &WindowSource) -> Result<Frame, ScreenshotError> {
    let window_rectangle = window_rectangle(window_source)?;
    let bounds = bounds_from_rectangle(&window_rectangle, &window_source.title)?;
    let client_bounds = client_bounds(window_source)?;
    let client_offset = client_offset(window_source, &window_rectangle)?;
    let byte_length = bounds.pixel_byte_length(&window_source.title)?;
    let device_context = MemoryDeviceContext::new(&window_source.title)?;
    let bitmap = WindowBitmap::new(
        device_context.handle(),
        &window_source.title,
        bounds,
        byte_length,
    )?;
    let captured = unsafe {
        PrintWindow(
            window_source.window,
            device_context.handle(),
            PRINT_WINDOW_RENDER_FULL_CONTENT,
        )
    };
    let mut rgba_pixels = bitmap.rgba_pixels();
    if captured == 0 || is_blank(&rgba_pixels) {
        copy_window_content(
            window_source.window,
            &device_context,
            &window_source.title,
            bounds,
        )?;
        rgba_pixels = bitmap.rgba_pixels();
    }
    if let Some(client_pixels) = capture_client_pixels(window_source, client_bounds)
        && !is_blank(&client_pixels)
    {
        composite_client_pixels(
            &mut rgba_pixels,
            bounds,
            client_bounds,
            client_offset,
            &client_pixels,
        );
    }
    Frame::new(
        window_source.title.clone(),
        bounds.width,
        bounds.height,
        rgba_pixels,
    )
    .map_err(ScreenshotError::from)
}

fn window_rectangle(window_source: &WindowSource) -> Result<Rect, ScreenshotError> {
    let mut rectangle = Rect::default();
    let succeeded = unsafe { GetWindowRect(window_source.window, &mut rectangle) };
    if succeeded == 0 {
        return Err(ScreenshotError::Capture {
            source: window_source.title.clone(),
            reason: format!("GetWindowRect failed: {}", std::io::Error::last_os_error()),
        });
    }
    Ok(rectangle)
}

fn client_bounds(window_source: &WindowSource) -> Result<Bounds, ScreenshotError> {
    let mut rectangle = Rect::default();
    let succeeded = unsafe { GetClientRect(window_source.window, &mut rectangle) };
    if succeeded == 0 {
        return Err(ScreenshotError::Capture {
            source: window_source.title.clone(),
            reason: format!("GetClientRect failed: {}", std::io::Error::last_os_error()),
        });
    }
    bounds_from_rectangle(&rectangle, &window_source.title)
}

fn bounds_from_rectangle(rectangle: &Rect, source: &str) -> Result<Bounds, ScreenshotError> {
    let width = i64::from(rectangle.right) - i64::from(rectangle.left);
    let height = i64::from(rectangle.bottom) - i64::from(rectangle.top);
    if width <= 0 || height <= 0 {
        return Err(ScreenshotError::InvalidDimensions {
            source: source.to_owned(),
            width: u32::try_from(width).unwrap_or(0),
            height: u32::try_from(height).unwrap_or(0),
        });
    }
    Ok(Bounds {
        width: width as u32,
        height: height as u32,
    })
}

fn client_offset(
    window_source: &WindowSource,
    window_rectangle: &Rect,
) -> Result<(u32, u32), ScreenshotError> {
    let mut point = Point::default();
    let succeeded = unsafe { ClientToScreen(window_source.window, &mut point) };
    if succeeded == 0 {
        return Err(ScreenshotError::Capture {
            source: window_source.title.clone(),
            reason: format!("ClientToScreen failed: {}", std::io::Error::last_os_error()),
        });
    }
    let x = i64::from(point.x) - i64::from(window_rectangle.left);
    let y = i64::from(point.y) - i64::from(window_rectangle.top);
    if x < 0 || y < 0 {
        return Err(ScreenshotError::Capture {
            source: window_source.title.clone(),
            reason: format!("client origin ({x}, {y}) is outside the window"),
        });
    }
    Ok((x as u32, y as u32))
}

fn capture_client_pixels(window_source: &WindowSource, bounds: Bounds) -> Option<Vec<u8>> {
    let byte_length = bounds.pixel_byte_length(&window_source.title).ok()?;
    let device_context = MemoryDeviceContext::new(&window_source.title).ok()?;
    let bitmap = WindowBitmap::new(
        device_context.handle(),
        &window_source.title,
        bounds,
        byte_length,
    )
    .ok()?;
    let captured = unsafe {
        PrintWindow(
            window_source.window,
            device_context.handle(),
            PRINT_WINDOW_CLIENT_ONLY | PRINT_WINDOW_RENDER_FULL_CONTENT,
        )
    };
    if captured == 0 {
        return None;
    }
    Some(bitmap.rgba_pixels())
}

fn composite_client_pixels(
    destination: &mut [u8],
    destination_bounds: Bounds,
    client_bounds: Bounds,
    offset: (u32, u32),
    client_pixels: &[u8],
) {
    let offset_x = offset.0.min(destination_bounds.width);
    let offset_y = offset.1.min(destination_bounds.height);
    let copy_width = client_bounds.width.min(destination_bounds.width - offset_x) as usize;
    let copy_height = client_bounds
        .height
        .min(destination_bounds.height - offset_y) as usize;
    let row_byte_length = copy_width * CHANNELS_PER_PIXEL;
    for row in 0..copy_height {
        let source_start = row * client_bounds.width as usize * CHANNELS_PER_PIXEL;
        let destination_start = ((offset_y as usize + row) * destination_bounds.width as usize
            + offset_x as usize)
            * CHANNELS_PER_PIXEL;
        destination[destination_start..destination_start + row_byte_length]
            .copy_from_slice(&client_pixels[source_start..source_start + row_byte_length]);
    }
}

fn is_blank(rgba_pixels: &[u8]) -> bool {
    rgba_pixels
        .as_chunks::<CHANNELS_PER_PIXEL>()
        .0
        .iter()
        .all(|pixel| pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0)
}

fn copy_window_content(
    window: WindowHandle,
    device_context: &MemoryDeviceContext,
    source: &str,
    bounds: Bounds,
) -> Result<(), ScreenshotError> {
    let window_device_context = WindowDeviceContext::new(window, source)?;
    let succeeded = unsafe {
        BitBlt(
            device_context.handle(),
            0,
            0,
            bounds.width as i32,
            bounds.height as i32,
            window_device_context.handle(),
            0,
            0,
            SRCCOPY,
        )
    };
    if succeeded == 0 {
        return Err(ScreenshotError::Capture {
            source: source.to_owned(),
            reason: format!("BitBlt failed: {}", std::io::Error::last_os_error()),
        });
    }
    Ok(())
}

struct MemoryDeviceContext {
    handle: DeviceContext,
}

impl MemoryDeviceContext {
    fn new(source: &str) -> Result<Self, ScreenshotError> {
        let handle = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if handle.is_null() {
            return Err(ScreenshotError::Capture {
                source: source.to_owned(),
                reason: format!(
                    "CreateCompatibleDC failed: {}",
                    std::io::Error::last_os_error()
                ),
            });
        }
        Ok(Self { handle })
    }

    fn handle(&self) -> DeviceContext {
        self.handle
    }
}

impl Drop for MemoryDeviceContext {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.handle);
        }
    }
}

struct WindowBitmap {
    device_context: DeviceContext,
    handle: BitmapHandle,
    previous_object: GdiObjectHandle,
    pixels: *mut c_void,
    byte_length: usize,
}

impl WindowBitmap {
    fn new(
        device_context: DeviceContext,
        source: &str,
        bounds: Bounds,
        byte_length: usize,
    ) -> Result<Self, ScreenshotError> {
        let bitmap_info = BitmapInfo {
            header: BitmapInfoHeader {
                size: size_of::<BitmapInfoHeader>() as u32,
                width: bounds.width as i32,
                height: -(bounds.height as i32),
                planes: 1,
                bit_count: 32,
                compression: BI_RGB,
                ..BitmapInfoHeader::default()
            },
            ..BitmapInfo::default()
        };
        let mut pixels: *mut c_void = ptr::null_mut();
        let handle = unsafe {
            CreateDIBSection(
                device_context,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut pixels,
                ptr::null_mut(),
                0,
            )
        };
        if handle.is_null() || pixels.is_null() {
            if !handle.is_null() {
                unsafe {
                    DeleteObject(handle);
                }
            }
            return Err(ScreenshotError::Capture {
                source: source.to_owned(),
                reason: format!(
                    "CreateDIBSection failed: {}",
                    std::io::Error::last_os_error()
                ),
            });
        }
        let previous_object = unsafe { SelectObject(device_context, handle) };
        if previous_object.is_null() {
            unsafe {
                DeleteObject(handle);
            }
            return Err(ScreenshotError::Capture {
                source: source.to_owned(),
                reason: "SelectObject failed".to_owned(),
            });
        }
        Ok(Self {
            device_context,
            handle,
            previous_object,
            pixels,
            byte_length,
        })
    }

    fn rgba_pixels(&self) -> Vec<u8> {
        let bgra_pixels =
            unsafe { std::slice::from_raw_parts(self.pixels.cast::<u8>(), self.byte_length) };
        bgra_to_rgba(bgra_pixels)
    }
}

impl Drop for WindowBitmap {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.device_context, self.previous_object);
            DeleteObject(self.handle);
        }
    }
}

struct WindowDeviceContext {
    window: WindowHandle,
    handle: DeviceContext,
}

impl WindowDeviceContext {
    fn new(window: WindowHandle, source: &str) -> Result<Self, ScreenshotError> {
        let handle = unsafe { GetWindowDC(window) };
        if handle.is_null() {
            return Err(ScreenshotError::Capture {
                source: source.to_owned(),
                reason: format!("GetWindowDC failed: {}", std::io::Error::last_os_error()),
            });
        }
        Ok(Self { window, handle })
    }

    fn handle(&self) -> DeviceContext {
        self.handle
    }
}

impl Drop for WindowDeviceContext {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(self.window, self.handle);
        }
    }
}

fn bgra_to_rgba(bgra_pixels: &[u8]) -> Vec<u8> {
    let mut rgba_pixels = Vec::with_capacity(bgra_pixels.len());
    for pixel in bgra_pixels.as_chunks::<CHANNELS_PER_PIXEL>().0 {
        rgba_pixels.extend_from_slice(&[pixel[2], pixel[1], pixel[0], u8::MAX]);
    }
    rgba_pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_bgra_pixels_to_opaque_rgba() {
        assert_eq!(
            bgra_to_rgba(&[1, 2, 3, 0, 4, 5, 6, 7]),
            vec![3, 2, 1, 255, 6, 5, 4, 255]
        );
    }
}
