//! Subcommand implementations.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use handprint_core::compare::DistanceMetric;
use handprint_core::contrast::{ContrastConfig, ContrastModel, Prior, PrivacyCull, Side, Variance};
use handprint_core::critique::{Critic, Mode};
use handprint_core::feature::lexicon::Severity;
use handprint_core::feature::vocab::Universe;
use handprint_core::feature::{
    BiberTier1, CharNgrams, ComparisonFrames, DeviceRates, LexiconFeature, MostFrequentWords,
    NormDensity, PunctTypography, Readability, RegisterClash, Richness, SentenceStats, SurprisalLm,
};
use handprint_core::reference::calibrate::CalibrationConfig;
use handprint_core::verify::{impostor_pool, Thresholds, VerifyConfig};
use handprint_core::{
    Calibration, Corpus, Document, Metric, Pipeline, Profile, Reference, ThresholdProfile,
};

use crate::config::Config;
use crate::io;
use crate::{Command, ExtractSource, FeatureArg, HnCommand, ModeArg, PackCommand};

/// Route a parsed command to its implementation.
pub fn dispatch(command: Command) -> Result<i32> {
    match command {
        Command::Fit { .. } => fit(command),
        Command::Calibrate { .. } => calibrate(command),
        Command::Profile { .. } => profile(command),
        Command::Compare { .. } => compare(command),
        Command::Rank { .. } => rank(command),
        Command::Verify { .. } => verify(command),
        Command::Contrast { .. } => contrast(command),
        Command::Explain { .. } => explain(command),
        Command::Critique { .. } => critique(command),
        Command::Extract { .. } => extract(command),
        Command::Hn { command } => hn(command),
        Command::Pack { command } => pack(command),
    }
}

fn fit(command: Command) -> Result<i32> {
    let Command::Fit {
        corpus,
        out,
        features,
        pack,
        pack_categories,
        vocab,
        config,
        no_config,
        name,
        version,
        kind,
        metric,
        top_words,
        top_ngrams,
        exemplars,
        private,
        note,
    } = command
    else {
        unreachable!()
    };

    let corpus = io::load_corpus(&corpus, private)?;
    let mut builder = Pipeline::builder()
        .name(&name)
        .version(&version)
        .kind(&kind)
        .metric(metric.into())
        .exemplars(exemplars);
    for note in note {
        builder = builder.note(note);
    }
    if private {
        builder = builder
            .note("fitted from a corpus flagged private; review the vocabulary before publishing");
    }
    for feature in &features {
        builder = match feature {
            FeatureArg::Punct => builder.feature(PunctTypography::default()),
            FeatureArg::Sentence => builder.feature(SentenceStats::default()),
            FeatureArg::Mfw => builder.feature(MostFrequentWords::default().top(top_words)),
            FeatureArg::FunctionWords => {
                builder.feature(MostFrequentWords::default().top(top_words).function_words())
            }
            FeatureArg::Ngrams => builder.feature(CharNgrams::new(3..=4).top(top_ngrams)),
            FeatureArg::Richness => builder.feature(Richness::default()),
            FeatureArg::Lexicon => builder.feature(LexiconFeature::default()),
            FeatureArg::Surprisal => builder.feature(SurprisalLm::default()),
            FeatureArg::Biber => builder.feature(BiberTier1::default()),
            FeatureArg::Readability => builder.feature(Readability::default()),
            FeatureArg::Hyland => builder.feature(builtin_pack_feature("hyland", true)?),
            FeatureArg::DocStyle => builder.feature(builtin_pack_feature("doc-style", false)?),
            FeatureArg::TechVoice => builder.feature(builtin_pack_feature("tech-voice", true)?),
            FeatureArg::SentenceRhythm => {
                builder.feature(SentenceStats::default().with_rhythm().with_md_extended())
            }
            FeatureArg::Punchline => builder.feature(SurprisalLm {
                word_bigrams: true,
                punchline: true,
                ..Default::default()
            }),
            FeatureArg::Frames => {
                let frozen = handprint_core::feature::packs::builtin("cliche-similes")
                    .expect("the frozen-simile pack is bundled");
                builder.feature(ComparisonFrames::default().with_frozen(frozen))
            }
            FeatureArg::Formality => builder.feature(RegisterClash::default()),
            FeatureArg::Devices => builder.feature(DeviceRates::default()),
            FeatureArg::Concreteness => builder.feature(NormDensity::new(
                handprint_core::feature::packs::concreteness_stub(),
            )),
            FeatureArg::Hyperbole => builder.feature(builtin_pack_feature("hyperbole", true)?),
            FeatureArg::Marketing => builder.feature(builtin_pack_feature("marketing-eval", true)?),
            FeatureArg::MarketingSlop => {
                builder.feature(builtin_pack_feature("marketing-slop", false)?)
            }
            FeatureArg::Epistemic => {
                builder.feature(builtin_pack_feature("epistemic-certainty", true)?)
            }
        };
    }

    // Packs and vocabularies come from the flags and from `handprint.toml`.
    // Both are additive: a project file declares what every contributor gets,
    // and a flag adds to it for a one-off fit.
    let project = if no_config {
        None
    } else {
        load_config(config.as_deref())?
    };
    for path in &pack {
        builder = builder.feature(load_pack_feature(path, pack_categories, None)?);
    }
    for path in &vocab {
        builder = builder.feature(load_vocab(path)?);
    }
    if let Some((project_config, root)) = &project {
        for declared in &project_config.packs {
            let feature = match &declared.path {
                Some(path) => load_pack_feature(
                    &Config::resolve(root, path),
                    declared.category_findings,
                    Some(declared),
                )?,
                None => {
                    let feature = builtin_pack_feature(&declared.name, declared.category_findings)?;
                    check_pack_pin(&feature.pack, declared)?;
                    feature
                }
            };
            builder = builder.feature(feature);
        }
        for declared in &project_config.vocabs {
            builder = builder.feature(load_vocab(&Config::resolve(root, &declared.path))?);
        }
    }

    let reference = builder.fit(&corpus).context("fitting the reference")?;
    io::save_reference(&out, &reference)?;

    let provenance = reference.provenance();
    println!(
        "fitted {} from {} document(s) by {} author(s), {} tokens",
        reference.fingerprint(),
        provenance.documents,
        provenance.authors,
        provenance.tokens
    );
    println!(
        "{} dimension(s) across {} families; {} exemplar profile(s) stored",
        reference.dims().len(),
        reference.families().len(),
        reference.exemplars().len()
    );
    if !provenance.licenses.is_empty() {
        println!("\nembedded data packs:");
        for license in &provenance.licenses {
            println!("  {license}");
        }
    }
    println!("\nwrote {}", out.display());
    if provenance.has_non_redistributable_pack() {
        eprintln!(
            "warning: this reference embeds a pack marked non-redistributable, so the reference \
             inherits that restriction. Keep it local; do not publish it as a shipped pack."
        );
    }
    println!(
        "note: this reference has no calibration, so `critique` cannot evaluate its pass gate. \
         Run `handprint calibrate` against a background corpus."
    );
    Ok(0)
}

