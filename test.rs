#[test]
fn test_red() {
    let hi = Red.paint("hi");
    assert!(hi == "\x1B[31mhi\x1B[0m".to_string());
}

#[test]
fn test_black() {
    let hi = Black.normal().paint("hi");
    assert!(hi == "\x1B[30mhi\x1B[0m".to_string());
}

#[test]
fn test_yellow_bold() {
    let hi = Yellow.bold().paint("hi");
    assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
}

#[test]
fn test_yellow_bold_2() {
    let hi = Yellow.normal().bold().paint("hi");
    assert!(hi == "\x1B[1;33mhi\x1B[0m".to_string());
}

#[test]
fn test_blue_underline() {
    let hi = Blue.underline().paint("hi");
    assert!(hi == "\x1B[4;34mhi\x1B[0m".to_string());
}

#[test]
fn test_green_bold_underline() {
    let hi = Green.bold().underline().paint("hi");
    assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
}

#[test]
fn test_green_bold_underline_2() {
    let hi = Green.underline().bold().paint("hi");
    assert!(hi == "\x1B[1;4;32mhi\x1B[0m".to_string());
}

#[test]
fn test_purple_on_white() {
    let hi = Purple.on(White).paint("hi");
    assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
}

#[test]
fn test_purple_on_white_2() {
    let hi = Purple.normal().on(White).paint("hi");
    assert!(hi == "\x1B[47;35mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_bold_on_white() {
    let hi = Cyan.bold().on(White).paint("hi");
    assert!(hi == "\x1B[1;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_underline_on_white() {
    let hi = Cyan.underline().on(White).paint("hi");
    assert!(hi == "\x1B[4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_bold_underline_on_white() {
    let hi = Cyan.bold().underline().on(White).paint("hi");
    assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_cyan_underline_bold_on_white() {
    let hi = Cyan.underline().bold().on(White).paint("hi");
    assert!(hi == "\x1B[1;4;47;36mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed() {
    let hi = Fixed(100).paint("hi");
    assert!(hi == "\x1B[38;5;100mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed_on_purple() {
    let hi = Fixed(100).on(Purple).paint("hi");
    assert!(hi == "\x1B[45;38;5;100mhi\x1B[0m".to_string());
}

#[test]
fn test_fixed_on_fixed() {
    let hi = Fixed(100).on(Fixed(200)).paint("hi");
    assert!(hi == "\x1B[48;5;200;38;5;100mhi\x1B[0m".to_string());
}
