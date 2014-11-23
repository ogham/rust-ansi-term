#![crate_name = "ansi_term"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![desc = "A rust library for ANSI terminal colours and styles (bold, underline)"]

//! This is a library for controlling colours and formatting, such as
//! red bold text or blue underlined text, on ANSI terminals.
//!
//! ```rust
//! extern crate ansi_term;
//! use ansi_term::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
//! use ansi_term::Paint;
//! ```
//!
//! Simple Colours
//! --------------
//!
//! You can format strings by calling the `paint` method on a Colour
//! or a Style object, passing in the string you want to format. For
//! example, to get some red text, call the `paint` method on `Red`:
//!
//! ```rust
//! Red.paint("Red!")
//! ```
//!
//! You can also call the `paint` method on a Style. To turn a Colour
//! into a Style, use the `normal` method, though this does the exact
//! some thing as above:
//!
//! ```rust
//! Red.normal().paint("Still red!")
//! ```
//!
//! Bold, Underline, and Background
//! -------------------------------
//!
//! To do anything more complex than just foreground colours, you need
//! to use Style objects. Calling the `bold` or `underline` method on
//! a Colour returns a Style that has the appropriate property set on
//! it:
//!
//! ```rust
//! Blue.bold().paint("Blue bold!")
//! Yellow.underline().paint("Yellow underline!")
//! ```
//!
//! Again, you can call these methods on Styles as well as just Colours:
//!
//! ```rust
//! Blue.normal().bold().paint("Still blue bold!")
//! ```
//!
//! Finally, you can set the background colour of a Style by using the
//! `on` method:
//!
//! ```rust
//! Blue.on(Yellow).paint("Blue on yellow!")
//! ```
//!
//! Extended Colours
//! ----------------
//!
//! You can access the extended range of 256 colours by using the
//! Fixed constructor, which takes an argument of the colour number to
//! use. This can be used wherever you would use a Colour:
//!
//! ```rust
//! Fixed(134).paint("A sort of light purple.")
//! ```
//!
//! This even works for background colours:
//!
//! ```rust
//! Fixed(221).on(Fixed(124)).paint("Mustard in the ketchup.")
//! ```
//!
//! No Formatting
//! -------------
//!
//! Finally, for the sake of completeness, the Plain style provides
//! neither colours nor formatting.
//!
//! ```rust
//! Plain.paint("No colours here.")
//! ```

#![feature(phase)] extern crate regex;
#[phase(plugin)] extern crate regex_macros;
use Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed};
use Style::{Plain, Foreground, Styled};

pub trait Paint {
    /// Paints the given text with this colour.
    fn paint(&self, input: &str) -> String;

    /// Returns a Style with the underline property set.
    fn underline(&self) -> Style;

    /// Return a Style with the bold property set.
    fn bold(&self) -> Style;

    /// Return a Style with the background colour set.
    fn on(&self, background: Colour) -> Style;
}

/// A colour is one specific type of ANSI escape code, and can refer
/// to either the foreground or background colour.
pub enum Colour {
    Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed(u8),
}

// These are the standard numeric sequences.
// See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html

impl Colour {
    fn foreground_code(&self) -> String {
        match *self {
            Black => "30".to_string(),
            Red => "31".to_string(),
            Green => "32".to_string(),
            Yellow => "33".to_string(),
            Blue => "34".to_string(),
            Purple => "35".to_string(),
            Cyan => "36".to_string(),
            White => "37".to_string(),
            Fixed(num) => format!("38;5;{}", num),
        }
    }

    fn background_code(&self) -> String {
        match *self {
            Black => "40".to_string(),
            Red => "41".to_string(),
            Green => "42".to_string(),
            Yellow => "43".to_string(),
            Blue => "44".to_string(),
            Purple => "45".to_string(),
            Cyan => "46".to_string(),
            White => "47".to_string(),
            Fixed(num) => format!("48;5;{}", num),
        }
    }
    
    /// Return a Style with the foreground colour set to this colour.
    pub fn normal(&self) -> Style {
        Styled { foreground: *self, background: None, bold: false, underline: false }
    }
}

/// The Paint trait represents a style or colour that can be applied
/// to a piece of text.
impl Paint for Colour {
    /// This is a short-cut so you don't have to use Blue.normal() just
    /// to get blue text.
    fn paint(&self, input: &str) -> String {
        let re = format!("\x1B[{}m{}\x1B[0m", self.foreground_code(), input);
        return re.to_string();
    }

    fn underline(&self) -> Style {
        Styled { foreground: *self, background: None, bold: false, underline: true }
    }

    fn bold(&self) -> Style {
        Styled { foreground: *self, background: None, bold: true, underline: false }
    }

    fn on(&self, background: Colour) -> Style {
        Styled { foreground: *self, background: Some(background), bold: false, underline: false }
    }
}

/// A style is a collection of properties that can format a string
/// using ANSI escape codes.
pub enum Style {

    /// The Plain style provides no formatting.
    Plain,

    /// The Foreground style provides just a foreground colour.
    Foreground(Colour),

