use ansi_term::Colour::{self, *};

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
    for line in content.lines() {
        println!("{}", colorize_yaml_line(line, highlight));
    }
}

fn colorize_yaml_line(line: &str, highlight: &[&str]) -> String {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    if trimmed.is_empty() {
        return line.to_string();
    }

    // Document markers
    if trimmed == "---" || trimmed == "..." {
        return format!("{}{}", indent, Colour::Fixed(240).paint(trimmed));
    }

    // Comments
    if trimmed.starts_with('#') {
        return Colour::Fixed(240).paint(line).to_string();
    }

    // List items:  "- value"
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return format!("{}{} {}", indent, Yellow.paint("-"), colorize_value(rest, highlight));
    }
    if trimmed == "-" {
        return format!("{}{}", indent, Yellow.paint("-"));
    }

    // Key: value  (or key: with nested block)
    if let Some(colon_pos) = find_key_colon(trimmed) {
        let key = &trimmed[..colon_pos];
        let value_str = trimmed[colon_pos + 1..].trim_start_matches(' ');
        if value_str.is_empty() {
            return format!("{}{}:", indent, Yellow.paint(key));
        }
        return format!("{}{}: {}", indent, Yellow.paint(key), colorize_value(value_str, highlight));
    }

    line.to_string()
}

fn colorize_value(value: &str, highlight: &[&str]) -> String {
    let v = value.trim();

    if v.is_empty() {
        return String::new();
    }

    // Inline sequence  [a, b, c]
    if v.starts_with('[') && v.ends_with(']') {
        return colorize_inline_seq(v, highlight);
    }

    // Highlighted value — rendered in bright amber bold regardless of type
    if !highlight.is_empty() && highlight.iter().any(|h| *h == v) {
        return Colour::Fixed(220).bold().paint(v).to_string();
    }

    // Quoted strings
    if (v.starts_with('"') && v.ends_with('"'))
        || (v.starts_with('\'') && v.ends_with('\''))
    {
        return Green.paint(v).to_string();
    }

    // Booleans / null
    match v {
        "true" | "false" | "null" | "~" => return Purple.paint(v).to_string(),
        _ => {}
    }

    // Numbers
    if v.parse::<f64>().is_ok() {
        return Cyan.paint(v).to_string();
    }

    // Paths, URLs, plain strings
    Green.paint(v).to_string()
}

fn colorize_inline_seq(s: &str, highlight: &[&str]) -> String {
    let inner = &s[1..s.len() - 1];
    let sep = format!("{} ", Colour::Fixed(250).paint(","));
    let items: Vec<String> = inner
        .split(',')
        .map(|item| {
            let t = item.trim();
            if !highlight.is_empty() && highlight.iter().any(|h| *h == t) {
                Colour::Fixed(220).bold().paint(t).to_string()
            } else {
                Green.paint(t).to_string()
            }
        })
        .collect();
    format!("{}{}{}", Yellow.paint("["), items.join(&sep), Yellow.paint("]"))
}

/// Find the position of the first `:` that acts as a YAML key separator
/// (followed by a space, end of string, or end of line) and is not inside quotes.
fn find_key_colon(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b':' if !in_single && !in_double => {
                match bytes.get(i + 1) {
                    None | Some(b' ') | Some(b'\n') | Some(b'\r') => return Some(i),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    None
}
