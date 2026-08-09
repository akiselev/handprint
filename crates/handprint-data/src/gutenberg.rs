//! Project Gutenberg corpus preparation.
//!
//! Two jobs, both mechanical and both easy to get subtly wrong:
//!
//! 1. **Strip the boilerplate.** A Gutenberg text is wrapped in a licence
//!    header and footer. Left in, they are the most distinctive thing in the
//!    file: every book by every author shares them word for word, so a
//!    stylometer fitted on unstripped texts learns Project Gutenberg's legal
//!    department and reports it as the author's voice.
//! 2. **Chapterize.** Per-chapter documents are what make within-author
//!    variance measurable, and within-author variance is what a two-sided band
//!    and a same-author calibration are made of. One document per book gives a
//!    calibration nothing to estimate from.
//!
//! # Jurisdiction
//!
//! US public domain is not worldwide public domain. Everything the capstone
//! fits is US-PD under the 95-year rule, and two of the authors — Wodehouse
//! (d. 1975) and Hemingway (d. 1961) — remain in copyright in life+70
//! jurisdictions until 2046 and 2032. [`PdBasis`] records the claim per author
//! so that a shipped artifact carries it, and the split is enforced by what
//! kind of artifact may be published rather than by hope: data-only artifacts
//! for all, text-bearing artifacts only where the text is worldwide-PD.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::record::{Record, Register};
use crate::{Error, Result};

/// The modern boilerplate markers.
const START_MARKERS: &[&str] = &[
    "*** START OF THE PROJECT GUTENBERG EBOOK",
    "*** START OF THIS PROJECT GUTENBERG EBOOK",
    "***START OF THE PROJECT GUTENBERG EBOOK",
];
const END_MARKERS: &[&str] = &[
    "*** END OF THE PROJECT GUTENBERG EBOOK",
    "*** END OF THIS PROJECT GUTENBERG EBOOK",
    "***END OF THE PROJECT GUTENBERG EBOOK",
];

/// Older header variants, which predate the `***` convention.
///
/// The fallback scanner exists because these are common in the archive's
/// earlier files and silently leaving their headers in is exactly the failure
/// mode described in the module docs.
const LEGACY_START: &[&str] = &[
    "*END*THE SMALL PRINT",
    "*END THE SMALL PRINT",
    "This etext was prepared by",
    "Project Gutenberg's Etext of",
];
const LEGACY_END: &[&str] = &[
    "End of Project Gutenberg",
    "End of the Project Gutenberg",
    "END OF THE PROJECT GUTENBERG",
];

/// The basis on which a text is claimed to be public domain.
///
/// Recorded per author on every shipped artifact. It is a claim, stated so a
/// reader can check it, not a legal opinion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdBasis {
    /// Short description, e.g. `"US (pre-1930 publication)"`.
    pub basis: String,
    /// Whether the text is public domain in life+70 jurisdictions too.
    pub worldwide: bool,
    /// A note explaining what may be published.
    pub note: String,
}

impl PdBasis {
    /// Public domain everywhere: the author died more than 70 years ago.
    pub fn worldwide(basis: impl Into<String>) -> PdBasis {
        PdBasis {
            basis: basis.into(),
            worldwide: true,
            note: "worldwide public domain: text-bearing artifacts (move-index sidecars, \
                   full transfer transcripts) may be published"
                .into(),
        }
    }

    /// Public domain in the US only.
    pub fn us_only(basis: impl Into<String>, until: u32) -> PdBasis {
        PdBasis {
            basis: basis.into(),
            worldwide: false,
            note: format!(
                "US public domain only; in copyright in life+70 jurisdictions until {until}. \
                 Data-only artifacts (fitted references, calibration, band edges) may be \
                 published with this basis stated; text-bearing artifacts may not."
            ),
        }
    }
}

/// A book stripped of boilerplate and split into chapters.
#[derive(Debug, Clone, PartialEq)]
pub struct Book {
    /// The author directory this belongs to.
    pub author: String,
    /// Slug derived from the source file name.
    pub slug: String,
    /// Chapters in order.
    pub chapters: Vec<Chapter>,
    /// Whether boilerplate markers were found. False means the file either was
    /// not a Gutenberg text or used a variant nothing here recognizes, and the
    /// caller should look before trusting it.
    pub stripped: bool,
    /// Editorial prose removed from inside the markers. Reported rather than
    /// silently dropped: every entry is a judgement call about who wrote
    /// something, and those are the ones worth showing a human.
    pub removed: Vec<Removal>,
}

/// One chapter.
#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    /// 1-based index within the book.
    pub index: usize,
    /// Heading line, when the splitter found one.
    pub heading: Option<String>,
    /// Body text.
    pub text: String,
}

