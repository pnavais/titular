use crate::fallback_map::FallbackMap;
use nu_ansi_term::{Color, Color::*, Style};
use once_cell::sync::Lazy;
use regex::Regex;

static RGB_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?i)^RGB\\([\\s]*([0-9]+)[\\s]*,[\\s]*([0-9]+)[\\s]*,[\\s]*([0-9]+)[\\s]*\\)$")
        .unwrap()
});

static FNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?i)^((FIXED)\\([\\s]*([0-9]+)[\\s]*\\)|(NAME)\\([\\s]*([[:alpha:]]+)[\\s]*\\))$")
        .unwrap()
});

pub struct ColorManager;

impl ColorManager {
    /// Formats the given string using a foreground/background color supplied and extracting color
    /// configuration from the given fallback map
    ///
    /// # Arguments
    ///
    /// * `colours` - A reference to the fallback map containing color configurations
    /// * `txt` - The string to format
    /// * `color_name` - The name of the color to use
    /// * `is_bg` - Whether to use the color as a background color
    ///
    /// # Returns
    ///
    /// A string with the color applied
    pub fn format<'a>(
        colours: &FallbackMap<str, String>,
        txt: &'a str,
        color_name: &str,
        is_bg: bool,
    ) -> String {
        match ColorManager::get_style(colours, color_name) {
            Some(c) => {
                let mut style = Style::new();
                style = if is_bg { style.on(c) } else { style.fg(c) };
                style.paint(txt).to_string()
            }
            None => txt.to_owned(),
        }
    }

    /// Process the colour style supplied in one of the following variants supported by the
    /// ansi_term crate :
    /// - RGB(r,g,b) : A colour specified using the RGB notation
    /// - FIXED(num) : A colour specified in fixed terms
    /// - NAME(name) : The name of the colour
    ///
    /// # Arguments
    ///
    /// * `colours` - A reference to the fallback map containing color configurations
    /// * `color_name` - The name of the color to use
    ///
    /// # Returns
    ///
    /// A color object
    ///
    fn get_style(colours: &FallbackMap<str, String>, color_name: &str) -> Option<Color> {
        match colours.get(color_name) {
            Some(c) => {
                if RGB_REGEX.is_match(c) {
                    let groups = RGB_REGEX.captures(c).unwrap();
                    let r: u8 = groups.get(1).map_or("", |m| m.as_str()).parse().unwrap();
                    let g: u8 = groups.get(2).map_or("", |m| m.as_str()).parse().unwrap();
                    let b: u8 = groups.get(3).map_or("", |m| m.as_str()).parse().unwrap();
                    Some(Color::Rgb(r, g, b))
                } else if FNAME_REGEX.is_match(c) {
                    let groups = FNAME_REGEX.captures(c).unwrap();
                    let operator = groups
                        .get(2)
                        .or_else(|| groups.get(4))
                        .map_or("", |m| m.as_str())
                        .to_uppercase();
                    if operator == "FIXED" {
                        Some(Fixed(
                            groups.get(3).map_or("", |m| m.as_str()).parse().unwrap(),
                        ))
                    } else if operator == "NAME" {
                        ColorManager::to_colour_name(groups.get(5).map_or("", |m| m.as_str()))
                    } else {
                        None
                    }
                } else {
                    ColorManager::get_style(colours, c)
                }
            }
            None => None,
        }
    }

    /// List of colours supported by the ansi_term crate
    ///
    /// # Arguments
    ///
    /// * `colour_name` - The name of the color to use
    ///
    /// # Returns
    ///
    /// A color object
    fn to_colour_name(colour_name: &str) -> Option<Color> {
        let colour = colour_name.to_uppercase();
        match &*colour {
            "BLACK" => Some(Black),
            "RED" => Some(Red),
            "GREEN" => Some(Green),
            "YELLOW" => Some(Yellow),
            "BLUE" => Some(Blue),
            "PURPLE" => Some(Purple),
            "CYAN" => Some(Cyan),
            "WHITE" => Some(White),
            _ => None,
        }
    }
}
