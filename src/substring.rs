//! Implementation of the substring operation on [`ANSIStrings`].
//!
//! See [`ANSIStrings::substring`] method.

use std::borrow::Cow;
use std::ops::Range;
use std::iter;

use display::{ANSIString, ANSIStrings};


/// Iterator over a substring of an [`ANSIStrings`].
///
/// Created by [`ANSIStrings::substring`].
///
/// `S` generic argument correspond to generic argument of `ANSIStrings`.  For
/// this type to be an iterator it `S` must implement [`Substringable`] trait.
pub struct Substring<'a, S: 'a> {
    strings: &'a [ANSIString<S>],
    start: usize,
    end: usize,
}


/// A value which implements substring operation.
pub trait Substringable {
    /// Type returned when creating a substring of the value.
    type Output : ?Sized;

    /// Returns length of the value.
    fn len(&self) -> usize;

    /// Returns substring of the given value.
    ///
    /// **Panics** if range is out of bounds.
    fn substr<'a>(&'a self, range: Range<usize>) -> &'a Self::Output;
}


// ---- Substringable ----

impl<'a> Substringable for &'a str {
    type Output = str;
    fn len(&self) -> usize { <str>::len(self) }
    fn substr(&self, r: Range<usize>) -> &str { self.get(r).unwrap() }
}

impl Substringable for String {
    type Output = str;
    fn len(&self) -> usize { String::len(self) }
    fn substr(&self, r: Range<usize>) -> &str { self.get(r).unwrap() }
}

impl<'a> Substringable for &'a [u8] {
    type Output = [u8];
    fn len(&self) -> usize { <[u8]>::len(self) }
    fn substr(&self, r: Range<usize>) -> &[u8] { self.get(r).unwrap() }
}

impl Substringable for Vec<u8> {
    type Output = [u8];
    fn len(&self) -> usize { Vec::len(self) }
    fn substr(&self, r: Range<usize>) -> &[u8] { self.get(r).unwrap() }
}

impl<'a, S> Substringable for Cow<'a, S>
where S: 'a + ToOwned + ?Sized + Substringable<Output = S>,
         <S as ToOwned>::Owned: Substringable<Output = S>,
{
    type Output = S;
    fn len(&self) -> usize { self.as_ref().len() }
    fn substr(&self, range: Range<usize>) -> &S {
        self.as_ref().substr(range)
    }
}


// ---- Substring ----

impl<'a, S: Substringable> ANSIStrings<'a, S> {
    /// Returns a substring with styles properly applied to remaining fragments.
    ///
    /// The substring is indexed on the positions within the strings excluding
    /// the ANSI control sequences.  That means a slice will never happen in the
    /// middle of an ANSI code.  Furthermore, leading and trailing strings will
    /// be styled properly even if they are sliced.
    ///
    /// The substring is returned as an iterator with items borrowing from this
    /// [`ANSIStrings`].  This means that creating a substring performs no
    /// allocations but on the other hand it means that this objects cannot be
    /// modified until the [`Substring`] iterator and all items it had returned
    /// are dropped.
    ///
    /// Unlike [`str::get`] and other indexing methods, this method doesn’t
    /// panic if `start` or `end` are out of bounds.  Instead, if `start` is out
    /// of bounds the resulting iterator will be empty and if `end` is out of
    /// bounds, the iterator behaves as if it was unbounded.  As such, to
    /// simulate `start..` range, an end bound of `usize::MAX` can be used.
    ///
    /// Note however, that slicing of each leading or trailing fragment may
    /// result in a panic.  For example, if the slice happens in the middle of
    /// an UTF-8 sequence on a `str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::{ANSIStrings, Colour, Style};
    ///
    /// let strings = [
    ///     Style::new().paint("The quick "),
    ///     Colour::Yellow.paint("brown"),
    ///     Style::new().paint(" fox jumped over the lazy dog")
    /// ];
    /// let strings = ANSIStrings(&strings[..]);
    /// let fox = strings.substring(4..19).collect::<Vec<_>>();
    /// let fox = ANSIStrings(fox.as_slice());
    /// assert_eq!("quick \u{1b}[33mbrown\u{1b}[0m fox", fox.to_string());
    /// ```
    pub fn substring(
        &self,
        range: Range<usize>,
    ) -> Substring<S> {
        if range.end <= range.start {
            Substring { strings: &[], start: 0, end: 0 }
        } else {
            let (idx, offset) = self.substring_start(range.start);
            Substring {
                strings: &self.0[idx..],
                start: range.start - offset,
                end: range.end - offset,
            }
        }
    }

    /// Finds index of the fragment which contains `start` character.
    ///
    /// Returns (index, offset) tuple where index is the fragment that includes
    /// character at `start` position and offset is total length of fragments
    /// prior to fragment at given index.  If `start` is out of bounds, returns
    /// index one-past-the-last fragment.
    fn substring_start(
        &self,
        start: usize
    ) -> (usize, usize) {
        let mut idx = 0;
        let mut offset = 0;
        while offset < start && idx < self.0.len() {
            let len = self.0[idx].value.len();
            if start < offset + len {
                break;
            }
            offset += len;
            idx += 1;
        }
        (idx, offset)
    }
}

