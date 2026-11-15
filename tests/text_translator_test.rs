mod support;

use std::error::Error;

use frames_translator::factory::create_text_translator;
use frames_translator::text_region::{Bounds, TextRegion};

const SOURCE_TEXT: &str = "Shadowrun";
const EXPECTED_TEXT: &str = "Shadowrun (translated)";

#[test]
fn text_translator_translates_known_text() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let mut text_translator = create_text_translator(&config)?;
    assert_eq!(text_translator.translate(SOURCE_TEXT)?, EXPECTED_TEXT);
    assert!(
        text_translator
            .translate("missing dictionary entry")
            .is_err()
    );
    Ok(())
}

#[test]
fn text_translator_translates_all_regions() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let mut text_translator = create_text_translator(&config)?;
    let region = TextRegion::new(Bounds::new(0, 0, 10, 10), SOURCE_TEXT);
    let translated_texts = text_translator.translate_all(std::slice::from_ref(&region))?;
    assert_eq!(translated_texts.len(), 1);
    assert_eq!(translated_texts[0].source_region(), &region);
    assert_eq!(translated_texts[0].text(), EXPECTED_TEXT);
    Ok(())
}
