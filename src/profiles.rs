extern crate xml;

use std::fs::File;
use std::io::Read;
use self::xml::*;
use errors::AppError;

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub bindings: Vec<Binding>
}

#[derive(Debug)]
pub enum Trigger {
    Window { name: String }
}

#[derive(Debug)]
pub struct Binding {
    pub vcode: u16,
    pub flags: u16,
    pub keys: Vec<Key>
}

#[derive(Debug)]
pub struct Key {
    pub vcode: u16
}

pub fn load_profiles() -> Result<Vec<Profile>, AppError> {
    let mut file = try!(File::open("resources/profiles.xml"));
    let mut buffer = String::new();
    try!(file.read_to_string(&mut buffer));

    let root: Element = try!(buffer
        .parse()
        .map_err(|_| AppError::new("Error parsing profiles.xml"))
    );
    read_children(&root, read_profile)
}

fn read_profile(e: &Element) -> Result<Profile, AppError> {
    let profile_name = e
        .get_attribute("name", None)
        .unwrap_or("")
        .to_string();

    let triggers = try!(read_section(e, "triggers", read_trigger));
    let bindings = try!(read_section(e, "bindings", read_binding));

    Ok(Profile {
        name: profile_name,
        triggers: triggers,
        bindings: bindings
    })
}

fn read_trigger(e: &Element) -> Result<Trigger, AppError> {
    let window_name = try!(e
        .get_attribute("name", None)
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::new("Window should have a name attribute."))
    );

    Ok(Trigger::Window {
        name: window_name
    })
}

fn read_binding(e: &Element) -> Result<Binding, AppError> {
    let vk_code = try!(e
        .get_attribute("vcode", None)
        .ok_or_else(||AppError::new("vcode is missing from binding")));
    let vk_code = try!(parse_hex(vk_code));

    let flags = match e.get_attribute("flags", None) {
        Some(text) => parse_hex(text),
        None => Ok(0x00),
    };
    let flags = try!(flags);

    let keys = try!(e.get_children("key", None).map(read_key).collect());

    Ok(Binding {
        vcode: vk_code,
        flags: flags,
        keys: keys
    })
}

fn read_key(e: &Element) -> Result<Key, AppError> {
    let vk_code = try!(e
        .get_attribute("vcode", None)
        .ok_or_else(||AppError::new("vcode is missing from key")));
    let vk_code = try!(parse_hex(vk_code));

    Ok(Key {
        vcode: vk_code
    })
}

fn read_section<T, F>(elem: &Element, section_name: &str, element_reader: F) -> Result<Vec<T>, AppError>
    where F: Fn(&Element) -> Result<T, AppError> {
    match elem.get_child(section_name, None) {
        Some(section) => read_children(section, element_reader),
        None => Ok(vec![]),
    }
}

fn read_children<T, F>(elem: &Element, element_reader: F) -> Result<Vec<T>, AppError>
    where F: Fn(&Element) -> Result<T, AppError> {
    let children: Result<Vec<T>, AppError> = elem.children
        .iter()
        .flat_map(|elem| match elem {
            &Xml::ElementNode(ref elem) => Some(element_reader(elem)),
            _ => None
        })
        .collect();
    children
}

fn parse_hex(text: &str) -> Result<u16, AppError> {
    let text = text.trim_left_matches("0x");
    u16::from_str_radix(text, 16)
        .map_err(|_| AppError::new(format!("Invalid hex number {}", text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_profiles_works() {
        let profiles = load_profiles();
        assert!(profiles.is_ok());
    }
}