/// Load the project config, either from an explicit path or by discovery.
fn load_config(explicit: Option<&Path>) -> Result<Option<(Config, PathBuf)>> {
    match explicit {
        Some(path) => Ok(Some((
            Config::load(path)?,
            path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        ))),
        None => Config::discover(Path::new(".")),
    }
}

/// Build a lexicon feature from a bundled pack.
fn builtin_pack_feature(name: &str, category_findings: bool) -> Result<LexiconFeature> {
    let pack = handprint_core::feature::packs::builtin(name).ok_or_else(|| {
        anyhow::anyhow!(
            "no bundled pack named {name:?}; `handprint pack list` shows what this binary carries"
        )
    })?;
    let feature = LexiconFeature::new(pack);
    Ok(if category_findings {
        feature.category_findings()
    } else {
        feature
    })
}

/// Load a lexicon pack from disk and wrap it as a feature.
fn load_pack_feature(
    path: &Path,
    category_findings: bool,
    declared: Option<&crate::config::Pack>,
) -> Result<LexiconFeature> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading pack {}", path.display()))?;
    let pack: handprint_core::feature::LexiconPack =
        serde_json::from_str(&text).with_context(|| format!("parsing pack {}", path.display()))?;
    // Compile the patterns here rather than at fit: a pack is a file someone
    // edited, and a pattern that silently never matches is indistinguishable
    // from a genuine zero rate.
    pack.validate_patterns()
        .with_context(|| format!("in pack {}", path.display()))?;
    if let Some(declared) = declared {
        check_pack_pin(&pack, declared)?;
    }
    let feature = LexiconFeature::new(pack);
    Ok(if category_findings {
        feature.category_findings()
    } else {
        feature
    })
}

/// Reject a pack whose identity does not match what the project pinned.
///
/// A pack is a dated artifact. Silently accepting `hyland@2027.03` where the
/// project asked for `hyland@2026.08` would change what the project considers
/// normal without anyone noticing — which is the exact failure the pin exists
/// to prevent.
fn check_pack_pin(
    pack: &handprint_core::feature::LexiconPack,
    declared: &crate::config::Pack,
) -> Result<()> {
    if pack.name != declared.name {
        bail!(
            "pack declares name {:?} but handprint.toml pinned {:?}",
            pack.name,
            declared.name
        );
    }
    if let Some(version) = &declared.version {
        if &pack.version != version {
            bail!(
                "pack {} is version {:?} but handprint.toml pinned {:?}; packs are dated \
                 artifacts, so update the pin deliberately",
                pack.name,
                pack.version,
                version
            );
        }
    }
    Ok(())
}

