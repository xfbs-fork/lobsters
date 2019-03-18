use termion::color::{AnsiValue, Color, Rgb, White};

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

pub static LOBSTERS_MONO: Theme<White, White, White, White, White, White, White, White> = Theme {
    score: White,
    ask_tag: White,
    media_tag: White,
    meta_tag: White,
    normal_tag: White,
    title: White,
    domain: White,
    metadata: White,
};

pub static LOBSTERS_GREY: Theme<
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
    ask_tag: AnsiValue(252),
    media_tag: AnsiValue(252),
    meta_tag: AnsiValue(252),
    normal_tag: AnsiValue(252),
    title: AnsiValue(254),
    domain: AnsiValue(245),
    metadata: AnsiValue(250),
};

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

pub static LOBSTERS_TRUE: Theme<Rgb, Rgb, Rgb, Rgb, Rgb, Rgb, Rgb, Rgb> = Theme {
    score: Rgb(170, 170, 170),
    ask_tag: Rgb(240, 178, 184),
    media_tag: Rgb(178, 204, 240),
    meta_tag: Rgb(200, 200, 200),
    normal_tag: Rgb(213, 212, 88),
    title: Rgb(37, 98, 220),
    domain: Rgb(153, 153, 153), // On the site this is actually the same as metadata
    metadata: Rgb(136, 136, 136),
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
        if tag.tag == "ask" || tag.tag == "show" {
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
