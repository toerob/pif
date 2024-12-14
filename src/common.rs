use std::fs;
use regex::Regex;
use std::error::Error;

use std::io::{ self, Write, Read };

pub fn yes_or_no(prompt: &str, default: bool) -> bool {
    loop {
        print!("{} (yes/no, default {}): ", prompt, if default == true { "yes" } else { "no" });
        io::stdout().flush().unwrap(); // Säkerställer att texten skrivs ut direkt

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "" => {
                return default;
            }
            "yes" | "y" => {
                return true;
            }
            "no" | "n" => {
                return false;
            }
            _ => println!("Please type 'yes', 'no', 'y' or 'n'."),
        }
    }
}
