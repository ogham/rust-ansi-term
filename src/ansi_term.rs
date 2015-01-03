#![crate_name = "ansi_term"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![feature(macro_rules)]

//! This is a library for controlling colours and formatting, such as
//! red bold text or blue underlined text, on ANSI terminals.
//!
//! ```rust
//! extern crate ansi_term;
//! use ansi_term::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
//! use ansi_term::Style::Plain;
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
//! println!("This is in red: {}!", Red.paint("a red string"));
//! ```
//!
//! The `paint` method returns an `ANSIString` object, which will get
//! automatically converted to the correct sequence of escape codes when
//! used in a `println!` or `format!` macro, or anything else that
//! supports using the `Show` trait. This means that if you just want a
//! string of the escape codes without anything else, you can still use
//! the `to_string` method:
//!
//! ```rust
//! let red_string: String = Red.paint("another red string").to_string();
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
//! println!("Demonstrating {} and {}!",
//!          Blue.bold().paint("blue bold"),
//!          Yellow.underline().paint("yellow underline"));
//! ```
//!
//! These methods chain, so you can call them on existing Style
//! objects to set more than one particular properly, like so:
//!
//! ```rust
//! Blue.underline().bold().paint("Blue underline bold!")
//! ```
//!
//! You can set the background colour of a Style by using the `on`
//! method:
//!
//! ```rust
//! Blue.on(Yellow).paint("Blue on yellow!")
//! ```
//!
//! Finally, you can turn a Colour into a Style with the `normal`
//! method, though it'll produce the exact same string if you just use
//! the Colour. It's only useful if you're writing a method that can
//! return either normal or bold (or underline) styles, and need to
//! return a Style object from it.
//!
//! ```rust
//! Red.normal().paint("yet another red string")
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
use std::fmt;

/// An ANSI String is a string coupled with the Style to display it
/// in a terminal.
///
/// Although not technically a string itself, it can be turned into
/// one with the `to_string` method.
pub struct ANSIString<'a> {
    string: &'a str,
    style: Style,
}

impl<'a> Copy for ANSIString<'a> { }

impl<'a> ANSIString<'a> {
    /// Creates a new ANSI String with the given contents and style.
    pub fn new(contents: &'a str, style: Style) -> ANSIString<'a> {
        ANSIString { string: contents, style: style }
    }
}

impl<'a> fmt::Show for ANSIString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.style {
            Plain => write!(f, "{}", self.string),
            Foreground(colour) => {
                try!(f.write_str("\x1B["));
                try!(colour.write_foreground_code(f));
                write!(f, "m{}\x1B[0m", self.string)
            },
            Styled { foreground, background, bold, underline } => {
                try!(f.write_str("\x1B["));

                if bold {
                    try!(f.write_str("1;"));
                }

                if underline {
                    try!(f.write_str("4;"));
                }

                match background {
                    Some(c) => {
                        try!(c.write_background_code(f));
                        try!(f.write_str(";"));
                    },
                    None => {},
                }

                try!(foreground.write_foreground_code(f));

                write!(f, "m{}\x1B[0m", self.string)
            }
        }
    }
}

/// A colour is one specific type of ANSI escape code, and can refer
/// to either the foreground or background colour.
///
/// These use the standard numeric sequences.
/// See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html
pub enum Colour {
    Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed(u8),
}

impl Copy for Colour { }

// I'm not beyond calling Colour Colour, rather than Color, but I did
// purposefully name this crate 'ansi-term' so people wouldn't get
// confused when they tried to install it.
//
// Only *after* they'd installed it.

impl Colour {
    fn write_foreground_code(self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Black  => f.write_str("30"),
            Red    => f.write_str("31"),
            Green  => f.write_str("32"),
            Yellow => f.write_str("33"),
            Blue   => f.write_str("34"),
            Purple => f.write_str("35"),
            Cyan   => f.write_str("36"),
            White  => f.write_str("37"),
            Fixed(num) => write!(f, "38;5;{}", num),
        }
    }

    fn write_background_code(self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Black  => f.write_str("40"),
            Red    => f.write_str("41"),
            Green  => f.write_str("42"),
            Yellow => f.write_str("43"),
            Blue   => f.write_str("44"),
            Purple => f.write_str("45"),
            Cyan   => f.write_str("46"),
            White  => f.write_str("47"),
            Fixed(num) => write!(f, "48;5;{}", num),
        }
    }

    /// Return a Style with the foreground colour set to this colour.
    pub fn normal(self) -> Style {
        Styled { foreground: self, background: None, bold: false, underline: false }
    }

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don't have to use Blue.normal() just
    /// to get blue text.
    pub fn paint(self, input: &str) -> ANSIString {
        ANSIString::new(input, Foreground(self))
    }

    /// Returns a Style with the underline property set.
    pub fn underline(self) -> Style {
        Styled { foreground: self, background: None, bold: false, underline: true }
    }

    /// Returns a Style with the bold property set.
    pub fn bold(self) -> Style {
        Styled { foreground: self, background: None, bold: true, underline: false }
    }

    /// Returns a Style with the background colour property set.
    pub fn on(self, background: Colour) -> Style {
        Styled { foreground: self, background: Some(background), bold: false, underline: false }
    }
}

