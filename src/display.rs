use std::borrow::Cow;
use std::fmt;
use std::io;

use ansi::PrefixBuffer;
use difference::Difference;
use style::{Style, Colour};


/// An `ANSIString` includes a generic string type and a `Style` to display that
/// string.
///
/// If the generic type implements `Display`, this value can be displayed or
/// turned into a string using `to_string` method.  Similarly, if the generic
/// type implements `AsRef<[u8]>`, this value can be written to arbitrary byte
/// stream with ANSI codes surrounding it.
///
/// # Examples
///
/// ```
/// use ansi_term::{ANSIString, Colour};
///
/// let red: ANSIString<_> = Colour::Red.paint("red");
/// println!("A {red} string");
///
/// let red = red.to_string();
/// let message = ["A ", &red, " string"].concat();
/// assert_eq!("A \x1b[31mred\x1B[0m string", message);
/// ```
///
/// ```
/// use ansi_term::{ANSIString, Colour};
///
/// let green = Colour::Green.paint("A green string".as_bytes());
/// green.write_to(&mut std::io::stdout()).unwrap();
/// let mut buf = [0; 23];
/// green.write_to(&mut &mut buf[..]).unwrap();
/// assert_eq!(b"\x1b[32mA green string\x1b[0m", &buf[..]);
/// ```
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ANSIString<S> {
    /// Style of the value.
    pub style: Style,
    /// Value of the string.
    pub value: S,
}

impl<'a, S, I> From<I> for ANSIString<Cow<'a, S>>
where S: 'a + ToOwned + ?Sized,
      I: Into<Cow<'a, S>> {
    fn from(input: I) -> Self {
        ANSIString {
            style: Style::default(),
            value: input.into(),
        }
    }
}

impl<S> ANSIString<S> {
    /// Creates a new object with default style.
    pub fn new<T: Into<S>>(value: T) -> Self {
        Self {
            style: Style::default(),
            value: value.into(),
        }
    }
}


/// A set of `ANSIString`s collected together, in order to be
/// written with a minimum of control characters.
#[derive(Debug, PartialEq)]
pub struct ANSIStrings<'a, S: 'a>(pub &'a [ANSIString<S>]);

impl<'a, S: 'a> ANSIStrings<'a, S> {
    /// Returns iterator over all string values stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{ANSIString, ANSIStrings, Colour, Style};
    ///
    /// let strings = [
    ///     Colour::Red.paint("Red"),
    ///     Style::default().paint(" "),
    ///     Colour::Green.paint("Green"),
    ///     Style::default().paint(" "),
    ///     Colour::Blue.paint("Blue"),
    /// ];
    /// let strings = ANSIStrings(&strings);
    ///
    /// let unstyled_len = strings.values().map(|val| val.len()).sum::<usize>();
    /// assert_eq!(14, unstyled_len);
    ///
    /// let unstyled = strings.values().map(|&value| value).collect::<String>();
    /// assert_eq!("Red Green Blue", unstyled);
    /// ```
    pub fn values(&self) -> impl Iterator<Item = &S> {
        self.0.iter().map(|string| &string.value)
    }
}

// ---- paint functions ----

impl Style {
    /// Paints the given text with this colour, returning an ANSI string.
    #[must_use]
    #[inline]
    pub fn paint<S>(self, input: S) -> ANSIString<S> {
        ANSIString {
            value: input,
            style:  self,
        }
    }

    /// Paints the given text with this colour, returning an ANSI string.
    #[must_use]
    #[inline]
    pub fn paint_cow<'a, S, I>(self, input: I) -> ANSIString<Cow<'a, S>>
    where S: 'a + ToOwned + ?Sized,
          I: Into<Cow<'a, S>> {
        ANSIString {
            value: input.into(),
            style:  self,
        }
    }
}


impl Colour {
    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don’t have to use `Blue.normal()` just
    /// to get blue text.
    ///
    /// ```
    /// use ansi_term::Colour::Blue;
    /// println!("{}", Blue.paint("da ba dee"));
    /// ```
    #[must_use]
    #[inline]
    pub fn paint<S>(self, input: S) -> ANSIString<S> {
        self.normal().paint(input)
    }

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don’t have to use `Blue.normal()` just
    /// to get blue text.
    ///
    /// ```
    /// use ansi_term::Colour::Blue;
    /// println!("{}", Blue.paint_cow("da ba dee"));
    /// ```
    #[must_use]
    #[inline]
    pub fn paint_cow<'a, S, I>(self, input: I) -> ANSIString<Cow<'a, S>>
    where S: 'a + ToOwned + ?Sized,
          I: Into<Cow<'a, S>> {
        self.normal().paint_cow(input)
    }
}