/// Load a contrast vocabulary produced by `handprint contrast --out`.
fn load_vocab(path: &Path) -> Result<handprint_core::feature::ContrastVocab> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading vocabulary {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing vocabulary {}", path.display()))
}

fn calibrate(command: Command) -> Result<i32> {
    let Command::Calibrate {
        reference,
        background,
        out,
        metric,
        bins,
        different_pairs,
        same_pairs,
        seed,
    } = command
    else {
        unreachable!()
    };

    let mut model = io::load_reference(&reference)?;
    let background_corpus = io::load_corpus(&background, false)?;
    let metric: Metric = metric.map(Into::into).unwrap_or_else(|| model.metric());

    let config = CalibrationConfig {
        bins,
        different_pairs,
        same_pairs,
        seed,
        ..Default::default()
    };
    let calibration = Calibration::fit_with_metric(&model, &background_corpus, &config, metric)
        .context("fitting calibration")?;

    println!(
        "calibrated {} for {} against {} background author(s)",
        model.fingerprint(),
        metric.name(),
        calibration.authors()
    );
    for bin in calibration.bins() {
        println!(
            "  {:>6} tokens: {:>5} unrelated pair(s), {:>5} same-author pair(s)",
            bin.tokens,
            bin.different.len(),
            bin.same.len()
        );
    }
    model.set_calibration(calibration);

    let out = out.unwrap_or(reference);
    io::save_reference(&out, &model)?;
    println!("wrote {}", out.display());
    Ok(0)
}

fn profile(command: Command) -> Result<i32> {
    let Command::Profile {
        reference,
        input,
        top,
        json,
    } = command
    else {
        unreachable!()
    };

    let reference = io::load_reference(&reference)?;
    let text = io::read_text(input.as_deref())?;
    let profile = reference.profile(&Document::new(text));

    if json {
        println!("{}", serde_json::to_string_pretty(&profile)?);
        return Ok(0);
    }

    println!("{} lexical tokens", profile.tokens());
    for warning in profile.warnings(&reference) {
        println!("  ! {warning}");
    }
    println!("\nconfidence by family:");
    for (family, confidence) in profile.confidence(&reference) {
        println!("  {:<12} {:?}", family.as_str(), confidence);
    }

    // Rank by how unusual each dimension is against the corpus.
    let mut ranked: Vec<(f64, usize)> = reference
        .stats()
        .iter()
        .enumerate()
        .map(|(i, stats)| (stats.z(profile.raw()[i]).abs(), i))
        .collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nmost unusual dimensions:");
    println!(
        "{:<36} {:>10} {:>10} {:>8}",
        "dimension", "observed", "corpus", "z"
    );
    for (z, index) in ranked.into_iter().take(top) {
        let dim = &reference.dims()[index];
        let stats = &reference.stats()[index];
        println!(
            "{:<36} {:>10.4} {:>10.4} {:>+8.2}",
            reference.name_of(dim.symbol),
            profile.raw()[index],
            stats.mean,
            if profile.raw()[index] < stats.mean {
                -z
            } else {
                z
            }
        );
    }
    Ok(0)
}

