use std::borrow::Cow;
use std::fmt;

use style::Style;
use super::ANSIGenericString;


/// A colour is one specific type of ANSI escape code, and can refer
/// to either the foreground or background colour.
///
/// These use the standard numeric sequences.
/// See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Colour {

    /// Colour #0 (foreground code `30`, background code `40`).
    ///
    /// This is not necessarily the background colour, and using it as one may
    /// render the text hard to read on terminals with dark backgrounds.
    Black,

    /// Colour #1 (foreground code `31`, background code `41`).
    Red,

    /// Colour #2 (foreground code `32`, background code `42`).
    Green,

    /// Colour #3 (foreground code `33`, background code `43`).
    Yellow,

    /// Colour #4 (foreground code `34`, background code `44`).
    Blue,

    /// Colour #5 (foreground code `35`, background code `45`).
    Purple,

    /// Colour #6 (foreground code `36`, background code `46`).
    Cyan,

    /// Colour #7 (foreground code `37`, background code `47`).
    ///
    /// As above, this is not necessarily the foreground colour, and may be
    /// hard to read on terminals with light backgrounds.
    White,

    /// A colour number from 0 to 255, for use in 256-colour terminal
    /// environments.
    ///
    /// - Colours 0 to 7 are the `Black` to `White` variants respectively.
    ///   These colours can usually be changed in the terminal emulator.
    /// - Colours 8 to 15 are brighter versions of the eight colours above.
    ///   These can also usually be changed in the terminal emulator, or it
    ///   could be configured to use the original colours and show the text in
    ///   bold instead. It varies depending on the program.
    /// - Colours 16 to 231 contain several palettes of bright colours,
    ///   arranged in six squares measuring six by six each.
    /// - Colours 232 to 255 are shades of grey from black to white.
    ///
    /// It might make more sense to look at a [colour chart][cc].
    /// [cc]: https://upload.wikimedia.org/wikipedia/en/1/15/Xterm_256color_chart.svg
    Fixed(u8),

    /// A 24-bit RGB color, as specified by ISO-8613-3.
    RGB(u8, u8, u8),
}


impl Colour {
    /// Return a Style with the foreground colour set to this colour.
    pub fn normal(self) -> Style {
        Style { foreground: Some(self), .. Style::default() }
    }

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don't have to use Blue.normal() just
    /// to get blue text.
    pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> ANSIGenericString<'a, S>
    where I: Into<Cow<'a, S>>,
          <S as ToOwned>::Owned: fmt::Debug {
        ANSIGenericString {
            string: input.into(),
            style:  self.normal(),
        }
    }

    /// Returns a Style with the bold property set.
    pub fn bold(self) -> Style {
        Style { foreground: Some(self), is_bold: true, .. Style::default() }
    }

    /// Returns a Style with the dimmed property set.
    pub fn dimmed(self) -> Style {
        Style { foreground: Some(self), is_dimmed: true, .. Style::default() }
    }

    /// Returns a Style with the italic property set.
    pub fn italic(self) -> Style {
        Style { foreground: Some(self), is_italic: true, .. Style::default() }
    }

    /// Returns a Style with the underline property set.
    pub fn underline(self) -> Style {
        Style { foreground: Some(self), is_underline: true, .. Style::default() }
    }

    /// Returns a Style with the blink property set.
    pub fn blink(self) -> Style {
        Style { foreground: Some(self), is_blink: true, .. Style::default() }
    }

    /// Returns a Style with the reverse property set.
    pub fn reverse(self) -> Style {
        Style { foreground: Some(self), is_reverse: true, .. Style::default() }
    }

    /// Returns a Style with the hidden property set.
    pub fn hidden(self) -> Style {
        Style { foreground: Some(self), is_hidden: true, .. Style::default() }
    }

    /// Returns a Style with the strikethrough property set.
    pub fn strikethrough(self) -> Style {
        Style { foreground: Some(self), is_strikethrough: true, .. Style::default() }
    }

    /// Returns a Style with the background colour property set.
    pub fn on(self, background: Colour) -> Style {
        Style { foreground: Some(self), background: Some(background), .. Style::default() }
    }
}