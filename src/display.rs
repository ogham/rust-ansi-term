use std::fmt;
use super::*;


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


impl Style {

    /// The prefix for this style.
    pub fn prefix(self) -> Prefix {
        Prefix(self)
    }

    /// The suffix for this style.
    pub fn suffix(self) -> Suffix {
        Suffix(self)
    }

    /// The infix between this style and another.
    pub fn infix(self, other: Style) -> Infix {
        Infix(self, other)
    }
}

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
        use self::Difference::*;

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

        if self.is_strikethrough && !next.is_strikethrough {
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

        if self.is_strikethrough != next.is_strikethrough {
            extra_styles.is_strikethrough = true;
        }

        if self.foreground != next.foreground {
            extra_styles.foreground = next.foreground;
        }

        if self.background != next.background {
            extra_styles.background = next.background;
        }

        ExtraStyles(extra_styles)
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


impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        try!(self.0.write_prefix(f));
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

impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Difference::*;

        match self.0.difference(&self.1) {
            ExtraStyles(style) => {
                let f: &mut fmt::Write = f;
                try!(style.write_prefix(f))
            },
            Reset => {
                let f: &mut fmt::Write = f;
                try!(f.write_str("\x1B[0m"));
                try!(self.0.write_prefix(f));
            },
            NoDifference => { /* Do nothing! */ },
        }

        Ok(())
    }
}


impl Colour {
    fn write_foreground_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
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
            RGB(r,g,b) => write!(f, "38;2;{};{};{}", &r, &g, &b),
        }
    }

    fn write_background_code<W: AnyWrite + ?Sized>(&self, f: &mut W) -> Result<(), W::Error> {
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
            RGB(r,g,b) => write!(f, "48;2;{};{};{}", &r, &g, &b),
        }
    }
}


impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericStrings<'a, S>
where <S as ToOwned>::Owned: std::fmt::Debug {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        use self::Difference::*;

        let first = match self.0.first() {
            None => return Ok(()),
            Some(f) => f,
        };

        try!(first.style.write_prefix(w));
        try!(w.write_str(&first.string));

        for window in self.0.windows(2) {
            match window[0].style.difference(&window[1].style) {
                ExtraStyles(style) => try!(style.write_prefix(w)),
                Reset => {
                    try!(write!(w, "\x1B[0m"));
                    try!(window[1].style.write_prefix(w));
                },
                NoDifference => { /* Do nothing! */ },
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

impl<'a> fmt::Display for ANSIStrings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let f: &mut fmt::Write = f;
        self.write_to_any(f)
    }
}

impl<'a, S: 'a + ToOwned + ?Sized> ANSIGenericString<'a, S>
where <S as ToOwned>::Owned: std::fmt::Debug {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        try!(self.style.write_prefix(w));
        try!(w.write_str(&self.string));
        self.style.write_suffix(w)
    }
}

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

impl<'a> ANSIByteStrings<'a> {
    /// Write ANSIByteStrings to an io::Write.  This writes the minimal
    /// escape sequences for the associated Styles around each set of
    /// bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let w: &mut io::Write = w;
        self.write_to_any(w)
    }
}


#[cfg(test)]
mod tests {
    pub use super::super::{Style, ANSIStrings};
    pub use super::Colour::*;

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
        fn removal_of_strikethrough() {
            let strikethrough = Style::new().strikethrough();
            let normal = Style::default();

            assert_eq!(Reset, strikethrough.difference(&normal));
        }

        #[test]
        fn addition_of_strikethrough() {
            let strikethrough = Style::new().strikethrough();
            let normal = Style::default();
            let extra_styles = ExtraStyles(strikethrough);

            assert_eq!(extra_styles, normal.difference(&strikethrough));
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
