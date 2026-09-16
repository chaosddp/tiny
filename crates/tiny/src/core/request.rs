use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize, ser::SerializeMap};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

impl Default for ImageDetail {
    fn default() -> Self {
        ImageDetail::Auto
    }
}

#[derive(Debug)]
pub enum ContentPart {
    Text(String),
    Image { url: String, detail: ImageDetail },
    Video(String),
    File(String),
}

pub type RichContent = Vec<ContentPart>;

// impl<'de> Deserialize<'de> for ContentPart {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de> {
//         deserializer.
//     }
// }

impl Serialize for ContentPart {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(2.into())?;

        match self {
            ContentPart::Text(text) => {
                map.serialize_entry("type", "text")?;
                map.serialize_entry("text", text)?;
            }
            ContentPart::File(file_url) => {
                map.serialize_entry("type", "file_url")?;

                let file_map = HashMap::from([("url", file_url)]);

                map.serialize_entry("file_url", &file_map)?;
            }
            ContentPart::Image { url, detail } => {
                map.serialize_entry("type", "image_url")?;

                let detail_str = match detail {
                    ImageDetail::Auto => "auto",
                    ImageDetail::High => "high",
                    ImageDetail::Low => "low",
                };

                // use BTreeMap so that we can consistant result
                let image_map = BTreeMap::from([("url", url.as_str()), ("detail", detail_str)]);

                map.serialize_entry("image_url", &image_map)?;
            }
            ContentPart::Video(url) => {
                map.serialize_entry("type", "video_url")?;

                let video_map = HashMap::from([("url", url)]);

                map.serialize_entry("video_url", &video_map)?;
            }
        }

        map.end()
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum UserMessage {
//     Text(String),
//     Rich(RichContent),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Message {
//     SystemMessage(String),
//     UserMessage(),
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_part_text_serialize() {
        let content = ContentPart::Text("hello".to_string());

        let content_str = serde_json::to_string(&content).unwrap();

        assert_eq!("{\"type\":\"text\",\"text\":\"hello\"}", &content_str)
    }

    #[test]
    fn test_content_part_image_serialize() {
        let url = "http://path/to/image.png";
        let content = ContentPart::Image {
            url: url.to_string(),
            detail: ImageDetail::Auto,
        };

        let content_str = serde_json::to_string(&content).unwrap();

        assert_eq!(
            format!(
                "{{\"type\":\"image_url\",\"image_url\":{{\"detail\":\"auto\",\"url\":\"{}\"}}}}",
                url
            ),
            content_str,
        )
    }

    #[test]
    fn test_content_part_video_serialize() {
        let content = ContentPart::Video("video_path".to_string());

        let content_str = serde_json::to_string(&content).unwrap();

        assert_eq!(
            "{\"type\":\"video_url\",\"video_url\":{\"url\":\"video_path\"}}",
            &content_str
        );
    }

    #[test]
    fn test_content_part_file_serialize() {
        let content = ContentPart::File("file_path".to_string());

        let content_str = serde_json::to_string(&content).unwrap();

        assert_eq!(
            "{\"type\":\"file_url\",\"file_url\":{\"url\":\"file_path\"}}",
            &content_str
        );
    }
}
