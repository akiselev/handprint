//! Hacker News, via the Algolia search API.
//!
//! HN is the human reference corpus of choice here: the register matches the
//! deployment register, per-user history is easy to pull, and the API is free
//! and needs no key.
//!
//! # The contamination cut-off
//!
//! Human reference corpora are cut at **2022-11** by default
//! ([`DEFAULT_CUTOFF`]). After ChatGPT's release, HN comments increasingly
//! contain LLM-assisted text, and — separately and more insidiously — AI-isms
//! have begun leaking into genuinely human usage. A "human" reference fitted on
//! 2024 text will treat "delve" as normal human vocabulary, which quietly
//! destroys the thing the reference exists to measure.
//!
//! # Network
//!
//! Fetching is behind the `net` feature. Everything that parses a response is
//! not, so the shape of the data is testable without a network.

use serde_json::Value;

use crate::record::{Record, Register};

/// Default contamination cut-off: 2022-11-01 as a Unix timestamp.
pub const DEFAULT_CUTOFF: i64 = 1_667_260_800;

/// The public Algolia endpoint for Hacker News.
pub const ALGOLIA_BASE: &str = "https://hn.algolia.com/api/v1";

/// One page of an Algolia response, in the shape the caller needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// Comment records from this page.
    pub records: Vec<Record>,
    /// Zero-based page index.
    pub page: usize,
    /// Total pages available for the query.
    pub pages: usize,
}

impl Page {
    /// Whether another page exists.
    pub fn has_more(&self) -> bool {
        self.page + 1 < self.pages
    }
}

/// Parse an Algolia `search_by_date` response into records.
///
/// Comments arrive as HTML; entities are decoded and tags are removed, because
/// `&#x27;` would otherwise be counted as punctuation and `<p>` as a word.
pub fn parse_page(json: &str, cutoff: Option<i64>) -> Result<Page, crate::Error> {
    let value: Value = serde_json::from_str(json).map_err(crate::Error::Json)?;
    let hits = value
        .get("hits")
        .and_then(Value::as_array)
        .ok_or_else(|| crate::Error::Format("Algolia response has no `hits` array".into()))?;

    let mut records = Vec::new();
    for hit in hits {
        let Some(text) = hit.get("comment_text").and_then(Value::as_str) else {
            continue;
        };
        let created = hit.get("created_at_i").and_then(Value::as_i64);
        if let (Some(created), Some(cutoff)) = (created, cutoff) {
            if created >= cutoff {
                continue;
            }
        }
        let author = hit
            .get("author")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let mut record = Record::new(clean_html(text), "hn:algolia", Register::ForumComment)
            .with_author(author)
            .with_meta(
                "license",
                "Hacker News content; profiles only, no text redistribution",
            );
        if let Some(ts) = hit.get("created_at").and_then(Value::as_str) {
            record = record.with_ts(ts);
        }
        if let Some(id) = hit.get("objectID").and_then(Value::as_str) {
            record = record.with_meta("id", id);
        }
        if let Some(story) = hit.get("story_id").and_then(Value::as_i64) {
            record = record.with_meta("story", story.to_string());
        }
        records.push(record);
    }

    Ok(Page {
        records,
        page: value.get("page").and_then(Value::as_u64).unwrap_or(0) as usize,
        pages: value.get("nbPages").and_then(Value::as_u64).unwrap_or(1) as usize,
    })
}

