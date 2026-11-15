#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Bounds {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn x(&self) -> u32 {
        self.x
    }

    pub const fn y(&self) -> u32 {
        self.y
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn right(&self) -> u32 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(&self) -> u32 {
        self.y.saturating_add(self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRegion {
    bounds: Bounds,
    text: String,
}

impl TextRegion {
    pub fn new(bounds: Bounds, text: impl Into<String>) -> Self {
        Self {
            bounds,
            text: text.into(),
        }
    }

    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatedText {
    source_region: TextRegion,
    text: String,
}

impl TranslatedText {
    pub fn new(source_region: TextRegion, text: impl Into<String>) -> Self {
        Self {
            source_region,
            text: text.into(),
        }
    }

    pub fn source_region(&self) -> &TextRegion {
        &self.source_region
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
