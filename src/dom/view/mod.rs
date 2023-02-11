use wasm_bindgen::prelude::wasm_bindgen;
use super::DomNode;

flat_mod! { span }

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    #[wasm_bindgen(extends = DomNode)]
    pub(super) type Text;

    #[wasm_bindgen(constructor)]
    fn new (s: &str) -> Text;
    #[wasm_bindgen(structural, method, setter, js_name = data)]
    fn set_data (this: &Text, s: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Alignment {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlignment {
    Left,
    #[default]
    Center,
    Right,
    Justified
}

impl Orientation {
    #[inline]
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Horizontal => "row",
            Self::Vertical => "column",
        }
    }
}

impl Alignment {
    #[inline]
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::Center => "center",
            Self::End => "flex-end",
        }
    }
}

impl TextAlignment {
    #[inline]
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Justified => "justify"
        }
    }
}

impl From<Alignment> for TextAlignment {
    #[inline]
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::Start => Self::Left,
            Alignment::Center => Self::Center,
            Alignment::End => Self::Right
        }
    }
}