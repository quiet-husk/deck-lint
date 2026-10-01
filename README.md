# decklint

A linter for spaced repetition deck files. It reads a plain text file of
scheduled cards and reports problems with a line number, the way a compiler
would, instead of letting a broken scheduling state silently corrupt review
timing.

## Why

Spaced repetition apps (Anki-style SM-2 schedulers, homegrown review queues,
export/import scripts between tools) store each card's ease factor,
interval, and repetition count as plain data. That data is easy to corrupt
by hand-editing an export, writing a buggy import script, or merging two
decks with colliding ids. The bugs are quiet: an ease factor of 1.1 does not
crash anything, it just means that card never leaves the daily review pile
again. This tool checks a deck file for that class of mistake before it
reaches a scheduler.

## Deck file format

One card per line, whitespace separated fields, first field is the card id:

```
<id> due=YYYY-MM-DD interval=<days> ease=<factor> reps=<count>
```

Blank lines and lines starting with `#` are ignored. Example:

```
# vocab deck, exported 2026-09-01
card-1042 due=2026-09-20 interval=6 ease=2.5 reps=3
card-1043 due=2026-09-21 interval=1 ease=1.1 reps=4
card-1042 due=2026-09-22 interval=1 ease=2.5 reps=1
```

Running `decklint` on that file reports:

```
deck.txt:2: error: card 'card-1043': ease 1.10 is below the minimum of 1.3 (the scheduler will stall this card)
deck.txt:3: error: duplicate card id 'card-1042' (first seen on line 2)
```

## Usage

```
decklint [--json] path/to/deck.txt
```

With `--json` the findings are printed as an array of objects with `file`,
`line`, `severity` and `message` keys (`[]` when the file is clean), instead
of the one-line-per-finding text form.

Exit code is nonzero if any finding is an error, so it can be dropped into a
pre-commit hook or CI step for a deck export pipeline.

## Checks implemented so far

- malformed `due` dates (bad format, invalid month or day, February 29th on
  a non-leap year)
- missing or unrecognized fields
- ease factor below the SM-2 floor of 1.3, or implausibly high
- negative interval or repetition counts
- interval and repetition counts that contradict each other (for example,
  zero reps with a multi-day interval)
- duplicate card ids within the same file

## Design note

Every check is a pure function: text in, a list of findings out, nothing
read from disk and nothing mutated. `lint::lint_source` takes a `&str` and
returns `Vec<Finding>`; `lint::check_card` takes an already-parsed `Card`
and returns its issues. That split makes it possible to unit test every
rule directly against constructed values, without touching the filesystem.

## Status

Early skeleton. The only flag so far is `--json` (no severity filtering yet),
and the rule set above is intentionally small.
