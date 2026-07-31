//! `handprint` — the command-line surface.
//!
//! Two audiences share one binary. A person runs `fit`, `compare`, `rank`,
//! `verify` and `explain` to investigate; an agent runs `critique --json` in a
//! loop. The forensic commands print human-readable tables; `critique` prints
//! the versioned JSON contract and uses Vale-style exit codes, so a rewrite loop
//! is a shell `while` statement.

mod commands;
mod config;
mod io;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Interpretable stylometry: profile, compare, and explain writing style.
#[derive(Debug, Parser)]
#[command(
    name = "handprint",
    version,
    about,
    long_about = "Interpretable stylometry: profile, compare, and explain writing style.\n\n\
                  handprint is an objective function and an explainer, not a verdict machine \
                  and not a rewriter. Distances come with per-feature decompositions and \
                  calibrated p-values; `critique --json` emits actionable findings for an \
                  agent to act on.\n\n\
                  Read `p_value_vs_unrelated` as \"how often unrelated authors land this \
                  close\", never as \"the probability these differ\"."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Which distance metric to use.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MetricArg {
    /// Angle between z-score vectors. The default for two-text comparison.
    Cosine,
    /// Burrows's Classic Delta. The right choice for one-class membership.
    Burrows,
    /// Eder's rank-weighted Delta.
    Eder,
    /// MinMax / Ruzicka on relative frequencies.
    Minmax,
}

impl From<MetricArg> for handprint_core::Metric {
    fn from(m: MetricArg) -> Self {
        match m {
            MetricArg::Cosine => handprint_core::Metric::CosineDelta,
            MetricArg::Burrows => handprint_core::Metric::BurrowsDelta,
            MetricArg::Eder => handprint_core::Metric::Eder,
            MetricArg::Minmax => handprint_core::Metric::MinMax,
        }
    }
}

/// Which feature families to include when fitting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FeatureArg {
    /// Punctuation and typography rates.
    Punct,
    /// Sentence, paragraph and markdown structure.
    Sentence,
    /// Most-frequent words.
    Mfw,
    /// Most-frequent function words only — the topic-robust setting.
    FunctionWords,
    /// Typed character n-grams.
    Ngrams,
    /// Length-corrected lexical richness.
    Richness,
    /// The built-in AI-style lexicon.
    Lexicon,
    /// Corpus language-model surprisal.
    Surprisal,
}

/// Which threshold profile to hold a draft to.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ProfileArg {
    /// Wide bands, low pass bar.
    Lenient,
    /// The default.
    Balanced,
    /// Narrow bands, high pass bar.
    Strict,
}

impl From<ProfileArg> for handprint_core::ThresholdProfile {
    fn from(p: ProfileArg) -> Self {
        match p {
            ProfileArg::Lenient => handprint_core::ThresholdProfile::Lenient,
            ProfileArg::Balanced => handprint_core::ThresholdProfile::Balanced,
            ProfileArg::Strict => handprint_core::ThresholdProfile::Strict,
        }
    }
}

/// Which critique mode to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    /// Match a corpus: "does this draft belong here yet?"
    Corpus,
    /// Move away from one reference point and toward another.
    Contrast,
}

