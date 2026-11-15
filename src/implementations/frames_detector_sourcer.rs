use std::collections::VecDeque;
use std::path::Path;

use image::RgbaImage;

use crate::frame::Frame;
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::text_region::Bounds;

const MINIMUM_FRAME_WIDTH: u32 = 16;
const MINIMUM_FRAME_HEIGHT: u32 = 16;
const MINIMUM_TEXT_PIXEL_COUNT: usize = 16;
const MINIMUM_BACKGROUND_SHARE_PERCENT: u64 = 50;
const MAXIMUM_RGB_VALUE: u32 = 0xFF_FFFF;
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const COLOR_CHANNEL_MASK: u32 = 0xFF;
const FRAME_NAME_PREFIX: &str = "Detected Frame";

#[derive(Debug)]
pub struct FramesDetectorSourcer {
    frames: VecDeque<Frame>,
}

impl FramesDetectorSourcer {
    pub fn open(
        path: impl AsRef<Path>,
        background_colors: &[u32],
        text_colors: &[u32],
    ) -> Result<Self, FrameSourceError> {
        let background_colors = RgbColorSet::from_rgb_values("background", background_colors)?;
        let text_colors = RgbColorSet::from_rgb_values("text", text_colors)?;
        let path = path.as_ref();
        let image = image::open(path).map_err(|error| {
            FrameSourceError::Decoding(format!("failed to decode {}: {error}", path.display()))
        })?;
        let rgba_image = image.to_rgba8();
        let frames = detect_frame_bounds(&rgba_image, &background_colors, &text_colors)
            .into_iter()
            .enumerate()
            .map(|(index, bounds)| extract_frame(&rgba_image, bounds, index + 1))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            frames: frames.into(),
        })
    }
}