impl Chapter {
    /// Words, by whitespace.
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

/// Remove the Project Gutenberg header and footer.
///
/// Returns the body and whether a marker pair was found. A text with no
/// recognized markers is returned unchanged with `false`, never silently
/// truncated: guessing where the licence ends would corrupt the corpus in a way
/// that looks exactly like an author with an unusual opening chapter.
pub fn strip_pg_boilerplate(text: &str) -> (&str, bool) {
    if let Some(body) = between(text, START_MARKERS, END_MARKERS) {
        return (body, true);
    }
    if let Some(body) = between(text, LEGACY_START, LEGACY_END) {
        return (body, true);
    }
    (text, false)
}

/// Openers of an editorial note written by whoever prepared the etext.
///
/// Lower-cased before matching, because the archive uses every casing.
const NOTE_OPENERS: &[&str] = &[
    "transcriber's note",
    "transcriber’s note",
    "transcribers note",
    "original transcriber's note",
    "original transcriber’s note",
    "editor's note for this etext",
    "note to this etext",
];

/// A legacy footer line that sits *inside* a body sliced by the modern markers.
///
/// `strip_pg_boilerplate` takes the earliest modern end marker, which is the
/// right rule, but the older sign-off line precedes it in files that carry
/// both. Left in, it teaches the reference a sentence naming the author.
const INNER_FOOTERS: &[&str] = &[
    "End of Project Gutenberg",
    "End of the Project Gutenberg",
    "End of The Project Gutenberg",
];

/// What was removed from one body, for the caller to report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    /// Which rule fired.
    pub rule: &'static str,
    /// Words removed.
    pub words: usize,
    /// First 80 characters of the removed span, so a human can check it.
    pub excerpt: String,
}

/// Remove the etext preparer's own prose from a stripped body.
///
/// The licence block is not the only thing in a Gutenberg file that nobody in
/// the corpus wrote. Four texts in this collection carry a transcriber's note,
/// two carry the older sign-off line inside the modern markers, and all of them
/// survive [`strip_pg_boilerplate`] because they sit between the markers rather
/// than outside them. The Pudd'nhead Wilson note is seventeen hundred words of
/// twenty-first-century historical essay filed under Mark Twain.
///
/// Three rules, each bounded, because an unbounded "delete until it looks like
/// prose again" rule is how a chapter goes missing:
///
/// 1. **Bracketed inline notes** — `[Transcriber's Note: ...]` is removed
///    exactly, span for span. Nothing is guessed.
/// 2. **A note in the last 15% of the body** — truncate to the end. A note that
///    late has no chapters after it. This is the Twain case, and the position
///    test is what keeps it away from the Dickinson case, where an identical
///    heading sits near the front with the entire book still to come.
/// 3. **A note anywhere else** — remove the heading and the one paragraph after
///    it, capped at 300 words, and report it. One paragraph is the observed
///    shape; the cap and the report are there because "observed shape" is not
///    a guarantee.
///
/// Returns the cleaned body and one [`Removal`] per rule that fired.
pub fn strip_editorial(body: &str) -> (String, Vec<Removal>) {
    let mut removals = Vec::new();
    // CRLF first, and before anything looks for a paragraph break. Half the
    // archive uses DOS line endings, `"\r\n\r\n"` does not contain `"\n\n"`,
    // and a blank-line search that silently never matches makes every note
    // look like it runs to the end of the book — which is exactly long enough
    // to trip the 300-word guard and leave the note in place. The failure is
    // invisible: the rule reports nothing, so it looks like nothing was there.
    let mut text = body.replace("\r\n", "\n");

    // Rule 1, first: a bracketed note can otherwise be re-matched by rule 3
    // and lose the paragraph that follows it, which is real prose.
    loop {
        let Some(open) = find_bracketed_note(&text) else {
            break;
        };
        let Some(close) = text[open..].find(']').map(|i| open + i + 1) else {
            break;
        };
        // A bracket that stays open for pages is not a note, it is prose that
        // happens to mention one.
        if text[open..close].split_whitespace().count() > 300 {
            break;
        }
        removals.push(removal("bracketed-note", &text[open..close]));
        text.replace_range(open..close, "");
    }

    // Rule 2 before rule 3: a trailing note should be truncated whole, not
    // trimmed to its first paragraph and left in.
    if let Some(at) = find_note_heading(&text) {
        if at as f64 > text.len() as f64 * 0.85 {
            removals.push(removal("trailing-note", &text[at..]));
            text.truncate(at);
        }
    }

    while let Some(at) = find_note_heading(&text) {
        let end = paragraph_end(&text, at);
        if text[at..end].split_whitespace().count() > 300 {
            break;
        }
        removals.push(removal("inline-note", &text[at..end]));
        text.replace_range(at..end, "");
    }

    for footer in INNER_FOOTERS {
        if let Some(at) = text.find(footer) {
            removals.push(removal("inner-footer", &text[at..]));
            text.truncate(at);
        }
    }

    text = strip_apparatus(&text, &mut removals);
    text = strip_plate_lists(&text, &mut removals);

    (text.trim().to_owned(), removals)
}