/// Which agent transcripts to read.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExtractSource {
    /// Claude Code transcripts.
    ClaudeCode,
    /// Codex CLI rollouts.
    Codex,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Fit a reference from a corpus.
    Fit {
        /// Corpus path: a JSONL record file, a directory of author
        /// subdirectories, or a flat directory of text files.
        corpus: PathBuf,
        /// Where to write the fitted reference.
        #[arg(short, long)]
        out: PathBuf,
        /// Feature families to include.
        #[arg(long, value_enum, value_delimiter = ',', default_values = ["punct", "sentence", "lexicon"])]
        features: Vec<FeatureArg>,
        /// Name recorded in the artifact's provenance.
        #[arg(long, default_value = "unnamed")]
        name: String,
        /// Version recorded in the artifact's provenance.
        #[arg(long, default_value = "0")]
        version: String,
        /// What the reference models.
        #[arg(long, default_value = "corpus")]
        kind: String,
        /// Default metric for this reference.
        #[arg(long, value_enum, default_value = "cosine")]
        metric: MetricArg,
        /// Words to keep for the frequent-word family.
        #[arg(long, default_value_t = 500)]
        top_words: usize,
        /// N-grams to keep per order.
        #[arg(long, default_value_t = 1000)]
        top_ngrams: usize,
        /// Representative document profiles to store in the artifact.
        #[arg(long, default_value_t = handprint_core::reference::DEFAULT_EXEMPLARS)]
        exemplars: usize,
        /// Mark the corpus private, which turns on privacy culling wherever a
        /// vocabulary would be derived from it.
        #[arg(long)]
        private: bool,
        /// Free-form provenance note; repeatable.
        #[arg(long)]
        note: Vec<String>,
    },

    /// Fit calibration distributions onto an existing reference.
    Calibrate {
        /// The reference to calibrate.
        reference: PathBuf,
        /// Background corpus: a population of unrelated authors, each with
        /// several documents.
        #[arg(long)]
        background: PathBuf,
        /// Where to write the calibrated reference. Defaults to overwriting.
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Metric to calibrate for. Defaults to the reference's own.
        #[arg(long, value_enum)]
        metric: Option<MetricArg>,
        /// Target lengths, in lexical tokens.
        #[arg(long, value_delimiter = ',', default_values_t = [128usize, 256, 512, 1024, 2048])]
        bins: Vec<usize>,
        /// Unrelated pairs sampled per bin.
        #[arg(long, default_value_t = 5000)]
        different_pairs: usize,
        /// Same-author pairs sampled per bin.
        #[arg(long, default_value_t = 2000)]
        same_pairs: usize,
        /// Sampling seed.
        #[arg(long, default_value_t = handprint_core::reference::calibrate::DEFAULT_SEED)]
        seed: u64,
    },

    /// Profile a document and print its notable dimensions.
    Profile {
        /// The reference to profile against.
        #[arg(short, long)]
        reference: PathBuf,
        /// Text file, or `-` for stdin.
        input: Option<PathBuf>,
        /// Dimensions to show.
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },

    /// Compare two documents.
    Compare {
        /// The reference to measure against.
        #[arg(short, long)]
        reference: PathBuf,
        /// First document.
        a: PathBuf,
        /// Second document.
        b: PathBuf,
        /// Metric override.
        #[arg(long, value_enum)]
        metric: Option<MetricArg>,
        /// Contributions to show.
        #[arg(long, default_value_t = 15)]
        top: usize,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },

    /// Rank candidate documents by distance from a query.
    Rank {
        /// The reference to measure against.
        #[arg(short, long)]
        reference: PathBuf,
        /// The query document.
        query: PathBuf,
        /// Candidate corpus.
        candidates: PathBuf,
        /// Metric override.
        #[arg(long, value_enum)]
        metric: Option<MetricArg>,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },

    /// Run General Imposters verification.
    Verify {
        /// The reference to measure against.
        #[arg(short, long)]
        reference: PathBuf,
        /// The questioned document.
        query: PathBuf,
        /// The candidate author's known writing.
        #[arg(long)]
        target: PathBuf,
        /// A crowd of unrelated writers, one per author.
        #[arg(long)]
        impostors: PathBuf,
        /// Bootstrap iterations.
        #[arg(long, default_value_t = 100)]
        iterations: usize,
        /// Lower grey-zone threshold.
        #[arg(long, default_value_t = 0.35)]
        p1: f64,
        /// Upper grey-zone threshold.
        #[arg(long, default_value_t = 0.65)]
        p2: f64,
        /// Sampling seed.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },

    /// Discover the vocabulary that distinguishes two corpora.
    Contrast {
        /// Side A.
        a: PathBuf,
        /// Side B.
        b: PathBuf,
        /// Background corpus for the informative prior.
        #[arg(long)]
        background: Option<PathBuf>,
        /// Term universe: words, bigrams, trigrams, or `chars:N`.
        #[arg(long, default_value = "words")]
        universe: String,
        /// Restrict the vocabulary to closed-class function words.
        ///
        /// Use this whenever the two corpora were not written about the same
        /// subject: an unrestricted contrast surfaces the topic, not the style.
        #[arg(long)]
        function_words: bool,
        /// Total prior mass. Defaults to `min(n_a, n_b)`, the neutral start.
        #[arg(long)]
        alpha0: Option<f64>,
        /// Use the reduced (Eq. 20) variance, which the Python
        /// `fightin-words` package implements.
        #[arg(long)]
        reduced_variance: bool,
        /// Minimum total occurrences for a term to be considered.
        #[arg(long, default_value_t = 5)]
        min_count: usize,
        /// Terms to show per side.
        #[arg(long, default_value_t = 25)]
        top: usize,
        /// Mark both corpora private, forcing privacy culling.
        #[arg(long)]
        private: bool,
        /// Write the discovered vocabulary as a feature to this path.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Minimum |z| for a term to enter the emitted vocabulary.
        #[arg(long, default_value_t = 2.0)]
        z_threshold: f64,
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },

    /// Explain a document against a reference, optionally as HTML.
    Explain {
        /// The reference to measure against.
        #[arg(short, long)]
        reference: PathBuf,
        /// Text file, or `-` for stdin.
        input: Option<PathBuf>,
        /// Write an HTML report here.
        #[arg(long)]
        html: Option<PathBuf>,
        /// Findings to show.
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Window size, in tokens, for the rolling scan. `0` disables it.
        #[arg(long, default_value_t = 0)]
        window: usize,
    },

    /// Critique a draft. The agent loop's entry point.
    ///
    /// Exit codes follow Vale: 0 pass, 1 findings, 2 error.
    Critique {
        /// The reference to match, or to move toward in contrast mode.
        #[arg(short, long)]
        reference: Option<PathBuf>,
        /// The reference point to move away from, in contrast mode. Must be a
        /// corpus measured against the same reference.
        #[arg(long)]
        away: Option<PathBuf>,
        /// Text file, or `-` for stdin.
        input: Option<PathBuf>,
        /// Which question to ask.
        #[arg(long, value_enum, default_value = "corpus")]
        mode: ModeArg,
        /// Threshold profile.
        #[arg(long, value_enum)]
        profile: Option<ProfileArg>,
        /// Findings to report per iteration.
        #[arg(long)]
        max_findings: Option<usize>,
        /// Drop findings below this severity: low, medium or high.
        #[arg(long)]
        min_severity: Option<String>,
        /// Print a human-readable summary instead of the JSON contract.
        #[arg(long)]
        no_json: bool,
        /// Path to an explicit `handprint.toml`.
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Extract a corpus from local agent transcripts.
    Extract {
        /// Which transcripts to read.
        #[arg(value_enum)]
        source: ExtractSource,
        /// Directory to read. Defaults to the tool's standard location.
        #[arg(long)]
        from: Option<PathBuf>,
        /// Where to write the JSONL records.
        #[arg(short, long)]
        out: PathBuf,
        /// Also extract reasoning traces, as a separate register.
        #[arg(long)]
        include_reasoning: bool,
        /// Reject messages with fewer prose words than this.
        #[arg(long, default_value_t = 12)]
        min_words: usize,
        /// Reject messages that are more than this share code.
        #[arg(long, default_value_t = 0.5)]
        max_code_share: f64,
    },

    /// Hacker News corpus tools.
    Hn {
        #[command(subcommand)]
        command: HnCommand,
    },

    /// Inspect or export packs.
    Pack {
        #[command(subcommand)]
        command: PackCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum HnCommand {
    /// Fetch a user's comments. Requires the `net` feature.
    Fetch {
        /// The username.
        #[arg(long)]
        user: String,
        /// Where to write the JSONL records.
        #[arg(short, long)]
        out: PathBuf,
        /// Drop comments at or after this Unix timestamp. Defaults to
        /// 2022-11-01, the contamination cut-off.
        #[arg(long)]
        before: Option<i64>,
        /// Stop after this many pages.
        #[arg(long, default_value_t = 20)]
        max_pages: usize,
        /// Cache raw responses here so a re-run costs nothing.
        #[arg(long)]
        cache: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PackCommand {
    /// Show a reference's provenance and contents.
    Show {
        /// The reference file.
        path: PathBuf,
    },
    /// Write the built-in AI-style lexicon to a file, as a starting point for a
    /// project-specific pack.
    Export {
        /// Where to write it.
        #[arg(short, long)]
        out: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match commands::dispatch(cli.command) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("handprint: {error:#}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn critique_defaults_to_json_and_corpus_mode() {
        let cli = Cli::try_parse_from(["handprint", "critique", "-r", "ref.json"]).unwrap();
        match cli.command {
            Command::Critique { no_json, mode, .. } => {
                assert!(!no_json, "the JSON contract is the default output");
                assert_eq!(mode, ModeArg::Corpus);
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn fit_takes_a_comma_separated_feature_list() {
        let cli = Cli::try_parse_from([
            "handprint",
            "fit",
            "corpus",
            "-o",
            "ref.json",
            "--features",
            "punct,ngrams,surprisal",
        ])
        .unwrap();
        match cli.command {
            Command::Fit { features, .. } => assert_eq!(
                features,
                vec![FeatureArg::Punct, FeatureArg::Ngrams, FeatureArg::Surprisal]
            ),
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn fit_defaults_to_the_short_text_families() {
        let cli = Cli::try_parse_from(["handprint", "fit", "corpus", "-o", "r.json"]).unwrap();
        match cli.command {
            Command::Fit { features, .. } => assert_eq!(
                features,
                vec![FeatureArg::Punct, FeatureArg::Sentence, FeatureArg::Lexicon]
            ),
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn unknown_subcommands_are_rejected() {
        assert!(Cli::try_parse_from(["handprint", "nope"]).is_err());
    }
}
