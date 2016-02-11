//! This is a library for controlling colours and formatting, such as
//! red bold text or blue underlined text, on ANSI terminals.
//!
//!
//! ## Basic usage
//!
//! There are two main data structures in this crate that you need to be
//! concerned with: `ANSIString` and `Style`. A `Style` holds stylistic
//! information: colours, whether the text should be bold, or blinking, or
//! whatever. There are also `Colour` variants that represent simple foreground
//! colour styles. An `ANSIString` is a string paired with a `Style`.
//!
//! (Yes, it’s British English, but you won’t have to write “colour” very often.
//! `Style` is used the majority of the time.)
//!
//! To format a string, call the `paint` method on a `Style` or a `Colour`,
//! passing in the string you want to format as the argument. For example,
//! here’s how to get some red text:
//!
//!     use ansi_term::Colour::Red;
//!     println!("This is in red: {}", Red.paint("a red string"));
//!
//! It’s important to note that the `paint` method does *not* actually return a
//! string with the ANSI control characters surrounding it. Instead, it returns
//! an `ANSIString` value that has a `Display` implementation that, when
//! formatted, returns the characters. This allows strings to be printed with a
//! minimum of `String` allocations being performed behind the scenes.
//!
//! If you *do* want to get at the escape codes, then you can convert the
//! `ANSIString` to a string as you would any other `Display` value:
//!
//!     use ansi_term::Colour::Red;
//!     use std::string::ToString;
//!     let red_string = Red.paint("a red string").to_string();
//!
//!
//! ## Bold, underline, background, and other styles
//!
//! For anything more complex than plain foreground colour changes, you need to
//! construct `Style` objects themselves, rather than beginning with a `Colour`.
//! You can do this by chaining methods based on a new `Style`, created with
//! `Style::new()`. Each method creates a new style that has that specific
//! property set. For example:
//!
//!     use ansi_term::Style;
//!     println!("How about some {} and {}?",
//!              Style::new().bold().paint("bold"),
//!              Style::new().underline().paint("underline"));
//!
//! For brevity, these methods have also been implemented for `Colour` values,
//! so you can give your styles a foreground colour without having to begin with
//! an empty `Style` value:
//!
//!     use ansi_term::Colour::{Blue, Yellow};
//!     println!("Demonstrating {} and {}!",
//!              Blue.bold().paint("blue bold"),
//!              Yellow.underline().paint("yellow underline"));
//!     println!("Yellow on blue: {}", Yellow.on(Blue).paint("wow!"));
//!
//! The complete list of styles you can use are: `bold`, `dimmed`, `italic`,
//! `underline`, `blink`, `reverse`, `hidden`, and `on` for background colours.
//!
//! Finally, you can turn a `Colour` into a `Style` with the `normal` method.
//! This will produce the exact same `ANSIString` as if you just used the
//! `paint` method on the `Colour` directly, but it’s useful in certain cases:
//! for example, you may have a method that returns `Styles`, and need to
//! represent both the “red bold” and “red, but not bold” styles with values of
//! the same type. The `Style` struct also has a `Default` implementation if you
//! want to have a style with *nothing* set.
//!
//!     use ansi_term::Style;
//!     use ansi_term::Colour::Red;
//!     Red.normal().paint("yet another red string");
//!     Style::default().paint("a completely regular string");
//!
//!
//! ## Extended colours
//!
//! You can access the extended range of 256 colours by using the `Fixed` colour
//! variant, which takes an argument of the colour number to use. This can be
//! included wherever you would use a `Colour`:
//!
//!     use ansi_term::Colour::Fixed;
//!     Fixed(134).paint("A sort of light purple");
//!     Fixed(221).on(Fixed(124)).paint("Mustard in the ketchup");
//!
//! The first sixteen of these values are the same as the normal and bold
//! standard colour variants. There’s nothing stopping you from using these as
//! `Fixed` colours instead, but there’s nothing to be gained by doing so
//! either.
//!
//!
//! ## Combining successive coloured strings
//!
//! The benefit of writing ANSI escape codes to the terminal is that they
//! *stack*: you do not need to end every coloured string with a reset code if
//! the text that follows it is of a similar style. For example, if you want to
//! have some blue text followed by some blue bold text, it’s possible to send
//! the ANSI code for blue, followed by the ANSI code for bold, and finishing
//! with a reset code without having to have an extra one between the two
//! strings.
//!
//! This crate can optimise the ANSI codes that get printed in situations like
//! this, making life easier for your terminal renderer. The `ANSIStrings`
//! struct takes a slice of several `ANSIString` values, and will iterate over
//! each of them, printing only the codes for the styles that need to be updated
//! as part of its formatting routine.
//!
//! The following code snippet uses this to enclose a binary number displayed in
//! red bold text inside some red, but not bold, brackets:
//!
//!     use ansi_term::Colour::Red;
//!     use ansi_term::{ANSIString, ANSIStrings};
//!     let some_value = format!("{:b}", 42);
//!     let strings: &[ANSIString<'static>] = &[
//!         Red.paint("["),
//!         Red.bold().paint(some_value),
//!         Red.paint("]"),
//!     ];
//!     println!("Value: {}", ANSIStrings(strings));
//!
//! There are several things to note here. Firstly, the `paint` method can take
//! *either* an owned `String` or a borrowed `&str`. Internally, an `ANSIString`
//! holds a copy-on-write (`Cow`) string value to deal with both owned and
//! borrowed strings at the same time. This is used here to display a `String`,
//! the result of the `format!` call, using the same mechanism as some
//! statically-available `&str` slices. Secondly, that the `ANSIStrings` value
//! works in the same way as its singular counterpart, with a `Display`
//! implementation that only performs the formatting when required.


