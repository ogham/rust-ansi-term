use style::{Colour, Style};

use std::fmt;
use std::str;


// ---- generating ANSI codes ----

/// A buffer to write prefix ANSI code into.  This allows the entire prefix code
/// to be formatted and then sent to Formatter or Write all at once.
// The length 54 corresponds to maximum number of bytes write_impl might
// write.  It is 2 bytes for `\x1B[` prefix, 9*2 bytes for all possible
// single-digit codes and 2*17 for foreground and background.
pub(super) struct PrefixBuffer([u8; 54]);

enum ColourCategory {
    Simple(u8),
    Fixed(u8),
    RGB(u8, u8, u8)
}

impl Default for PrefixBuffer {
    fn default() -> Self {
        PrefixBuffer([0; 54])
    }
}

impl PrefixBuffer {
    /// Returns ANSI code for given style.
    pub fn write(&'_ mut self, style: &Style) -> &'_ str {
        self.write_impl(style, false)
    }

    /// Returns ANSI code for given style including a reset sequence.
    pub fn write_with_reset(&'_ mut self, style: &Style) -> &'_ str {
        self.write_impl(style, true)
    }

    /// Returns ANSI code for given style optionally including a reset sequence.
    fn write_impl(&'_ mut self, style: &Style, with_reset: bool) -> &'_ str {
        // If there are actually no styles here, then don’t write *any* codes
        // as the prefix. An empty ANSI code may not affect the terminal
        // output at all, but a user may just want a code-free string.
        if style.is_plain() {
            return if with_reset { RESET } else { "" };
        }

        // Write the codes’ prefix, then write numbers, separated by
        // semicolons, for each text style we want to apply.
        self.0[..2].copy_from_slice(b"\x1B[");
        let mut idx = 2;

        {
            let mut write_char = |byte: u8| {
                self.0[idx] = byte;
                self.0[idx + 1] = b';';
                idx += 2;
            };

            if with_reset             { write_char(b'0'); }
            if style.is_bold          { write_char(b'1'); }
            if style.is_dimmed        { write_char(b'2'); }
            if style.is_italic        { write_char(b'3'); }
            if style.is_underline     { write_char(b'4'); }
            if style.is_blink         { write_char(b'5'); }
            if style.is_reverse       { write_char(b'7'); }
            if style.is_hidden        { write_char(b'8'); }
            if style.is_strikethrough { write_char(b'9'); }
        }

        // The foreground and background colours, if specified, need to be
        // handled specially because the number codes are more complicated.
        // (see `write_colour_category`)
        if let Some(bg) = style.background {
            idx = self.write_colour_category(idx, b'4', bg.colour_category());
        }
        if let Some(fg) = style.foreground {
            idx = self.write_colour_category(idx, b'3', fg.colour_category());
        }

        // Replace final `;` with a `m` which indicates end of the ANSI code.
        self.0[idx - 1] = b'm';

        // SAFETY: We’ve only ever written bytes <128 so everything written is
        // ASCII and thus valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.0[..idx]) }
    }

    /// Writes colour code at given position in the buffer.  Ends the sequence
    /// with a semicolon.  Returns index past the last written byte.
    ///
    /// May write up to 17 bytes.
    fn write_colour_category(
        &mut self,
        idx: usize,
        typ: u8,
        category: ColourCategory,
    ) -> usize {
        use std::io::Write;

        self.0[idx] = typ;
        match category {
            ColourCategory::Simple(digit) => {
                self.0[idx + 1] = digit;
                self.0[idx + 2] = b';';
                idx + 3
            },
            ColourCategory::Fixed(num) => {
                self.0.len() - {
                    let mut wr = &mut self.0[idx+1..];
                    write!(wr, "8;5;{};", num).unwrap();
                    wr.len()
                }
            }
            ColourCategory::RGB(r, g, b) => {
                self.0.len() - {
                    let mut wr = &mut self.0[idx+1..];
                    write!(wr, "8;2;{};{};{};", r, g, b).unwrap();
                    wr.len()
                }
            }
        }
    }
}

impl Style {
    /// Returns any bytes that go *after* a piece of text.
    pub(super) fn suffix_str(&self) -> &'static str {
        if self.is_plain() {
            ""
        } else {
            RESET
        }
    }
}


/// The code to send to reset all styles and return to `Style::default()`.
pub static RESET: &str = "\x1B[0m";


impl Colour {
    fn colour_category(&self) -> ColourCategory {
        match *self {
            Colour::Black      => ColourCategory::Simple(b'0'),
            Colour::Red        => ColourCategory::Simple(b'1'),
            Colour::Green      => ColourCategory::Simple(b'2'),
            Colour::Yellow     => ColourCategory::Simple(b'3'),
            Colour::Blue       => ColourCategory::Simple(b'4'),
            Colour::Purple     => ColourCategory::Simple(b'5'),
            Colour::Cyan       => ColourCategory::Simple(b'6'),
            Colour::White      => ColourCategory::Simple(b'7'),
            Colour::Fixed(num) => ColourCategory::Fixed(num),
            Colour::RGB(r,g,b) => ColourCategory::RGB(r, g, b),
        }
    }
}


/// Like `ANSIString`, but only displays the style prefix.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::prefix`](struct.Style.html#method.prefix).
#[derive(Clone, Copy, Debug)]
pub struct Prefix(Style);

/// Like `ANSIString`, but only displays the difference between two
/// styles.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::infix`](struct.Style.html#method.infix).
#[derive(Clone, Copy, Debug)]
pub struct Infix(Style, Style);

/// Like `ANSIString`, but only displays the style suffix.
///
/// This type implements the `Display` trait, meaning it can be written to a
/// `std::fmt` formatting without doing any extra allocation, and written to a
/// string with the `.to_string()` method. For examples, see
/// [`Style::suffix`](struct.Style.html#method.suffix).
#[derive(Clone, Copy, Debug)]
pub struct Suffix(Style);


impl Style {

    /// The prefix bytes for this style. These are the bytes that tell the
    /// terminal to use a different colour or font style.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Blue};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[1m",
    ///            style.prefix().to_string());
    ///
    /// let style = Blue.bold();
    /// assert_eq!("\x1b[1;34m",
    ///            style.prefix().to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.prefix().to_string());
    /// ```
    pub fn prefix(self) -> Prefix {
        Prefix(self)
    }

    /// The infix bytes between this style and `next` style. These are the bytes
    /// that tell the terminal to change the style to `next`. These may include
    /// a reset followed by the next colour and style, depending on the two styles.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Green};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[32m",
    ///            style.infix(Green.bold()).to_string());
    ///
    /// let style = Green.normal();
    /// assert_eq!("\x1b[1m",
    ///            style.infix(Green.bold()).to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.infix(style).to_string());
    /// ```
    pub fn infix(self, next: Style) -> Infix {
        Infix(self, next)
    }

    /// The suffix for this style. These are the bytes that tell the terminal
    /// to reset back to its normal colour and font style.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{Style, Colour::Green};
    ///
    /// let style = Style::default().bold();
    /// assert_eq!("\x1b[0m",
    ///            style.suffix().to_string());
    ///
    /// let style = Green.normal().bold();
    /// assert_eq!("\x1b[0m",
    ///            style.suffix().to_string());
    ///
    /// let style = Style::default();
    /// assert_eq!("",
    ///            style.suffix().to_string());
    /// ```
    pub fn suffix(self) -> Suffix {
        Suffix(self)
    }
}


impl Colour {

    /// The prefix bytes for this colour as a `Style`. These are the bytes
    /// that tell the terminal to use a different colour or font style.
    ///
    /// See also [`Style::prefix`](struct.Style.html#method.prefix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::Green;
    ///
    /// assert_eq!("\x1b[0m",
    ///            Green.suffix().to_string());
    /// ```
    pub fn prefix(self) -> Prefix {
        Prefix(self.normal())
    }

    /// The infix bytes between this colour and `next` colour. These are the bytes
    /// that tell the terminal to use the `next` colour, or to do nothing if
    /// the two colours are equal.
    ///
    /// See also [`Style::infix`](struct.Style.html#method.infix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::{Red, Yellow};
    ///
    /// assert_eq!("\x1b[33m",
    ///            Red.infix(Yellow).to_string());
    /// ```
    pub fn infix(self, next: Colour) -> Infix {
        Infix(self.normal(), next.normal())
    }

    /// The suffix for this colour as a `Style`. These are the bytes that
    /// tell the terminal to reset back to its normal colour and font style.
    ///
    /// See also [`Style::suffix`](struct.Style.html#method.suffix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Colour::Purple;
    ///
    /// assert_eq!("\x1b[0m",
    ///            Purple.suffix().to_string());
    /// ```
    pub fn suffix(self) -> Suffix {
        Suffix(self.normal())
    }
}


impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(PrefixBuffer::default().write(&self.0))
    }
}


impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use difference::Difference;
        let mut buf = PrefixBuffer::default();
        let prefix = match Difference::between(&self.0, &self.1) {
            Difference::ExtraStyles(style) => buf.write(&style),
            Difference::Reset => buf.write_with_reset(&self.1),
            Difference::NoDifference => return Ok(()),
        };
        f.write_str(prefix)
    }
}


impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.suffix_str())
    }
}



#[cfg(test)]
mod test {
    use style::Style;
    use style::Colour::*;

    macro_rules! test {
        ($name: ident: $style: expr; $input: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style.paint($input).to_string(), $result.to_string());

                let mut v = Vec::new();
                $style.paint($input.as_bytes()).write_to(&mut v).unwrap();
                assert_eq!(v.as_slice(), $result.as_bytes());
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
    test!(yellow_on_blue:        Style::new().on(Blue).fg(Yellow);  "hi" => "\x1B[44;33mhi\x1B[0m");
    test!(yellow_on_blue_2:      Cyan.on(Blue).fg(Yellow);          "hi" => "\x1B[44;33mhi\x1B[0m");
    test!(cyan_bold_on_white:    Cyan.bold().on(White);             "hi" => "\x1B[1;47;36mhi\x1B[0m");
    test!(cyan_ul_on_white:      Cyan.underline().on(White);        "hi" => "\x1B[4;47;36mhi\x1B[0m");
    test!(cyan_bold_ul_on_white: Cyan.bold().underline().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(cyan_ul_bold_on_white: Cyan.underline().bold().on(White); "hi" => "\x1B[1;4;47;36mhi\x1B[0m");
    test!(fixed:                 Fixed(100);                        "hi" => "\x1B[38;5;100mhi\x1B[0m");
    test!(fixed_on_purple:       Fixed(100).on(Purple);             "hi" => "\x1B[45;38;5;100mhi\x1B[0m");
    test!(fixed_on_fixed:        Fixed(100).on(Fixed(200));         "hi" => "\x1B[48;5;200;38;5;100mhi\x1B[0m");
    test!(rgb:                   RGB(70,130,180);                   "hi" => "\x1B[38;2;70;130;180mhi\x1B[0m");
    test!(rgb_on_blue:           RGB(70,130,180).on(Blue);          "hi" => "\x1B[44;38;2;70;130;180mhi\x1B[0m");
    test!(blue_on_rgb:           Blue.on(RGB(70,130,180));          "hi" => "\x1B[48;2;70;130;180;34mhi\x1B[0m");
    test!(rgb_on_rgb:            RGB(70,130,180).on(RGB(5,10,15));  "hi" => "\x1B[48;2;5;10;15;38;2;70;130;180mhi\x1B[0m");
    test!(bold:                  Style::new().bold();               "hi" => "\x1B[1mhi\x1B[0m");
    test!(underline:             Style::new().underline();          "hi" => "\x1B[4mhi\x1B[0m");
    test!(bunderline:            Style::new().bold().underline();   "hi" => "\x1B[1;4mhi\x1B[0m");
    test!(dimmed:                Style::new().dimmed();             "hi" => "\x1B[2mhi\x1B[0m");
    test!(italic:                Style::new().italic();             "hi" => "\x1B[3mhi\x1B[0m");
    test!(blink:                 Style::new().blink();              "hi" => "\x1B[5mhi\x1B[0m");
    test!(reverse:               Style::new().reverse();            "hi" => "\x1B[7mhi\x1B[0m");
    test!(hidden:                Style::new().hidden();             "hi" => "\x1B[8mhi\x1B[0m");
    test!(stricken:              Style::new().strikethrough();      "hi" => "\x1B[9mhi\x1B[0m");

    #[test]
    fn test_infix() {
        assert_eq!(Style::new().dimmed().infix(Style::new()).to_string(), "\x1B[0m");
        assert_eq!(White.dimmed().infix(White.normal()).to_string(), "\x1B[0;37m");
        assert_eq!(White.normal().infix(White.bold()).to_string(), "\x1B[1m");
        assert_eq!(White.normal().infix(Blue.normal()).to_string(), "\x1B[34m");
        assert_eq!(Blue.bold().infix(Blue.bold()).to_string(), "");
    }
}
