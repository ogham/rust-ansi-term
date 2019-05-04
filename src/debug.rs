use std::fmt;

use style::Style;

/// Styles have a special `Debug` implementation that only shows the fields that
/// are set. Fields that haven’t been touched aren’t included in the output.
///
/// This behaviour gets bypassed when using the alternate formatting mode
/// `format!("{:#?}")`.
///
///     use ansi_term::Colour::{Red, Blue};
///     assert_eq!("Style { fg(Red), on(Blue), bold, italic }",
///                format!("{:?}", Red.on(Blue).bold().italic()));
impl fmt::Debug for Style {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Style")
                .field("foreground", &self.foreground)
                .field("background", &self.background)
                .field("blink", &self.is_blink)
                .field("bold", &self.is_bold)
                .field("dimmed", &self.is_dimmed)
                .field("hidden", &self.is_hidden)
                .field("italic", &self.is_italic)
                .field("reverse", &self.is_reverse)
                .field("strikethrough", &self.is_strikethrough)
                .field("underline", &self.is_underline)
                .finish()
        } else if self.is_plain() {
            fmt.write_str("Style {}")
        } else {
            fmt.write_str("Style { ")?;

                        let mut parts = Vec::new();

            if let Some(fg) = self.foreground {
                parts.push(format!("fg({:?})", fg));
            }

            if let Some(bg) = self.background {
                parts.push(format!("on({:?})", bg));
            }

            {
                let mut push_flag = |name| {
                    parts.push(name);
                };

                if self.is_blink {
                    push_flag("blink".to_string())
                }
                if self.is_bold {
                    push_flag("bold".to_string())
                }
                if self.is_dimmed {
                    push_flag("dimmed".to_string())
                }
                if self.is_hidden {
                    push_flag("hidden".to_string())
                }
                if self.is_italic {
                    push_flag("italic".to_string())
                }
                if self.is_reverse {
                    push_flag("reverse".to_string())
                }
                if self.is_strikethrough {
                    push_flag("strikethrough".to_string())
                }
                if self.is_underline {
                    push_flag("underline".to_string())
                }
            }
            write!(fmt, "{}", parts.join(", "))?;

            write!(fmt, " }}")
        }
    }
}

#[cfg(test)]
mod test {
    use style::Colour::*;
    use style::Style;

    fn style() -> Style {
        Style::new()
    }

    macro_rules! test {
        ($name: ident: $obj: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($result, format!("{:?}", $obj));
            }
        };
    }

    test!(empty:   style()                  => "Style {}");
    test!(bold:    style().bold()           => "Style { bold }");
    test!(italic:  style().italic()         => "Style { italic }");
    test!(both:    style().bold().italic()  => "Style { bold, italic }");

    test!(red:     Red.normal()                     => "Style { fg(Red) }");
    test!(redblue: Red.normal().on(RGB(3, 2, 4))    => "Style { fg(Red), on(RGB(3, 2, 4)) }");

    test!(everything:
            Red.on(Blue).blink().bold().dimmed().hidden().italic().reverse().strikethrough().underline() =>
            "Style { fg(Red), on(Blue), blink, bold, dimmed, hidden, italic, reverse, strikethrough, underline }");

    #[test]
    fn long_and_detailed() {
        let debug = r##"Style {
    foreground: Some(
        Blue
    ),
    background: None,
    blink: false,
    bold: true,
    dimmed: false,
    hidden: false,
    italic: false,
    reverse: false,
    strikethrough: false,
    underline: false
}"##;
        assert_eq!(debug, format!("{:#?}", Blue.bold()));
    }
}