fn compare(command: Command) -> Result<i32> {
    let Command::Compare {
        reference,
        a,
        b,
        metric,
        top,
        json,
    } = command
    else {
        unreachable!()
    };

    let reference = io::load_reference(&reference)?;
    let pa = reference.profile(&Document::new(io::read_text(Some(&a))?));
    let pb = reference.profile(&Document::new(io::read_text(Some(&b))?));
    let metric: Metric = metric.map(Into::into).unwrap_or_else(|| reference.metric());
    let comparison = reference.compare_with(&pa, &pb, metric)?;
    let contributions = comparison.contributions();

    if json {
        let value = serde_json::json!({
            "metric": metric.name(),
            "distance": comparison.distance,
            "offset": comparison.offset(),
            "tokens": [comparison.tokens.0, comparison.tokens.1],
            "assessment": comparison.assessment,
            "contributions": contributions.iter().take(top).collect::<Vec<_>>(),
            "by_family": comparison.by_family().iter()
                .map(|(f, v)| serde_json::json!({"family": f.as_str(), "value": v}))
                .collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(0);
    }

    println!(
        "{} distance {:.6}  ({} vs {} tokens)",
        metric.name(),
        comparison.distance,
        comparison.tokens.0,
        comparison.tokens.1
    );
    match &comparison.assessment {
        Some(a) => {
            println!(
                "  p_value_vs_unrelated {:.4}   p_same_author {:.4}   likelihood ratio {:.2}{}",
                a.p_value_vs_unrelated,
                a.p_same_author,
                a.likelihood_ratio,
                if a.extrapolated {
                    "  (extrapolated beyond the calibrated length range)"
                } else {
                    ""
                }
            );
            println!(
                "  p_value_vs_unrelated is P(distance <= d | different authors). It is NOT the \
                 probability the authors differ."
            );
        }
        None => println!("  {}", uncalibrated_note(&reference, metric)),
    }

    println!("\nby family:");
    for (family, value) in comparison.by_family() {
        println!("  {:<12} {:+.6}", family.as_str(), value);
    }

    println!("\ntop contributions:");
    println!(
        "{:<36} {:>10} {:>10} {:>10}",
        "dimension", "a", "b", "share"
    );
    for c in contributions.iter().take(top) {
        println!(
            "{:<36} {:>10.4} {:>10.4} {:>+10.6}",
            c.name, c.observed_a, c.observed_b, c.value
        );
    }
    Ok(0)
}

/// Explain why a comparison came back without a calibrated assessment.
///
/// The commonest cause is not a missing calibration but one fitted for a
/// different metric, which is easy to hit because `fit` and `calibrate` take
/// the metric separately.
fn uncalibrated_note(reference: &Reference, metric: Metric) -> String {
    match reference.calibration() {
        Some(existing) => format!(
            "uncalibrated for {}: this reference is calibrated for {}. Pass --metric {} or \
             re-run `handprint calibrate --metric {}`.",
            metric.name(),
            existing.metric().name(),
            existing.metric().name(),
            metric.name()
        ),
        None => format!(
            "uncalibrated: run `handprint calibrate --metric {} --background <corpus>` to turn \
             this distance into evidence",
            metric.name()
        ),
    }
}

fn rank(command: Command) -> Result<i32> {
    let Command::Rank {
        reference,
        query,
        candidates,
        metric,
        json,
    } = command
    else {
        unreachable!()
    };

    let reference = io::load_reference(&reference)?;
    let metric: Metric = metric.map(Into::into).unwrap_or_else(|| reference.metric());
    let query_profile = reference.profile(&Document::new(io::read_text(Some(&query))?));
    let corpus = io::load_corpus(&candidates, false)?;

    // One profile per author, aggregating their documents: individual documents
    // are usually far below every length floor.
    let mut rows: Vec<(String, f64, Option<handprint_core::Assessment>)> = Vec::new();
    for group in corpus.authors() {
        let candidate = reference.profile_aggregate(&group.docs);
        let comparison = reference.compare_with(&query_profile, &candidate, metric)?;
        rows.push((
            group.author.to_string(),
            comparison.distance,
            comparison.assessment,
        ));
    }
    rows.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if json {
        let value: Vec<_> = rows
            .iter()
            .map(|(label, distance, assessment)| {
                serde_json::json!({
                    "label": label,
                    "distance": distance,
                    "assessment": assessment,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(0);
    }

    if rows.iter().all(|(_, _, a)| a.is_none()) {
        println!("{}", uncalibrated_note(&reference, metric));
    }
    println!(
        "{:<28} {:>12} {:>14} {:>14}",
        "candidate", "distance", "p_vs_unrel", "p_same_author"
    );
    for (label, distance, assessment) in &rows {
        match assessment {
            Some(a) => println!(
                "{label:<28} {distance:>12.6} {:>14.4} {:>14.4}",
                a.p_value_vs_unrelated, a.p_same_author
            ),
            None => println!("{label:<28} {distance:>12.6} {:>14} {:>14}", "-", "-"),
        }
    }
    println!(
        "\nRanking is not identification: the closest candidate is closest among those supplied, \
         which says nothing about whether the true author is in the list at all."
    );
    Ok(0)
}

fn verify(command: Command) -> Result<i32> {
    let Command::Verify {
        reference,
        query,
        target,
        impostors,
        iterations,
        p1,
        p2,
        seed,
        json,
    } = command
    else {
        unreachable!()
    };

    let reference = io::load_reference(&reference)?;
    let query_profile = reference.profile(&Document::new(io::read_text(Some(&query))?));
    let target_corpus = io::load_corpus(&target, false)?;
    let impostor_corpus = io::load_corpus(&impostors, false)?;

    let targets: Vec<Profile> = target_corpus
        .authors()
        .iter()
        .map(|a| reference.profile_aggregate(&a.docs))
        .collect();
    let pool = impostor_pool(&reference, &impostor_corpus, &[]);

    let config = VerifyConfig {
        iterations,
        thresholds: Thresholds::new(p1, p2)?,
        seed,
        ..Default::default()
    };
    let score =
        handprint_core::verify::verify(&reference, &query_profile, &targets, &pool, &config)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&score)?);
    } else {
        println!("{}", score.describe());
        println!(
            "  {} target profile(s), {} dimension(s) sampled per iteration, metric {}",
            score.targets,
            score.features_per_iteration,
            score.metric.name()
        );
        println!(
            "\nA favoured verdict means \"consistent with common authorship\", not \"same author\". \
             The dominant failure mode is topic confound: if the impostor pool writes about \
             different subjects than the query, the target wins for the wrong reason."
        );
    }
    Ok(0)
}

fn contrast(command: Command) -> Result<i32> {
    let Command::Contrast {
        a,
        b,
        background,
        universe,
        function_words,
        alpha0,
        reduced_variance,
        min_count,
        top,
        private,
        out,
        z_threshold,
        json,
    } = command
    else {
        unreachable!()
    };

    let corpus_a = io::load_corpus(&a, private)?;
    let corpus_b = io::load_corpus(&b, private)?;
    let background_corpus = match &background {
        Some(path) => Some(io::load_corpus(path, false)?),
        None => None,
    };
    let universe = parse_universe(&universe)?;

    let tokenizer = handprint_core::Tokenizer::default();
    // alpha0 is a pseudo-sample size in tokens, calibrated against the corpora
    // being compared - never against the background.
    let n_a = count_terms(&corpus_a, &tokenizer, universe);
    let n_b = count_terms(&corpus_b, &tokenizer, universe);
    let prior = match alpha0 {
        Some(alpha0) => Prior::Background { alpha0 },
        None => Prior::neutral(n_a, n_b),
    };

    let mut config = ContrastConfig::default()
        .universe(universe)
        .prior(prior)
        .min_count(min_count)
        .variance(if reduced_variance {
            Variance::Reduced
        } else {
            Variance::Full
        });
    if private {
        config = config.privacy(PrivacyCull::default());
    }
    if function_words {
        config = config.function_words();
    }

    let model = ContrastModel::fit(
        "contrast",
        &corpus_a,
        &corpus_b,
        background_corpus.as_ref(),
        &tokenizer,
        &config,
    )?;

    if json {
        println!("{}", serde_json::to_string_pretty(&model)?);
    } else {
        println!("{}", model.describe());
        println!("\ncharacteristic of A ({}):", a.display());
        print_terms(&model.top(top, Side::A));
        println!("\ncharacteristic of B ({}):", b.display());
        print_terms(&model.top(top, Side::B));
        println!(
            "\nz-scores are comparable only within this term universe ({}). Never rank terms \
             from different universes against each other.",
            universe.as_str()
        );
        if !function_words {
            println!(
                "If these corpora are about different subjects, much of what you see above is \
                 topic rather than style. Re-run with --function-words for the topic-robust view."
            );
        }
    }

    if let Some(path) = out {
        let vocab = model.into_feature(z_threshold)?;
        let json = serde_json::to_string_pretty(&vocab)?;
        std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
        println!(
            "\nwrote {} term(s) above |z| >= {z_threshold} to {}",
            vocab.terms.len(),
            path.display()
        );
    }
    Ok(0)
}

fn print_terms(terms: &[&handprint_core::contrast::TermStats]) {
    println!(
        "{:<28} {:>8} {:>10} {:>8} {:>8}",
        "term", "z", "delta", "count_a", "count_b"
    );
    for term in terms {
        println!(
            "{:<28} {:>+8.2} {:>+10.4} {:>8} {:>8}",
            term.text, term.z, term.delta, term.count_a, term.count_b
        );
    }
}

fn count_terms(
    corpus: &Corpus,
    tokenizer: &handprint_core::Tokenizer,
    universe: Universe,
) -> usize {
    corpus
        .documents()
        .map(|d| universe.extract(&d.analyze(tokenizer)).len())
        .sum()
}

fn parse_universe(spec: &str) -> Result<Universe> {
    Ok(match spec {
        "words" | "w1" => Universe::Words,
        "bigrams" | "w2" => Universe::WordBigrams,
        "trigrams" | "w3" => Universe::WordTrigrams,
        other => match other.strip_prefix("chars:") {
            Some(n) => Universe::CharNgrams {
                n: n.parse().context("character n-gram order")?,
            },
            None => bail!(
                "unknown term universe {other:?}; expected words, bigrams, trigrams or chars:N"
            ),
        },
    })
}

fn explain(command: Command) -> Result<i32> {
    let Command::Explain {
        reference,
        input,
        html,
        top,
        window,
    } = command
    else {
        unreachable!()
    };

    let reference = io::load_reference(&reference)?;
    let text = io::read_text(input.as_deref())?;
    let profile = reference.profile_tracked(&Document::new(text.clone()));

    let target = reference.exemplars().first().cloned().ok_or_else(|| {
        anyhow::anyhow!("this reference stores no exemplar profiles; refit with --exemplars N")
    })?;
    let comparison = reference.compare_with(&profile, &target, reference.metric())?;
    let contributions = comparison.contributions();

    println!(
        "{} distance {:.6} against a representative corpus document",
        reference.metric().name(),
        comparison.distance
    );
    for warning in profile.warnings(&reference) {
        println!("  ! {warning}");
    }
    println!("\ntop contributions:");
    for c in contributions.iter().take(top) {
        let spans = profile.spans(c.symbol);
        let excerpt = spans
            .first()
            .and_then(|s| s.slice(&text))
            .map(|s| format!("   e.g. {s:?}"))
            .unwrap_or_default();
        println!("  {:<34} {:>+10.6}{excerpt}", c.name, c.value);
    }

    if window > 0 {
        println!("\nrolling scan (window {window} tokens):");
        let windows = handprint_core::explain::rolling(
            &reference,
            &text,
            &target,
            window,
            window / 2,
            reference.metric(),
        )?;
        for (span, distance) in windows.iter().take(top) {
            let excerpt: String = span
                .slice(&text)
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>()
                .replace('\n', " ");
            println!("  {distance:.4}  {excerpt}...");
        }
    }

    if let Some(path) = html {
        let weights: Vec<(handprint_core::Symbol, f64)> = contributions
            .iter()
            .take(top)
            .map(|c| (c.symbol, c.value))
            .collect();
        let highlights = handprint_core::explain::highlights(&reference, &profile, &weights, 200);
        std::fs::write(&path, render_html(&text, &highlights))
            .with_context(|| format!("writing {}", path.display()))?;
        println!("\nwrote {}", path.display());
    }
    Ok(0)
}

/// Minimal self-contained HTML report.
///
/// The core crate's renderer is behind a feature flag; the CLI always has one
/// so that `explain --html` never depends on how the library was built.
fn render_html(text: &str, highlights: &[handprint_core::Highlight]) -> String {
    let mut sorted: Vec<&handprint_core::Highlight> = highlights.iter().collect();
    sorted.sort_by_key(|h| h.span.start);

    let mut body = String::new();
    let mut cursor = 0usize;
    for h in sorted {
        if h.span.start < cursor || h.span.end > text.len() {
            continue;
        }
        escape_into(&text[cursor..h.span.start], &mut body);
        body.push_str(&format!(
            "<mark title=\"{} ({:+.4})\">",
            escape(&h.name),
            h.weight
        ));
        escape_into(&text[h.span.range()], &mut body);
        body.push_str("</mark>");
        cursor = h.span.end;
    }
    escape_into(&text[cursor..], &mut body);

    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\">\n\
         <title>handprint explain</title>\n\
         <style>body{{font:16px/1.6 system-ui,sans-serif;max-width:44rem;margin:3rem auto;\
         padding:0 1rem}}pre{{white-space:pre-wrap;word-wrap:break-word}}\
         mark{{background:#ffe58f;padding:.05em .1em;border-radius:.2em}}</style>\n\
         </head><body><h1>handprint explain</h1><pre>{body}</pre></body></html>\n"
    )
}

fn escape(s: &str) -> String {
    let mut out = String::new();
    escape_into(s, &mut out);
    out
}

fn escape_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
}

fn critique(command: Command) -> Result<i32> {
    let Command::Critique {
        reference,
        away,
        input,
        mode,
        profile,
        max_findings,
        min_severity,
        no_json,
        config,
    } = command
    else {
        unreachable!()
    };

    // Project configuration fills in whatever the flags did not.
    let discovered = match &config {
        Some(path) => Some((
            Config::load(path)?,
            path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        )),
        None => Config::discover(Path::new("."))?,
    };

    let reference_path = reference
        .or_else(|| {
            discovered.as_ref().and_then(|(c, root)| {
                c.project
                    .reference
                    .as_ref()
                    .map(|p| Config::resolve(root, p))
            })
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no reference given; pass --reference or set `reference` in handprint.toml"
            )
        })?;
    let reference = io::load_reference(&reference_path)?;

    let profile_arg = profile
        .map(Into::into)
        .or_else(|| {
            discovered
                .as_ref()
                .and_then(|(c, _)| c.project.profile.as_deref())
                .and_then(parse_threshold_profile)
        })
        .unwrap_or(ThresholdProfile::Balanced);

    let mut critique_config = profile_arg.config();
    critique_config.mode = match mode {
        ModeArg::Corpus => Mode::CorpusMimic,
        ModeArg::Contrast => Mode::Contrast,
    };
    if critique_config.mode == Mode::CorpusMimic {
        // Magnitude, not angle: one-class membership needs Burrows.
        critique_config.metric = Some(Metric::BurrowsDelta);
    }
    if let Some(n) = max_findings.or_else(|| {
        discovered
            .as_ref()
            .and_then(|(c, _)| c.project.max_findings)
    }) {
        critique_config.max_findings = n;
    }
    let severity = min_severity.or_else(|| {
        discovered
            .as_ref()
            .and_then(|(c, _)| c.project.min_severity.clone())
    });
    if let Some(s) = severity {
        critique_config.min_severity = parse_severity(&s)?;
    }

    let text = io::read_text(input.as_deref())?;

    let away_path = away.or_else(|| {
        discovered
            .as_ref()
            .and_then(|(c, root)| c.project.away.as_ref().map(|p| Config::resolve(root, p)))
    });

    let mut critic = match critique_config.mode {
        Mode::CorpusMimic => {
            let targets = reference.exemplars().to_vec();
            if targets.is_empty() {
                bail!(
                    "this reference stores no exemplar profiles, so there is nothing to match \
                     against; refit it with --exemplars N"
                );
            }
            Critic::with_config(&reference, targets, None, critique_config)
        }
        Mode::Contrast => {
            let path = away_path.ok_or_else(|| {
                anyhow::anyhow!("contrast mode needs --away pointing at a corpus to move away from")
            })?;
            let away_corpus = io::load_corpus(&path, false)?;
            let away_profile =
                reference.profile_aggregate(&away_corpus.documents().cloned().collect::<Vec<_>>());
            let toward = reference
                .exemplars()
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("this reference stores no exemplar profiles"))?;
            Critic::with_config(
                &reference,
                vec![toward],
                Some(away_profile),
                critique_config,
            )
        }
    };

    let report = critic.review(&text)?;
    if no_json {
        println!("{}", report.summary());
    } else {
        println!("{}", report.to_json());
    }
    Ok(report.exit_code())
}

