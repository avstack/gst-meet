// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::pubsub::PubSubPayload;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

generate_elem_id!(
    /// The artist or performer of the song or piece.
    Artist,
    "artist",
    TUNE
);

generate_elem_id!(
    /// The duration of the song or piece in seconds.
    Length,
    "length",
    TUNE,
    u16
);

generate_elem_id!(
    /// The user's rating of the song or piece, from 1 (lowest) to 10 (highest).
    Rating,
    "rating",
    TUNE,
    u8
);

generate_elem_id!(
    /// The collection (e.g., album) or other source (e.g., a band website that hosts streams or
    /// audio files).
    Source,
    "source",
    TUNE
);

generate_elem_id!(
    /// The title of the song or piece.
    Title,
    "title",
    TUNE
);

generate_elem_id!(
    /// A unique identifier for the tune; e.g., the track number within a collection or the
    /// specific URI for the object (e.g., a stream or audio file).
    Track,
    "track",
    TUNE
);

generate_elem_id!(
    /// A URI or URL pointing to information about the song, collection, or artist.
    Uri,
    "uri",
    TUNE
);

/// Container for formatted text.
#[derive(Debug, Clone)]
pub struct Tune {
    /// The artist or performer of the song or piece.
    artist: Option<Artist>,

    /// The duration of the song or piece in seconds.
    length: Option<Length>,

    /// The user's rating of the song or piece, from 1 (lowest) to 10 (highest).
    rating: Option<Rating>,

    /// The collection (e.g., album) or other source (e.g., a band website that hosts streams or
    /// audio files).
    source: Option<Source>,

    /// The title of the song or piece.
    title: Option<Title>,

    /// A unique identifier for the tune; e.g., the track number within a collection or the
    /// specific URI for the object (e.g., a stream or audio file).
    track: Option<Track>,

    /// A URI or URL pointing to information about the song, collection, or artist.
    uri: Option<Uri>,
}

impl PubSubPayload for Tune {}

impl Tune {
    fn new() -> Tune {
        Tune {
            artist: None,
            length: None,
            rating: None,
            source: None,
            title: None,
            track: None,
            uri: None,
        }
    }
}

impl TryFrom<Element> for Tune {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Tune, Error> {
        check_self!(elem, "tune", TUNE);
        check_no_attributes!(elem, "tune");

        let mut tune = Tune::new();
        for child in elem.children() {
            if child.is("artist", ns::TUNE) {
                if tune.artist.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one artist."));
                }
                tune.artist = Some(Artist::try_from(child.clone())?);
            } else if child.is("length", ns::TUNE) {
                if tune.length.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one length."));
                }
                tune.length = Some(Length::try_from(child.clone())?);
            } else if child.is("rating", ns::TUNE) {
                if tune.rating.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one rating."));
                }
                tune.rating = Some(Rating::try_from(child.clone())?);
            } else if child.is("source", ns::TUNE) {
                if tune.source.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one source."));
                }
                tune.source = Some(Source::try_from(child.clone())?);
            } else if child.is("title", ns::TUNE) {
                if tune.title.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one title."));
                }
                tune.title = Some(Title::try_from(child.clone())?);
            } else if child.is("track", ns::TUNE) {
                if tune.track.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one track."));
                }
                tune.track = Some(Track::try_from(child.clone())?);
            } else if child.is("uri", ns::TUNE) {
                if tune.uri.is_some() {
                    return Err(Error::ParseError("Tune can’t have more than one uri."));
                }
                tune.uri = Some(Uri::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown element in User Tune."));
            }
        }

        Ok(tune)
    }
}

impl From<Tune> for Element {
    fn from(tune: Tune) -> Element {
        Element::builder("tune", ns::TUNE)
            .append_all(tune.artist)
            .append_all(tune.length)
            .append_all(tune.rating)
            .append_all(tune.source)
            .append_all(tune.title)
            .append_all(tune.track)
            .append_all(tune.uri)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Tune, 68);
        assert_size!(Artist, 12);
        assert_size!(Length, 2);
        assert_size!(Rating, 1);
        assert_size!(Source, 12);
        assert_size!(Title, 12);
        assert_size!(Track, 12);
        assert_size!(Uri, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Tune, 128);
        assert_size!(Artist, 24);
        assert_size!(Length, 2);
        assert_size!(Rating, 1);
        assert_size!(Source, 24);
        assert_size!(Title, 24);
        assert_size!(Track, 24);
        assert_size!(Uri, 24);
    }

    #[test]
    fn empty() {
        let elem: Element = "<tune xmlns='http://jabber.org/protocol/tune'/>"
            .parse()
            .unwrap();
        let elem2 = elem.clone();
        let tune = Tune::try_from(elem).unwrap();
        assert!(tune.artist.is_none());
        assert!(tune.length.is_none());
        assert!(tune.rating.is_none());
        assert!(tune.source.is_none());
        assert!(tune.title.is_none());
        assert!(tune.track.is_none());
        assert!(tune.uri.is_none());

        let elem3 = tune.into();
        assert_eq!(elem2, elem3);
    }

    #[test]
    fn full() {
        let elem: Element = "<tune xmlns='http://jabber.org/protocol/tune'><artist>Yes</artist><length>686</length><rating>8</rating><source>Yessongs</source><title>Heart of the Sunrise</title><track>3</track><uri>http://www.yesworld.com/lyrics/Fragile.html#9</uri></tune>"
            .parse()
            .unwrap();
        let tune = Tune::try_from(elem).unwrap();
        assert_eq!(tune.artist, Some(Artist::from_str("Yes").unwrap()));
        assert_eq!(tune.length, Some(Length(686)));
        assert_eq!(tune.rating, Some(Rating(8)));
        assert_eq!(tune.source, Some(Source::from_str("Yessongs").unwrap()));
        assert_eq!(
            tune.title,
            Some(Title::from_str("Heart of the Sunrise").unwrap())
        );
        assert_eq!(tune.track, Some(Track::from_str("3").unwrap()));
        assert_eq!(
            tune.uri,
            Some(Uri::from_str("http://www.yesworld.com/lyrics/Fragile.html#9").unwrap())
        );
    }
}
