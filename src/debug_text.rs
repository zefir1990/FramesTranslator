use sdl3::pixels::Color;
use sdl3::render::Canvas;
use sdl3::video::Window;

use crate::text_region::{Bounds, TranslatedText};

pub const DEBUG_FONT_CHARACTER_WIDTH: f32 = 8.0;
pub const DEBUG_FONT_CHARACTER_HEIGHT: f32 = 8.0;
pub const DEBUG_TEXT_PADDING: f32 = 2.0;
const MINIMUM_GLYPH_SCALE: f32 = 0.25;
const GLYPH_SCALE_STEP: f32 = 0.05;

#[derive(Debug, Clone, PartialEq)]
pub struct DebugTextEntry<'a> {
    text: &'a str,
    preferred_offset_y: f32,
}

impl<'a> DebugTextEntry<'a> {
    pub const fn new(text: &'a str, preferred_offset_y: f32) -> Self {
        Self {
            text,
            preferred_offset_y,
        }
    }

    pub const fn text(&self) -> &'a str {
        self.text
    }

    pub const fn preferred_offset_y(&self) -> f32 {
        self.preferred_offset_y
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugTextRow {
    text: String,
    offset_y: f32,
}

impl DebugTextRow {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn offset_y(&self) -> f32 {
        self.offset_y
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugTextLayout {
    rows: Vec<DebugTextRow>,
    glyph_scale: f32,
}

impl DebugTextLayout {
    pub fn for_translated_texts(translated_texts: &[TranslatedText], bounds: Bounds) -> Self {
        let entries = translated_texts
            .iter()
            .map(|translated_text| {
                DebugTextEntry::new(
                    translated_text.text(),
                    translated_text.source_region().bounds().y() as f32,
                )
            })
            .collect::<Vec<_>>();
        Self::new(&entries, available_width(bounds), available_height(bounds))
    }

    pub fn new(
        entries: &[DebugTextEntry<'_>],
        available_width: f32,
        available_height: f32,
    ) -> Self {
        let mut glyph_scale = 1.0;
        loop {
            let layout = Self::layout_at_scale(entries, available_width, glyph_scale);
            if layout.required_height() <= available_height {
                return layout;
            }
            if glyph_scale <= MINIMUM_GLYPH_SCALE {
                return layout.shift_into_height(available_height);
            }
            glyph_scale = (glyph_scale - GLYPH_SCALE_STEP).max(MINIMUM_GLYPH_SCALE);
        }
    }

    pub fn rows(&self) -> &[DebugTextRow] {
        &self.rows
    }

    pub const fn glyph_scale(&self) -> f32 {
        self.glyph_scale
    }

    pub fn required_height(&self) -> f32 {
        self.rows
            .iter()
            .map(|row| row.offset_y + self.character_height())
            .fold(0.0, f32::max)
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        origin: (f32, f32),
        scene_scale: f32,
        color: Color,
    ) -> Result<(), sdl3::Error> {
        let draw_scale = scene_scale * self.glyph_scale;
        canvas.set_scale(draw_scale, draw_scale)?;
        canvas.set_draw_color(color);
        let text_x = (origin.0 + DEBUG_TEXT_PADDING) / self.glyph_scale;
        for row in &self.rows {
            let text_y = (origin.1 + DEBUG_TEXT_PADDING + row.offset_y) / self.glyph_scale;
            canvas.draw_debug_text(row.text(), (text_x, text_y))?;
        }
        canvas.set_scale(scene_scale, scene_scale)?;
        Ok(())
    }

    fn layout_at_scale(
        entries: &[DebugTextEntry<'_>],
        available_width: f32,
        glyph_scale: f32,
    ) -> Self {
        let character_capacity = (available_width / character_width(glyph_scale))
            .floor()
            .max(1.0) as usize;
        let row_height = character_height(glyph_scale);
        let mut rows = Vec::new();
        let mut cursor_y = 0.0;
        for entry in entries {
            let wrapped_rows = wrap_text(entry.text(), character_capacity);
            let mut row_y = (entry.preferred_offset_y() * glyph_scale).max(cursor_y);
            for text in wrapped_rows {
                rows.push(DebugTextRow {
                    text,
                    offset_y: row_y,
                });
                row_y += row_height;
            }
            cursor_y = row_y;
        }
        Self { rows, glyph_scale }
    }

    fn character_height(&self) -> f32 {
        character_height(self.glyph_scale)
    }

    fn shift_into_height(mut self, available_height: f32) -> Self {
        let overflow = self.required_height() - available_height;
        if overflow > 0.0 {
            for row in &mut self.rows {
                row.offset_y = (row.offset_y - overflow).max(0.0);
            }
        }
        self
    }
}

fn character_width(glyph_scale: f32) -> f32 {
    DEBUG_FONT_CHARACTER_WIDTH * glyph_scale
}

fn character_height(glyph_scale: f32) -> f32 {
    DEBUG_FONT_CHARACTER_HEIGHT * glyph_scale
}

fn available_width(bounds: Bounds) -> f32 {
    (bounds.width() as f32 - 2.0 * DEBUG_TEXT_PADDING).max(0.0)
}

fn available_height(bounds: Bounds) -> f32 {
    (bounds.height() as f32 - 2.0 * DEBUG_TEXT_PADDING).max(0.0)
}

fn wrap_text(text: &str, character_capacity: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current_row = String::new();
    for word in text.split_whitespace() {
        let word_width = word.chars().count();
        if !current_row.is_empty() {
            if current_row.chars().count() + 1 + word_width <= character_capacity {
                current_row.push(' ');
                current_row.push_str(word);
                continue;
            }
            rows.push(std::mem::take(&mut current_row));
        }
        current_row = take_word_tail(&mut rows, word, character_capacity);
    }
    if !current_row.is_empty() || rows.is_empty() {
        rows.push(current_row);
    }
    rows
}

fn take_word_tail(rows: &mut Vec<String>, word: &str, character_capacity: usize) -> String {
    let characters = word.chars().collect::<Vec<_>>();
    let chunks = characters.chunks(character_capacity).collect::<Vec<_>>();
    let Some((last_chunk, full_chunks)) = chunks.split_last() else {
        return String::new();
    };
    for chunk in full_chunks {
        rows.push(chunk.iter().collect());
    }
    last_chunk.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_text_into_rows() {
        let entries = [DebugTextEntry::new("one two three four five", 0.0)];
        let layout = DebugTextLayout::new(&entries, 72.0, 100.0);
        assert_eq!(
            layout
                .rows()
                .iter()
                .map(DebugTextRow::text)
                .collect::<Vec<_>>(),
            vec!["one two", "three", "four five"]
        );
        assert_eq!(layout.glyph_scale(), 1.0);
    }

    #[test]
    fn splits_words_wider_than_the_frame() {
        let entries = [DebugTextEntry::new("abcdefghij", 0.0)];
        let layout = DebugTextLayout::new(&entries, 40.0, 100.0);
        assert_eq!(
            layout
                .rows()
                .iter()
                .map(DebugTextRow::text)
                .collect::<Vec<_>>(),
            vec!["abcde", "fghij"]
        );
    }

    #[test]
    fn preserves_preferred_vertical_positions() {
        let entries = [
            DebugTextEntry::new("first", 0.0),
            DebugTextEntry::new("second", 32.0),
        ];
        let layout = DebugTextLayout::new(&entries, 200.0, 100.0);
        assert_eq!(layout.rows()[1].offset_y(), 32.0);
    }

    #[test]
    fn stacks_rows_that_would_overlap() {
        let entries = [
            DebugTextEntry::new("first line", 0.0),
            DebugTextEntry::new("second line", 0.0),
        ];
        let layout = DebugTextLayout::new(&entries, 200.0, 100.0);
        assert_eq!(layout.rows()[1].offset_y(), DEBUG_FONT_CHARACTER_HEIGHT);
    }

    #[test]
    fn shrinks_text_to_fit_the_frame_height() {
        let entries = [DebugTextEntry::new(
            "one two three four five six seven eight nine ten",
            0.0,
        )];
        let layout = DebugTextLayout::new(&entries, 40.0, 16.0);
        assert!(layout.glyph_scale() < 1.0);
        assert!(layout.required_height() <= 16.0);
    }

    #[test]
    fn fits_text_positioned_near_the_frame_bottom() {
        let entries = [DebugTextEntry::new(
            "one two three four five six seven eight nine ten",
            100.0,
        )];
        let layout = DebugTextLayout::new(&entries, 40.0, 20.0);
        assert!(layout.required_height() <= 20.0);
    }

    #[test]
    fn keeps_empty_text_as_a_single_row() {
        let entries = [DebugTextEntry::new("", 0.0)];
        let layout = DebugTextLayout::new(&entries, 100.0, 100.0);
        assert_eq!(layout.rows().len(), 1);
        assert_eq!(layout.rows()[0].text(), "");
    }
}