/// Phrases no novelist writes, safe to match inside a bounded paragraph.
const APPARATUS: &[&str] = &[
    "this etext",
    "project gutenberg",
    "illustrations taken from an",
    "etext was prepared",
];

/// Phrases that *are* written in prose, and so need a much shorter paragraph.
///
/// Poe's essay on Maelzel's chess automaton contains "His Essay was first
/// published in a Baltimore weekly paper". Twain's Gutenberg file contains
/// "First published in 1880" as a four-word paragraph of its own. Only the
/// length separates them.
const APPARATUS_SHORT: &[&str] = &["first published in 1", "illustrations taken from"];

/// Remove whole paragraphs that are the etext's apparatus rather than the book.
///
/// Paradise Lost opens with a page about a 486 running DOS; A Tramp Abroad
/// restarts three times with a publication slug. Both survive the marker strip
/// because they sit inside the markers, and both are filed under the author.
fn strip_apparatus(text: &str, removals: &mut Vec<Removal>) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for paragraph in text.split("\n\n") {
        let words = paragraph.split_whitespace().count();
        let lowered = paragraph.to_lowercase();
        let long_hit = words <= 300 && APPARATUS.iter().any(|m| lowered.contains(m));
        let short_hit = words <= 25 && APPARATUS_SHORT.iter().any(|m| lowered.contains(m));
        if (long_hit || short_hit) && words > 0 {
            removals.push(removal("apparatus", paragraph));
            continue;
        }
        kept.push(paragraph);
    }
    kept.join("\n\n")
}

/// Remove a run of numbered plate captions, e.g. `41.  AN OBJECT OF ADMIRATION`.
///
/// One such line is a list item in a book that has lists. Five in a row, set in
/// capitals, is the front-of-volume illustration index, repeated once per part.
///
/// The capitals requirement is not decoration. Without it this rule deletes
/// Watson's list of Sherlock Holmes's limits — "1. Knowledge of Literature.—Nil.
/// 2. Philosophy.—Nil. 3. Astronomy.—Nil." — which is a numbered run of short
/// items and is also one of the most characteristic passages Doyle ever wrote.
/// Plate captions are typeset in caps; a novelist's list is written in
/// sentence case, and that is the whole of the difference.
fn strip_plate_lists(text: &str, removals: &mut Vec<Removal>) -> String {
    const RUN: usize = 5;
    let lines: Vec<&str> = text.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let mut j = i;
        while j < lines.len() && is_plate_caption(lines[j]) {
            j += 1;
        }
        if j - i >= RUN {
            removals.push(removal("plate-list", &lines[i..j].join("\n")));
            i = j;
            continue;
        }
        kept.push(lines[i]);
        i += 1;
    }
    kept.join("\n")
}

fn is_plate_caption(line: &str) -> bool {
    let trimmed = line.trim();
    let Some((number, rest)) = trimmed.split_once('.') else {
        return false;
    };
    let rest = rest.trim();
    if number.is_empty()
        || number.len() > 4
        || !number.chars().all(|c| c.is_ascii_digit())
        || rest.is_empty()
        || rest.split_whitespace().count() > 12
    {
        return false;
    }
    let letters = rest.chars().filter(|c| c.is_alphabetic()).count();
    let upper = rest.chars().filter(|c| c.is_uppercase()).count();
    letters > 0 && upper * 10 >= letters * 8
}

fn removal(rule: &'static str, span: &str) -> Removal {
    Removal {
        rule,
        words: span.split_whitespace().count(),
        excerpt: span.split_whitespace().take(14).collect::<Vec<_>>().join(" "),
    }
}

/// Byte offset of a `[` that opens a bracketed transcriber's note.
fn find_bracketed_note(text: &str) -> Option<usize> {
    text.match_indices('[')
        .find(|(i, _)| {
            let tail = &text[*i + 1..];
            let head = tail.get(..40).unwrap_or(tail).to_lowercase();
            NOTE_OPENERS.iter().any(|opener| head.starts_with(opener))
        })
        .map(|(i, _)| i)
}

