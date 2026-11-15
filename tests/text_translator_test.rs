mod support;

use std::error::Error;

use frames_translator::factory::create_text_translator;
use frames_translator::text_region::{Bounds, TextRegion};

#[test]
fn text_translator_translates_configured_text() -> Result<(), Box<dyn Error>> {
    let config = support::load_config()?;
    let source_text = config.require("TEXT_TRANSLATOR_TEST_SOURCE_TEXT")?;
    let expected_text = config.require("TEXT_TRANSLATOR_TEST_EXPECTED_TEXT")?;
    let mut text_translator = create_text_translator(&config)?;
    assert_eq!(text_translator.translate(source_text)?, expected_text);
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
    let source_text = config.require("TEXT_TRANSLATOR_TEST_SOURCE_TEXT")?;
    let expected_text = config.require("TEXT_TRANSLATOR_TEST_EXPECTED_TEXT")?;
    let mut text_translator = create_text_translator(&config)?;
    let region = TextRegion::new(Bounds::new(0, 0, 10, 10), source_text);
    let translated_texts = text_translator.translate_all(std::slice::from_ref(&region))?;
    assert_eq!(translated_texts.len(), 1);
    assert_eq!(translated_texts[0].source_region(), &region);
    assert_eq!(translated_texts[0].text(), expected_text);
    Ok(())
}
