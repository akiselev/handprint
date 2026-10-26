#!/usr/bin/env python3
"""Render one experiment document (markdown) to an artifact page.

usage: render.py DOC_ID SOURCE_MD OUT_HTML META_JSON
META_JSON: {"project": "cadatomic", "kind": "Tutorial", "provenance": [..lines..],
            "report_url": "...", "images": {"placeholder": "path.png"}}
"""
import base64, html, json, re, sys
from pathlib import Path
import markdown

doc_id, src, out, meta_path = sys.argv[1:5]
meta = json.loads(Path(meta_path).read_text())
text = Path(src).read_text()

# Title from the first H1; the page header shows it, so drop it from the body.
m = re.search(r"^# (.+)$", text, re.M)
title = m.group(1).strip() if m else doc_id
if m:
    text = text[:m.start()] + text[m.end():]

body = markdown.markdown(text, extensions=["tables", "fenced_code", "sane_lists", "toc"],
                         extension_configs={"toc": {"permalink": False}})
body = body.replace("<table>", '<div class="tablewrap"><table>').replace("</table>", "</table></div>")

# Optional figure inserted after a marker paragraph.
for marker, img in (meta.get("images") or {}).items():
    data = base64.b64encode(Path(img["path"]).read_bytes()).decode()
    fig = (f'<figure><img src="data:image/png;base64,{data}" alt="{html.escape(img["alt"])}">'
           f'<figcaption>{html.escape(img["caption"])}</figcaption></figure>')
    if marker.startswith("afterhead:"):
        h = html.escape(marker[len("afterhead:"):], quote=False)
        mm = re.search(r">" + re.escape(h) + r"</h[23]>", body)
        assert mm, h
        end = body.index("</p>", mm.end()) + 4
        body = body[:end] + fig + body[end:]
    elif marker.startswith("before:"):
        m2 = marker[len("before:"):]
        assert m2 in body, m2
        body = body.replace(m2, fig + m2, 1)
    else:
        assert marker in body, marker
        body = body.replace(marker, marker + fig, 1)

accent = {"cadatomic": ("#1d6b4f", "#7fd1ae"), "svelters": ("#c2410c", "#fb9a6b")}[meta["project"]]
prov = "".join(f"<li>{line}</li>" for line in meta["provenance"])
page = f"""<title>{html.escape(meta.get("short_title", title))}</title>
<meta name="claude:viewport-fit-sides" content="cover">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Archivo:wght@500;700&family=Public+Sans:ital,wght@0,400;0,600;1,400&family=JetBrains+Mono:wght@400;600&display=swap">
<style>
/* Layout: one reading column (~68ch), a quiet header band, provenance as a margin note. */
:root {{
  --bg: #fbfbf9; --fg: #1d2421; --muted: #5b6662; --rule: #dfe3df;
  --panel: #f1f3f0; --code-bg: #f0f2ef; --accent: {accent[0]};
  --display: "Archivo", "Helvetica Neue", Arial, sans-serif;
  --body: "Public Sans", "Segoe UI", system-ui, sans-serif;
  --mono: "JetBrains Mono", ui-monospace, "SFMono-Regular", Menlo, monospace;
}}
@media (prefers-color-scheme: dark) {{ :root:not([data-theme="light"]) {{
  --bg: #141917; --fg: #e3e8e5; --muted: #9aa6a1; --rule: #2c3431;
  --panel: #1b2220; --code-bg: #1f2725; --accent: {accent[1]}; color-scheme: dark; }} }}
:root[data-theme="dark"] {{
  --bg: #141917; --fg: #e3e8e5; --muted: #9aa6a1; --rule: #2c3431;
  --panel: #1b2220; --code-bg: #1f2725; --accent: {accent[1]}; color-scheme: dark; }}
body {{ background: var(--bg); color: var(--fg); font: 16px/1.65 var(--body); }}
.wrap {{ max-width: 46rem; margin: 0 auto; padding-block: 2rem 4rem;
  padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px)); }}
header {{ border-bottom: 1px solid var(--rule); padding-bottom: 1.25rem; margin-bottom: 1.5rem; }}
.eyebrow {{ font: 600 .75rem/1 var(--mono); letter-spacing: .08em; text-transform: uppercase; color: var(--accent); }}
h1, h2, h3 {{ font-family: var(--display); text-wrap: balance; line-height: 1.2; }}
h1 {{ font-size: clamp(1.7rem, 4.5vw, 2.3rem); font-weight: 700; margin: .5rem 0 0; }}
h2 {{ font-size: 1.35rem; font-weight: 700; margin: 2.4rem 0 .6rem; padding-top: .4rem; border-top: 1px solid var(--rule); }}
h3 {{ font-size: 1.08rem; font-weight: 700; margin: 1.6rem 0 .4rem; }}
p, li {{ max-width: 68ch; }}
a {{ color: var(--accent); }}
code {{ font: .88em var(--mono); background: var(--code-bg); padding: .08em .3em; border-radius: 3px; overflow-wrap: anywhere; }}
pre {{ background: var(--code-bg); border: 1px solid var(--rule); border-radius: 6px; padding: .9rem 1rem; overflow-x: auto; font-size: .85rem; line-height: 1.5; }}
pre code {{ background: none; padding: 0; overflow-wrap: normal; }}
.tablewrap {{ overflow-x: auto; margin: 1rem 0; }}
table {{ border-collapse: collapse; font-size: .9rem; min-width: 32rem; }}
th, td {{ text-align: left; vertical-align: top; padding: .45rem .6rem; border-bottom: 1px solid var(--rule); }}
th {{ font-family: var(--display); font-weight: 700; }}
blockquote {{ margin: 1rem 0; padding: .2rem 1rem; border-left: 3px solid var(--accent); color: var(--muted); }}
figure {{ margin: 1.2rem 0; }} figure img {{ max-width: 100%; height: auto; border: 1px solid var(--rule); border-radius: 6px; display: block; }}
figcaption {{ font-size: .85rem; color: var(--muted); margin-top: .4rem; }}
.made {{ background: var(--panel); border-radius: 6px; padding: .8rem 1rem; font-size: .85rem; color: var(--muted); margin-top: 1rem; }}
.made strong {{ color: var(--fg); }} .made ul {{ margin: .3rem 0 0; padding-left: 1.1rem; }}
:focus-visible {{ outline: 2px solid var(--accent); outline-offset: 2px; }}
</style>
<div class="wrap">
<header>
  <div class="eyebrow">{html.escape(meta["project"])} · {html.escape(meta["kind"])}</div>
  <h1>{html.escape(title)}</h1>
  <div class="made"><strong>How this page was made.</strong>
    <ul>{prov}</ul></div>
</header>
<main>
{body}
</main>
</div>
"""
Path(out).write_text(page)
print(out, len(page))