/// Byte offset of a line that is nothing but a transcriber's-note heading.
///
/// Line-anchored on purpose: a novel is free to contain the words "the
/// transcriber's note lay on the table", and that sentence is the author's.
fn find_note_heading(text: &str) -> Option<usize> {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        let lowered = trimmed.trim_end_matches([':', '.']).to_lowercase();
        if NOTE_OPENERS.contains(&lowered.as_str())
            || NOTE_OPENERS
                .iter()
                .any(|o| lowered.starts_with(o) && trimmed.split_whitespace().count() <= 6)
        {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

/// End of the paragraph block that starts at `from`: the next blank line after
/// the heading's own line, or the end of the text.
fn paragraph_end(text: &str, from: usize) -> usize {
    let rest = &text[from..];
    let after_heading = rest.find('\n').map_or(rest.len(), |i| i + 1);
    let body = &rest[after_heading..];
    // Skip the blank line separating heading from note, then run to the next.
    let start = body
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map_or(body.len(), |(i, _)| i);
    match body[start..].find("\n\n") {
        Some(i) => from + after_heading + start + i,
        None => text.len(),
    }
}

/// Slice from after the *last* start marker to the *first* end marker.
///
/// Last and first, not first and last: a file can carry several header
/// variants stacked (a title line, a "prepared by" line, then the small-print
/// block), and taking the first would leave the rest of the header in the body.
/// The end is the earliest footer marker for the mirror reason.
fn between<'a>(text: &'a str, starts: &[&str], ends: &[&str]) -> Option<&'a str> {
    let start =
        starts
            .iter()
            .filter_map(|m| text.rfind(m))
            .max()
            .map(|i| match text[i..].find('\n') {
                Some(nl) => i + nl + 1,
                None => text.len(),
            })?;
    let end = ends
        .iter()
        .filter_map(|m| text.find(m))
        .filter(|&e| e > start)
        .min()
        .unwrap_or(text.len());
    Some(text[start..end].trim())
}

/// Whether a line opens a chapter.
///
/// `CHAPTER I`, `CHAPTER 12`, a bare roman numeral, a markdown `#` heading, or
/// `BOOK THE FIRST`. The rule is deliberately conservative — a false split
/// costs a chapter boundary, a false *merge* costs a whole document's worth of
/// within-author variance, and only one of those is recoverable by looking.
pub fn is_chapter_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 60 {
        return false;
    }
    if let Some(rest) = trimmed.strip_prefix('#') {
        return rest.starts_with(' ') || rest.starts_with('#');
    }
    let upper = trimmed.to_uppercase();
    for prefix in ["CHAPTER", "BOOK", "PART", "CANTO", "LETTER", "ACT", "SCENE"] {
        if let Some(rest) = upper.strip_prefix(prefix) {
            if rest.starts_with(' ') || rest.starts_with('.') {
                return true;
            }
        }
    }
    // A line that is nothing but a roman numeral, optionally with a period.
    //
    // Bounded at 100: "MIX" and "DID" are letters from the numeral alphabet and
    // one of them even parses (1009), but no book has a chapter MIX.
    roman_value(upper.trim_end_matches('.')).is_some_and(|n| (1..=100).contains(&n))
}

/// Parse a well-formed roman numeral, rejecting anything that is not one.
///
/// Well-formed matters: a bare "all characters are in IVXLCDM" test claims
/// every English word spelled from those letters, and `DIM`, `MILD` and `CIVIL`
/// all appear as headings in ordinary prose.
fn roman_value(s: &str) -> Option<u32> {
    if s.is_empty() || s.len() > 8 {
        return None;
    }
    let digit = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let values: Vec<u32> = s.chars().map(digit).collect::<Option<Vec<u32>>>()?;
    let mut total = 0u32;
    let mut i = 0usize;
    // The largest step allowed next. A plain digit `d` permits another `d`;
    // a subtractive pair `(s, b)` permits nothing at or above `s`, which is
    // what rejects "IVI" while accepting "XIV".
    let mut max_next = u32::MAX;
    while i < values.len() {
        let current = values[i];
        let next = values.get(i + 1).copied();
        let (step, ceiling) = match next {
            Some(n) if n > current => {
                let valid = matches!(
                    (current, n),
                    (1, 5) | (1, 10) | (10, 50) | (10, 100) | (100, 500) | (100, 1000)
                );
                if !valid {
                    return None;
                }
                i += 2;
                (n - current, current.saturating_sub(1))
            }
            _ => {
                i += 1;
                (current, current)
            }
        };
        if step > max_next {
            return None;
        }
        max_next = ceiling;
        total += step;
    }
    Some(total)
}

