use std::collections::HashMap;
use std::ops::Range;

use crate::util;
use lobsters::models::{ShortTag, Story, Tag};
use lobsters::url::{self, Url};

const STORY_HEIGHT: usize = 2;

pub struct State {
    tag_map: TagMap,
    stories: Vec<Story>,
    current_story: usize,
    row_offset: usize,
    col_offset: usize,
}

pub struct TagMap {
    tags: HashMap<String, Tag>,
}

impl State {
    pub fn new(stories: Vec<Story>, tags: Vec<Tag>) -> Self {
        assert!(!stories.is_empty(), "no stories");

        let tag_map = TagMap::new(tags);
        State {
            stories,
            tag_map,
            current_story: 0,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn stories(&self) -> &[Story] {
        &self.stories
    }

    pub fn current_story_index(&self) -> usize {
        self.current_story
    }

    pub fn current_story_offset(&self) -> usize {
        self.current_story * STORY_HEIGHT
    }

    pub fn visible_range(&self, height: usize) -> Range<usize> {
        self.row_offset..self.row_offset + height
    }

    pub fn story_range(&self) -> Range<usize> {
        self.current_story_offset()..self.current_story_offset() + STORY_HEIGHT
    }

    pub fn current_story(&self) -> &Story {
        &self.stories[self.current_story]
    }

    pub fn story_url(&self) -> Result<Url, url::ParseError> {
        match self.current_story().url.as_str() {
            "" => self.comments_url(),
            url => url.parse::<Url>(),
        }
    }

    pub fn comments_url(&self) -> Result<Url, url::ParseError> {
        self.current_story().comments_url.parse::<Url>()
    }

    pub fn max_score_digits(&self) -> Option<usize> {
        self.stories
            .iter()
            .map(|story| util::count_digits(story.score))
            .max()
    }

    pub fn get_tag(&self, tag: &ShortTag) -> Option<&Tag> {
        self.tag_map.get(&tag)
    }

    pub fn row_offset_get_mut(&mut self) -> &mut usize {
        &mut self.row_offset
    }

    pub fn col_offset(&self) -> usize {
        self.col_offset
    }

    pub fn next_story(&mut self) -> bool {
        if self.current_story < (self.stories.len() - 1) {
            self.current_story += 1;
            true
        } else {
            false
        }
    }

    pub fn prev_story(&mut self) -> bool {
        if let Some(index) = self.current_story.checked_sub(1) {
            self.current_story = index;
            true
        } else {
            false
        }
    }

    pub fn scroll_left(&mut self, amount: usize) -> bool {
        // TODO: Limit the number of cols
        self.col_offset += amount;
        true
    }

    pub fn scroll_right(&mut self, amount: usize) -> bool {
        if let Some(new_offset) = self.col_offset.checked_sub(amount) {
            self.col_offset = new_offset;
            true
        } else {
            false
        }
    }
}

impl TagMap {
    pub fn new(tags: Vec<Tag>) -> Self {
        let tags = tags.into_iter().fold(HashMap::new(), |mut map, tag| {
            map.insert(tag.tag.clone(), tag);
            map
        });

        TagMap { tags }
    }

    pub fn get<'a>(&'a self, name: &ShortTag) -> Option<&'a Tag> {
        self.tags.get(&name.0)
    }
}
