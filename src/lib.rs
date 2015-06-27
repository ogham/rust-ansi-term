#![crate_name = "ansi_term"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

//! This is a library for controlling colours and formatting, such as
//! red bold text or blue underlined text, on ANSI terminals.
//!
//! ```rust
//! extern crate ansi_term;
//! use ansi_term::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
//! use ansi_term::Style;
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
//! Finally, for the sake of completeness, the default style provides
//! neither colours nor formatting.
//!
//! ```rust
//! Style::default().paint("No colours here.")
//! ```

use std::fmt;
use std::default::Default;

use Colour::*;
use Difference::*;


/// An ANSI String is a string coupled with the Style to display it
/// in a terminal.
///
/// Although not technically a string itself, it can be turned into
/// one with the `to_string` method.
#[derive(Clone, Copy)]
pub struct ANSIString<'a> {
    string: &'a str,
    style: Style,
}

impl<'a> ANSIString<'a> {
    /// Creates a new ANSI String with the given contents and style.
    pub fn new(contents: &'a str, style: Style) -> ANSIString<'a> {
        ANSIString { string: contents, style: style }
    }
}

impl<'a> fmt::Display for ANSIString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.style.write_prefix(f));
        try!(write!(f, "{}", self.string));
        self.style.write_suffix(f)
    }
}

/// A colour is one specific type of ANSI escape code, and can refer
/// to either the foreground or background colour.
///
/// These use the standard numeric sequences.
/// See http://invisible-island.net/xterm/ctlseqs/ctlseqs.html
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Colour {
    Black, Red, Green, Yellow, Blue, Purple, Cyan, White, Fixed(u8),
}

// I'm not beyond calling Colour Colour, rather than Color, but I did
// purposefully name this crate 'ansi-term' so people wouldn't get
// confused when they tried to install it.
//
// Only *after* they'd installed it.

impl Colour {
    fn foreground_code(&self) -> String {
        match *self {
            Black  => "30".to_string(),
            Red    => "31".to_string(),
            Green  => "32".to_string(),
            Yellow => "33".to_string(),
            Blue   => "34".to_string(),
            Purple => "35".to_string(),
            Cyan   => "36".to_string(),
            White  => "37".to_string(),
            Fixed(num) => format!("38;5;{}", &num),
        }
    }

    fn background_code(&self) -> String {
        match *self {
            Black  => "40".to_string(),
            Red    => "41".to_string(),
            Green  => "42".to_string(),
            Yellow => "43".to_string(),
            Blue   => "44".to_string(),
            Purple => "45".to_string(),
            Cyan   => "46".to_string(),
            White  => "47".to_string(),
            Fixed(num) => format!("48;5;{}", &num),
        }
    }

    /// Return a Style with the foreground colour set to this colour.
    pub fn normal(self) -> Style {
        Style { foreground: Some(self), .. Style::default() }
    }

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don't have to use Blue.normal() just
    /// to get blue text.
    pub fn paint(self, input: &str) -> ANSIString {
        ANSIString::new(input, self.normal())
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

    /// Returns a Style with the background colour property set.
    pub fn on(self, background: Colour) -> Style {
        Style { foreground: Some(self), background: Some(background), .. Style::default() }
    }
}

/// A style is a collection of properties that can format a string
/// using ANSI escape codes.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Style {
    foreground: Option<Colour>,
    background: Option<Colour>,
    is_bold: bool,
    is_dimmed: bool,
    is_italic: bool,
    is_underline: bool,
    is_blink: bool,
    is_reverse: bool,
    is_hidden: bool
}

impl Style {
    /// Creates a new Style with no differences.
    pub fn new() -> Style {
        Style::default()
    }