// ---- Display et al ----

macro_rules! display_impl {
    ($trait:ident, $write:ident) => {
        impl<S: fmt::$trait> fmt::$trait for ANSIString<S> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(PrefixBuffer::default().write(&self.style))?;
                self.value.fmt(fmt)?;
                fmt.write_str(self.style.suffix_str())
            }
        }

        struct $write<'a, 'b: 'a>(pub &'a mut fmt::Formatter<'b>);

        impl<'a, 'b, V: fmt::$trait> AnyWrite<V> for $write<'a, 'b> {
            type Error = fmt::Error;

            fn write(&mut self, code: &str, value: &V) -> Result<(), Self::Error> {
                self.0.write_str(code)?;
                value.fmt(self.0)
            }

            fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
                self.0.write_str(s)
            }
        }

        impl<'a, S: fmt::$trait> fmt::$trait for ANSIStrings<'a, S> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.write_to_any($write(f))
            }
        }
    }
}

display_impl!(Binary, BinaryWrite);
display_impl!(Display, DisplayWrite);
display_impl!(LowerExp, LowerExpWrite);
display_impl!(LowerHex, LowerHexWrite);
display_impl!(Octal, OctalWrite);
display_impl!(Pointer, PointerWrite);
display_impl!(UpperExp, UpperExpWrite);
display_impl!(UpperHex, UpperHexWrite);


// ---- Writes for binary strings ----

impl<S: AsRef<[u8]>> ANSIString<S> {
    /// Write an `ANSIString` to an `io::Write`.  This writes the escape
    /// sequences for the associated `Style` around the bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(PrefixBuffer::default().write(&self.style).as_bytes())?;
        w.write_all(self.value.as_ref())?;
        w.write_all(self.style.suffix_str().as_bytes())?;
        Ok(())
    }
}

impl<'a, S: AsRef<[u8]>> ANSIStrings<'a, S> {
    /// Write `ANSIStrings` to an `io::Write`.  This writes the minimal
    /// escape sequences for the associated `Style`s around each set of bytes.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        self.write_to_any(IOWrite(w))
    }
}


// ---- writer for combined ANSI strings ----

impl<'a, S> ANSIStrings<'a, S> {
    fn write_to_any<W: AnyWrite<S>>(&self, mut wr: W) -> Result<(), W::Error> {
        use self::Difference::*;

        let mut buf = PrefixBuffer::default();
        match self.0.first() {
            None => return Ok(()),
            Some(first) => wr.write(buf.write(&first.style), &first.value)?,
        }

        for window in self.0.windows(2) {
            let code = match Difference::between(&window[0].style, &window[1].style) {
                ExtraStyles(style) => buf.write(&style),
                Reset              => buf.write_with_reset(&window[1].style),
                NoDifference       => "",
            };
            wr.write(code, &window[1].value)?;
        }

        if let Some(last) = self.0.last() {
            wr.write_str(last.style.suffix_str())?;
        }

        Ok(())
    }
}

trait AnyWrite<V> {
    type Error;

    fn write(&mut self, code: &str, value: &V) -> Result<(), Self::Error>;
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;
}

struct IOWrite<'a, W: 'a>(pub &'a mut W);

impl<'a, W: io::Write, V: AsRef<[u8]>> AnyWrite<V> for IOWrite<'a, W> {
    type Error = io::Error;

    fn write(&mut self, code: &str, value: &V) -> Result<(), Self::Error> {
        self.0.write_all(code.as_bytes())?;
        self.0.write_all(value.as_ref())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())
    }
}


// ---- tests ----

#[test]
fn no_control_codes_for_plain() {
    use std::borrow::Cow;

    let one = Style::default().paint(Cow::Borrowed("one"));
    let two = Style::default().paint(Cow::Borrowed("two"));
    let output = format!("{}", ANSIStrings( &[ one, two ] ));
    assert_eq!(&*output, "onetwo");
}
