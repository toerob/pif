use ansi_term::Colour::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SyntectColor, FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub fn _colorize_message(colour: Option<ansi_term::Colour>, msg: String) -> String {
  let text = match colour {
      Some(c) => format!("{}", c.paint(msg).to_owned()),
      None => msg.to_owned(),
  };
  return text;
}

pub fn create_success_msg(colour: bool, msg: String) -> String {
  return match colour {
      true => format!("{}", Green.paint(msg).to_owned().to_string()),
      _ => msg.to_string(),
  };
}


pub fn create_info_message(colour: bool, msg: String) -> String {
  return match colour {
      true => format!("{}", Blue.bold().paint(msg).to_owned().to_string()),
      _ => msg.to_string(),
  };
}

pub fn create_warning_msg(colour: bool, msg: String) -> String {
  return match colour {
      true => format!("{}", Red.paint(msg).to_owned()).to_string(),
      _ => msg.to_string(),
  };
}

pub fn print_success_msg(colour: bool, msg: String) -> () {
  print!("{}", create_success_msg(colour, msg));
}


pub fn print_warning_msg(colour: bool, msg: String) -> () {
  print!("{}", create_warning_msg(colour, msg));
}

// ── YAML syntax colorizer ─────────────────────────────────────────────────────

pub fn print_yaml_colored(content: &str, use_color: bool, highlight: &[&str]) {
    if !use_color {
        print!("{}", content);
        return;
    }

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps
        .find_syntax_by_extension("yaml")
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let theme_name = "base16-eighties.dark";    
    let mut h = HighlightLines::new(syntax, &ts.themes[theme_name]);

    for line in LinesWithEndings::from(content) {
        let mut ranges = match h.highlight_line(line, &ps) {
            Ok(r) => r,
            Err(_) => { print!("{}", line); continue; }
        };
        if !highlight.is_empty() {
            for i in 0..ranges.len() {
                if highlight.iter().any(|&hl| hl == ranges[i].1.trim()) {
                    ranges[i].0.foreground = SyntectColor { r: 220, g: 163, b: 0, a: 255 };
                    ranges[i].0.font_style = FontStyle::BOLD;
                }
            }
        }
        print!("{}", as_24_bit_terminal_escaped(&ranges, false));
    }
}
