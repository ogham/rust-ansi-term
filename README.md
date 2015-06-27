# rust-ansi-term [![Build Status](https://travis-ci.org/ogham/rust-ansi-term.svg?branch=master)](https://travis-ci.org/ogham/rust-ansi-term)

This is a library for controlling colours and formatting, such as red
bold text or blue underlined text, on ANSI terminals.

### [View the Rustdoc](http://bsago.me/doc/ansi_term/)


## Installation

It uses [Cargo](http://crates.io/), Rust's package manager. You can
depend on this library by adding `ansi_term` to your Cargo dependencies:

```toml
[dependencies]
ansi_term = "*"
```

Or, to use the Git repo directly:

```toml
[dependencies.ansi_term]
git = "https://github.com/ogham/rust-ansi-term.git"
```


# Usage

```rust
extern crate ansi_term;
use ansi_term::Colour::{Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
use ansi_term::Style;
```


## Simple Colours

You can format strings by calling the `paint` method on a Colour
or a Style object, passing in the string you want to format. For
example, to get some red text, call the `paint` method on `Red`:

```rust
println!("This is in red: {}!", Red.paint("a red string"));
```

The `paint` method returns an `ANSIString` object, which will get
automatically converted to the correct sequence of escape codes when
used in a `println!` or `format!` macro, or anything else that
supports using the `Show` trait. This means that if you just want a
string of the escape codes without anything else, you can still use
the `to_string` method:

```rust
let red_string: String = Red.paint("another red string").to_string();
```


## Bold, Underline, and Background

To do anything more complex than just foreground colours, you need
to use Style objects. Calling the `bold` or `underline` method on
a Colour returns a Style that has the appropriate property set on
it:

```rust
println!("Demonstrating {} and {}!",
         Blue.bold().paint("blue bold"),
         Yellow.underline().paint("yellow underline"));
```

These methods chain, so you can call them on existing Style
objects to set more than one particular properly, like so:

```rust
Blue.underline().bold().paint("Blue underline bold!")
```

You can set the background colour of a Style by using the `on`
method:

```rust
Blue.on(Yellow).paint("Blue on yellow!")
```

Finally, you can turn a Colour into a Style with the `normal`
method, though it'll produce the exact same string if you just use
the Colour. It's only useful if you're writing a method that can
return either normal or bold (or underline) styles, and need to
return a Style object from it.

```rust
Red.normal().paint("yet another red string")
```


## Extended Colours

You can access the extended range of 256 colours by using the
Fixed constructor, which takes an argument of the colour number to
use. This can be used wherever you would use a Colour:

```rust
Fixed(134).paint("A sort of light purple.")
```

This even works for background colours:

```rust
Fixed(221).on(Fixed(124)).paint("Mustard in the ketchup.")
```


## No Formatting

Finally, for the sake of completeness, the default style provides
neither colours nor formatting.

```rust
Style::default().paint("No colours here.")
```
