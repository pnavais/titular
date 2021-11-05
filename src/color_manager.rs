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

use ansi_term::Colour;
use ansi_term::Colour::*;

impl ColorManager {

    pub fn format<'a>(colours: &FallbackMap<String, String>, txt: &'a str, color_name: &str) -> String {
        match ColorManager::get_style(colours, color_name) {
            Some(c) => c.paint(txt).to_string(),
            None => txt.to_owned(),
        }
    }

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
                    let operator = groups.get(2).or(groups.get(4)).map_or("", |m| m.as_str()).to_uppercase();                       
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