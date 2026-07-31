//! Reading and writing the normalized JSONL record stream.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::record::Record;
use crate::{Error, Result};

/// Write records as JSONL.
pub fn write(path: &Path, records: &[Record]) -> Result<()> {
    let file = std::fs::File::create(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut out = BufWriter::new(file);
    for record in records {
        let line = serde_json::to_string(record)?;
        writeln!(out, "{line}").map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
    }
    out.flush().map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Read records from JSONL, skipping unparseable lines.
///
/// Returns the records and the number of lines that could not be parsed. A
/// file being appended to while it is read will always have a truncated last
/// line, so a count of one is normal.
pub fn read(path: &Path) -> Result<(Vec<Record>, usize)> {
    let file = std::fs::File::open(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut records = Vec::new();
    let mut bad = 0usize;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Record>(&line) {
            Ok(record) => records.push(record),
            Err(_) => bad += 1,
        }
    }
    Ok((records, bad))
}

/// Read every `.jsonl` file in a directory tree.
pub fn read_dir(dir: &Path) -> Result<(Vec<Record>, usize)> {
    let mut records = Vec::new();
    let mut bad = 0usize;
    for path in crate::agent::jsonl_files(dir)? {
        let (mut found, skipped) = read(&path)?;
        records.append(&mut found);
        bad += skipped;
    }
    Ok((records, bad))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Register;

    #[test]
    fn round_trips_through_a_file() {
        let dir = std::env::temp_dir().join(format!("handprint-jsonl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("records.jsonl");

        let records = vec![
            Record::new("first record here", "test", Register::AgentProse).with_model("m1"),
            Record::new("second record here", "test", Register::ForumComment).with_author("a"),
        ];
        write(&path, &records).unwrap();
        let (back, bad) = read(&path).unwrap();
        assert_eq!(back, records);
        assert_eq!(bad, 0);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn truncated_lines_are_counted_not_fatal() {
        let dir = std::env::temp_dir().join(format!("handprint-jsonl2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("partial.jsonl");
        std::fs::write(
            &path,
            "{\"text\":\"ok\",\"source\":\"s\",\"register\":\"agent_prose\"}\n{\"text\":\"trunc",
        )
        .unwrap();

        let (records, bad) = read(&path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(bad, 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
