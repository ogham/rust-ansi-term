# nu-ansi-term changes

## 2021-02-22

- forked rust-ansi-term
- renamed to nu-ansi-term
- added nushell project contributors to the authors
- updated readme.md
- renamed `Colour` to `Color`
- renamed some files ending in `colour` to `color`
- added "bright" colors ansi 90-97 (foreground) and 100-107 (background)
- ran cargo fmt


## 2021-03-26

- fix warnings for rust 1.51

## 2021-04-09

- make ansi pub to more easily use `ansi::RESET`
- ignore some doc warnings

## 2021-06-07

- add gradient functionality
- rework Rgb to support gradient

## 2021-06-11

- enable `Default` for `Color`

## 2021-09-09

- general minor refactorings

## 2021-09-27

- remove some dependencies

## 2021-11-14

- tweaked assert

## 2022-01-18

- fix some clippy lints

## 2022-03-13

- change authors text
- update license

## 2022-03-14

- add ci

## 2022-03-15

- bump to 0.45
- rename AnsiByteStrings for consistency
- update cargo.toml

## 2022-03-26

- add ansi default `39` foreground and default `49` background

## 2022-06-03

- code deref cleanup
- update docs url
- bump to 0.46

