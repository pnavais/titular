use crate::{
    config::{MainConfig, TemplateConfig},
    error::*,
    context::Context,
    fallback_map::FallbackMap,
    color_manager::ColorManager,
    term::TERM_SIZE,
};

use std::collections::VecDeque;
use truncrate::*;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use lazy_static::lazy_static;

lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new("([\\$|\\#])\\{([^}]+)\\}").unwrap();
    static ref OP_REGEX: Regex = Regex::new("((:|\\+|-)((fg|bg){0,1}\\[([^\\]]+)\\]|(pad)))").unwrap();
    static ref ITEM_REGEX: Regex = Regex::new("^([^:|^\\+|^-]+)").unwrap();
    static ref FILLER_REGEX: Regex = Regex::new("^f[\\d]*$").unwrap();
}

#[derive(Debug)]
struct VarContent<'a> {
    item: &'a str,    
    surround: bool,
    is_filler: bool,
    transforms: VecDeque<Transform<'a>>,
}

#[derive(Debug)]
struct Transform<'a> {
    operator: &'a str,
    value: &'a str,
}

struct ResolveStats {
    current_length: usize,
    num_groups_pad : usize,
}

struct FormattedItem {
    value: String,
    length: usize,
}

pub struct TemplateFormatter<'a> {
    main_config: &'a MainConfig,
}

impl<'a> TemplateFormatter<'a> {
    
    pub fn new(main_config: &'a MainConfig) -> Self {
        TemplateFormatter { main_config }
    }

    pub fn format(&self, context: &Context, template_config: &TemplateConfig) -> Result<bool> {        
        let mut result = Ok(true);

        // Create a fallback map
        let mut fallback_map: FallbackMap<String, String> = FallbackMap::from(Box::new(context));
        fallback_map.add(Box::new(template_config));
        fallback_map.add(Box::new(self.main_config));

        // Compute max term size
        let max_term_size = self.compute_max_term_size(&fallback_map)?;
        
        for pattern in template_config.pattern.data.split("\n") {
            result = self.format_line(&fallback_map, pattern, max_term_size);
        }
        result
    }

    pub fn format_line(&self, fallback_map: &FallbackMap<String, String>, pattern: &str, max_term_size: usize) -> Result<bool> {        
        let mut line = pattern.to_owned();
        
        // Compute max padding left
        let fixed_length = VAR_REGEX.replace_all(pattern, "").graphemes(true).count();
        let mut space_left = max_term_size - fixed_length;

        // Resolve normal groups
        let resolve_stats = self.format_items(&mut line, fallback_map, false, 0, &mut space_left)?;
        //space_left-=resolve_stats.current_length;
        let max_pad_length = (max_term_size - fixed_length - resolve_stats.current_length) / resolve_stats.num_groups_pad;
    
        // Resolve padding groups
        self.format_items(&mut line, fallback_map, true, max_pad_length, &mut space_left)?;

        println!("{}", line);

        Ok(true)
    }

    fn format_items(&self, items: &mut String, context: &FallbackMap<String, String>, apply_padding: bool, max_pad_length: usize, space_left: &mut usize) -> Result<ResolveStats> {               
        let mut num_groups_pad = 0;
        let mut current_length = 0;
        
        for group in VAR_REGEX.captures_iter(&items.clone()) {
            let item_group = group.get(2).map_or("", |m| m.as_str());
            let item_name = ITEM_REGEX.captures(item_group).unwrap().get(1).map_or("", |m| m.as_str());
            let mut has_padding: bool = false;
            let var_content = VarContent {
                item: item_name,
                surround: group.get(1).map_or("", |m| m.as_str()) == "#",
                is_filler: FILLER_REGEX.is_match(item_name),
                transforms: self.get_transforms(item_group, &mut has_padding),
            };
            if (!apply_padding && !has_padding) || apply_padding {
                let item = self.format_item(context, &var_content, max_pad_length + if *space_left - (max_pad_length+1) == 0 { 1 } else { 0 });
                *items = items.replace(group.get(0).map_or("", |m| m.as_str()), &item.value);
                
                current_length+=item.length;
                *space_left-=item.length;
            }
            if has_padding {
                num_groups_pad+=1;
            }
        }

        Ok(ResolveStats { current_length, num_groups_pad})
    }

