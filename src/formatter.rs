use crate::{
    config::{MainConfig, TemplateConfig},
    error::*,
    context::Context,
    fallback_map::{FallbackMap,MapProvider},
    styler::ItemStyler,
    transform::Transform,
    term::TERM_SIZE,
};

use std::collections::HashSet;

use regex::Regex;
use unicode_width::UnicodeWidthStr;
use lazy_static::lazy_static;

lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new("([\\$|#|%])\\{([^}]+)\\}").unwrap();
    static ref OP_REGEX: Regex = Regex::new("((:|\\+|\\-|\\*)((fg|bg){0,1}\\[([^\\]]+)\\]|(pad|fit|inv)))").unwrap();
    static ref ITEM_REGEX: Regex = Regex::new("^([^:|^\\+|^\\-|^\\*]+)").unwrap();
    static ref FILLER_REGEX: Regex = Regex::new("^f[\\d]*$").unwrap();
}

pub static KILL_LINE: &str = "\x1B[2K";
pub static MOVE_CURSOR_TO_START: &str = "\x1B[0G";

#[derive(Debug)]
struct VarContent<'a> {
    item: &'a str,    
    marker: char,    
    is_filler: bool,
    transforms: Vec<Transform<'a>>,
}

#[derive(Debug)]
struct ResolveStats {
    current_length: usize,
    num_groups_pad : usize,
}

#[derive(Debug)]
struct FormattedItem {
    value: String,
    length: usize,
}

pub struct TemplateFormatter<'a> {
    main_config: &'a MainConfig,    
}

/// Default template formatter implementation. Process each line in the template pattern
/// separately and thus only provides backward referencing. 
impl<'a> TemplateFormatter<'a> {
    
    pub fn new(main_config: &'a MainConfig) -> Self {
        TemplateFormatter { main_config }
    }

    /// Formats the given template pattern using a configured context by spliting it line
    /// by line.
    pub fn format(&self, context: &Context, template_config: &TemplateConfig) -> Result<bool> {        
        let mut result = Ok(true);

        // Create a fallback map
        let mut fallback_map: FallbackMap<String, String> = FallbackMap::from(context);
        fallback_map.add(template_config);
        fallback_map.add(self.main_config);

        // Compute max term size
        let max_term_size = self.compute_max_term_size(&fallback_map)?;
        let mut previous_line_size = 0;

        // Handle pre actions on the terminal
        self.prepare_term(context);
        
        for pattern in template_config.pattern.data.split('\n') {
            result = self.format_line(&fallback_map, pattern, max_term_size, &mut previous_line_size);
        }
        result
    }

    /// Formats a line in the pattern in a 2-step operations. First loop processes normal capture groups,
    /// second loop processes remaining groups  needing padding/fiting since those operations need to know 
    /// remaining space to render properly.
    pub fn format_line(&self, fallback_map: &FallbackMap<String, String>, pattern: &str, max_term_size: usize, previous_line_size: &mut usize) -> Result<bool> {        
        let mut line = pattern.to_owned();

        if fallback_map.contains(&"with-time".to_owned()) {
            self.add_time(&mut line);
        }
        
        // Compute max padding left
        let fixed_length = VAR_REGEX.replace_all(&line, "").width();
        let mut space_left = max_term_size - fixed_length;

        // Resolve normal groups        
        let resolve_stats = self.format_items(&mut line, fallback_map, false, 0, &mut space_left,  previous_line_size)?;
        *previous_line_size = if resolve_stats.current_length == 0 { *previous_line_size } else { previous_line_size.checked_sub(resolve_stats.current_length).unwrap_or(0) };        
        let max_pad_length = (max_term_size.saturating_sub(fixed_length + resolve_stats.current_length)) / std::cmp::max(resolve_stats.num_groups_pad,1);
                        
        // Resolve padding groups
        let resolve_stats_padding = self.format_items(&mut line, fallback_map, true, max_pad_length, &mut space_left, previous_line_size)?;

        if !line.is_empty() {
            print!("{}{}", line, if !fallback_map.contains(&"skip-newline".to_owned()) { "\n" } else { ""});
        }

        *previous_line_size = std::cmp::max(resolve_stats.current_length, resolve_stats_padding.current_length);

        Ok(true)
    }