/// Strip HTML tags and decode the entities HN emits.
pub fn clean_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => {
                // `<p>` marks a paragraph break; every other tag just goes.
                let mut tag = String::new();
                for c in chars.by_ref() {
                    if c == '>' {
                        break;
                    }
                    tag.push(c);
                }
                if tag.eq_ignore_ascii_case("p") || tag.eq_ignore_ascii_case("br") {
                    out.push_str("\n\n");
                }
            }
            '&' => {
                let mut entity = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ';' {
                        chars.next();
                        break;
                    }
                    if entity.len() > 8 || c.is_whitespace() {
                        break;
                    }
                    entity.push(c);
                    chars.next();
                }
                out.push_str(&decode_entity(&entity));
            }
            c => out.push(c),
        }
    }
    // HN paragraphs come through as doubled newlines; collapse longer runs.
    let mut collapsed = String::with_capacity(out.len());
    let mut blank = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        collapsed.push_str(line.trim_end());
        collapsed.push('\n');
    }
    collapsed.trim().to_owned()
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" | "#x27" | "#39" => "'".into(),
        "nbsp" => "\u{00A0}".into(),
        "#x2F" | "#47" => "/".into(),
        // Named typographic entities. These matter more than they look: the
        // difference between an em dash and a hyphen is a feature dimension, so
        // leaving `&mdash;` undecoded would count it as five letters and a
        // semicolon instead.
        "mdash" | "#x2014" | "#8212" => "\u{2014}".into(),
        "ndash" | "#x2013" | "#8211" => "\u{2013}".into(),
        "rsquo" | "#x2019" | "#8217" => "\u{2019}".into(),
        "lsquo" | "#x2018" | "#8216" => "\u{2018}".into(),
        "ldquo" | "#x201C" | "#8220" => "\u{201C}".into(),
        "rdquo" | "#x201D" | "#8221" => "\u{201D}".into(),
        "laquo" => "\u{00AB}".into(),
        "raquo" => "\u{00BB}".into(),
        "middot" | "#183" => "\u{00B7}".into(),
        "bull" => "\u{2022}".into(),
        "deg" => "\u{00B0}".into(),
        "times" => "\u{00D7}".into(),
        "copy" => "\u{00A9}".into(),
        "reg" => "\u{00AE}".into(),
        "trade" => "\u{2122}".into(),
        "hellip" | "#8230" => "\u{2026}".into(),
        other => {
            // Numeric entities we do not special-case.
            if let Some(hex) = other
                .strip_prefix("#x")
                .or_else(|| other.strip_prefix("#X"))
            {
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(n) {
                        return c.to_string();
                    }
                }
            } else if let Some(dec) = other.strip_prefix('#') {
                if let Ok(n) = dec.parse::<u32>() {
                    if let Some(c) = char::from_u32(n) {
                        return c.to_string();
                    }
                }
            }
            // Unknown entity: put it back rather than silently dropping text.
            format!("&{other};")
        }
    }
}

/// The URL for one page of a user's comments.
pub fn user_comments_url(base: &str, user: &str, page: usize, hits_per_page: usize) -> String {
    format!(
        "{base}/search_by_date?tags=comment,author_{user}&page={page}&hitsPerPage={hits_per_page}"
    )
}

#[cfg(feature = "net")]
pub use net::{Client, FetchConfig};

#[cfg(feature = "net")]
mod net {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;