    /// Paints the given text with this colour, returning an ANSI string.
    pub fn paint(self, input: &str) -> ANSIString {
        ANSIString::new(input, self)
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

    /// Returns a Style with the background colour property set.
    pub fn on(&self, background: Colour) -> Style {
        Style { background: Some(background), .. *self }
    }

    fn prefix(&self) -> String {
        let mut prefix = String::new();
        let mut semicolon = false;

        if self.is_bold {
            prefix.push('1');
            semicolon = true;
        }

        if self.is_dimmed {
            if semicolon { prefix.push(';') }
            prefix.push('2');
            semicolon = true;
        }

        if self.is_italic {
            if semicolon { prefix.push(';') }
            prefix.push('3');
            semicolon = true;
        }

        if self.is_underline {
            if semicolon { prefix.push(';') }
            prefix.push('4');
            semicolon = true;
        }

        if self.is_blink {
            if semicolon { prefix.push(';') }
            prefix.push('5');
            semicolon = true;
        }

        if self.is_reverse {
            if semicolon { prefix.push(';') }
            prefix.push('6');
            semicolon = true;
        }

        if self.is_hidden {
            if semicolon { prefix.push(';') }
            prefix.push('7');
            semicolon = true;
        }

        if let Some(bg) = self.background {
            if semicolon { prefix.push(';'); }
            prefix.push_str(&bg.background_code());
            semicolon = true;
        }

        if let Some(fg) = self.foreground {
            if semicolon { prefix.push(';'); }
            prefix.push_str(&fg.foreground_code());
        }

        if prefix.len() != 0 {
            prefix = "\x1B[".to_string() + &prefix;
            prefix.push('m');
            prefix
        } else {
            prefix
        }
    }

    fn write_prefix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prefix())
    }

    fn write_suffix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self == &Style::default() {
            write!(f, "")
        } else {
            write!(f, "\x1B[0m")
        }
    }

    /// Compute the 'style difference' required to turn an existing style into
    /// the given, second style.
    ///
    /// For example, to turn green text into green bold text, it's redundant
    /// to write a reset command then a second green+bold command, instead of
    /// just writing one bold command. This method should see that both styles
    /// use the foreground colour green, and reduce it to a single command.
    ///
    /// This method returns an enum value because it's not actually always
    /// possible to turn one style into another: for example, text could be
    /// made bold and underlined, but you can't remove the bold property
    /// without also removing the underline property. So when this has to
    /// happen, this function returns None, meaning that the entire set of
    /// styles should be reset and begun again.
    fn difference(&self, next: &Style) -> Difference {
        // XXX(Havvy): This algorithm is kind of hard to replicate without
        // having the Plain/Foreground enum variants, so I'm just leaving
        // it commented out for now, and defaulting to Reset.

        if self == next {
            return NoDifference;
        }

        // Cannot un-bold, so must Reset.
        if self.is_bold && !next.is_bold {
            return Reset;
        }

        if self.is_dimmed && !next.is_dimmed {
            return Reset;
        }

        if self.is_italic && !next.is_italic {
            return Reset;
        }

        // Cannot un-underline, so must Reset.
        if self.is_underline && !next.is_underline {
            return Reset;
        }

        if self.is_blink && !next.is_blink {
            return Reset;
        }

        if self.is_reverse && !next.is_reverse {
            return Reset;
        }

        if self.is_hidden && !next.is_hidden {
            return Reset;
        }

        // Cannot go from foreground to no foreground, so must Reset.
        if self.foreground.is_some() && next.foreground.is_none() {
            return Reset;
        }

        // Cannot go from background to no background, so must Reset.
        if self.background.is_some() && next.background.is_none() {
            return Reset;
        }

        let mut extra_styles = Style::default();

        if self.is_bold != next.is_bold {
            extra_styles.is_bold = true;
        }

        if self.is_dimmed != next.is_dimmed {
            extra_styles.is_dimmed = true;
        }

        if self.is_italic != next.is_italic {
            extra_styles.is_italic = true;
        }

        if self.is_underline != next.is_underline {
            extra_styles.is_underline = true;
        }

        if self.is_blink != next.is_blink {
            extra_styles.is_blink = true;
        }

        if self.is_reverse != next.is_reverse {
            extra_styles.is_reverse = true;
        }

        if self.is_hidden != next.is_hidden {
            extra_styles.is_hidden = true;
        }

        if self.foreground != next.foreground {
            extra_styles.foreground = next.foreground;
        }

        if self.background != next.background {
            extra_styles.background = next.background;
        }

        ExtraStyles(extra_styles)
    }

    /// Return true if this `Style` has no actual styles, and can be written
    /// without any control characters.
    fn is_plain(self) -> bool {
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
            is_hidden: false
        }
    }
}