    fn get_transforms(&self, item_group: &'a str, has_padding: &mut bool) -> VecDeque<Transform> {
        let mut transform_list: VecDeque<Transform> = VecDeque::new();
            OP_REGEX.captures_iter(item_group).for_each(|m| {                
                let t = Transform {
                    operator: m.get(6).or(m.get(4)).or(m.get(2)).map(|s| s.as_str()).unwrap(),
                    value: m.get(5).map_or("", |s| s.as_str()),
                };
                *has_padding = t.operator == "pad";
                if *has_padding || t.operator == "+" || t.operator == "-" { 
                    transform_list.push_front(t) 
                } else { transform_list.push_back(t) }
            });
        transform_list
    }

    fn format_item(&self, context: &'a FallbackMap<String, String>, var_content: &VarContent, max_pad_length: usize) -> FormattedItem {        
        // Try to resolve the variable using the context or take it from the template if not available
        let item_ctx = context.get(&var_content.item.to_owned());                 
        // Process the item operation
        let item_val = if item_ctx.is_some() { item_ctx.unwrap() } else { if var_content.is_filler { &self.main_config.defaults.fill_char } else { "" } };
        let mut item_name = item_val.to_owned();

        let mut styled_length = 0;

        // Apply style
        for transform in &var_content.transforms {            
            styled_length = item_name.graphemes(true).count();
            self.style(&mut item_name, transform, context, max_pad_length);
            styled_length = item_name.graphemes(true).count() - styled_length;
        }
        
        // Surround
        if var_content.surround {
            self.surround(&mut item_name, context);
        }

        let item_length = item_name.graphemes(true).count() - styled_length;
        
        FormattedItem { value: item_name, length: item_length }
    }

    fn style(&self, item_name: &'a mut String, transform: &Transform, context: &'a FallbackMap<String, String>, max_pad_length: usize) {        
        if transform.operator == "+" { *item_name = format!("{}{}", item_name, transform.value); }
        else if transform.operator == "-" { *item_name = format!("{}{}", transform.value, item_name);  }
        else if transform.operator == "fg" {                           
            *item_name = ColorManager::format(&context, &item_name, transform.value);
        }
        else if transform.operator == "pad" {
            self.pad(item_name, max_pad_length);
        }
    }
    
    fn surround(&self, txt: &mut String, context: &'a FallbackMap<String, String>) {
        let s_start = context.get(&"surround_start".to_owned()).or(Some(&self.main_config.defaults.surround_start)).unwrap();
        let s_end = context.get(&"surround_end".to_owned()).or(Some(&self.main_config.defaults.surround_start)).unwrap();
        if txt.len() > 0 {
            *txt = format!("{}{}{}", s_start, txt, s_end);
        }
    }

    fn pad(&self, txt: &'a mut String, max_length: usize) -> &'a String {        
        let pattern = txt.to_owned();

        while txt.graphemes(true).count() < max_length {
            *txt+=&pattern;            
        }
        
        *txt = txt.truncate_to_boundary(max_length).to_owned();        
        txt
    }

    fn compute_max_term_size(&self, context: &'a FallbackMap<String, String>) -> Result<usize> {
        let perc_spec = context.get(&"width".to_owned()).or(Some(&self.main_config.defaults.width)).unwrap();
        let percentage = match perc_spec.parse::<usize>() {
            Ok(n) => std::cmp::min(n,100),
            Err(_) => {
                if perc_spec == "full" { 100 }
                else {
                    return Err(Error::ArgsProcessingError(format!("Invalid width supplied \"{}\" (Must be in [0-100] percentage format)", perc_spec)))
                }
            }
        };

        Ok(TERM_SIZE.get_term_width() * percentage / 100)
    }
}