    /// formats the items in the given line. Accounts for the remaining space left and also considers
    /// the previous line size for fiting operations.
    fn format_items(&self, items: &mut String, context: &FallbackMap<String, String>, apply_padding: bool, max_pad_length: usize, space_left: &mut usize, previous_line_size: &usize) -> Result<ResolveStats> {               
        let mut num_groups_pad = 0;
        let mut current_length = 0;
        
        for group in VAR_REGEX.captures_iter(&items.clone()) {
            let item_group = group.get(2).map_or("", |m| m.as_str());
            let item_name = match ITEM_REGEX.captures(item_group) {
                Some(i) => i.get(1).map_or("", |m| m.as_str()),
                None => return Err(Error::from(format!("Error processing pattern {}", item_group))),
            };
            
            let mut has_padding: bool = false;
            let var_content = VarContent {
                item: item_name,
                marker: group.get(1).map_or('\0', |m| m.as_str().chars().next().unwrap()),
                is_filler: FILLER_REGEX.is_match(item_name),
                transforms: self.get_transforms(item_group, &mut has_padding),
            };
            
            if apply_padding || !has_padding {                   
                let excess = if max_pad_length+1 == *space_left { 1 } else { 0 };
                let item = self.format_item(context, &var_content, max_pad_length + excess, previous_line_size);
                
                                
                *items = items.replacen(group.get(0).map_or("", |m| m.as_str()), &item.value, 1);                
                current_length+=item.length;

                *space_left = *space_left - std::cmp::min(item.length, *space_left);
            }       

            if has_padding {
                num_groups_pad+=1;
            }
        }

        Ok(ResolveStats { current_length, num_groups_pad})
    }

    /// Retrieves the list of transforms (i.e. rendering operations) specifies for the given item
    fn get_transforms(&self, item_group: &'a str, has_padding: &mut bool) -> Vec<Transform> {
        let mut transform_set: HashSet<Transform> = HashSet::new();
            OP_REGEX.captures_iter(item_group).for_each(|m| {                
                let t = Transform {
                    operator: m.get(6).or_else(|| m.get(4)).or_else(|| m.get(2)).map(|s| s.as_str()).unwrap(),
                    value: m.get(5).map_or("", |s| s.as_str()),
                };
                *has_padding = *has_padding || t.operator == "pad" || t.operator == "fit";
                transform_set.insert(t);
            });
        let mut transform_list = transform_set.into_iter().collect::<Vec<Transform>>();
        transform_list.sort();
        transform_list
    }

    /// Formats a single item in the group of items extracted from the line.
    fn format_item(&self, context: &'a FallbackMap<String, String>, var_content: &VarContent, max_pad_length: usize, previous_line_size: &usize) -> FormattedItem {        
        // Try to resolve the variable using the context or take it from the template if not available
        let item_ctx = self.get_item_name(context, var_content.item);
        let mut item_name;
        let item_length;
        let mut invisible = false;

        if let Some(value) = item_ctx {

            // Process item option
            item_name = value.to_owned();

            let mut excess_length = 0;

            // Surround (Prefix -> include text in style)
            if var_content.marker == '%' {
                ItemStyler::surround(&mut item_name, context);
            }
            
            // Apply style
            for transform in &var_content.transforms {
                excess_length += ItemStyler::style(&mut item_name, transform, context, if transform.operator == "fit" { *previous_line_size } else { max_pad_length });
                invisible = if context.is_active(&"hide".to_owned()) && transform.operator == "inv" { true } else { invisible };
            }
            
            // Surround (Postfix -> exclude text in style)
            if var_content.marker == '#' {
                ItemStyler::surround(&mut item_name, context);
            }
            
            item_length = item_name.width() - excess_length;

            // Hide text by effectively erasing it while keeping the length
            item_name = if invisible { String::from("") } else { item_name };
        } else {
            item_name = if var_content.is_filler { self.main_config.defaults.fill_char.clone() } else { String::from("") };
            item_length = item_name.width();
        }
        
        FormattedItem { value: item_name, length: item_length }
    }

    /// Retrieves the given item name from the context, in case the 
    /// resolved entry refers to a variable it is queries recursively
    /// until a terminal element is retrieved
    fn get_item_name(&self,context: &'a FallbackMap<String, String>, item_name: &str) -> Option<&String> {
        match context.get(&item_name.to_owned()) {
            Some(v) => {
                
                let item_name = match VAR_REGEX.captures(v) {
                    Some(i) => i.get(2).map_or("", |m| m.as_str()),
                    None => return Some(v),
                };
                self.get_item_name(context, item_name)

            },
            None => None,
        }
    }

    /// Computes the maximum terminal width allowed for rendering. 
    /// The configured width ratio is used to restrict actual usage.
    fn compute_max_term_size(&self, context: &'a FallbackMap<String, String>) -> Result<usize> {
        let default_str = String::from(&self.main_config.defaults.width);
        let perc_spec = context.get(&"width".to_owned()).unwrap_or(&default_str);
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

    /// Adds a trailing time pattern to a given line in the template
    fn add_time(&self, line: &mut String) {        
        line.push_str(&self.main_config.defaults.time_pattern);
    }

    /// Perfoms some cleanup and preparation functions in the terminal before using it
    fn prepare_term(&self, context: &Context) {
        if context.is_active(&"clear".to_owned()) {
            print!("{}{}", KILL_LINE, MOVE_CURSOR_TO_START);
        }        
    }
}
