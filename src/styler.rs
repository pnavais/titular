
use crate::{
    color_manager::ColorManager,
    fallback_map::FallbackMap,
};

use truncrate::*;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Transform<'a> {
    pub operator: &'a str,
    pub value: &'a str,
}

pub struct ItemStyler {}

impl<'a> ItemStyler {

    pub fn style(item_name: &'a mut String, transform: &Transform, context: &'a FallbackMap<String, String>, max_pad_length: usize) -> usize {        
        let mut excess = 0;
        if transform.operator == "+" { *item_name = format!("{}{}", item_name, transform.value); }
        else if transform.operator == "-" { *item_name = format!("{}{}", transform.value, item_name);  }
        else if transform.operator == "fg" {
            if item_name.len() > 0 {
                excess = item_name.width();
                *item_name = ColorManager::format(&context, &item_name, transform.value);
                excess = item_name.width().checked_sub(excess).unwrap_or(0);
            }
        }
        else if transform.operator == "pad" {            
            ItemStyler::pad(item_name, max_pad_length);
        }

        excess
    }

    pub fn surround(txt: &mut String, context: &'a FallbackMap<String, String>) {
        let s_start = context.get(&"surround_start".to_owned()).or(context.get(&"defaults.surround_start".to_owned())).unwrap();
        let s_end = context.get(&"surround_end".to_owned()).or(context.get(&"defaults.surround_end".to_owned())).unwrap();
        if txt.len() > 0 {
            *txt = format!("{}{}{}", s_start, txt, s_end);
        }
    }

    fn pad(txt: &'a mut String, max_length: usize) -> &'a String {        
        let pattern = txt.to_owned();

        while txt.width() < max_length {
            *txt+=&pattern;            
        }
        
        *txt = txt.truncate_to_boundary(max_length).to_owned();        
        txt
    }

}