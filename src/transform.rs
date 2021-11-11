use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq)]
pub struct Transform<'a> {
    pub operator: &'a str,
    pub value: &'a str,
}

impl <'a> Hash for Transform<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operator.hash(state);
    }
}

impl <'a> Ord for Transform<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.operator == "+" || self.operator == "-" || self.operator == "*" {
            if other.operator == "+" || other.operator == "*" || self.operator == "*" {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if self.operator == "pad" || self.operator == "fit" {
            if other.operator == "+" || other.operator == "-" || self.operator == "*" {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        } else {
            Ordering::Greater
        }    
    }
}

impl <'a> PartialOrd for Transform<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <'a> PartialEq for Transform<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator || (self.operator == "fit" && other.operator == "pad")
        || (self.operator == "pad" && other.operator == "fit")
    }
}