/// When printing out one coloured string followed by another, use one of
/// these rules to figure out which *extra* control codes need to be sent.
#[derive(PartialEq, Clone, Copy, Debug)]
enum Difference {

    /// Print out the control codes specified by this style to end up looking
    /// like the second string's styles.
    ExtraStyles(Style),

    /// Converting between these two is impossible, so just send a reset
    /// command and then the second string's styles.
    Reset,

    /// The before style is exactly the same as the after style, so no further
    /// control codes need to be printed.
    NoDifference,
}

/// A set of `ANSIString`s collected together, in order to be written with a
/// minimum of control characters.
pub struct ANSIStrings<'a>(pub &'a [ANSIString<'a>]);

impl<'a> fmt::Display for ANSIStrings<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	    let first = match self.0.first() {
	        None => return Ok(()),
	        Some(f) => f,
	    };

        try!(first.style.write_prefix(f));
	    try!(write!(f, "{}", first.string));

	    for window in self.0.windows(2) {
	        match window[0].style.difference(&window[1].style) {
	            ExtraStyles(style) => try!(style.write_prefix(f)),
	            Reset => {
                    try!(f.write_str("\x1B[0m"));
                    try!(window[1].style.write_prefix(f));
	            },
	            NoDifference => { /* Do nothing! */ },
	        }

            try!(write!(f, "{}", window[1].string));
	    }

        // Write the final reset string after all of the ANSIStrings have been
        // written, *except* if the last one has no styles, because it would
        // have already been written by this point.
        if let Some(last) = self.0.last() {
            if !last.style.is_plain() {
                try!(f.write_str("\x1B[0m"));
            }
        }

	    Ok(())
	}
}

#[cfg(test)]
mod tests {
    pub use super::Style;
    pub use super::Colour::*;
    pub use super::ANSIStrings;

