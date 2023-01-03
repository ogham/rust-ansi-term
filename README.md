# rust-ansi-term [![ansi-term on crates.io](http://meritbadge.herokuapp.com/ansi-term)](https://crates.io/crates/ansi_term) [![Build status](https://img.shields.io/travis/ogham/rust-ansi-term/master.svg?style=flat)](https://travis-ci.org/ogham/rust-ansi-term) [![Build status](https://img.shields.io/appveyor/ci/ogham/rust-ansi-term/master.svg?style=flat&logo=AppVeyor&logoColor=silver)](https://ci.appveyor.com/project/ogham/rust-ansi-term) [![Coverage status](https://coveralls.io/repos/ogham/rust-ansi-term/badge.svg?branch=master&service=github)](https://coveralls.io/github/ogham/rust-ansi-term?branch=master)

This is a library for controlling colours and formatting, such as red bold text or blue underlined text, on ANSI terminals.

### [View the Rustdoc](https://docs.rs/ansi_term/)


# Installation

This crate works with [Cargo](http://crates.io). Add the following to your `Cargo.toml` dependencies section:

```toml
[dependencies]
ansi_term = "0.12"
```


## Basic usage

There are three main types in this crate that you need to be
concerned with: `ANSIString`, `Style`, and `Colour`.

A `Style` holds stylistic information: foreground and background colours,
whether the text should be bold, or blinking, or other properties. The
`Colour` enum represents the available colours. And an `ANSIString` is
a string paired with a `Style`.

`Color` is also available as an alias to `Colour`.

To format a string, call the `Style::paint` or `Colour::paint` method,
passing in the string you want to format as the argument.  For example,
here’s how to get some red text:

```rust
use ansi_term::Colour::Red;

println!("This is in red: {}", Red.paint("a red string"));
```

Note that the `paint` method doesn’t return a string with the ANSI control
sequence surrounding it.  Instead, it returns an `ANSIString` value which
has a `Display` implementation that outputs the sequence.  This allows
strings to be printed without additional `String` allocations.

In fact, `ANSIString` is a generic type which doesn’t require the element
to be a `String` at all.  Any type which implements `Display` can be
painted.  Other related traits (such as `LowerHex`) are supported as well.
For example:

```rust
use ansi_term::Colour::{Red, Green, Blue};

let red = Red.paint(255);
let green = Green.paint(248);
let blue = Blue.paint(231);

let latte = format!("rgb({red}, {green}, {blue})");
assert_eq!("rgb(\u{1b}[31m255\u{1b}[0m, \
                \u{1b}[32m248\u{1b}[0m, \
                \u{1b}[34m231\u{1b}[0m)", latte);

let latte = format!("#{red:02x}{green:02x}{blue:02x}");
assert_eq!("#\u{1b}[31mff\u{1b}[0m\
             \u{1b}[32mf8\u{1b}[0m\
             \u{1b}[34me7\u{1b}[0m", latte);
```

If you want to get at the escape codes, you can convert an `ANSIString` to
a string with `to_string` method as you would any other `Display` value:

```rustrust
use ansi_term::Colour::Red;

let red_string = Red.paint("a red string").to_string();
```

## Bold, underline, background, and other styles

For anything more complex than plain foreground colour changes, you need to
construct `Style` values.  You can do this by chaining methods based on
a object created with `Style::new`.  Each method creates a new style that
has that specific property set.  For example:

```rust
use ansi_term::Style;

println!("How about some {} and {}?",
         Style::new().bold().paint("bold"),
         Style::new().underline().paint("underline"));
```

For brevity, these methods have also been implemented for `Colour` values,
so you can give your styles a foreground colour without having to begin with
an empty `Style` value:

```rust
use ansi_term::Colour::{Blue, Yellow};

println!("Demonstrating {} and {}!",
         Blue.bold().paint("blue bold"),
         Yellow.underline().paint("yellow underline"));

println!("Yellow on blue: {}", Yellow.on(Blue).paint("wow!"));
```

The complete list of styles you can use are: `bold`, `dimmed`, `italic`,
`underline`, `blink`, `reverse`, `hidden`, `strikethrough`, and `on` for
background colours.

In some cases, you may find it easier to change the foreground on an
existing `Style` rather than starting from the appropriate `Colour`.
You can do this using the `fg` method:

```rust
use ansi_term::Style;
use ansi_term::Colour::{Blue, Cyan, Yellow};

println!("Yellow on blue: {}", Style::new().on(Blue).fg(Yellow).paint("yow!"));
println!("Also yellow on blue: {}", Cyan.on(Blue).fg(Yellow).paint("zow!"));
```

You can turn a `Colour` into a `Style` with the `normal` method.  This
produces the exact same `ANSIString` as if you just used the
`Colour::paint` method directly, but it’s useful if you need to represent
both the “red bold” and “red, but not bold” with values of the same type.

```rust
use ansi_term::Style;
use ansi_term::Colour::Red;

Red.normal().paint("yet another red string");
Style::default().paint("a completely regular string");
```

## Extended colours

You can access the 256-colour palette by using the `Colour::Fixed`
variant.  It takes an argument of the colour number to use.  This can be
included wherever you would use a `Colour`:

```rust
use ansi_term::Colour::Fixed;

Fixed(134).paint("A sort of light purple");
Fixed(221).on(Fixed(124)).paint("Mustard in the ketchup");
```

You can also access full 24-bit colour by using the `Colour::RGB` variant,
which takes separate red, green, and blue arguments:

```rust
use ansi_term::Colour::RGB;

RGB(70, 130, 180).paint("Steel blue");
```

## Combining successive coloured strings

The benefit of writing ANSI escape codes to the terminal is that they
*stack*: you do not need to end every coloured string with a reset code if
the text that follows it is of a similar style. For example, if you want to
have some blue text followed by some blue bold text, it’s possible to send
the ANSI code for blue, followed by the ANSI code for bold, and finishing
with a reset code without having to have an extra one between the two
strings.

This crate can optimise the ANSI codes that get printed in situations like
this, making life easier for your terminal renderer. The `ANSIStrings`
type takes a slice of several `ANSIString` values, and will iterate over
each of them, printing only the codes for the styles that need to be updated
as part of its formatting routine.

The following code snippet uses this to enclose a binary number displayed in
red bold text inside some red, but not bold, brackets:

```rust
use ansi_term::Colour::Red;
use ansi_term::{ANSIString, ANSIStrings};

let some_value = format!("{:b}", 42);
let strings: &[ANSIString<_>] = &[
    Red.paint_cow("["),
    Red.bold().paint_cow(some_value),
    Red.paint_cow("]"),
];

println!("Value: {}", ANSIStrings(strings));
```

In this example, the `paint_cow` method can take *either* an owned `String`
or a borrowed `&str` value.  It converts the argument into a copy-on-write
string (`Cow`) and wraps that inside of an `ANSIString`.

The `ANSIStrings` value works in the same way as its singular counterpart,
with a `Display` implementation that only performs the formatting when
required.

## Byte strings

This library also handles formatting `[u8]` byte strings.  This supports
applications working with text in an unknown encoding.  More specifically,
any type which implements `AsRef<[u8]>` can be painted.  For such types
`ANSIString::write_to` method is provided to write the value to any object
that implements `Write`:

```rust
use ansi_term::Colour::Green;

Green.paint("user data".as_bytes())
    .write_to(&mut std::io::stdout()).unwrap();
```
