use std::path::Path;

use image::RgbaImage;

use crate::frame::Frame;
use crate::frames_detector_sourcer::{
    DetectedFrame, DetectedFrameQueue, FramesDetectorSourcer, decode_image,
    detected_frames_from_bounds, validate_colors,
};
use crate::frames_sourcer::{FrameSourceError, FramesSourcer};
use crate::text_region::Bounds;

const MINIMUM_FRAME_WIDTH: u32 = 16;
const MINIMUM_FRAME_HEIGHT: u32 = 16;
const MINIMUM_TEXT_PIXEL_COUNT: usize = 16;
const MINIMUM_BACKGROUND_SHARE_PERCENT: u64 = 50;
const BACKGROUND_COLOR_TOLERANCE: u8 = 8;
const TEXT_COLOR_TOLERANCE: u8 = 32;
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const COLOR_CHANNEL_MASK: u32 = 0xFF;

pub struct ProceduralFramesDetectorSourcer {
    frames: DetectedFrameQueue,
    settings: PixelDetectionSettings,
}

impl ProceduralFramesDetectorSourcer {
    pub fn new(background_colors: &[u32], text_colors: &[u32]) -> Result<Self, FrameSourceError> {
        Ok(Self {
            frames: DetectedFrameQueue::default(),
            settings: PixelDetectionSettings::new(background_colors, text_colors)?,
        })
    }

    pub fn open(
        path: impl AsRef<Path>,
        background_colors: &[u32],
        text_colors: &[u32],
    ) -> Result<Self, FrameSourceError> {
        let image = decode_image(path.as_ref())?;
        let mut sourcer = Self::new(background_colors, text_colors)?;
        let detected_frames = sourcer.detect(&image)?;
        sourcer.frames = DetectedFrameQueue::from_detected_frames(detected_frames);
        Ok(sourcer)
    }
}

impl FramesSourcer for ProceduralFramesDetectorSourcer {
    fn next_frame(&mut self) -> Result<Option<Frame>, FrameSourceError> {
        self.frames.next_frame()
    }
}

impl FramesDetectorSourcer for ProceduralFramesDetectorSourcer {
    fn detect(&self, image: &RgbaImage) -> Result<Vec<DetectedFrame>, FrameSourceError> {
        let detected_bounds = detect_frame_bounds(
            image,
            &self.settings.background_colors,
            &self.settings.text_colors,
        );
        detected_frames_from_bounds(image, detected_bounds)
    }
}

#[derive(Debug, Clone)]
struct PixelDetectionSettings {
    background_colors: RgbColorSet,
    text_colors: RgbColorSet,
}

impl PixelDetectionSettings {
    fn new(background_colors: &[u32], text_colors: &[u32]) -> Result<Self, FrameSourceError> {
        validate_colors("background", background_colors)?;
        validate_colors("text", text_colors)?;
        Ok(Self {
            background_colors: RgbColorSet::from_rgb_values(background_colors),
            text_colors: RgbColorSet::from_rgb_values(text_colors),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    fn from_rgb_value(value: u32) -> Self {
        Self {
            red: ((value >> RED_SHIFT) & COLOR_CHANNEL_MASK) as u8,
            green: ((value >> GREEN_SHIFT) & COLOR_CHANNEL_MASK) as u8,
            blue: (value & COLOR_CHANNEL_MASK) as u8,
        }
    }

    fn matches_within(self, pixel: [u8; 4], tolerance: u8) -> bool {
        pixel[0].abs_diff(self.red) <= tolerance
            && pixel[1].abs_diff(self.green) <= tolerance
            && pixel[2].abs_diff(self.blue) <= tolerance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RgbColorSet {
    colors: Vec<RgbColor>,
}

impl RgbColorSet {
    fn from_rgb_values(values: &[u32]) -> Self {
        Self {
            colors: values
                .iter()
                .copied()
                .map(RgbColor::from_rgb_value)
                .collect(),
        }
    }

    fn matches_within(&self, pixel: [u8; 4], tolerance: u8) -> bool {
        self.colors
            .iter()
            .any(|color| color.matches_within(pixel, tolerance))
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
        .map(|pixel| background_colors.matches_within(pixel.0, BACKGROUND_COLOR_TOLERANCE))
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
    if count_text_pixels(image, bounds, text_colors) < MINIMUM_TEXT_PIXEL_COUNT {
        return false;
    }
    let area = bounds.width() as u64 * bounds.height() as u64;
    component.background_pixel_count as u64 * 100 >= area * MINIMUM_BACKGROUND_SHARE_PERCENT
}

fn count_text_pixels(image: &RgbaImage, bounds: Bounds, colors: &RgbColorSet) -> usize {
    (bounds.y()..bounds.bottom())
        .flat_map(|y| (bounds.x()..bounds.right()).map(move |x| (x, y)))
        .filter(|&(x, y)| colors.matches_within(image.get_pixel(x, y).0, TEXT_COLOR_TOLERANCE))
        .count()
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
    const SCALED_TEXT_COLOR: RgbColor = RgbColor {
        red: 230,
        green: 230,
        blue: 230,
    };
    const FAR_TEXT_COLOR: RgbColor = RgbColor {
        red: 180,
        green: 180,
        blue: 180,
    };
    const SCALED_BACKGROUND_COLOR: RgbColor = RgbColor {
        red: 5,
        green: 5,
        blue: 5,
    };
    const SURROUND_COLOR: RgbColor = RgbColor {
        red: 64,
        green: 64,
        blue: 64,
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
    fn detects_scaled_background_colors_within_tolerance() {
        let mut image = blank_image(32, 32);
        fill_rectangle(
            &mut image,
            Bounds::new(2, 2, 20, 20),
            SCALED_BACKGROUND_COLOR,
        );
        fill_rectangle(&mut image, Bounds::new(4, 4, 4, 4), TEXT_COLOR);
        let detected_bounds = detect_frame_bounds(
            &image,
            &color_set(&[BACKGROUND_COLOR]),
            &color_set(&[TEXT_COLOR]),
        );
        assert_eq!(detected_bounds, vec![Bounds::new(2, 2, 20, 20)]);
    }

    #[test]
    fn detects_scaled_text_colors_within_tolerance() {
        let mut image = blank_image(64, 32);
        fill_rectangle(&mut image, Bounds::new(2, 2, 20, 20), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(42, 2, 20, 20), BACKGROUND_COLOR);
        fill_rectangle(&mut image, Bounds::new(4, 4, 4, 4), SCALED_TEXT_COLOR);
        fill_rectangle(&mut image, Bounds::new(44, 4, 4, 4), FAR_TEXT_COLOR);
        let detected_bounds = detect_frame_bounds(
            &image,
            &color_set(&[BACKGROUND_COLOR]),
            &color_set(&[TEXT_COLOR]),
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