    macro_rules! test {
        ($name: ident: $style: expr; $input: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style.paint($input).to_string(), $result.to_string())
            }
        };
    }

    test!(plain:                 Style::default();                  "text/plain" => "text/plain");
    test!(red:                   Red;                               "hi" => "\x1B[31mhi\x1B[0m");
    test!(black:                 Black.normal();                    "hi" => "\x1B[30mhi\x1B[0m");
    test!(yellow_bold:           Yellow.bold();                     "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(yellow_bold_2:         Yellow.normal().bold();            "hi" => "\x1B[1;33mhi\x1B[0m");
    test!(blue_underline:        Blue.underline();                  "hi" => "\x1B[4;34mhi\x1B[0m");
    test!(green_bold_ul:         Green.bold().underline();          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(green_bold_ul_2:       Green.underline().bold();          "hi" => "\x1B[1;4;32mhi\x1B[0m");
    test!(purple_on_white:       Purple.on(White);                  "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(purple_on_white_2:     Purple.normal().on(White);         "hi" => "\x1B[47;35mhi\x1B[0m");
    test!(cyan_bold_on_white:    Cyan.bold().on(White);             "hi" => "\x1B[1;47;36mhi\x1B[0m");
    test!(cyan_ul_on_white:      Cyan.underline().on(White);        "hi" => "\x1B[4;47;36mhi\x1B[0m");
    test!(cyan_bold_ul_on_white: Cyan.bold().underline().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(cyan_ul_bold_on_white: Cyan.underline().bold().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(fixed:                 Fixed(100);                        "hi" => "\x1B[38;5;100mhi\x1B[0m");
    test!(fixed_on_purple:       Fixed(100).on(Purple);             "hi" => "\x1B[45;38;5;100mhi\x1B[0m");
    test!(fixed_on_fixed:        Fixed(100).on(Fixed(200));         "hi" => "\x1B[48;5;200;38;5;100mhi\x1B[0m");
    test!(bold:                  Style::new().bold();               "hi" => "\x1B[1mhi\x1B[0m");
    test!(underline:             Style::new().underline();          "hi" => "\x1B[4mhi\x1B[0m");
    test!(bunderline:            Style::new().bold().underline();   "hi" => "\x1B[1;4mhi\x1B[0m");
    test!(dimmed:                Style::new().dimmed();             "hi" => "\x1B[2mhi\x1B[0m");
    test!(italic:                Style::new().italic();             "hi" => "\x1B[3mhi\x1B[0m");
    test!(blink:                 Style::new().blink();              "hi" => "\x1B[5mhi\x1B[0m");
    test!(reverse:               Style::new().reverse();            "hi" => "\x1B[6mhi\x1B[0m");
    test!(hidden:                Style::new().hidden();             "hi" => "\x1B[7mhi\x1B[0m");

    mod difference {
        pub use ::Difference::*;
        pub use super::*;

        #[test]
        fn diff() {
            let expected = ExtraStyles(Style::new().bold());
            let got = Green.normal().difference(&Green.bold());
            assert_eq!(expected, got)
        }

        #[test]
        fn dlb() {
            let got = Green.bold().difference(&Green.normal());
            assert_eq!(Reset, got)
        }

        #[test]
        fn nothing() {
            assert_eq!(NoDifference, Green.bold().difference(&Green.bold()));
        }

        #[test]
        fn nothing_2() {
            assert_eq!(NoDifference, Green.normal().difference(&Green.normal()));
        }

        #[test]
        fn colour_change() {
            assert_eq!(ExtraStyles(Blue.normal()), Red.normal().difference(&Blue.normal()))
        }

        #[test]
        fn removal_of_dimmed() {
            let dimmed = Style::new().dimmed();
            let normal = Style::default();

            assert_eq!(Reset, dimmed.difference(&normal));
        }

        #[test]
        fn addition_of_dimmed() {
            let dimmed = Style::new().dimmed();
            let normal = Style::default();
            let extra_styles = ExtraStyles(dimmed);

            assert_eq!(extra_styles, normal.difference(&dimmed));
        }

        #[test]
        fn removal_of_blink() {
            let blink = Style::new().blink();
            let normal = Style::default();

            assert_eq!(Reset, blink.difference(&normal));
        }

        #[test]
        fn addition_of_blink() {
            let blink = Style::new().blink();
            let normal = Style::default();
            let extra_styles = ExtraStyles(blink);

            assert_eq!(extra_styles, normal.difference(&blink));
        }

        #[test]
        fn removal_of_reverse() {
            let reverse = Style::new().reverse();
            let normal = Style::default();

            assert_eq!(Reset, reverse.difference(&normal));
        }

        #[test]
        fn addition_of_reverse() {
            let reverse = Style::new().reverse();
            let normal = Style::default();
            let extra_styles = ExtraStyles(reverse);

            assert_eq!(extra_styles, normal.difference(&reverse));
        }

        #[test]
        fn removal_of_hidden() {
            let hidden = Style::new().hidden();
            let normal = Style::default();

            assert_eq!(Reset, hidden.difference(&normal));
        }

        #[test]
        fn addition_of_hidden() {
            let hidden = Style::new().hidden();
            let normal = Style::default();
            let extra_styles = ExtraStyles(hidden);

            assert_eq!(extra_styles, normal.difference(&hidden));
        }

        #[test]
        fn no_control_codes_for_plain() {
            let one = Style::default().paint("one");
            let two = Style::default().paint("two");
            let output = format!("{}", ANSIStrings( &[ one, two ] ));
            assert_eq!(&*output, "onetwo");
        }
    }
}