    /// How to fetch.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FetchConfig {
        /// API base URL.
        pub base: String,
        /// Results per page; Algolia caps this at 1000.
        pub hits_per_page: usize,
        /// Stop after this many pages.
        pub max_pages: usize,
        /// Drop comments at or after this Unix timestamp.
        pub cutoff: Option<i64>,
        /// Pause between requests. The API tolerates roughly 10k requests per
        /// hour; this keeps a bulk pull comfortably under that.
        pub delay: Duration,
        /// Directory to cache raw pages in, so a re-run costs nothing.
        pub cache: Option<PathBuf>,
    }

    impl Default for FetchConfig {
        fn default() -> Self {
            FetchConfig {
                base: ALGOLIA_BASE.to_owned(),
                hits_per_page: 100,
                max_pages: 50,
                cutoff: Some(DEFAULT_CUTOFF),
                delay: Duration::from_millis(400),
                cache: None,
            }
        }
    }

    /// A rate-limited, caching Algolia client.
    #[derive(Debug)]
    pub struct Client {
        agent: ureq::Agent,
        config: FetchConfig,
    }

    impl Client {
        /// Build a client.
        pub fn new(config: FetchConfig) -> Client {
            let agent = ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(30))
                .timeout_write(Duration::from_secs(30))
                .user_agent(concat!("handprint/", env!("CARGO_PKG_VERSION")))
                .build();
            Client { agent, config }
        }

        /// Fetch every available page of a user's comments.
        pub fn user_comments(&self, user: &str) -> crate::Result<Vec<Record>> {
            let mut out = Vec::new();
            for page in 0..self.config.max_pages {
                let body = self.fetch_page(user, page)?;
                let parsed = super::parse_page(&body, self.config.cutoff)?;
                let more = parsed.has_more();
                out.extend(parsed.records);
                if !more {
                    break;
                }
                std::thread::sleep(self.config.delay);
            }
            Ok(out)
        }

        fn fetch_page(&self, user: &str, page: usize) -> crate::Result<String> {
            let cache_path = self.config.cache.as_ref().map(|dir| {
                dir.join(format!(
                    "hn-{user}-{page}-{}.json",
                    self.config.hits_per_page
                ))
            });
            if let Some(path) = &cache_path {
                if let Ok(cached) = std::fs::read_to_string(path) {
                    return Ok(cached);
                }
            }
            let url =
                super::user_comments_url(&self.config.base, user, page, self.config.hits_per_page);
            let body = self
                .agent
                .get(&url)
                .call()
                .map_err(|e| crate::Error::Network(e.to_string()))?
                .into_string()
                .map_err(|e| crate::Error::Network(e.to_string()))?;
            if let Some(path) = &cache_path {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, &body);
            }
            Ok(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "hits": [
        {
          "objectID": "1001",
          "author": "someone",
          "comment_text": "I don&#x27;t buy the argument. <p>The numbers in the paper don&#x27;t support it &mdash; and the sample is tiny.",
          "created_at": "2019-05-01T12:00:00Z",
          "created_at_i": 1556712000,
          "story_id": 555
        },
        {
          "objectID": "1002",
          "author": "someone",
          "comment_text": "A later comment, after the cut-off.",
          "created_at": "2024-01-01T12:00:00Z",
          "created_at_i": 1704110400
        },
        {
          "objectID": "1003",
          "author": "someone",
          "story_text": "not a comment"
        }
      ],
      "page": 0,
      "nbPages": 3
    }"#;

    #[test]
    fn parses_hits_into_records() {
        let page = parse_page(FIXTURE, None).unwrap();
        assert_eq!(page.records.len(), 2);
        assert_eq!(page.page, 0);
        assert_eq!(page.pages, 3);
        assert!(page.has_more());
        assert_eq!(page.records[0].author.as_deref(), Some("someone"));
        assert_eq!(page.records[0].register, Register::ForumComment);
        assert_eq!(
            page.records[0].meta.get("id").map(String::as_str),
            Some("1001")
        );
    }

    #[test]
    fn the_cutoff_drops_post_chatgpt_comments() {
        let page = parse_page(FIXTURE, Some(DEFAULT_CUTOFF)).unwrap();
        assert_eq!(page.records.len(), 1, "{:?}", page.records);
        assert_eq!(
            page.records[0].meta.get("id").map(String::as_str),
            Some("1001")
        );
    }

    #[test]
    fn html_is_decoded_not_counted_as_style() {
        let page = parse_page(FIXTURE, None).unwrap();
        let text = &page.records[0].text;
        assert!(text.contains("don't"), "{text}");
        assert!(text.contains('\u{2014}'), "{text}");
        assert!(!text.contains("&#x27;"));
        assert!(!text.contains("<p>"));
        // The paragraph break survives as a blank line.
        assert!(text.contains("\n\n"), "{text:?}");
    }

    #[test]
    fn entity_decoding_covers_the_common_cases() {
        assert_eq!(clean_html("a &amp; b"), "a & b");
        assert_eq!(clean_html("&lt;tag&gt;"), "<tag>");
        assert_eq!(clean_html("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(clean_html("it&#8217;s"), "it\u{2019}s");
        assert_eq!(clean_html("wait&hellip;"), "wait\u{2026}");
        assert_eq!(clean_html("a &mdash; b"), "a \u{2014} b");
        assert_eq!(clean_html("&ldquo;quoted&rdquo;"), "\u{201C}quoted\u{201D}");
        // An entity we do not know is preserved rather than silently dropped.
        assert_eq!(clean_html("&weird;"), "&weird;");
    }

    #[test]
    fn malformed_responses_are_errors() {
        assert!(parse_page("not json", None).is_err());
        assert!(parse_page("{}", None).is_err());
    }

    #[test]
    fn urls_are_built_correctly() {
        assert_eq!(
            user_comments_url(ALGOLIA_BASE, "pg", 2, 100),
            "https://hn.algolia.com/api/v1/search_by_date?tags=comment,author_pg&page=2&hitsPerPage=100"
        );
    }

    #[test]
    fn the_cutoff_constant_is_november_2022() {
        // Sanity-check the magic number: 2022-11-01T00:00:00Z, the month
        // ChatGPT shipped and human forum text stopped being reliably human.
        let days = DEFAULT_CUTOFF / 86_400;
        let years_since_epoch = days as f64 / 365.2425;
        assert!(
            (52.8..53.0).contains(&years_since_epoch),
            "DEFAULT_CUTOFF is not late 2022: {years_since_epoch} years after the epoch"
        );
    }
}
