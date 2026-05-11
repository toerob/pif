use ansi_term::Colour::*;

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