/// Split a stripped body into chapters.
///
/// Everything before the first heading is dropped as front matter unless there
/// are no headings at all, in which case the whole body is one chapter — an
/// honest "we could not split this" rather than a fabricated structure.
pub fn chapterize(body: &str) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut heading: Option<String> = None;
    let mut buffer = String::new();
    let mut seen_heading = false;

    for line in body.lines() {
        if is_chapter_heading(line) {
            if seen_heading && !buffer.trim().is_empty() {
                chapters.push(Chapter {
                    index: chapters.len() + 1,
                    heading: heading.clone(),
                    text: buffer.trim().to_owned(),
                });
            }
            heading = Some(line.trim().trim_start_matches(['#', ' ']).to_owned());
            buffer.clear();
            seen_heading = true;
            continue;
        }
        buffer.push_str(line);
        buffer.push('\n');
    }
    if seen_heading && !buffer.trim().is_empty() {
        chapters.push(Chapter {
            index: chapters.len() + 1,
            heading,
            text: buffer.trim().to_owned(),
        });
    }
    if chapters.is_empty() && !body.trim().is_empty() {
        chapters.push(Chapter {
            index: 1,
            heading: None,
            text: body.trim().to_owned(),
        });
    }
    chapters
}

/// How to prepare a Gutenberg corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GutenbergConfig {
    /// Chapters shorter than this many words are merged into the previous one.
    ///
    /// Front matter, dedications and one-line "CHAPTER XII" stubs otherwise
    /// become documents, and a corpus of hundred-word documents estimates a
    /// calibration from noise.
    pub min_words: usize,
    /// Chapters longer than this are left alone rather than split further.
    pub max_words: usize,
}

impl Default for GutenbergConfig {
    fn default() -> Self {
        GutenbergConfig {
            // Around the low end of the plan's 2-5k target: long enough to be
            // above the per-document reliability floor after aggregation.
            min_words: 1_500,
            max_words: 12_000,
        }
    }
}

/// Prepare one file.
pub fn prepare(path: &Path, author: &str, config: &GutenbergConfig) -> Result<Book> {
    let raw = std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let (body, stripped) = strip_pg_boilerplate(&raw);
    let (body, removed) = strip_editorial(body);
    let chapters = merge_short(chapterize(&body), config.min_words);
    Ok(Book {
        author: author.to_owned(),
        slug: slug(path),
        chapters,
        stripped,
        removed,
    })
}

/// Merge chapters below the floor into their predecessor.
fn merge_short(chapters: Vec<Chapter>, min_words: usize) -> Vec<Chapter> {
    let mut out: Vec<Chapter> = Vec::new();
    for chapter in chapters {
        if chapter.word_count() < min_words {
            if let Some(previous) = out.last_mut() {
                previous.text.push_str("\n\n");
                previous.text.push_str(&chapter.text);
                continue;
            }
        }
        out.push(chapter);
    }
    // A trailing short chapter with no predecessor to merge into is kept, and
    // the indices are renumbered so they stay contiguous.
    for (i, chapter) in out.iter_mut().enumerate() {
        chapter.index = i + 1;
    }
    out
}

/// Turn a book into one [`Record`] per chapter.
pub fn to_records(book: &Book) -> Vec<Record> {
    book.chapters
        .iter()
        .map(|chapter| {
            let mut record = Record::new(
                chapter.text.clone(),
                format!("gutenberg:{}/{}", book.author, book.slug),
                Register::Article,
            )
            .with_author(book.author.clone())
            .with_meta("book", book.slug.clone())
            .with_meta("chapter", chapter.index.to_string());
            if let Some(heading) = &chapter.heading {
                record = record.with_meta("heading", heading.clone());
            }
            record
        })
        .collect()
}

/// Write a book as one text file per chapter under `out/{author}/`.
pub fn write_chapters(book: &Book, out: &Path) -> Result<Vec<PathBuf>> {
    let dir = out.join(&book.author);
    std::fs::create_dir_all(&dir).map_err(|source| Error::Io {
        path: dir.clone(),
        source,
    })?;
    let mut written = Vec::new();
    for chapter in &book.chapters {
        let path = dir.join(format!("{}-ch{:02}.md", book.slug, chapter.index));
        std::fs::write(&path, &chapter.text).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        written.push(path);
    }
    Ok(written)
}

/// A file-name slug: lowercase, alphanumeric and dashes.
fn slug(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");
    let mut out = String::new();
    let mut dash = false;
    for c in stem.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_owned()
}

