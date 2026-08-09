# Preparing a Project Gutenberg Corpus

## Overview

The `handprint gutenberg` command provides a streamlined workflow for
transforming raw Project Gutenberg text files into a structured corpus suitable
for stylometric analysis. This guide walks you through the entire process, from
downloading source texts to verifying the output.

## Prerequisites

Before you begin, ensure that you have the following:

- A working installation of the handprint CLI
- A directory of raw Project Gutenberg text files, organized by author
- Sufficient disk space for the generated corpus

## Understanding the Pipeline

It's important to understand that the corpus preparation pipeline consists of
several distinct stages, each of which addresses a specific challenge:

1. **Boilerplate removal.** Project Gutenberg files are wrapped in licensing
   text that would otherwise be learned as part of the author's style.
2. **Editorial stripping.** Transcriber's notes, publication slugs, and plate
   indexes are removed, as these represent the work of the etext preparer rather
   than the original author.
3. **Chapterization.** The text is split into per-chapter documents, which is
   essential for measuring within-author variance.

Each of these stages plays a crucial role in ensuring that the resulting corpus
accurately reflects the author's voice.

## Directory Structure

The command expects your raw files to be organized as follows:

It's worth noting that the directory name determines the author attribution, so
it's important to ensure that these are correct before proceeding.

## Running the Command

To prepare your corpus, simply run:

This will process every file in every author directory and write the results to
your output directory. The `--jsonl` flag is optional but recommended, as it
produces a machine-readable record of the entire corpus.

## Interpreting the Output

Upon completion, the command reports several important pieces of information:

- The number of books processed and chapters produced
- Any files where boilerplate markers could not be found
- A detailed list of editorial blocks that were removed

It's essential to review these warnings carefully. A file with no recognized
markers still contains its licensing boilerplate, which will be learned as style
and reported as the author's voice.

## Best Practices

To get the most out of the corpus preparation pipeline, consider the following
recommendations:

- **Always review the warnings.** Silent failures are the most dangerous kind.
- **Verify document lengths.** Documents that are too short will produce
  unreliable per-document vectors.
- **Check for duplicates.** Some sources include the same content multiple
  times, which can skew your results.

## Troubleshooting

If you encounter issues, here are some common problems and their solutions:

**Problem: No markers found.** This typically indicates that the file uses a
boilerplate variant that the parser does not recognize. You may need to remove
the header and footer manually.

**Problem: One enormous document.** If the chapterizer cannot find chapter
markers, it will emit the entire text as a single document. Running the output
through `chapterize.py` with a target word count resolves this.

## Conclusion

By following the steps outlined in this guide, you can reliably transform raw
Project Gutenberg texts into a clean, structured corpus. Remember that the
quality of your corpus directly impacts the quality of your analysis, so it's
always worth taking the time to verify your results carefully.
