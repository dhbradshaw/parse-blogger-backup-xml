/// Tools for exploring an xml document on top of quick-xml.  For example contains
///
/// - Functions to convert different references and byte slices into strings.
/// - An XPath struct with a nice string representation that you can push and pop to and from.
/// - Functions to selectively print out different aspects of an xml file.
use quick_xml::events::{BytesEnd, BytesStart, BytesText};
use quick_xml::{events::Event, Reader};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;

pub fn string_from_bytes_text(bytes_text: BytesText) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = bytes_text.unescaped()?.into_owned();
    Ok(String::from_utf8(bytes)?)
}

pub fn start_tag_string(bytes_start: &BytesStart) -> Result<String, Box<dyn std::error::Error>> {
    let tag = bytes_start.name();
    let tag = tag.to_owned();
    let tag = String::from_utf8(tag)?;
    Ok(tag)
}
pub fn end_tag_string(bytes_end: &BytesEnd) -> Result<String, Box<dyn std::error::Error>> {
    let tag = bytes_end.name();
    let tag = tag.to_owned();
    let tag = String::from_utf8(tag)?;
    Ok(tag)
}

pub fn string_from_cow(cow: Cow<[u8]>) -> Result<String, Box<dyn std::error::Error>> {
    let string = match cow {
        Cow::Owned(internal) => String::from_utf8(internal)?,
        Cow::Borrowed(internal) => String::from_utf8(internal.to_owned())?,
    };
    Ok(string)
}

struct Stringable(String);

impl fmt::Display for Stringable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&BytesStart<'_>> for Stringable {
    fn from(start: &BytesStart) -> Self {
        let tag = start.name();
        let tag = tag.to_owned();
        let tag = String::from_utf8(tag).expect("Tag not in utf8");
        Self(tag)
    }
}

pub struct XPath(Vec<String>);

impl XPath {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn push(&mut self, tag: String) {
        self.0.push(tag);
    }
    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }
    pub fn pop_checked(&mut self, tag: String) {
        assert_eq!(self.pop().expect("can't end without starting."), tag);
    }
    pub fn as_string(&self) -> String {
        self.0.join("=>")
    }
}

impl Default for XPath {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for XPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl fmt::Display for XPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Get a list of all tags in an xml file.
pub fn tag_names(path: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(path)?;
    let mut buf = Vec::new();
    let mut tag_names: HashSet<String> = HashSet::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = start_tag_string(e)?;
                tag_names.insert(tag);
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    Ok(tag_names)
}

/// Get a list of all text children of all elements in your xml document.
pub fn all_text(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(path)?;
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    Ok(txt)
}

/// List all xml paths (as a set, where siblings have the same path if they have the same element type).
pub fn paths(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(path)?;
    let mut xpath: XPath = XPath::new();
    let mut buf = Vec::new();
    let mut xpath_strings = HashSet::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                xpath.push(start_tag_string(e)?);
            }
            Ok(Event::End(ref e)) => {
                xpath.pop_checked(end_tag_string(e)?);
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        xpath_strings.insert(xpath.as_string());
    }
    let mut xpath_strings: Vec<String> = xpath_strings.into_iter().collect();
    xpath_strings.sort();
    for xpath_string in xpath_strings {
        println!("{}", xpath_string);
    }
    Ok(())
}

/// All attributes in all the elements
pub fn all_attributes(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(file_path)?;
    let mut xpath = XPath::new();
    let mut buf = Vec::new();
    let mut attributes = HashSet::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                xpath.push(start_tag_string(e)?);
                for attr in e.attributes() {
                    let attr_string = format!("{:?}", attr.unwrap());
                    attributes.insert(attr_string);
                }
            }
            Ok(Event::End(ref e)) => xpath.pop_checked(end_tag_string(e)?),
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(_event) => {}
        }
    }
    let mut attributes: Vec<String> = attributes.into_iter().collect();
    attributes.sort();
    for attr in &attributes {
        println!("{}", attr);
    }
    Ok(attributes)
}

/// Print out all events found under a specific xpath leaf type.
pub fn path_contents(
    file_path: &str,
    x_path: &str,
    first: u32,
    last: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(file_path)?;
    let mut xpath = XPath::new();
    let mut xpath_string = "".to_owned();
    let mut buf = Vec::new();
    let mut index = 0;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                xpath.push(start_tag_string(e)?);
                xpath_string = xpath.as_string();
            }
            Ok(Event::End(ref e)) => {
                if x_path == xpath_string {
                    index += 1;
                    if first <= index && index <= last {
                        println!()
                    };
                }
                xpath.pop_checked(end_tag_string(e)?);
                xpath_string = xpath.as_string();
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(event) => {
                if x_path == xpath_string && first <= index && index <= last {
                    println!("{:?}", event);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_path_contents() -> Result<(), Box<dyn std::error::Error>> {
        // feed
        // feed=>author
        // feed=>author=>email
        // feed=>author=>name
        // feed=>entry
        // feed=>entry=>app:control
        // feed=>entry=>app:control=>app:draft
        // feed=>entry=>author
        // feed=>entry=>author=>email
        // feed=>entry=>author=>name
        // feed=>entry=>author=>uri
        // feed=>entry=>content
        // feed=>entry=>id
        // feed=>entry=>published
        // feed=>entry=>thr:total
        // feed=>entry=>title
        // feed=>entry=>updated
        // feed=>generator
        // feed=>id
        // feed=>title
        // feed=>updated
        let entry_number = 102;
        path_contents(
            "data/harris_backup.xml",
            "feed=>entry=>app:control=>app:draft",
            0,
            entry_number,
        )?;
        Ok(())
    }
    #[test]
    fn run_all_attributes() -> Result<(), Box<dyn std::error::Error>> {
        all_attributes("data/harris_backup.xml")?;
        Ok(())
    }
    #[test]
    fn run_paths() -> Result<(), Box<dyn std::error::Error>> {
        paths("data/harris_backup.xml")?;
        Ok(())
    }
    #[test]
    fn print_tag_names() -> Result<(), Box<dyn std::error::Error>> {
        let tags = tag_names("data/harris_backup.xml")?;
        dbg!(tags);
        Ok(())
    }
    #[test]
    fn print_all_text() -> Result<(), Box<dyn std::error::Error>> {
        let tags = all_text("data/harris_backup.xml")?;
        dbg!(tags);
        Ok(())
    }
}
