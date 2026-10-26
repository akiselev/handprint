# Blind judging protocol

You are a reader, not an editor. You get pairs of documentation pages, labelled X and Y, about the
same task. You are not told who wrote them, how, or which one anyone hopes will win. Do not look
for, open or guess at any other file than the ones named in your instructions.

For each pair:

1. **Read both. You need to do the task the pages describe, and you can keep only one page. Which?**
   Record `keep: X|Y`, `margin: clear|slight|coin-flip`, and a one-sentence reason (the deciding
   factor, not a summary). Decide this before any analysis.
2. Then note, for each page, with quotes:
   - `attention_dropped`: the sentence where you stopped wanting to read, or null.
   - `confusing`: any instruction you could not follow or that contradicts itself, or null.
   - `machine_tells`: template headings, list-closers that restate the list, filler intensifiers
     ("crucial", "seamless", "simply"), "It's <adjective> to" openers, a closing paragraph that
     restates the page, a sentence explaining what the previous one showed, heavy bold or
     em-dashes. Quote each.
   - `called_machine_written`: yes / no / can't tell, with confidence 1–5.
3. A yes/no checklist per page (answer each, one word): states prerequisites; steps have one
   action each; says what you should see after key steps; has a recovery tip where a step can fail;
   says plainly what doesn't work yet; headings name the real subject; no restating closer.

Write your verdict file as JSON (a list, one object per pair) to the path you were given.
