use crate::{
    color_manager::ColorManager,
    fallback_map::FallbackMap,
    transform::Transform,
};

use truncrate::*;
use unicode_width::UnicodeWidthStr;

pub struct ItemStyler {}

impl<'a> ItemStyler {

    /// Performs the transform operation of a single item using the supplied context data and maximum terminal width
    pub fn style(item_name: &'a mut String, transform: &Transform, context: &'a FallbackMap<String, String>, max_pad_length: usize) -> usize {        
        let mut excess = 0;
        if transform.operator == "+" { *item_name = format!("{}{}", item_name, transform.value); }
        else if transform.operator == "-" { *item_name = format!("{}{}", transform.value, item_name);  }
        else if transform.operator == "*" { ItemStyler::pad(item_name, transform.value.parse::<usize>().unwrap_or(1));  }
        else if transform.operator == "fg" || transform.operator == "bg"{
            if item_name.len() > 0 {
                excess = item_name.width();
                *item_name = ColorManager::format(&context, &item_name, transform.value, transform.operator == "bg");
                excess = item_name.width().checked_sub(excess).unwrap_or(0);
            }
        }
        else if transform.operator == "pad" || transform.operator == "fit" {
            ItemStyler::pad(item_name, max_pad_length);
        }

        excess
    }

    /// Performs a surround operation by applying the configured surround start/end characters
    /// to the given input text.
    pub fn surround(txt: &mut String, context: &'a FallbackMap<String, String>) {
        let s_start = context.get(&"surround_start".to_owned()).or(context.get(&"defaults.surround_start".to_owned())).unwrap();
        let s_end = context.get(&"surround_end".to_owned()).or(context.get(&"defaults.surround_end".to_owned())).unwrap();
        if txt.len() > 0 {
            *txt = format!("{}{}{}", s_start, txt, s_end);
        }
    }

    /// Pads a given text to the maxium length specified and truncates
    /// in its boundaries in case the pattern exceeds the final length 
    /// after padding.
    fn pad(txt: &'a mut String, max_length: usize) -> &'a String {        
        let pattern = txt.to_owned();

        if ! txt.is_empty() {
            while txt.width() < max_length {
                *txt+=&pattern;
            }
            
            *txt = txt.truncate_to_boundary(max_length).to_owned();
        }
        txt
    }

}