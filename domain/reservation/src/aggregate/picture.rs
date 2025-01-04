use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{alphanumeric, capital, end, Parser, sym};
use prelude::url::Url;

#[derive(Error, Debug, PartialEq)]
pub enum PictureError {
    #[error("url not secure")]
    UrlNotSecure,

    #[error("malformed url")]
    MalformedUrl,

    #[error("malformed caption")]
    MalformedCaption,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Picture {
    pub url: Url,
    pub caption: String,
}

impl Picture {
    fn caption_parser<'a>() -> Parser<'a, &'a str> {
        (capital()
            + (alphanumeric() | sym(' ') | sym('-')).repeat(5..255).collect()
            + end::<char>()
        ).collect()
    }

    pub fn build(url: Url, caption: String) -> Result<Self, PictureError> {
        if url.scheme() != "https" {
            return Err(PictureError::UrlNotSecure);
        }

        if Self::caption_parser().parse_str(&caption).is_err() {
            return Err(PictureError::MalformedCaption)
        }

        Ok(Self {
            url,
            caption,
        })
    }
}

impl PartialEq for Picture {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

#[cfg(test)]
mod tests {
    use prelude::url::Url;
    use crate::aggregate::{Picture, PictureError};

    #[test]
    fn is_buildable() {
        let picture = Picture::build(url(), caption());
        assert!(picture.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // non https:// url
        let picture = Picture::build("http://stuff.com/image.jpg".parse().unwrap(), caption());
        assert_eq!(picture, Err(PictureError::UrlNotSecure));

        // empty caption
        let picture = Picture::build(url(), "".to_owned());
        assert_eq!(picture, Err(PictureError::MalformedCaption));

        // invalid symbols
        let picture = Picture::build(url(), "🍩".to_owned());
        assert_eq!(picture, Err(PictureError::MalformedCaption));
    }

    #[test]
    fn is_serializable() {
        let original = Picture::build(url(), caption()).unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Picture = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.url, deserialized.url);
        assert_eq!(original.caption, deserialized.caption);
    }

    #[test]
    fn equals_by_url() {
        let picture1 = Picture::build(url(), caption()).unwrap();
        let picture2 = Picture::build(url(), caption()).unwrap();
        assert_eq!(picture1, picture2);

        let picture3 = Picture::build(url(), "Far south resort".to_owned()).unwrap();
        assert_eq!(picture1, picture3);

        let picture4 = Picture::build("https://www.visitnorway.com/img/bergen.jpg".parse().unwrap(), caption()).unwrap();
        assert_ne!(picture1, picture4);
    }

    fn url() -> Url {
        "https://www.visitnorway.com/img/farsund.jpg".parse().unwrap()
    }

    fn caption() -> String {
        "Farsund Resort".to_owned()
    }
}