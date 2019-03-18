use termion::color::{AnsiValue, Color};

use lobsters::models;

pub struct Theme<Score, Title, Meta, Ask, Media, Tag, Domain, Metadata>
where
    Score: Color + Copy,
    Meta: Color + Copy,
    Ask: Color + Copy,
    Media: Color + Copy,
    Tag: Color + Copy,
    Title: Color + Copy,
    Domain: Color + Copy,
{
    pub score: Score,
    pub meta_tag: Meta,
    pub ask_tag: Ask,
    pub media_tag: Media,
    pub normal_tag: Tag,
    pub title: Title,
    pub domain: Domain,
    pub metadata: Metadata,
}

pub static LOBSTERS_256: Theme<
    AnsiValue,
    AnsiValue,
    AnsiValue,
    AnsiValue,
    AnsiValue,
    AnsiValue,
    AnsiValue,
    AnsiValue,
> = Theme {
    score: AnsiValue(248),
    ask_tag: AnsiValue(1), // TODO: Find a better colour for this one
    media_tag: AnsiValue(195),
    meta_tag: AnsiValue(252),
    normal_tag: AnsiValue(229),
    title: AnsiValue(33),
    domain: AnsiValue(245),
    metadata: AnsiValue(250),
};

macro_rules! get {
    ($field:ident) => {
        pub fn $field(&self) -> Box<dyn Color> {
            Box::new(self.$field)
        }
    };
}

impl<
        Score: 'static,
        Title: 'static,
        Meta: 'static,
        Ask: 'static,
        Media: 'static,
        Tag: 'static,
        Domain: 'static,
        Metadata: 'static,
    > Theme<Score, Title, Meta, Ask, Media, Tag, Domain, Metadata>
where
    Score: Color + Copy,
    Title: Color + Copy,
    Meta: Color + Copy,
    Ask: Color + Copy,
    Media: Color + Copy,
    Tag: Color + Copy,
    Domain: Color + Copy,
    Metadata: Color + Copy,
{
    get!(score);
    get!(title);
    get!(meta_tag);
    get!(ask_tag);
    get!(media_tag);
    get!(normal_tag);
    get!(domain);
    get!(metadata);

    pub fn tag_colour(&self, tag: &models::Tag) -> Box<dyn Color> {
        if tag.tag == "ask" {
            self.ask_tag()
        } else if tag.tag == "meta" {
            self.meta_tag()
        } else if tag.is_media {
            self.media_tag()
        } else {
            self.normal_tag()
        }
    }
}