impl<'a, S: Substringable> Iterator for Substring<'a, S> {
    type Item = ANSIString<&'a <S as Substringable>::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.strings.get(0)?;
        let len = string.value.len().min(self.end);
        let string = ANSIString {
            style: string.style,
            value: string.value.substr(self.start..len),
        };

        self.start = 0;
        self.end -= len;
        // size_hint uses strings.is_empty to check if this is an empty iterator
        // so update it to one if we’ve reached end bound.  Otherwise, advance
        // by one fragment.
        self.strings = if self.end == 0 {
            &[]
        } else {
            &self.strings[1..]
        };

        Some(string)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.strings.is_empty() {
            (0, Some(0))
        } else {
            (1, Some(self.strings.len()))
        }
    }
}

impl<'a, S: Substringable> iter::FusedIterator for Substring<'a, S> {}


#[test]
fn test_substring() {
    use crate::Colour;

    let strings = [
        Colour::Black.paint("foo"),
        Colour::Red.paint("bar"),
        Colour::White.paint("baz"),
    ];
    let strings = ANSIStrings(&strings[..]);
    let mut results = vec![];
    for start in 0..10 {
        for end in start..11 {
            let substring = strings
                .substring(start..end)
                .map(|fragment| fragment.to_string())
                .collect::<String>();
            results.push(format!("{start}..{end:2} ‘{substring}’\n"));
        }
    }
    let got = results
        .into_iter()
        .collect::<String>()
        .replace("\x1b[0m", ">")
        .replace("\x1b[30m", "B<")
        .replace("\x1b[31m", "R<")
        .replace("\x1b[37m", "W<");
    assert_eq!("0.. 0 ‘’\n\
                0.. 1 ‘B<f>’\n\
                0.. 2 ‘B<fo>’\n\
                0.. 3 ‘B<foo>’\n\
                0.. 4 ‘B<foo>R<b>’\n\
                0.. 5 ‘B<foo>R<ba>’\n\
                0.. 6 ‘B<foo>R<bar>’\n\
                0.. 7 ‘B<foo>R<bar>W<b>’\n\
                0.. 8 ‘B<foo>R<bar>W<ba>’\n\
                0.. 9 ‘B<foo>R<bar>W<baz>’\n\
                0..10 ‘B<foo>R<bar>W<baz>’\n\
                1.. 1 ‘’\n\
                1.. 2 ‘B<o>’\n\
                1.. 3 ‘B<oo>’\n\
                1.. 4 ‘B<oo>R<b>’\n\
                1.. 5 ‘B<oo>R<ba>’\n\
                1.. 6 ‘B<oo>R<bar>’\n\
                1.. 7 ‘B<oo>R<bar>W<b>’\n\
                1.. 8 ‘B<oo>R<bar>W<ba>’\n\
                1.. 9 ‘B<oo>R<bar>W<baz>’\n\
                1..10 ‘B<oo>R<bar>W<baz>’\n\
                2.. 2 ‘’\n\
                2.. 3 ‘B<o>’\n\
                2.. 4 ‘B<o>R<b>’\n\
                2.. 5 ‘B<o>R<ba>’\n\
                2.. 6 ‘B<o>R<bar>’\n\
                2.. 7 ‘B<o>R<bar>W<b>’\n\
                2.. 8 ‘B<o>R<bar>W<ba>’\n\
                2.. 9 ‘B<o>R<bar>W<baz>’\n\
                2..10 ‘B<o>R<bar>W<baz>’\n\
                3.. 3 ‘’\n\
                3.. 4 ‘R<b>’\n\
                3.. 5 ‘R<ba>’\n\
                3.. 6 ‘R<bar>’\n\
                3.. 7 ‘R<bar>W<b>’\n\
                3.. 8 ‘R<bar>W<ba>’\n\
                3.. 9 ‘R<bar>W<baz>’\n\
                3..10 ‘R<bar>W<baz>’\n\
                4.. 4 ‘’\n\
                4.. 5 ‘R<a>’\n\
                4.. 6 ‘R<ar>’\n\
                4.. 7 ‘R<ar>W<b>’\n\
                4.. 8 ‘R<ar>W<ba>’\n\
                4.. 9 ‘R<ar>W<baz>’\n\
                4..10 ‘R<ar>W<baz>’\n\
                5.. 5 ‘’\n\
                5.. 6 ‘R<r>’\n\
                5.. 7 ‘R<r>W<b>’\n\
                5.. 8 ‘R<r>W<ba>’\n\
                5.. 9 ‘R<r>W<baz>’\n\
                5..10 ‘R<r>W<baz>’\n\
                6.. 6 ‘’\n\
                6.. 7 ‘W<b>’\n\
                6.. 8 ‘W<ba>’\n\
                6.. 9 ‘W<baz>’\n\
                6..10 ‘W<baz>’\n\
                7.. 7 ‘’\n\
                7.. 8 ‘W<a>’\n\
                7.. 9 ‘W<az>’\n\
                7..10 ‘W<az>’\n\
                8.. 8 ‘’\n\
                8.. 9 ‘W<z>’\n\
                8..10 ‘W<z>’\n\
                9.. 9 ‘’\n\
                9..10 ‘’\n\
                ", got);
}
