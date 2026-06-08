use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "opml")]
pub struct Opml {
    #[serde(rename = "@version")]
    pub version: String,
    pub head: Head,
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "outline")]
    pub outlines: Vec<Outline>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Outline {
    #[serde(rename = "@text")]
    pub text: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(rename = "@xmlUrl", skip_serializing_if = "Option::is_none")]
    pub xml_url: Option<String>,
    #[serde(rename = "outline", skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<Outline>,
}

impl Opml {
    pub fn new(outlines: Vec<Outline>) -> Self {
        Self {
            version: "2.0".to_string(),
            head: Head {
                title: "Wyrm RSS".to_string(),
            },
            body: Body { outlines },
        }
    }

    pub fn from_xml(bytes: &[u8]) -> WyrmResult<Self> {
        quick_xml::de::from_reader(bytes).map_err(WyrmError::XmlDeserializeError)
    }

    pub fn to_xml(&self) -> WyrmResult<String> {
        quick_xml::se::to_string(self)
            .map(|xml| format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{xml}"))
            .map_err(WyrmError::XmlSerializeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hn() -> Outline {
        Outline {
            text: "Hacker News".to_string(),
            title: "Hacker News".to_string(),
            kind: Some("rss".to_string()),
            xml_url: Some("https://news.ycombinator.com/rss".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn untagged_feed_becomes_leaf_outline() {
        let expected = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<opml version="2.0">"#,
            r#"<head><title>Wyrm RSS</title></head>"#,
            r#"<body>"#,
            r#"<outline text="Hacker News" title="Hacker News" type="rss" xmlUrl="https://news.ycombinator.com/rss"/>"#,
            r#"</body>"#,
            r#"</opml>"#,
        );
        assert_eq!(Opml::new(vec![hn()]).to_xml().unwrap(), expected);
    }

    #[test]
    fn tagged_feed_becomes_folder_with_child() {
        let folder = Outline {
            text: "Tech".to_string(),
            title: "Tech".to_string(),
            children: vec![hn()],
            ..Default::default()
        };
        let expected = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<opml version="2.0">"#,
            r#"<head><title>Wyrm RSS</title></head>"#,
            r#"<body>"#,
            r#"<outline text="Tech" title="Tech">"#,
            r#"<outline text="Hacker News" title="Hacker News" type="rss" xmlUrl="https://news.ycombinator.com/rss"/>"#,
            r#"</outline>"#,
            r#"</body>"#,
            r#"</opml>"#,
        );
        assert_eq!(Opml::new(vec![folder]).to_xml().unwrap(), expected);
    }
}
