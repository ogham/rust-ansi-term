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
//! `underline`, `blink`, `reverse`, `hidden`, `strikethrough`, and `on` for
//! background colours.
//!
//! In some cases, you may find it easier to change the foreground on an
//! existing `Style` rather than starting from the appropriate `Colour`.
//! You can do this using the `fg` method:
//!
//!     use ansi_term::Style;
//!     use ansi_term::Colour::{Blue, Cyan, Yellow};
//!     println!("Yellow on blue: {}", Style::new().on(Blue).fg(Yellow).paint("yow!"));
//!     println!("Also yellow on blue: {}", Cyan.on(Blue).fg(Yellow).paint("zow!"));
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
//! You can also access full 24-bit color by using the `RGB` colour variant,
//! which takes separate `u8` arguments for red, green, and blue:
//!
//!     use ansi_term::Colour::RGB;
//!     RGB(70, 130, 180).paint("Steel blue");
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
//!
//! ## Byte strings
//!
//! This library also supports formatting `[u8]` byte strings; this supports
//! applications working with text in an unknown encoding.  `Style` and
//! `Color` support painting `[u8]` values, resulting in an `ANSIByteString`.
//! This type does not implement `Display`, as it may not contain UTF-8, but
//! it does provide a method `write_to` to write the result to any
//! `io::Write`:
//!
//!     use ansi_term::Colour::Green;
//!     Green.paint("user data".as_bytes()).write_to(&mut std::io::stdout()).unwrap();
//!
//! Similarly, the type `ANSIByteStrings` supports writing a list of
//! `ANSIByteString` values with minimal escape sequences:
//!
//!     use ansi_term::Colour::Green;
//!     use ansi_term::ANSIByteStrings;
//!     ANSIByteStrings(&[
//!         Green.paint("user data 1\n".as_bytes()),
//!         Green.bold().paint("user data 2\n".as_bytes()),
//!     ]).write_to(&mut std::io::stdout()).unwrap();


#![crate_name = "ansi_term"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_extern_crates, unused_qualifications)]


use std::borrow::Cow;
use std::default::Default;
use std::io;
use std::ops::Deref;

use Colour::*;

mod difference;
use difference::Difference;

mod display;

mod write;
use write::AnyWrite;


/// An ANSIGenericString includes a generic string type and a Style to
/// display that string.  ANSIString and ANSIByteString are aliases for
/// this type on str and [u8], respectively.
#[derive(PartialEq, Debug, Clone)]
pub struct ANSIGenericString<'a, S: 'a + ToOwned + ?Sized>
where <S as ToOwned>::Owned: std::fmt::Debug {
    style: Style,
    string: Cow<'a, S>,
}


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
pub type ANSIString<'a> = ANSIGenericString<'a, str>;

/// An ANSIByteString represents a formatted series of bytes.  Use
/// ANSIByteString when styling text with an unknown encoding.
pub type ANSIByteString<'a> = ANSIGenericString<'a, [u8]>;

/// Like `ANSIString`, but only displays the style prefix.
#[derive(Clone, Copy, Debug)]
pub struct Prefix(Style);

/// Like `ANSIString`, but only displays the style suffix.
#[derive(Clone, Copy, Debug)]
pub struct Suffix(Style);

/// Like `ANSIString`, but only displays the difference between two
/// styles.
#[derive(Clone, Copy, Debug)]
pub struct Infix(Style, Style);



impl<'a, I, S: 'a + ToOwned + ?Sized> From<I> for ANSIGenericString<'a, S>
where I: Into<Cow<'a, S>>,
      <S as ToOwned>::Owned: std::fmt::Debug {
    fn from(input: I) -> ANSIGenericString<'a, S> {
        ANSIGenericString {
            string: input.into(),
            style:  Style::default(),
        }
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> Deref for ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: std::fmt::Debug {
    type Target = S;

    fn deref(&self) -> &S {
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
    /// It might make more sense to look at a [colour chart][cc].
    /// [cc]: https://upload.wikimedia.org/wikipedia/en/1/15/Xterm_256color_chart.svg
    Fixed(u8),

    /// A 24-bit RGB color, as specified by ISO-8613-3.
    RGB(u8, u8, u8),
}

/// Color is a type alias for Colour for those who can't be bothered.
pub use Colour as Color;

// I'm not beyond calling Colour Colour, rather than Color, but I did
// purposefully name this crate 'ansi-term' so people wouldn't get
// confused when they tried to install it.
//
// Only *after* they'd installed it.

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
          <S as ToOwned>::Owned: std::fmt::Debug {
        ANSIGenericString {
            string: input.into(),
            style:  self.normal(),
        }
    }

    /// The prefix for this colour.
    pub fn prefix(self) -> Prefix {
        Prefix(self.normal())
    }

    /// The suffix for this colour.
    pub fn suffix(self) -> Suffix {
        Suffix(self.normal())
    }

    /// The infix between this colour and another.
    pub fn infix(self, other: Colour) -> Infix {
        Infix(self.normal(), other.normal())
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
    is_hidden: bool,
    is_strikethrough: bool
}

impl Style {
    /// Creates a new Style with no differences.
    pub fn new() -> Style {
        Style::default()
    }

    /// Paints the given text with this colour, returning an ANSI string.
    pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> ANSIGenericString<'a, S>
    where I: Into<Cow<'a, S>>,
          <S as ToOwned>::Owned: std::fmt::Debug {
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
            is_hidden: false,
            is_strikethrough: false,
        }
    }
}


/// A set of `ANSIGenericString`s collected together, in order to be
/// written with a minimum of control characters.
pub struct ANSIGenericStrings<'a, S: 'a + ToOwned + ?Sized>
    (pub &'a [ANSIGenericString<'a, S>])
    where <S as ToOwned>::Owned: std::fmt::Debug;

/// A set of `ANSIString`s collected together, in order to be written with a
/// minimum of control characters.
pub type ANSIStrings<'a> = ANSIGenericStrings<'a, str>;

/// A function to construct an ANSIStrings instance.
#[allow(non_snake_case)]
pub fn ANSIStrings<'a>(arg: &'a [ANSIString<'a>]) -> ANSIStrings<'a> {
    ANSIGenericStrings(arg)
}

/// A set of `ANSIByteString`s collected together, in order to be
/// written with a minimum of control characters.
pub type ANSIByteStrings<'a> = ANSIGenericStrings<'a, [u8]>;

/// A function to construct an ANSIByteStrings instance.
#[allow(non_snake_case)]
pub fn ANSIByteStrings<'a>(arg: &'a [ANSIByteString<'a>]) -> ANSIByteStrings<'a> {
    ANSIGenericStrings(arg)
}



/// Enable ansi code support on windows 10
/// Returns a Result with the windows error code if unsuccessful
#[cfg(windows)]
pub fn enable_ansi_support() -> Result<(), u64> {
    #[link(name = "kernel32")]
    extern {
        fn GetStdHandle(handle: u64) -> *const i32;
        fn SetConsoleMode(handle: *const i32, mode: u32) -> bool;
        fn GetLastError() -> u64;
    }

    unsafe {
        const STD_OUT_HANDLE: u64 = -11i32 as u64;
        const ENABLE_ANSI_CODES: u32 = 7;

        let std_out_handle = GetStdHandle(STD_OUT_HANDLE);
        let error_code = GetLastError();
        if error_code != 0 { return Err(error_code); }

        SetConsoleMode(std_out_handle, ENABLE_ANSI_CODES);
        let error_code = GetLastError();
        if error_code != 0 { return Err(error_code); }
    }
    return Ok(());
}
