use std::fmt;
use std::io;

use difference::Difference;
use write::AnyWrite;
use ansi::RESET;
use super::{ANSIGenericStrings, ANSIString, ANSIStrings, ANSIGenericString, ANSIByteString, ANSIByteStrings};


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
where <S as ToOwned>::Owned: fmt::Debug, &'a S: AsRef<[u8]> {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        write!(w, "{}", self.style.prefix())?;
        w.write_str(self.string.as_ref())?;
        write!(w, "{}", self.style.suffix())
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
where <S as ToOwned>::Owned: fmt::Debug, &'a S: AsRef<[u8]> {
    fn write_to_any<W: AnyWrite<wstr=S> + ?Sized>(&self, w: &mut W) -> Result<(), W::Error> {
        use self::Difference::*;

        let first = match self.0.first() {
            None => return Ok(()),
            Some(f) => f,
        };

        write!(w, "{}", first.style.prefix())?;
        w.write_str(first.string.as_ref())?;

        for window in self.0.windows(2) {
            match Difference::between(&window[0].style, &window[1].style) {
                ExtraStyles(style) => write!(w, "{}", style.prefix())?,
                Reset              => write!(w, "{}{}", RESET, window[1].style.prefix())?,
                NoDifference       => {/* Do nothing! */},
            }

            w.write_str(&window[1].string)?;
        }

        // Write the final reset string after all of the ANSIStrings have been
        // written, *except* if the last one has no styles, because it would
        // have already been written by this point.
        if let Some(last) = self.0.last() {
            if !last.style.is_plain() {
                write!(w, "{}", RESET)?;
            }
        }

        Ok(())
    }
}


// ---- tests ----

#[cfg(test)]
mod tests {
    pub use super::super::ANSIStrings;
    pub use style::Style;
    pub use style::Colour::*;

    #[test]
    fn no_control_codes_for_plain() {
        let one = Style::default().paint("one");
        let two = Style::default().paint("two");
        let output = format!("{}", ANSIStrings( &[ one, two ] ));
        assert_eq!(&*output, "onetwo");
    }
}
