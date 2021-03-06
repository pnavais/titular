use crate::{
    fallback_map::FallbackMap,
};

use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RGB_REGEX : Regex = Regex::new("(?i)^RGB\\([\\s]*([0-9]+)[\\s]*,[\\s]*([0-9]+)[\\s]*,[\\s]*([0-9]+)[\\s]*\\)$").unwrap();
    static ref FNAME_REGEX : Regex = Regex::new("(?i)^((FIXED)\\([\\s]*([0-9]+)[\\s]*\\)|(NAME)\\([\\s]*([[:alpha:]]+)[\\s]*\\))$").unwrap();
}

pub struct ColorManager;

use ansi_term::Style;
use ansi_term::Colour;
use ansi_term::Colour::*;

impl ColorManager {

    /// Formats the given string using a foreground/background color supplied and extracting color
    /// configuration from the given fallback map
    pub fn format<'a>(colours: &FallbackMap<String, String>, txt: &'a str, color_name: &str, is_bg: bool) -> String {
        match ColorManager::get_style(colours, color_name) {
            Some(c) => {
                let mut style = Style::new();
                style = if is_bg { style.on(c) } else { style.fg(c) };
                style.paint(txt).to_string()
            },
            None => txt.to_owned(),
        }
    }

    /// Process the colour style supplied in one of the following variants supported by the 
    /// ansi_term crate : 
    /// - RGB(r,g,b) : A colour specified using the RGB notation
    /// - FIXED(num) : A colour specified in fixed terms
    /// - NAME(name) : The name of the colour
    fn get_style(colours: &FallbackMap<String, String>, color_name: &str) -> Option<Colour> {
        match colours.get(&color_name.to_string()) {
            Some(c) => {                
                if RGB_REGEX.is_match(c) {
                    let groups = RGB_REGEX.captures(c).unwrap();
                    Some(RGB(groups.get(1).map_or("", |m| m.as_str()).parse().unwrap(), 
                            groups.get(2).map_or("", |m| m.as_str()).parse().unwrap(),
                            groups.get(3).map_or("", |m| m.as_str()).parse().unwrap()))
                } else if FNAME_REGEX.is_match(c) {                    
                    let groups = FNAME_REGEX.captures(c).unwrap();
                    let operator = groups.get(2).or_else(|| groups.get(4)).map_or("", |m| m.as_str()).to_uppercase();                       
                    if operator == "FIXED" {
                        Some(Fixed(groups.get(3).map_or("", |m| m.as_str()).parse().unwrap()))
                    } else if operator == "NAME" {
                        ColorManager::to_colour_name(groups.get(5).map_or("", |m| m.as_str()))
                    }
                    else {
                        None
                    }
                } else {
                    ColorManager::get_style(colours, c)
                }
            },
            None => None
        }
        
    }

    /// List of colours supported by the ansi_term crate
    fn to_colour_name(colour_name: &str) -> Option<Colour> {
        let colour = colour_name.to_uppercase();
        match &*colour {
            "BLACK"  => Some(Black),
            "RED"    => Some(Red), 
            "GREEN"  => Some(Green),
            "YELLOW" => Some(Yellow),
            "BLUE"   => Some(Blue),
            "PURPLE" => Some(Purple),
            "CYAN"   => Some(Cyan),
            "WHITE"  => Some(White),
            _        => None
        }
    }

}
