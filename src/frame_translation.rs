use crate::frame::Frame;
use crate::text_region::{TextRegion, TranslatedText};
use crate::text_translator::{TextTranslator, TranslationError};

const MINIMUM_BLOCK_LINE_COUNT: usize = 3;
const MINIMUM_BLOCK_ASPECT_RATIO: u32 = 2;

pub fn translate_frame_regions(
    frame: &Frame,
    regions: &[TextRegion],
    text_translator: &mut dyn TextTranslator,
) -> Result<Vec<TranslatedText>, TranslationError> {
    if !should_translate_as_block(frame, regions) {
        return text_translator.translate_all(regions);
    }
    let source_text = regions
        .iter()
        .map(TextRegion::text)
        .collect::<Vec<_>>()
        .join(" ");
    let translated_text = text_translator.translate(&source_text)?;
    let translated_lines = split_translation_into_lines(&translated_text, regions.len());
    if translated_lines.iter().any(String::is_empty) {
        return text_translator.translate_all(regions);
    }
    Ok(regions
        .iter()
        .zip(translated_lines)
        .map(|(region, line)| TranslatedText::new(region.clone(), line))
        .collect())
}

pub fn should_translate_as_block(frame: &Frame, regions: &[TextRegion]) -> bool {
    regions.len() >= MINIMUM_BLOCK_LINE_COUNT
        && frame.width() >= frame.height().saturating_mul(MINIMUM_BLOCK_ASPECT_RATIO)
}

pub fn split_translation_into_lines(text: &str, line_count: usize) -> Vec<String> {
    if line_count == 0 {
        return Vec::new();
    }
    let words = text.split_whitespace().collect::<Vec<_>>();
    let words_per_line = words.len() / line_count;
    let extra_word_count = words.len() % line_count;
    let mut lines = Vec::with_capacity(line_count);
    let mut word_index = 0;
    for line_index in 0..line_count {
        let line_word_count = words_per_line + usize::from(line_index < extra_word_count);
        lines.push(words[word_index..word_index + line_word_count].join(" "));
        word_index += line_word_count;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE_FRAME_WIDTH: u32 = 267;
    const WIDE_FRAME_HEIGHT: u32 = 52;
    const MENU_FRAME_WIDTH: u32 = 91;
    const MENU_FRAME_HEIGHT: u32 = 68;

    #[test]
    fn groups_wide_three_line_dialogs() {
        let frame = frame(WIDE_FRAME_WIDTH, WIDE_FRAME_HEIGHT);
        let regions = regions(3);
        assert!(should_translate_as_block(&frame, &regions));
    }

    #[test]
    fn does_not_group_narrow_menu_frames() {
        let frame = frame(MENU_FRAME_WIDTH, MENU_FRAME_HEIGHT);
        let regions = regions(4);
        assert!(!should_translate_as_block(&frame, &regions));
    }

    #[test]
    fn does_not_group_two_line_dialogs() {
        let frame = frame(WIDE_FRAME_WIDTH, WIDE_FRAME_HEIGHT);
        let regions = regions(2);
        assert!(!should_translate_as_block(&frame, &regions));
    }

    #[test]
    fn splits_translation_evenly_across_lines() {
        let lines = split_translation_into_lines("I'd like to be motivated by my wife.", 3);
        assert_eq!(lines, vec!["I'd like to", "be motivated by", "my wife."]);
    }

    #[test]
    fn returns_empty_lines_when_there_are_no_words() {
        assert_eq!(
            split_translation_into_lines("", 3),
            vec![String::new(), String::new(), String::new()]
        );
    }

    #[test]
    fn translates_wide_dialogs_as_one_block() {
        let frame = frame(WIDE_FRAME_WIDTH, WIDE_FRAME_HEIGHT);
        let regions = regions(3);
        let mut translator = FixedTranslator::new("one two three four five six");
        let translated_texts = translate_frame_regions(&frame, &regions, &mut translator).unwrap();
        assert_eq!(translator.call_count, 1);
        assert_eq!(
            translated_texts
                .iter()
                .map(TranslatedText::text)
                .collect::<Vec<_>>(),
            vec!["one two", "three four", "five six"]
        );
    }

    #[test]
    fn translates_menu_lines_individually() {
        let frame = frame(MENU_FRAME_WIDTH, MENU_FRAME_HEIGHT);
        let regions = regions(4);
        let mut translator = FixedTranslator::new("translated");
        let translated_texts = translate_frame_regions(&frame, &regions, &mut translator).unwrap();
        assert_eq!(translator.call_count, 4);
        assert_eq!(translated_texts.len(), 4);
    }

    struct FixedTranslator {
        translation: String,
        call_count: usize,
    }

    impl FixedTranslator {
        fn new(translation: &str) -> Self {
            Self {
                translation: translation.to_owned(),
                call_count: 0,
            }
        }
    }

    impl TextTranslator for FixedTranslator {
        fn translate(&mut self, _text: &str) -> Result<String, TranslationError> {
            self.call_count += 1;
            Ok(self.translation.clone())
        }
    }

    fn frame(width: u32, height: u32) -> Frame {
        Frame::new(
            "Frame",
            width,
            height,
            vec![0; width as usize * height as usize * 4],
        )
        .unwrap()
    }

    fn regions(count: usize) -> Vec<TextRegion> {
        (0..count)
            .map(|index| {
                TextRegion::new(
                    crate::text_region::Bounds::new(0, index as u32 * 16, WIDE_FRAME_WIDTH, 16),
                    format!("line {index}"),
                )
            })
            .collect()
    }
}
