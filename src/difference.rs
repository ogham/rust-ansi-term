use super::Style;


/// When printing out one coloured string followed by another, use one of
/// these rules to figure out which *extra* control codes need to be sent.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Difference {

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


impl Style {

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
    pub fn difference(&self, next: &Style) -> Difference {
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


#[cfg(test)]
mod test {
    use super::*;
    use super::Difference::*;
    use colour::Colour::*;

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
}
