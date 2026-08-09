#!/usr/bin/env python3
"""Apply a target corpus's quotation typography to a draft, and nothing else.

The punctuation family reads quote marks as style, and it is right to: they are
a real, measurable habit. But they are the *publisher's* habit, not the
author's, and they are free to match — one pass of search-and-replace moves
`punct.quote_double_straight_rate` from a z of +8 to zero without a single word
changing.

So a transfer scored against a draft that did not match them is measuring two
different things at once. This script produces the third arm: same words as the
plain retelling, target typography. Whatever separates *that* from the styled
draft is the part that required actually writing.

    python3 typography.py plain.md > plain-typo.md
"""

import re
import sys

text = open(sys.argv[1], encoding="utf-8").read()

# Dialogue: British single curly quotes, the convention throughout the
# Pratchett corpus. Applied to double quotes only, so apostrophes are untouched.
out, opening = [], True
for character in text:
    if character == '"':
        out.append("‘" if opening else "’")
        opening = not opening
    else:
        out.append(character)
text = "".join(out)

# Apostrophes: curly. Done after the quote pass so it cannot consume one.
text = re.sub(r"(?<=\w)'(?=\w)", "’", text)
text = text.replace("'", "’")

# Spaced em dash to spaced en dash, which is what the corpus uses.
text = text.replace(" — ", " – ")

sys.stdout.write(text)