impl FramesSourcer for FramesDetectorSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        Ok(self.frames.pop_front())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    fn from_rgb_value(name: &'static str, value: u32) -> Result<Self, FrameSourceError> {
        if value > MAXIMUM_RGB_VALUE {
            return Err(FrameSourceError::InvalidColor { name, value });
        }
        Ok(Self {
            red: ((value >> RED_SHIFT) & COLOR_CHANNEL_MASK) as u8,
            green: ((value >> GREEN_SHIFT) & COLOR_CHANNEL_MASK) as u8,
            blue: (value & COLOR_CHANNEL_MASK) as u8,
        })
    }

    fn matches(self, pixel: [u8; 4]) -> bool {
        pixel[0] == self.red && pixel[1] == self.green && pixel[2] == self.blue
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RgbColorSet {
    colors: Vec<RgbColor>,
}

impl RgbColorSet {
    fn from_rgb_values(name: &'static str, values: &[u32]) -> Result<Self, FrameSourceError> {
        let colors = values
            .iter()
            .copied()
            .map(|value| RgbColor::from_rgb_value(name, value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { colors })
    }

    fn matches(&self, pixel: [u8; 4]) -> bool {
        self.colors.iter().any(|color| color.matches(pixel))
    }
}

#[derive(Debug, Clone, Copy)]
struct BackgroundComponent {
    bounds: Bounds,
    background_pixel_count: usize,
}

fn detect_frame_bounds(
    image: &RgbaImage,
    background_colors: &RgbColorSet,
    text_colors: &RgbColorSet,
) -> Vec<Bounds> {
    let background_mask = background_mask(image, background_colors);
    let mut visited = vec![false; background_mask.len()];
    let mut detected_bounds = Vec::new();
    for y in 0..image.height() {
        for x in 0..image.width() {
            let index = pixel_index(x, y, image.width());
            if !visited[index] && background_mask[index] {
                let component = trace_component(
                    &background_mask,
                    (image.width(), image.height()),
                    (x, y),
                    &mut visited,
                );
                if is_text_frame(image, &component, text_colors) {
                    detected_bounds.push(component.bounds);
                }
            }
        }
    }
    detected_bounds.sort_by_key(|bounds| (bounds.y(), bounds.x()));
    detected_bounds
}

fn background_mask(image: &RgbaImage, background_colors: &RgbColorSet) -> Vec<bool> {
    image
        .pixels()
        .map(|pixel| background_colors.matches(pixel.0))
        .collect()
}

fn trace_component(
    background_mask: &[bool],
    image_size: (u32, u32),
    start: (u32, u32),
    visited: &mut [bool],
) -> BackgroundComponent {
    let width = image_size.0;
    let mut stack = vec![start];
    visited[pixel_index(start.0, start.1, width)] = true;
    let mut minimum_x = start.0;
    let mut maximum_x = start.0;
    let mut minimum_y = start.1;
    let mut maximum_y = start.1;
    let mut background_pixel_count = 0;
    while let Some((x, y)) = stack.pop() {
        background_pixel_count += 1;
        minimum_x = minimum_x.min(x);
        maximum_x = maximum_x.max(x);
        minimum_y = minimum_y.min(y);
        maximum_y = maximum_y.max(y);
        for (neighbour_x, neighbour_y) in neighbour_positions((x, y), image_size) {
            let neighbour_index = pixel_index(neighbour_x, neighbour_y, width);
            if background_mask[neighbour_index] && !visited[neighbour_index] {
                visited[neighbour_index] = true;
                stack.push((neighbour_x, neighbour_y));
            }
        }
    }
    BackgroundComponent {
        bounds: Bounds::new(
            minimum_x,
            minimum_y,
            maximum_x - minimum_x + 1,
            maximum_y - minimum_y + 1,
        ),
        background_pixel_count,
    }
}

fn neighbour_positions(
    position: (u32, u32),
    image_size: (u32, u32),
) -> impl Iterator<Item = (u32, u32)> {
    let (x, y) = (position.0 as i64, position.1 as i64);
    let (width, height) = (image_size.0 as i64, image_size.1 as i64);
    [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
        .into_iter()
        .filter(move |&(neighbour_x, neighbour_y)| {
            neighbour_x >= 0 && neighbour_y >= 0 && neighbour_x < width && neighbour_y < height
        })
        .map(|(neighbour_x, neighbour_y)| (neighbour_x as u32, neighbour_y as u32))
}

fn pixel_index(x: u32, y: u32, width: u32) -> usize {
    y as usize * width as usize + x as usize
}

fn is_text_frame(
    image: &RgbaImage,
    component: &BackgroundComponent,
    text_colors: &RgbColorSet,
) -> bool {
    let bounds = component.bounds;
    if bounds.width() < MINIMUM_FRAME_WIDTH || bounds.height() < MINIMUM_FRAME_HEIGHT {
        return false;
    }
    if count_color_pixels(image, bounds, text_colors) < MINIMUM_TEXT_PIXEL_COUNT {
        return false;
    }
    let area = bounds.width() as u64 * bounds.height() as u64;
    component.background_pixel_count as u64 * 100 >= area * MINIMUM_BACKGROUND_SHARE_PERCENT
}

fn count_color_pixels(image: &RgbaImage, bounds: Bounds, colors: &RgbColorSet) -> usize {
    (bounds.y()..bounds.bottom())
        .flat_map(|y| (bounds.x()..bounds.right()).map(move |x| (x, y)))
        .filter(|&(x, y)| colors.matches(image.get_pixel(x, y).0))
        .count()
}

fn extract_frame(
    image: &RgbaImage,
    bounds: Bounds,
    frame_number: usize,
) -> Result<Frame, FrameSourceError> {
    let cropped_image = image::imageops::crop_imm(
        image,
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height(),
    )
    .to_image();
    Frame::new(
        format!("{FRAME_NAME_PREFIX} {frame_number}"),
        cropped_image.width(),
        cropped_image.height(),
        cropped_image.into_raw(),
    )
    .map_err(|error| FrameSourceError::Decoding(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BACKGROUND_COLOR: RgbColor = RgbColor {
        red: 0,
        green: 0,
        blue: 0,
    };
    const TEXT_COLOR: RgbColor = RgbColor {
        red: 252,
        green: 252,
        blue: 252,
    };
    const ALTERNATIVE_TEXT_COLOR: RgbColor = RgbColor {
        red: 255,
        green: 255,
        blue: 255,
    };
    const SURROUND_COLOR: RgbColor = RgbColor {
        red: 1,
        green: 1,
        blue: 1,
    };

    #[test]
    fn detects_background_components_that_contain_text() {
        let width = 64;
        let height = 32;
        let mut image = blank_image(width, height);
        fill_rectangle(&mut image, Bounds::new(2, 2, 20, 20), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(42, 2, 20, 20), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(4, 4, 4, 4), TEXT_COLOR);
        let detected_bounds = detect_frame_bounds(
            &image,
            &color_set(&[BACKGROUND_COLOR]),
            &color_set(&[TEXT_COLOR]),
        );
        assert_eq!(detected_bounds, vec![Bounds::new(2, 2, 20, 20)]);
    }

    #[test]
    fn detects_text_with_any_configured_text_color() {
        let mut image = blank_image(32, 32);
        fill_rectangle(&mut image, Bounds::new(2, 2, 20, 20), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(4, 4, 4, 4), ALTERNATIVE_TEXT_COLOR);
        let detected_bounds = detect_frame_bounds(
            &image,
            &color_set(&[BACKGROUND_COLOR]),
            &color_set(&[TEXT_COLOR, ALTERNATIVE_TEXT_COLOR]),
        );
        assert_eq!(detected_bounds, vec![Bounds::new(2, 2, 20, 20)]);
    }

    #[test]
    fn rejects_background_components_that_are_too_small() {
        let mut image = blank_image(32, 32);
        fill_rectangle(&mut image, Bounds::new(1, 1, 8, 8), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(2, 2, 4, 4), TEXT_COLOR);
        assert!(
            detect_frame_bounds(
                &image,
                &color_set(&[BACKGROUND_COLOR]),
                &color_set(&[TEXT_COLOR])
            )
            .is_empty()
        );
    }

    #[test]
    fn rejects_color_values_outside_the_rgb_range() {
        let result = RgbColorSet::from_rgb_values("background", &[0x1_000_000]);
        assert!(matches!(
            result,
            Err(FrameSourceError::InvalidColor {
                name: "background",
                value: 0x1_000_000
            })
        ));
    }

    fn color_set(colors: &[RgbColor]) -> RgbColorSet {
        RgbColorSet {
            colors: colors.to_vec(),
        }
    }

    fn blank_image(width: u32, height: u32) -> RgbaImage {
        let mut image = RgbaImage::new(width, height);
        fill_rectangle(&mut image, Bounds::new(0, 0, width, height), SURROUND_COLOR);
        image
    }

    fn fill_rectangle(image: &mut RgbaImage, bounds: Bounds, color: RgbColor) {
        for y in bounds.y()..bounds.bottom() {
            for x in bounds.x()..bounds.right() {
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([color.red, color.green, color.blue, u8::MAX]),
                );
            }
        }
    }
}