    ///
    /// The Styled style is a catch-all for anything more complicated
    /// than that. It's technically possible for there to be other
    /// cases, such as "bold foreground", but probably isn't worth it.
    Styled { foreground: Colour, background: Option<Colour>, bold: bool, underline: bool, },
}

impl Paint for Style {
    fn paint(&self, input: &str) -> String {
        match *self {
            Plain => input.to_string(),
            Foreground(c) => c.paint(input),
            Styled(foreground, background, bold, underline) => {
                let bg = match background {
                    Some(c) => format!("{};", c.background_code()),
                    None => "".to_string()
                };
                let bo = if bold { "1;" } else { "" };
                let un = if underline { "4;" } else { "" };
                let painted = format!("\x1B[{}{}{}{}m{}\x1B[0m", bo, un, bg, foreground.foreground_code(), input.to_string());
                return painted.to_string();
            }
        }
    }

    fn bold(&self) -> Style {
        match *self {
            Plain =>                Styled { foreground: White,  background: None,  bold: true,  underline: false },
            Foreground(c) =>        Styled { foreground: c,      background: None,  bold: true,  underline: false },
            Styled(fg, bg, _, u) => Styled { foreground: fg,     background: bg,    bold: true,  underline: u },
        }
    }

    fn underline(&self) -> Style {
        match *self {
            Plain =>                Styled { foreground: White,  background: None,  bold: false,  underline: true },
            Foreground(c) =>        Styled { foreground: c,      background: None,  bold: false,  underline: true },
            Styled(fg, bg, b, _) => Styled { foreground: fg,     background: bg,    bold: b,      underline: true },
        }
    }

    fn on(&self, background: Colour) -> Style {
        match *self {
            Plain =>               Styled { foreground: White,  background: Some(background),  bold: false,  underline: false },
            Foreground(c) =>       Styled { foreground: c,      background: Some(background),  bold: false,  underline: false },
            Styled(fg, _, b, u) => Styled { foreground: fg,     background: Some(background),  bold: b,      underline: u },
        }
    }
}

/// Return a String with all ANSI escape codes removed. For example:
///
/// ```rust
/// strip_formatting(Blue.paint("hello!")) == "hello!"
/// ```
pub fn strip_formatting(input: String) -> String {
    // What's blue and smells like red paint? Blue paint.
    let re = regex!("\x1B\\[.+?m");
    re.replace_all(input.as_slice(), "").to_string()
}

#[cfg(test)]
mod tests {
    use super::strip_formatting;
    use super::Style::Plain;
    use super::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed};
    use super::Paint;

    #[test]
    fn test_plain() {
        let hi = Plain.paint("text/plain");
        assert!(hi == "text/plain".to_string());
    }

    #[test]
    fn test_red() {
        let hi = Red.paint("hi");
        assert!(hi == "\x1B[31mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_black() {
        let hi = Black.normal().paint("hi");
        assert!(hi == "\x1B[30mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_yellow_bold() {
        let hi = Yellow.bold().paint("hi");
        assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_yellow_bold_2() {
        let hi = Yellow.normal().bold().paint("hi");
        assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_blue_underline() {
        let hi = Blue.underline().paint("hi");
        assert!(hi == "\x1B[4;34mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_green_bold_underline() {
        let hi = Green.bold().underline().paint("hi");
        assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_green_bold_underline_2() {
        let hi = Green.underline().bold().paint("hi");
        assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_purple_on_white() {
        let hi = Purple.on(White).paint("hi");
        assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_purple_on_white_2() {
        let hi = Purple.normal().on(White).paint("hi");
        assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_cyan_bold_on_white() {
        let hi = Cyan.bold().on(White).paint("hi");
        assert!(hi == "\x1B[1;47;36mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_cyan_underline_on_white() {
        let hi = Cyan.underline().on(White).paint("hi");
        assert!(hi == "\x1B[4;47;36mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_cyan_bold_underline_on_white() {
        let hi = Cyan.bold().underline().on(White).paint("hi");
        assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_cyan_underline_bold_on_white() {
        let hi = Cyan.underline().bold().on(White).paint("hi");
        assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_fixed() {
        let hi = Fixed(100).paint("hi");
        assert!(hi == "\x1B[38;5;100mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_fixed_on_purple() {
        let hi = Fixed(100).on(Purple).paint("hi");
        assert!(hi == "\x1B[45;38;5;100mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_fixed_on_fixed() {
        let hi = Fixed(100).on(Fixed(200)).paint("hi");
        assert!(hi == "\x1B[48;5;200;38;5;100mhi\x1B[0m".to_string());
    }

    #[test]
    fn test_strip_formatting() {
        let hi = strip_formatting(Blue.paint("hi"));
        assert!(hi == "hi".to_string());
    }

    #[test]
    fn test_strip_formatting_2() {
        let hi = strip_formatting(Blue.on(Fixed(230)).bold().paint("hi"));
        assert!(hi == "hi".to_string());
    }
}