#![crate_name = "ansi_term"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_extern_crates, unused_qualifications)]


use std::borrow::Cow;
use std::default::Default;
use std::fmt;
use std::ops::Deref;

use Colour::*;
use Difference::*;


/// An ANSI String is a string coupled with the Style to display it
/// in a terminal.
///
/// Although not technically a string itself, it can be turned into
/// one with the `to_string` method.
///
/// ### Examples
///
/// ```no_run
/// use ansi_term::ANSIString;
/// use ansi_term::Colour::Red;
///
/// let red_string = Red.paint("a red string");
/// println!("{}", red_string);
/// ```
///
/// ```
/// use ansi_term::ANSIString;
///
/// let plain_string = ANSIString::from("a plain string");
/// assert_eq!(&*plain_string, "a plain string");
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct ANSIString<'a> {
    string: Cow<'a, str>,
    style: Style,
}

impl<'a> fmt::Display for ANSIString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.style.write_prefix(f));
        try!(write!(f, "{}", self.string));
        self.style.write_suffix(f)
    }
}

impl<'a, S> From<S> for ANSIString<'a>
where S: Into<Cow<'a, str>> {
    fn from(input: S) -> ANSIString<'a> {
        ANSIString {
            string: input.into(),
            style:  Style::default(),
        }
    }
}

impl<'a> Deref for ANSIString<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.string.deref()
    }
}


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
    /// It might make more sense to look at a [colour chart](^cc).
    /// [^cc]: https://upload.wikimedia.org/wikipedia/en/1/15/Xterm_256color_chart.svg
    Fixed(u8),
}

// I'm not beyond calling Colour Colour, rather than Color, but I did
// purposefully name this crate 'ansi-term' so people wouldn't get
// confused when they tried to install it.
//
// Only *after* they'd installed it.

impl Colour {
    fn write_foreground_code(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Black      => write!(f, "30"),
            Red        => write!(f, "31"),
            Green      => write!(f, "32"),
            Yellow     => write!(f, "33"),
            Blue       => write!(f, "34"),
            Purple     => write!(f, "35"),
            Cyan       => write!(f, "36"),
            White      => write!(f, "37"),
            Fixed(num) => write!(f, "38;5;{}", &num),
        }
    }

    fn write_background_code(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Black      => write!(f, "40"),
            Red        => write!(f, "41"),
            Green      => write!(f, "42"),
            Yellow     => write!(f, "43"),
            Blue       => write!(f, "44"),
            Purple     => write!(f, "45"),
            Cyan       => write!(f, "46"),
            White      => write!(f, "47"),
            Fixed(num) => write!(f, "48;5;{}", &num),
        }
    }

    /// Return a Style with the foreground colour set to this colour.
    pub fn normal(self) -> Style {
        Style { foreground: Some(self), .. Style::default() }
    }

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don't have to use Blue.normal() just
    /// to get blue text.
    pub fn paint<'a, S>(self, input: S) -> ANSIString<'a>
    where S: Into<Cow<'a, str>> {
        ANSIString {
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
    pub fn paint<'a, S>(self, input: S) -> ANSIString<'a>
    where S: Into<Cow<'a, str>> {
        ANSIString {
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

    /// Returns a Style with the background colour property set.
    pub fn on(&self, background: Colour) -> Style {
        Style { background: Some(background), .. *self }
    }

    /// Write any ANSI codes that go *before* a piece of text. These should be
    /// the codes to set the terminal to a different colour or font style.
    fn write_prefix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Write;

        // If there are actually no styles here, then don’t write *any* codes
        // as the prefix. An empty ANSI code may not affect the terminal
        // output at all, but a user may just want a code-free string.
        if self.is_plain() {
            return Ok(());
        }

        // Write the codes’ prefix, then write numbers, separated by
        // semicolons, for each text style we want to apply.
        try!(write!(f, "\x1B["));
        let mut written_anything = false;

        {
            let mut write_char = |c| {
                if written_anything { try!(f.write_char(';')); }
                written_anything = true;
                try!(f.write_char(c));
                Ok(())
            };

            if self.is_bold       { try!(write_char('1')); }
            if self.is_dimmed     { try!(write_char('2')); }
            if self.is_italic     { try!(write_char('3')); }
            if self.is_underline  { try!(write_char('4')); }
            if self.is_blink      { try!(write_char('5')); }
            if self.is_reverse    { try!(write_char('6')); }
            if self.is_hidden     { try!(write_char('7')); }
        }

        // The foreground and background colours, if specified, need to be
        // handled specially because the number codes are more complicated.
        // (see `write_background_code` and `write_foreground_code`)
        if let Some(bg) = self.background {
            if written_anything { try!(f.write_char(';')); }
            written_anything = true;

            try!(bg.write_background_code(f));
        }

        if let Some(fg) = self.foreground {
            if written_anything { try!(f.write_char(';')); }

            try!(fg.write_foreground_code(f));
        }

        // All the codes end with an `m`, because reasons.
        try!(f.write_char('m'));
        Ok(())
    }

    /// Write any ANSI codes that go *after* a piece of text. These should be
    /// the codes to *reset* the terminal back to its normal colour and style.
    fn write_suffix(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_plain() {
            Ok(())
        }
        else {
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
    pub use super::{Style, ANSIStrings};
    pub use super::Colour::*;

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
        use super::*;
        use super::super::Difference::*;

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
