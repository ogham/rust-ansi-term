use std::fmt;
use std::io;
use style::{Style, Colour};

use difference::Difference;
use write::AnyWrite;
use super::{ANSIGenericStrings, ANSIString, ANSIStrings, ANSIGenericString, ANSIByteString, ANSIByteStrings};

// ---- generating ANSI codes ----

impl Style {

    /// Write any ANSI codes that go *before* a piece of text. These should be
    /// the codes to set the terminal to a different colour or font style.
    fn write_prefix<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
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
                if written_anything { try!(write!(f, ";")); }
                written_anything = true;
                try!(write!(f, "{}", c));
                Ok(())
            };

            if self.is_bold           { try!(write_char('1')); }
            if self.is_dimmed         { try!(write_char('2')); }
            if self.is_italic         { try!(write_char('3')); }
            if self.is_underline      { try!(write_char('4')); }
            if self.is_blink          { try!(write_char('5')); }
            if self.is_reverse        { try!(write_char('7')); }
            if self.is_hidden         { try!(write_char('8')); }
            if self.is_strikethrough  { try!(write_char('9')); }
        }

        // The foreground and background colours, if specified, need to be
        // handled specially because the number codes are more complicated.
        // (see `write_background_code` and `write_foreground_code`)
        if let Some(bg) = self.background {
            if written_anything { try!(write!(f, ";")); }
            written_anything = true;

            try!(bg.write_background_code(f));
        }

        if let Some(fg) = self.foreground {
            if written_anything { try!(write!(f, ";")); }

            try!(fg.write_foreground_code(f));
        }

        // All the codes end with an `m`, because reasons.
        try!(write!(f, "m"));
        Ok(())
    }

    /// Write any ANSI codes that go *after* a piece of text. These should be
    /// the codes to *reset* the terminal back to its normal colour and style.
    fn write_suffix<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        if self.is_plain() {
            Ok(())
        }
        else {
            write!(f, "\x1B[0m")
        }
    }
}


impl Colour {
    fn write_foreground_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        match *self {
            Colour::Black      => write!(f, "30"),
            Colour::Red        => write!(f, "31"),
            Colour::Green      => write!(f, "32"),
            Colour::Yellow     => write!(f, "33"),
            Colour::Blue       => write!(f, "34"),
            Colour::Purple     => write!(f, "35"),
            Colour::Cyan       => write!(f, "36"),
            Colour::White      => write!(f, "37"),
            Colour::Fixed(num) => write!(f, "38;5;{}", &num),
            Colour::RGB(r,g,b) => write!(f, "38;2;{};{};{}", &r, &g, &b),
        }
    }

    fn write_background_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
        match *self {
            Colour::Black      => write!(f, "40"),
            Colour::Red        => write!(f, "41"),
            Colour::Green      => write!(f, "42"),
            Colour::Yellow     => write!(f, "43"),
            Colour::Blue       => write!(f, "44"),
            Colour::Purple     => write!(f, "45"),
            Colour::Cyan       => write!(f, "46"),
            Colour::White      => write!(f, "47"),
            Colour::Fixed(num) => write!(f, "48;5;{}", &num),
            Colour::RGB(r,g,b) => write!(f, "48;2;{};{};{}", &r, &g, &b),
        }
    }
}


// ---- writers for individual ANSI strings ----

impl<'a> fmt::Display for ANSIString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: &mut fmt::Write = f;
        self.write_to_any(w)
    }
}

impl<'a> ANSIByteString<'a> {
    /// Write an ANSIByteString to an io::Write.  This writes the escape
    /// sequences for the associated Style around the bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let w: &mut io::Write = w;
        self.write_to_any(w)
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: fmt::Debug {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        try!(self.style.write_prefix(w));
        try!(w.write_str(&self.string));
        self.style.write_suffix(w)
    }
}


// ---- writers for combined ANSI strings ----

impl<'a> fmt::Display for ANSIStrings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        self.write_to_any(f)
    }
}

impl<'a> ANSIByteStrings<'a> {
    /// Write ANSIByteStrings to an io::Write.  This writes the minimal
    /// escape sequences for the associated Styles around each set of
    /// bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let w: &mut io::Write = w;
        self.write_to_any(w)
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericStrings<'a, S>
where <S as ToOwned>::Owned: fmt::Debug {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        let first = match self.0.first() {
            None => return Ok(()),
            Some(f) => f,
        };

        try!(first.style.write_prefix(w));
        try!(w.write_str(&first.string));

        for window in self.0.windows(2) {
            match window[0].style.difference(&window[1].style) {
                Difference::ExtraStyles(style) => try!(style.write_prefix(w)),
                Difference::Reset => {
                    try!(write!(w, "\x1B[0m"));
                    try!(window[1].style.write_prefix(w));
                },
                Difference::NoDifference => { /* Do nothing! */ },
            }

            try!(w.write_str(&window[1].string));
        }

        // Write the final reset string after all of the ANSIStrings have been
        // written, *except* if the last one has no styles, because it would
        // have already been written by this point.
        if let Some(last) = self.0.last() {
            if !last.style.is_plain() {
                try!(write!(w, "\x1B[0m"));
            }
        }

        Ok(())
    }
}


// ---- special structs for printing out style parts ----

/// Like `ANSIString`, but only displays the style prefix.
#[derive(Clone, Copy, Debug)]
pub struct Prefix(Style);

/// Like `ANSIString`, but only displays the difference between two
/// styles.
#[derive(Clone, Copy, Debug)]
pub struct Infix(Style, Style);

/// Like `ANSIString`, but only displays the style suffix.
#[derive(Clone, Copy, Debug)]
pub struct Suffix(Style);


impl Style {

    /// The prefix for this style.
    pub fn prefix(self) -> Prefix {
        Prefix(self)
    }

    /// The infix between this style and another.
    pub fn infix(self, other: Style) -> Infix {
        Infix(self, other)
    }

    /// The suffix for this style.
    pub fn suffix(self) -> Suffix {
        Suffix(self)
    }
}


impl Colour {

    /// The prefix for this colour.
    pub fn prefix(self) -> Prefix {
        Prefix(self.normal())
    }

    /// The infix between this colour and another.
    pub fn infix(self, other: Colour) -> Infix {
        Infix(self.normal(), other.normal())
    }

    /// The suffix for this colour.
    pub fn suffix(self) -> Suffix {
        Suffix(self.normal())
    }
}


impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        try!(self.0.write_prefix(f));
        Ok(())
    }
}


impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.difference(&self.1) {
            Difference::ExtraStyles(style) => {
                let f: &mut fmt::Write = f;
                try!(style.write_prefix(f))
            },
            Difference::Reset => {
                let f: &mut fmt::Write = f;
                try!(f.write_str("\x1B[0m"));
                try!(self.0.write_prefix(f));
            },
            Difference::NoDifference => { /* Do nothing! */ },
        }

        Ok(())
    }
}


impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        try!(self.0.write_suffix(f));
        Ok(())
    }
}


// ---- tests ----

#[cfg(test)]
mod tests {
    pub use super::super::ANSIStrings;
    pub use style::Style;
    pub use style::Colour::*;

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
    fn no_control_codes_for_plain() {
        let one = Style::default().paint("one");
        let two = Style::default().paint("two");
        let output = format!("{}", ANSIStrings( &[ one, two ] ));
        assert_eq!(&*output, "onetwo");
    }
}
