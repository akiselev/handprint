//! Loading corpora, references and text from the filesystem.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use handprint_core::{Corpus, Document, Reference};

/// Extensions treated as plain-text documents.
const TEXT_EXTENSIONS: &[&str] = &["txt", "md", "text"];

/// Load a corpus from a path.
///
/// Three layouts are accepted, distinguished by what is on disk:
///
/// * a **JSONL file** of `handprint-data` records, grouped by model or author;
/// * a **directory of subdirectories**, where each subdirectory is one author
///   and each file inside it is one document;
/// * a **flat directory** of text files, each treated as its own author, which
///   is the right shape for a background corpus of unrelated writers.
pub fn load_corpus(path: &Path, private: bool) -> Result<Corpus> {
    if !path.exists() {
        bail!("{} does not exist", path.display());
    }
    let corpus = if path.is_file() {
        match path.extension().and_then(|e| e.to_str()) {
            Some("jsonl") => load_jsonl(path)?,
            _ => {
                let text = read_file(path)?;
                let author = stem(path);
                Corpus::new().with(author, [Document::new(text)])
            }
        }
    } else {
        load_directory(path)?
    };
    if corpus.is_empty() {
        bail!("{} contained no documents", path.display());
    }
    Ok(corpus.private(private))
}

fn load_jsonl(path: &Path) -> Result<Corpus> {
    let (records, bad) = handprint_data::jsonl::read(path)
        .with_context(|| format!("reading {}", path.display()))?;
    if bad > 0 {
        eprintln!(
            "warning: skipped {bad} unparseable line(s) in {}",
            path.display()
        );
    }
    Ok(handprint_data::to_corpus(records))
}

fn load_directory(dir: &Path) -> Result<Corpus> {
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if is_text(&path) || path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
    subdirs.sort();
    files.sort();

    let mut corpus = Corpus::new();
    // Author-per-subdirectory takes precedence: it is the only layout that can
    // express "these documents share an author", which everything downstream
    // needs.
    for subdir in &subdirs {
        let author = stem(subdir);
        let mut docs = Vec::new();
        for path in text_files(subdir)? {
            docs.push(Document::new(read_file(&path)?).with_id(path.display().to_string()));
        }
        if !docs.is_empty() {
            corpus.add(author, docs);
        }
    }
    for path in files {
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            let extra = load_jsonl(&path)?;
            for group in extra.authors() {
                corpus.add(group.author.clone(), group.docs.clone());
            }
            continue;
        }
        corpus.add(
            stem(&path),
            [Document::new(read_file(&path)?).with_id(path.display().to_string())],
        );
    }
    Ok(corpus)
}

fn text_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file() && is_text(&path) {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn is_text(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| TEXT_EXTENSIONS.contains(&e))
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

/// Read text from a file, or from stdin when the path is `-` or absent.
pub fn read_text(path: Option<&Path>) -> Result<String> {
    match path {
        None => read_stdin(),
        Some(p) if p.as_os_str() == "-" => read_stdin(),
        Some(p) => read_file(p),
    }
}

fn read_stdin() -> Result<String> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .context("reading stdin")?;
    Ok(buffer)
}

/// Load a fitted reference.
pub fn load_reference(path: &Path) -> Result<Reference> {
    let text = read_file(path)?;
    serde_json::from_str(&text)
        .with_context(|| format!("{} is not a handprint reference", path.display()))
}

/// Save a fitted reference.
pub fn save_reference(path: &Path, reference: &Reference) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(reference)?;
    std::fs::write(path, json).with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("handprint-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn author_per_subdirectory() {
        let dir = scratch("subdirs");
        std::fs::create_dir_all(dir.join("alice")).unwrap();
        std::fs::create_dir_all(dir.join("bob")).unwrap();
        std::fs::write(dir.join("alice/one.txt"), "alice wrote this").unwrap();
        std::fs::write(dir.join("alice/two.md"), "and this too").unwrap();
        std::fs::write(dir.join("bob/one.txt"), "bob wrote this").unwrap();

        let corpus = load_corpus(&dir, false).unwrap();
        assert_eq!(corpus.author_count(), 2);
        assert_eq!(corpus.author(&"alice".into()).unwrap().len(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn flat_directory_is_one_author_per_file() {
        let dir = scratch("flat");
        std::fs::write(dir.join("a.txt"), "first").unwrap();
        std::fs::write(dir.join("b.txt"), "second").unwrap();
        std::fs::write(dir.join("ignored.bin"), "not text").unwrap();

        let corpus = load_corpus(&dir, false).unwrap();
        assert_eq!(corpus.author_count(), 2);
        assert_eq!(corpus.len(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn jsonl_records_group_by_model() {
        let dir = scratch("jsonl");
        let path = dir.join("records.jsonl");
        std::fs::write(
            &path,
            "{\"text\":\"one\",\"source\":\"s\",\"register\":\"agent_prose\",\"model\":\"m1\"}\n\
             {\"text\":\"two\",\"source\":\"s\",\"register\":\"agent_prose\",\"model\":\"m1\"}\n\
             {\"text\":\"three\",\"source\":\"s\",\"register\":\"agent_prose\",\"model\":\"m2\"}\n",
        )
        .unwrap();

        let corpus = load_corpus(&path, false).unwrap();
        assert_eq!(corpus.author_count(), 2);
        assert_eq!(corpus.author(&"m1".into()).unwrap().len(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn private_flag_propagates() {
        let dir = scratch("private");
        std::fs::write(dir.join("a.txt"), "text").unwrap();
        assert!(load_corpus(&dir, true).unwrap().is_private());
        assert!(!load_corpus(&dir, false).unwrap().is_private());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_and_empty_paths_are_errors() {
        assert!(load_corpus(Path::new("/nonexistent/handprint-x"), false).is_err());
        let dir = scratch("empty");
        assert!(load_corpus(&dir, false).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn references_round_trip_on_disk() {
        use handprint_core::feature::PunctTypography;
        use handprint_core::Pipeline;

        let dir = scratch("refs");
        let corpus = Corpus::new()
            .with("a", [Document::new("some text here, with commas")])
            .with("b", [Document::new("other text\u{2014}with dashes")]);
        let reference = Pipeline::builder()
            .feature(PunctTypography::default())
            .name("t")
            .fit(&corpus)
            .unwrap();

        let path = dir.join("nested").join("ref.json");
        save_reference(&path, &reference).unwrap();
        let back = load_reference(&path).unwrap();
        assert_eq!(back.fingerprint(), reference.fingerprint());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