fn parse_threshold_profile(name: &str) -> Option<ThresholdProfile> {
    match name {
        "lenient" => Some(ThresholdProfile::Lenient),
        "balanced" => Some(ThresholdProfile::Balanced),
        "strict" => Some(ThresholdProfile::Strict),
        _ => None,
    }
}

fn parse_severity(name: &str) -> Result<Severity> {
    Ok(match name {
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        other => bail!("unknown severity {other:?}; expected low, medium or high"),
    })
}

fn extract(command: Command) -> Result<i32> {
    let Command::Extract {
        source,
        from,
        out,
        include_reasoning,
        min_words,
        max_code_share,
    } = command
    else {
        unreachable!()
    };

    let config = handprint_data::AgentConfig {
        include_reasoning,
        strip: handprint_data::StripConfig {
            min_words,
            max_code_share,
            ..Default::default()
        },
    };

    let roots: Vec<PathBuf> = match (&from, source) {
        (Some(path), _) => vec![path.clone()],
        (None, ExtractSource::ClaudeCode) => handprint_data::agent::claude_code_root()
            .into_iter()
            .collect(),
        (None, ExtractSource::Codex) => handprint_data::agent::codex_roots(),
    };
    if roots.is_empty() {
        bail!("could not determine a default transcript directory; pass --from");
    }

    let mut extraction = handprint_data::Extraction::default();
    for root in &roots {
        if !root.exists() {
            eprintln!("warning: {} does not exist, skipping", root.display());
            continue;
        }
        let found = match source {
            ExtractSource::ClaudeCode => handprint_data::agent::extract_claude_code(root, &config)?,
            ExtractSource::Codex => handprint_data::agent::extract_codex(root, &config)?,
        };
        extraction.merge(found);
    }

    if extraction.records.is_empty() {
        bail!("no records extracted from {roots:?}");
    }
    handprint_data::jsonl::write(&out, &extraction.records)?;
    println!("{}", extraction.summary());

    let mut by_model: std::collections::BTreeMap<&str, usize> = Default::default();
    for record in &extraction.records {
        *by_model.entry(record.group()).or_insert(0) += record.word_count();
    }
    println!("\nwords by model:");
    for (model, words) in by_model {
        println!("  {model:<28} {words:>10}");
    }
    println!("\nwrote {}", out.display());
    println!(
        "These transcripts are private data. Any vocabulary derived from them must go through \
         privacy culling (`handprint contrast --private`) and a manual review before it leaves \
         this machine."
    );
    Ok(0)
}

