/// A blogger backup has a schema with the following path types:
/// - feed
/// - feed=>author
/// - feed=>author=>email
/// - feed=>author=>name
/// - feed=>entry
/// - feed=>entry=>app:control
/// - feed=>entry=>app:control=>app:draft
/// - feed=>entry=>author
/// - feed=>entry=>author=>email
/// - feed=>entry=>author=>name
/// - feed=>entry=>author=>uri
/// - feed=>entry=>content
/// - feed=>entry=>id
/// - feed=>entry=>published
/// - feed=>entry=>thr:total
/// - feed=>entry=>title
/// - feed=>entry=>updated
/// - feed=>generator
/// - feed=>id
/// - feed=>title
/// - feed=>updated
///
/// In other words, there are a few main entity types:
/// feed, author, and entry.
/// Of those, only entry corresponds to actual blog posts.
/// However, both comments and posts are entries.
/// get_posts figures all that out.
use std::collections::HashMap;
use std::str::FromStr;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::models::Entry;
use crate::models::EntryKind;
use crate::models::Post;
use crate::xml_tools::end_tag_string;
use crate::xml_tools::start_tag_string;
use crate::xml_tools::string_from_bytes_text;
use crate::xml_tools::string_from_cow;
use crate::xml_tools::XPath;

// const COMMENT_KIND: &[u8] = b"http://schemas.google.com/blogger/2008/kind#comment";
const POST_ID_PREFIX: &[u8] = b"tag:blogger.com,1999:blog";
const POST_KIND: &[u8] = b"http://schemas.google.com/blogger/2008/kind#post";
const SETTINGS_KIND: &[u8] = b"http://schemas.google.com/blogger/2008/kind#settings";
const TEMPLATE_KIND: &[u8] = b"http://schemas.google.com/blogger/2008/kind#template";

/// Logic in this crate
/// - finds entries,
/// - determines whether they are posts or comments, and
/// - assigns comments to their posts
pub fn get_posts(file_path: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let mut comments = Vec::new();
    let mut entry = Entry::new();
    let mut posts = HashMap::new();
    let mut reader = Reader::from_file(file_path)?;
    let mut xpath = XPath::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref bytes_start)) => {
                xpath.push(start_tag_string(bytes_start)?);
            }
            Ok(Event::End(ref bytes_end)) => {
                if xpath.as_string() == "feed=>entry" {
                    match entry.kind {
                        Some(EntryKind::Comment) => comments.push(entry.to_comment().unwrap()),
                        Some(EntryKind::Post) => {
                            let post = entry.to_post().unwrap();
                            posts.insert(post.id.to_owned(), post);
                        }
                        _ => (),
                    }
                    entry.clear();
                }
                xpath.pop_checked(end_tag_string(bytes_end)?);
            }
            Ok(Event::Empty(byte_start)) => {
                for attribute in byte_start.attributes().flatten() {
                    match attribute.value {
                        value if value == POST_KIND => entry.kind = Some(EntryKind::Post),
                        value if value == SETTINGS_KIND => entry.kind = Some(EntryKind::Settings),
                        value if value == TEMPLATE_KIND => entry.kind = Some(EntryKind::Template),
                        value if value.starts_with(POST_ID_PREFIX) => {
                            entry.kind = Some(EntryKind::Comment);
                            entry.post_id = Some(string_from_cow(value)?);
                        }
                        _value => {
                            // let value = string_from_cow(value)?;
                            // dbg!(value);
                        }
                    }
                }
            }
            Ok(Event::Text(bytes_text)) => {
                let text = Some(string_from_bytes_text(bytes_text)?);
                match xpath.as_string().as_str() {
                    "feed=>entry=>author=>name" => entry.author_name = text,
                    "feed=>entry=>published" => {
                        let text = text.unwrap();
                        let published = parse_published(&text)?;
                        entry.published = Some(published);
                    }
                    "feed=>entry=>id" => entry.id = text,
                    "feed=>entry=>title" => entry.title = text,
                    "feed=>entry=>content" => entry.content = text,
                    "feed=>entry=>app:control=>app:draft" => {
                        if text.unwrap() == "yes" {
                            entry.draft = true;
                            println!("This post is a draft")
                        }
                    }
                    "feed=>entry" => println!("{}", text.unwrap()),
                    _ => (),
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(_event) => {}
        }
    }
    for comment in comments {
        if let Some(post) = posts.get_mut(&comment.post_id) {
            post.comments.push(comment);
        } else {
            println!("missing post for comment {:?}", comment);
        }
    }
    let mut posts: Vec<Post> = posts.into_iter().map(|(_, post)| post).collect();
    posts.sort_by(|a, b| a.published.cmp(&b.published));
    Ok(posts)
}

pub fn parse_published(
    published: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, Box<dyn std::error::Error>> {
    let dt = chrono::DateTime::from_str(published)?;
    Ok(dt)
}

#[cfg(test)]
mod tests {
    use super::get_posts;

    #[test]
    fn test_get_posts() {
        let posts = get_posts("data/backup.xml").unwrap();
        dbg!(posts);
    }
}
