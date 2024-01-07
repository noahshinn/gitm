pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
    Grey,
}

pub fn colorize_string(input: &str, color: Color) -> String {
    let color_code = match color {
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Blue => "\x1b[34m",
        Color::Yellow => "\x1b[33m",
        Color::Cyan => "\x1b[36m",
        Color::Magenta => "\x1b[35m",
        Color::White => "\x1b[37m",
        Color::Black => "\x1b[30m",
        Color::Grey => "\x1b[90m",
    };

    format!("{}{}{}", color_code, input, "\x1b[0m")
}

pub fn indent_string(input: &str, indent: usize) -> String {
    let mut output = String::new();
    for line in input.lines() {
        output.push_str(&" ".repeat(indent));
        output.push_str(line);
        output.push('\n');
    }
    output
}
