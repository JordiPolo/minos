use std::fmt::Display;
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub fn print_success(message: impl Display) {
    print_color(message, Color::Green);
}

pub fn print_error(error: impl Display) {
    print_color(error, Color::Red);
}

fn print_color(error: impl Display, color: Color) {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
    writeln!(&mut buffer, "{}", error).unwrap();
    bufwtr.print(&buffer).unwrap();
  //  buffer.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap();
}