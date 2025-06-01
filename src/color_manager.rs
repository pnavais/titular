use crate::utils::safe_parse;
use nu_ansi_term::{Color, Color::*, Style};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

use crate::context::Context;

#[derive(Debug, Clone, Copy)]
pub enum StyleScope {
    FG,
    BG,
    BOTH,
}

#[derive(Debug)]
pub struct StyleFormat {
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub scope: StyleScope,
}

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
    /// * `style` - The style format containing color information and scope
    ///
    /// # Returns
    ///
    /// A string with the color applied
    pub fn format<'a>(colours: &Context, txt: &'a str, style: StyleFormat) -> String {
        let mut style_obj = Style::new();

        // Apply foreground color if present
        if let Some(fg) = style.fg_color {
            if let Some(c) = ColorManager::get_style(colours, &fg) {
                style_obj = style_obj.fg(c);
            }
        }

        // Apply background color if present
        if let Some(bg) = style.bg_color {
            if let Some(c) = ColorManager::get_style(colours, &bg) {
                style_obj = style_obj.on(c);
            }
        }
        style_obj.paint(txt).to_string()
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
    fn get_style(colours: &Context, color_name: &str) -> Option<Color> {
        ColorManager::resolve_color_safely(colours, color_name, &mut HashSet::new())
    }

    /// Internal method to process a color with cycle detection
    ///
    /// # Arguments
    ///
    /// * `colours` - A reference to the fallback map containing color configurations
    /// * `color_name` - The name of the color to use
    /// * `visited` - A set to track visited colors and detect cycles
    ///
    /// # Returns
    ///
    /// A color object if the color can be resolved, None if a cycle is detected or the color cannot be resolved
    fn resolve_color_safely(
        colours: &Context,
        color_name: &str,
        visited: &mut HashSet<String>,
    ) -> Option<Color> {
        // Check for cycles
        if !visited.insert(color_name.to_string()) {
            return None;
        }

        ColorManager::process_color(
            colours,
            colours.get(color_name).unwrap_or(color_name),
            visited,
        )
    }

    /// Process a color string into a Color object
    ///
    /// # Arguments
    ///
    /// * `colours` - A reference to the fallback map containing color configurations
    /// * `color_str` - The color string to process
    ///
    /// # Returns
    ///
    /// A color object
    fn process_color(
        colours: &Context,
        color_str: &str,
        visited: &mut HashSet<String>,
    ) -> Option<Color> {
        if RGB_REGEX.is_match(color_str) {
            let groups = RGB_REGEX.captures(color_str).unwrap();
            let r: u8 = safe_parse(groups.get(1).map_or("", |m| m.as_str()));
            let g: u8 = safe_parse(groups.get(2).map_or("", |m| m.as_str()));
            let b: u8 = safe_parse(groups.get(3).map_or("", |m| m.as_str()));
            Some(Color::Rgb(r, g, b))
        } else if FNAME_REGEX.is_match(color_str) {
            let groups = FNAME_REGEX.captures(color_str).unwrap();
            let operator = groups
                .get(2)
                .or_else(|| groups.get(4))
                .map_or("", |m| m.as_str())
                .to_uppercase();
            if operator == "FIXED" {
                Some(Fixed(safe_parse::<u8>(
                    groups.get(3).map_or("", |m| m.as_str()),
                )))
            } else if operator == "NAME" {
                ColorManager::to_colour_name(groups.get(5).map_or("", |m| m.as_str()))
            } else {
                None
            }
        } else {
            ColorManager::resolve_color_safely(colours, color_str, visited)
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