/// A style is a collection of properties that can format a string
/// using ANSI escape codes.
pub enum Style {

    /// The Plain style provides no formatting.
    Plain,

    /// The Foreground style provides just a foreground colour.
    Foreground(Colour),

    /// The Styled style is a catch-all for anything more complicated
    /// than that. It's technically possible for there to be other
    /// cases, such as "bold foreground", but probably isn't worth it.
    Styled { foreground: Colour, background: Option<Colour>, bold: bool, underline: bool, },
}

impl Copy for Style { }

impl Style {
    /// Paints the given text with this colour, returning an ANSI string.
    pub fn paint(self, input: &str) -> ANSIString {
        ANSIString::new(input, self)
    }

    /// Returns a Style with the bold property set.
    pub fn bold(self) -> Style {
        match self {
            Plain => Styled { foreground: White, background: None, bold: true, underline: false },
            Foreground(c) => Styled { foreground: c, background: None, bold: true, underline: false },
            Styled { foreground, background, bold: _, underline } => Styled { foreground: foreground, background: background, bold: true, underline: underline },
        }
    }

    /// Returns a Style with the underline property set.
    pub fn underline(self) -> Style {
        match self {
            Plain => Styled { foreground: White, background: None, bold: false, underline: true },
            Foreground(c) => Styled { foreground: c, background: None, bold: false, underline: true },
            Styled { foreground, background, bold, underline: _ } => Styled { foreground: foreground, background: background, bold: bold, underline: true },
        }
    }

    /// Returns a Style with the background colour property set.
    pub fn on(self, background: Colour) -> Style {
        match self {
            Plain => Styled { foreground: White,background: Some(background), bold: false, underline: false },
            Foreground(c) => Styled { foreground: c, background: Some(background), bold: false, underline: false },
            Styled { foreground, background: _, bold, underline } => Styled { foreground: foreground, background: Some(background), bold: bold, underline: underline },
        }
    }
}

/// Return a String with all ANSI escape codes removed.
///
/// For example:
///
/// ```rust
/// strip_formatting(Blue.paint("hello!").to_string()) == "hello!"
/// ```
pub fn strip_formatting(input: &str) -> String {
    // What's blue and smells like red paint? Blue paint.
    let re = regex!("\x1B\\[.+?m");
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::strip_formatting;
    use super::Style::Plain;
    use super::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed};

    macro_rules! test {
        ($name: ident: $style: expr $input: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style.paint($input).to_string(), $result.to_string())
            }
        };
    }

    test!(plain:                 Plain                             "text/plain" => "text/plain");
    test!(red:                   Red                               "hi" => "\x1B[31mhi\x1B[0m");
    test!(black:                 Black.normal()                    "hi" => "\x1B[30mhi\x1B[0m");
    test!(yellow_bold:           Yellow.bold()                     "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(yellow_bold_2:         Yellow.normal().bold()            "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(blue_underline:        Blue.underline()                  "hi" => "\x1B[4;34mhi\x1B[0m");
    test!(green_bold_ul:         Green.bold().underline()          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(green_bold_ul_2:       Green.underline().bold()          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(purple_on_white:       Purple.on(White)                  "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(purple_on_white_2:     Purple.normal().on(White)         "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(cyan_bold_on_white:    Cyan.bold().on(White)             "hi" => "\x1B[1;47;36mhi\x1B[0m");
    test!(cyan_ul_on_white:      Cyan.underline().on(White)        "hi" => "\x1B[4;47;36mhi\x1B[0m");
    test!(cyan_bold_ul_on_white: Cyan.bold().underline().on(White) "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(cyan_ul_bold_on_white: Cyan.underline().bold().on(White) "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(fixed:                 Fixed(100)                        "hi" => "\x1B[38;5;100mhi\x1B[0m");
    test!(fixed_on_purple:       Fixed(100).on(Purple)             "hi" => "\x1B[45;38;5;100mhi\x1B[0m");
    test!(fixed_on_fixed:        Fixed(100).on(Fixed(200))         "hi" => "\x1B[48;5;200;38;5;100mhi\x1B[0m");

    #[test]
    fn test_strip_formatting() {
        let hi = strip_formatting(Blue.paint("hi").to_string().as_slice());
        assert!(hi == "hi".to_string());
    }

    #[test]
    fn test_strip_formatting_2() {
        let hi = strip_formatting(Blue.on(Fixed(230)).bold().paint("hi").to_string().as_slice());
        assert!(hi == "hi".to_string());
    }
}
