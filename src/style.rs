use std::borrow::Cow;
use std::fmt;

use colour::Colour;
use super::ANSIGenericString;


/// A style is a collection of properties that can format a string
/// using ANSI escape codes.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Style {
    pub foreground: Option<Colour>,
    pub background: Option<Colour>,
    pub is_bold: bool,
    pub is_dimmed: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub is_blink: bool,
    pub is_reverse: bool,
    pub is_hidden: bool,
    pub is_strikethrough: bool
}

impl Style {
    /// Creates a new Style with no differences.
    pub fn new() -> Style {
        Style::default()
    }

    /// Paints the given text with this colour, returning an ANSI string.
    pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> ANSIGenericString<'a, S>
    where I: Into<Cow<'a, S>>,
          <S as ToOwned>::Owned: fmt::Debug {
        ANSIGenericString {
            string: input.into(),
            style:  self,
        }
    }

    /// Returns a Style with the bold property set.
    pub fn bold(&self) -> Style {
        Style { is_bold: true, .. *self }
    }

    /// Returns a Style with the dimmed property set.
    pub fn dimmed(&self) -> Style {
        Style { is_dimmed: true, .. *self }
    }

    /// Returns a Style with the italic property set.
    pub fn italic(&self) -> Style {
        Style { is_italic: true, .. *self }
    }

    /// Returns a Style with the underline property set.
    pub fn underline(&self) -> Style {
        Style { is_underline: true, .. *self }
    }

    /// Returns a Style with the blink property set.
    pub fn blink(&self) -> Style {
        Style { is_blink: true, .. *self }
    }

    /// Returns a Style with the reverse property set.
    pub fn reverse(&self) -> Style {
        Style { is_reverse: true, .. *self }
    }

    /// Returns a Style with the hidden property set.
    pub fn hidden(&self) -> Style {
        Style { is_hidden: true, .. *self }
    }

    /// Returns a Style with the hidden property set.
    pub fn strikethrough(&self) -> Style {
        Style { is_strikethrough: true, .. *self }
    }

    /// Returns a Style with the foreground colour property set.
    pub fn fg(&self, foreground: Colour) -> Style {
        Style { foreground: Some(foreground), .. *self }
    }

    /// Returns a Style with the background colour property set.
    pub fn on(&self, background: Colour) -> Style {
        Style { background: Some(background), .. *self }
    }

    /// Return true if this `Style` has no actual styles, and can be written
    /// without any control characters.
    pub fn is_plain(self) -> bool {
        self == Style::default()
    }
}

impl Default for Style {
    fn default() -> Style {
        Style {
            foreground: None,
            background: None,
            is_bold: false,
            is_dimmed: false,
            is_italic: false,
            is_underline: false,
            is_blink: false,
            is_reverse: false,
            is_hidden: false,
            is_strikethrough: false,
        }
    }
}