fn hn(command: HnCommand) -> Result<i32> {
    let HnCommand::Fetch {
        user,
        out,
        before,
        max_pages,
        cache,
    } = command;

    #[cfg(feature = "net")]
    {
        let config = handprint_data::hn::FetchConfig {
            max_pages,
            cutoff: Some(before.unwrap_or(handprint_data::hn::DEFAULT_CUTOFF)),
            cache,
            ..Default::default()
        };
        let client = handprint_data::hn::Client::new(config);
        let records = client.user_comments(&user)?;
        if records.is_empty() {
            bail!("no comments found for {user} before the cut-off");
        }
        handprint_data::jsonl::write(&out, &records)?;
        let words: usize = records.iter().map(handprint_data::Record::word_count).sum();
        println!(
            "{} comment(s), {words} words -> {}",
            records.len(),
            out.display()
        );
        println!(
            "Comments at or after the cut-off were dropped: post-2022 forum text is \
             contaminated with LLM-assisted writing, which is exactly what a human reference \
             must not contain."
        );
        Ok(0)
    }

    #[cfg(not(feature = "net"))]
    {
        let _ = (user, out, before, max_pages, cache);
        bail!(
            "network support is not compiled in; rebuild with `--features net` to fetch from \
             Hacker News"
        )
    }
}