/// The capstone's authors and their public-domain bases.
///
/// Exposed as data so the experiment scripts and the shipped provenance read
/// the same claim, rather than each restating it.
pub fn capstone_authors() -> Vec<(&'static str, PdBasis)> {
    vec![
        (
            "wodehouse",
            PdBasis::us_only("US (pre-1930 publication)", 2046),
        ),
        (
            "hemingway-early",
            PdBasis::us_only(
                "US (In Our Time 1925, Men Without Women 1927, A Farewell to Arms 1929)",
                2032,
            ),
        ),
        ("twain", PdBasis::worldwide("worldwide (d. 1910)")),
        ("poe", PdBasis::worldwide("worldwide (d. 1849)")),
        ("austen", PdBasis::worldwide("worldwide (d. 1817)")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODERN: &str = "The Project Gutenberg eBook of Something\n\
        \n\
        This ebook is for the use of anyone anywhere.\n\
        \n\
        *** START OF THE PROJECT GUTENBERG EBOOK SOMETHING ***\n\
        \n\
        CHAPTER I\n\
        \n\
        It was a bright cold day in April.\n\
        \n\
        CHAPTER II\n\
        \n\
        The clocks were striking thirteen.\n\
        \n\
        *** END OF THE PROJECT GUTENBERG EBOOK SOMETHING ***\n\
        \n\
        Updated editions will replace the previous one.\n";

    const LEGACY: &str = "Project Gutenberg's Etext of Something\n\
        \n\
        *END*THE SMALL PRINT! FOR PUBLIC DOMAIN ETEXTS*Ver.04.29.93*END*\n\
        \n\
        I.\n\
        \n\
        It was a bright cold day in April.\n\
        \n\
        End of Project Gutenberg Etext of Something\n";

    #[test]
    fn the_modern_markers_are_stripped_from_both_ends() {
        let (body, stripped) = strip_pg_boilerplate(MODERN);
        assert!(stripped);
        assert!(body.starts_with("CHAPTER I"), "{body:?}");
        assert!(!body.contains("Updated editions"));
        assert!(!body.contains("PROJECT GUTENBERG EBOOK"));
    }

    #[test]
    fn the_legacy_variant_is_handled_by_the_fallback_scanner() {
        let (body, stripped) = strip_pg_boilerplate(LEGACY);
        assert!(stripped);
        assert!(body.starts_with("I."), "{body:?}");
        assert!(!body.contains("SMALL PRINT"));
        assert!(!body.contains("End of Project Gutenberg"));
    }

    #[test]
    fn a_trailing_transcribers_note_is_truncated_whole() {
        // The Pudd'nhead Wilson shape: the story ends, and seventeen hundred
        // words of modern historical essay follow under the author's name.
        let body = format!(
            "{}\n\nTranscriber's Notes\n\nWelcome to Project Gutenberg's \
             presentation of this book. Homer Plessy was arrested in 1892.\n",
            "He pardoned Tom at once, and the creditors sold him. ".repeat(40)
        );
        let (out, removed) = strip_editorial(&body);
        assert!(!out.contains("Transcriber"), "{out:?}");
        assert!(out.contains("pardoned Tom"));
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].rule, "trailing-note");
    }

    #[test]
    fn a_note_near_the_front_loses_its_paragraph_and_not_the_book() {
        // The Moby-Dick shape, and the reason the trailing rule is
        // position-gated: an identical heading sits near the front of the
        // Dickinson file with the whole book still to come.
        let body = "Original Transcriber's Notes:\n\n\
                    This text is a combination of etexts from two archives.\n\n\
                    ETYMOLOGY.\n\n\
                    The pale Usher—threadbare in coat, heart, body, and brain.\n";
        let (out, removed) = strip_editorial(body);
        assert!(!out.contains("combination of etexts"), "{out:?}");
        assert!(out.contains("pale Usher"), "the book survives: {out:?}");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].rule, "inline-note");
    }

    #[test]
    fn a_note_in_a_dos_file_is_found_too() {
        // Moby-Dick is CRLF. Before the normalisation this test guards, the
        // paragraph scan found no blank line, treated the note as running to
        // the end of the book, tripped the length guard and reported nothing.
        let body = "Original Transcriber's Notes:\r\n\r\n\
                    This text is a combination of etexts from two archives.\r\n\r\n\
                    ETYMOLOGY.\r\n\r\n\
                    The pale Usher, threadbare in coat, heart, body, and brain.\r\n";
        let (out, removed) = strip_editorial(body);
        assert_eq!(removed.len(), 1, "{removed:?}");
        assert!(removed[0].words < 20, "the note, not the book: {removed:?}");
        assert!(out.contains("pale Usher"), "{out:?}");
    }

    #[test]
    fn a_bracketed_note_is_removed_span_for_span() {
        let body = "He looked at the leaky roof. [Transcriber's Note for edition 11: \
                    the word \"leafy\" has been changed to \"leaky\".--jt] Then he left.\n";
        let (out, removed) = strip_editorial(body);
        assert!(out.contains("leaky roof"));
        assert!(out.contains("Then he left"));
        assert!(!out.contains("edition 11"), "{out:?}");
        assert_eq!(removed[0].rule, "bracketed-note");
    }

    #[test]
    fn the_older_sign_off_inside_the_modern_markers_is_cut() {
        let body = "'Can you beat it?' said Henry, silently, to himself.\n\n\
                    End of Project Gutenberg's The Man with Two Left Feet, by P. G. Wodehouse\n";
        let (out, removed) = strip_editorial(body);
        assert!(out.ends_with("to himself."), "{out:?}");
        assert_eq!(removed[0].rule, "inner-footer");
    }

    #[test]
    fn the_etexts_own_provenance_page_is_not_the_author() {
        let body = "Introduction (one page)\n\n\
                    This etext was originally created in 1964-1965 according to \
                    Dr. Joseph Raben of Queens College, NY.\n\n\
                    Of Man's first disobedience, and the fruit of that forbidden tree.\n";
        let (out, removed) = strip_editorial(body);
        assert!(!out.contains("Queens College"), "{out:?}");
        assert!(out.contains("first disobedience"));
        assert_eq!(removed[0].rule, "apparatus");
    }

    #[test]
    fn a_publication_slug_goes_but_a_sentence_about_publishing_stays() {
        // Twain's file restarts each part with a four-word slug. Poe's essay
        // says the same words inside a paragraph of argument. Length is the
        // only thing that separates them, so the rule is length-gated.
        let slug = "First published in 1880\n\nHe went down the road.\n";
        let (out, removed) = strip_editorial(slug);
        assert!(!out.contains("1880"), "{out:?}");
        assert_eq!(removed.len(), 1);

        let prose = "His Essay was first published in 1836 in a Baltimore weekly \
                     paper, was illustrated by cuts, and was entitled \"An attempt \
                     to analyse the Automaton Chess Player\", a title we cannot \
                     consider altogether the true one, although the solution is \
                     ingenious and deserves a hearing on its own terms.\n";
        let (out, removed) = strip_editorial(prose);
        assert!(out.contains("Baltimore"), "{out:?}");
        assert!(removed.is_empty(), "{removed:?}");
    }

    #[test]
    fn a_run_of_plate_captions_goes_and_a_lone_numbered_line_stays() {
        let index = "ILLUSTRATIONS:\n\
                     1.   PORTRAIT OF THE AUTHOR\n\
                     2.   TITIAN'S MOSES\n\
                     3.   THE AUTHOR'S MEMORIES\n\
                     4.   FRENCH CALM\n\
                     5.   THE CHALLENGE ACCEPTED\n\
                     \n\
                     It was a bright cold day in April.\n";
        let (out, removed) = strip_editorial(index);
        assert!(!out.contains("TITIAN"), "{out:?}");
        assert!(out.contains("bright cold day"));
        assert_eq!(removed[0].rule, "plate-list");

        let prose = "3. He had three rules, and the third was the one that mattered.\n\n\
                     He never wrote it down.\n";
        let (out, removed) = strip_editorial(prose);
        assert!(out.contains("three rules"), "{out:?}");
        assert!(removed.is_empty());
    }

    #[test]
    fn watsons_list_of_holmess_limits_is_not_a_plate_index() {
        // Sentence case, not capitals: a numbered run a novelist wrote.
        let body = "His limits were as follows:\n\
                    1. Knowledge of Literature.—Nil.\n\
                    2. Philosophy.—Nil.\n\
                    3. Astronomy.—Nil.\n\
                    4. Politics.—Feeble.\n\
                    5. Botany.—Variable.\n\
                    6. Geology.—Practical, but limited.\n";
        let (out, removed) = strip_editorial(body);
        assert!(out.contains("Botany"), "{out:?}");
        assert!(removed.is_empty(), "{removed:?}");
    }

    #[test]
    fn prose_that_merely_mentions_a_transcriber_is_left_alone() {
        // Line-anchored matching: the heading rule must not fire on a sentence.
        let body = "The transcriber's note lay on the table where he had left it, \
                    and nobody read it for a week.\n";
        let (out, removed) = strip_editorial(body);
        assert_eq!(out, body.trim());
        assert!(removed.is_empty());
    }

    #[test]
    fn an_unrecognized_file_is_returned_whole_and_flagged() {
        // Guessing where the licence ends would corrupt the corpus in a way
        // that looks exactly like an author with an unusual opening chapter.
        let text = "Just some prose with no markers at all.\n";
        let (body, stripped) = strip_pg_boilerplate(text);
        assert!(!stripped);
        assert_eq!(body, text);
    }

    #[test]
    fn boilerplate_would_dominate_a_profile_if_it_survived() {
        // The reason this module exists, asserted rather than assumed: the
        // header is a large fraction of a short file and is identical across
        // every book in the archive.
        let (body, _) = strip_pg_boilerplate(MODERN);
        assert!(
            body.len() * 2 < MODERN.len(),
            "the fixture must be mostly boilerplate for this to mean anything"
        );
    }

    #[test]
    fn chapter_headings_are_recognized_in_their_usual_forms() {
        assert!(is_chapter_heading("CHAPTER I"));
        assert!(is_chapter_heading("Chapter 12"));
        assert!(is_chapter_heading("  CHAPTER XIV  "));
        assert!(is_chapter_heading("# The Beginning"));
        assert!(is_chapter_heading("## Part Two"));
        assert!(is_chapter_heading("XIV"));
        assert!(is_chapter_heading("IV."));
        assert!(is_chapter_heading("BOOK THE FIRST"));
        assert!(is_chapter_heading("ACT I"));
        // And not in these.
        assert!(!is_chapter_heading("It was a bright cold day in April."));
        assert!(!is_chapter_heading(""));
        assert!(!is_chapter_heading(
            "I went to the shop and bought a chapter of my life"
        ));
        // Letters from the numeral alphabet that are English words. "MIX"
        // even parses as 1009; no book has a chapter MIX.
        assert!(!is_chapter_heading("MIX"));
        assert!(!is_chapter_heading("DIM"));
        assert!(!is_chapter_heading("CIVIL"));
        assert_eq!(roman_value("XIV"), Some(14));
        assert_eq!(roman_value("MIX"), Some(1009));
        assert_eq!(roman_value("DIM"), None);
        assert_eq!(roman_value("IVI"), None);
    }

    #[test]
    fn chapterize_splits_on_headings_and_drops_front_matter() {
        let (body, _) = strip_pg_boilerplate(MODERN);
        let chapters = chapterize(body);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].heading.as_deref(), Some("CHAPTER I"));
        assert_eq!(chapters[0].text, "It was a bright cold day in April.");
        assert_eq!(chapters[1].index, 2);
    }

    #[test]
    fn a_book_with_no_headings_becomes_one_chapter_rather_than_none() {
        let chapters = chapterize("Just prose, all the way down, with no structure at all.");
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].heading, None);
        assert!(chapterize("   \n\n  ").is_empty());
    }

    #[test]
    fn short_chapters_merge_into_their_predecessor() {
        // A "CHAPTER XII" stub or a dedication must not become a document: a
        // corpus of hundred-word documents estimates a calibration from noise.
        let chapters = vec![
            Chapter {
                index: 1,
                heading: Some("I".into()),
                text: "word ".repeat(2_000),
            },
            Chapter {
                index: 2,
                heading: Some("II".into()),
                text: "short".into(),
            },
            Chapter {
                index: 3,
                heading: Some("III".into()),
                text: "word ".repeat(2_000),
            },
        ];
        let merged = merge_short(chapters, 1_500);
        assert_eq!(merged.len(), 2);
        assert!(merged[0].text.contains("short"));
        assert_eq!(merged[1].index, 2, "indices stay contiguous");
    }

    #[test]
    fn records_carry_the_book_and_chapter_in_provenance() {
        let book = Book {
            author: "twain".into(),
            slug: "jumping-frog".into(),
            chapters: chapterize("CHAPTER I\n\nSome prose here.\n"),
            stripped: true,
            removed: Vec::new(),
        };
        let records = to_records(&book);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].author.as_deref(), Some("twain"));
        assert_eq!(
            records[0].meta.get("book").map(String::as_str),
            Some("jumping-frog")
        );
        assert_eq!(
            records[0].meta.get("chapter").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn slugs_are_filesystem_safe() {
        assert_eq!(
            slug(Path::new("/x/The Jumping Frog!.txt")),
            "the-jumping-frog"
        );
        assert_eq!(slug(Path::new("pg1234.txt")), "pg1234");
    }

    #[test]
    fn the_jurisdiction_split_is_data_not_folklore() {
        let authors = capstone_authors();
        let worldwide: Vec<&str> = authors
            .iter()
            .filter(|(_, b)| b.worldwide)
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(worldwide, vec!["twain", "poe", "austen"]);

        for (name, basis) in &authors {
            assert!(!basis.basis.is_empty(), "{name} states no basis");
            if basis.worldwide {
                assert!(basis.note.contains("may be published"));
            } else {
                // The two US-only authors must say what may *not* be published.
                assert!(basis.note.contains("may not"), "{name}: {}", basis.note);
                assert!(basis.basis.starts_with("US"));
            }
        }
    }
}
