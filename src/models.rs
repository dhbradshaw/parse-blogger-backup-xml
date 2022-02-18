use crate::errors::EmptyResult;
use crate::utilities;
use chrono::DateTime;
use chrono::Datelike;
use chrono::FixedOffset;
use chrono::Timelike;
#[derive(Clone, Debug)]
pub struct Comment {
    pub author_name: String,
    pub content: String,
    pub id: String,
    pub post_id: String,
    pub published: DateTime<FixedOffset>,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct Post {
    pub author_name: String,
    pub comments: Vec<Comment>,
    pub content: String,
    pub draft: bool,
    pub id: String,
    pub published: DateTime<FixedOffset>,
    pub title: String,
}

impl Post {
    pub fn save_content(&self) -> EmptyResult {
        let path = format!(
            "data/bookroot/post_content_for_{}-{}-{}-{}-{}-{}",
            self.published.year(),
            self.published.month(),
            self.published.day(),
            self.published.hour(),
            self.published.minute(),
            self.published.second(),
        );
        let content = self.content.clone();
        utilities::save(&path, content)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum EntryKind {
    Comment,
    Post,
    Settings,
    Template,
}
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub author_name: Option<String>,
    pub content: Option<String>,
    pub draft: bool,
    pub id: Option<String>,
    pub kind: Option<EntryKind>,
    pub post_id: Option<String>,
    pub published: Option<DateTime<FixedOffset>>,
    pub title: Option<String>,
}

impl Entry {
    pub fn new() -> Entry {
        Entry {
            author_name: None,
            content: None,
            draft: false,
            id: None,
            kind: None,
            post_id: None,
            published: None,
            title: None,
        }
    }
    pub fn to_post(&self) -> Option<Post> {
        if let Entry {
            author_name: Some(author_name),
            content: Some(content),
            draft,
            kind: Some(EntryKind::Post),
            id: Some(id),
            published: Some(published),
            title: Some(title),
            ..
        } = self
        {
            Some(Post {
                author_name: author_name.to_owned(),
                comments: vec![],
                content: content.to_owned(),
                draft: draft.to_owned(),
                id: id.to_owned(),
                published: published.to_owned(),
                title: title.to_owned(),
            })
        } else {
            None
        }
    }
    pub fn to_comment(&self) -> Option<Comment> {
        if let Entry {
            author_name: Some(author_name),
            content: Some(content),
            draft: _draft,
            kind: Some(EntryKind::Comment),
            id: Some(id),
            published: Some(published),
            title: Some(title),
            post_id: Some(post_id),
        } = self
        {
            Some(Comment {
                author_name: author_name.to_owned(),
                content: content.to_owned(),
                id: id.to_owned(),
                post_id: post_id.to_owned(),
                published: published.to_owned(),
                title: title.to_owned(),
            })
        } else {
            None
        }
    }
    pub fn clear(&mut self) {
        self.author_name = None;
        self.content = None;
        self.draft = false;
        self.id = None;
        self.kind = None;
        self.post_id = None;
        self.published = None;
        self.title = None;
    }
}