fn pack(command: PackCommand) -> Result<i32> {
    match command {
        PackCommand::Show { path } => {
            let reference: Reference = io::load_reference(&path)?;
            let provenance = reference.provenance();
            println!("{}", reference.fingerprint());
            println!("  name       {}", provenance.name);
            println!("  version    {}", provenance.version);
            println!("  kind       {}", provenance.kind);
            println!("  handprint  {}", provenance.handprint);
            println!(
                "  corpus     {} document(s), {} author(s), {} tokens{}",
                provenance.documents,
                provenance.authors,
                provenance.tokens,
                if provenance.private { " (private)" } else { "" }
            );
            println!("  metric     {}", reference.metric().name());
            println!("  dimensions {}", reference.dims().len());
            println!("  exemplars  {}", reference.exemplars().len());
            match reference.calibration() {
                Some(cal) => println!(
                    "  calibrated for {} over {} bin(s), {} background author(s)",
                    cal.metric().name(),
                    cal.bins().len(),
                    cal.authors()
                ),
                None => println!("  calibrated no"),
            }
            println!("\n  families:");
            for family in reference.families() {
                let count = reference
                    .dims()
                    .iter()
                    .filter(|d| d.family == family)
                    .count();
                let (low, ok) = family.token_floors();
                println!(
                    "    {:<12} {:>6} dim(s)   usable from {low} tokens, supported from {ok}",
                    family.as_str(),
                    count
                );
            }
            if !provenance.licenses.is_empty() {
                println!("\n  embedded data packs:");
                for license in &provenance.licenses {
                    println!("    - {license}");
                }
                if provenance.has_non_redistributable_pack() {
                    println!(
                        "    ! this reference embeds a non-redistributable pack and inherits \
                         that restriction"
                    );
                }
            }
            if !provenance.notes.is_empty() {
                println!("\n  notes:");
                for note in &provenance.notes {
                    println!("    - {note}");
                }
            }
            Ok(0)
        }
        PackCommand::Export { pack: name, out } => {
            let pack = handprint_core::feature::packs::builtin(&name).ok_or_else(|| {
                anyhow::anyhow!(
                    "no bundled pack named {name:?}; `handprint pack list` shows what this \
                     binary carries"
                )
            })?;
            let json = serde_json::to_string_pretty(&pack)?;
            std::fs::write(&out, json).with_context(|| format!("writing {}", out.display()))?;
            println!(
                "wrote {} ({} term(s), {} phrase(s)) to {}",
                pack.qualified_name(),
                pack.terms.len(),
                pack.phrases.len(),
                out.display()
            );
            println!(
                "This is a dated list, not a fixture. Vocabularies decay — per model and per \
                 year for AI-isms, per register for everything else. Refit against a current \
                 corpus before relying on it."
            );
            Ok(0)
        }
        PackCommand::List => {
            println!("{:<22} {:>7} {:>8}  license", "pack", "terms", "phrases");
            for name in handprint_core::feature::packs::BUILTIN_PACKS {
                let pack = handprint_core::feature::packs::builtin(name)
                    .expect("every listed pack resolves");
                println!(
                    "{:<22} {:>7} {:>8}  {}",
                    pack.qualified_name(),
                    pack.terms.len(),
                    pack.phrases.len(),
                    pack.license
                );
            }
            println!(
                "\nExport one with `handprint pack export --pack <name> -o <file>`, then pin it \
                 in handprint.toml. A pack in a file is a pack your project controls."
            );
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universe_specs_parse() {
        assert_eq!(parse_universe("words").unwrap(), Universe::Words);
        assert_eq!(parse_universe("w2").unwrap(), Universe::WordBigrams);
        assert_eq!(parse_universe("trigrams").unwrap(), Universe::WordTrigrams);
        assert_eq!(
            parse_universe("chars:4").unwrap(),
            Universe::CharNgrams { n: 4 }
        );
        assert!(parse_universe("nonsense").is_err());
        assert!(parse_universe("chars:x").is_err());
    }

    #[test]
    fn severity_and_profile_names_parse() {
        assert_eq!(parse_severity("high").unwrap(), Severity::High);
        assert!(parse_severity("critical").is_err());
        assert!(matches!(
            parse_threshold_profile("strict"),
            Some(ThresholdProfile::Strict)
        ));
        assert!(parse_threshold_profile("nope").is_none());
    }

    #[test]
    fn html_rendering_escapes_and_marks() {
        let text = "a <b> delve here";
        let highlights = [handprint_core::Highlight {
            span: handprint_core::Span::new(6, 11),
            name: "lex:ai-slop:word.delve".into(),
            weight: 1.5,
        }];
        let html = render_html(text, &highlights);
        assert!(html.contains("&lt;b&gt;"));
        assert!(html.contains("<mark title=\"lex:ai-slop:word.delve (+1.5000)\">delve</mark>"));
        assert!(html.starts_with("<!doctype html>"));
    }
}
