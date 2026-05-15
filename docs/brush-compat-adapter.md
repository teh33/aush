# AUSH external shell corpus adapter

AUSH has a small adapter for consuming Brush's Bash-oriented shell behavior
corpus without vendoring Brush's Rust test harness. This is a test-corpus
integration, not a product compatibility promise: Brush provides useful external
shell cases that help harden AUSH before release.

## Source corpus

By default the adapter reads Brush YAML cases from:

```text
/Users/asher/brush/brush-shell/tests/cases/compat
```

Override with:

```sh
AUSH_BRUSH_CASES=/path/to/brush-shell/tests/cases/compat \
  cargo test --test brush_compat_tests
```

Run a bounded report while iterating over the external corpus:

```sh
AUSH_BRUSH_REPORT_LIMIT=50 \
  cargo test --test brush_compat_tests brush_compat_full_corpus_report -- --ignored --nocapture
```

Run one Brush YAML file:

```sh
AUSH_BRUSH_CASE_FILE=options/set-u.yaml \
  cargo test --test brush_compat_tests brush_compat_full_corpus_report -- --ignored --nocapture
```

Run one case by exact name or substring:

```sh
AUSH_BRUSH_CASE_FILE=pipeline.yaml \
AUSH_BRUSH_CASE_NAME='Exit codes for piped commands' \
  cargo test --test brush_compat_tests brush_compat_full_corpus_report -- --ignored --nocapture
```

`AUSH_BRUSH_CASE_FILE` matches either the full relative path or a suffix. It can
therefore be `pipeline.yaml` or `compound_cmds/case.yaml`. `AUSH_BRUSH_CASE_NAME`
matches exact case names or substrings.

Write a Markdown report:

```sh
AUSH_BRUSH_REPORT=reports/brush-corpus.md \
  cargo test --test brush_compat_tests brush_compat_full_corpus_report -- --ignored --nocapture
```

The initial test, `brush_compat_smoke_subset_matches_bash`, runs a focused
passing subset of Brush cases through:

- `bash --norc --noprofile`
- `aush --no-rc`

and compares stdout, stderr, and exit status.

Supported case fields today:

- `name`
- `stdin`
- `args`
- `env`
- `test_files`
- `ignore_stdout`
- `ignore_stderr`
- `skip`
- `known_failure`

Classification is intentionally coarse at first. Current report classes:

- `must-fix`: core automation/common-shell behavior AUSH should pursue soon.
- `should-fix`: useful Bash/common scripting compatibility, usually later.
- `deferred`: intentionally low priority or deep Bash behavior for now.

The report summary includes raw counts plus `must-fix failed`, `should-fix
failed`, `deferred failed`, and `deferred ignored`.

## Known early mismatch

Brush `pipeline.yaml::Basic pipe` exposed a real AUSH shell-behavior difference:

```sh
echo hi | grep -l h
```

Bash prints:

```text
(standard input)
```

AUSH currently prints nothing. Keep this as a future targeted compatibility
regression rather than adding it to the passing smoke subset.

## Direction

Expand the adapter by classification, not by assuming all Brush cases should
pass immediately:

- `must-pass`: core syntax/semantics AUSH intends to match
- `known-gap`: not yet supported but desired
- `intentional-difference`: AUSH deliberately differs from Bash
- `aush-extension`: structured/native behavior outside Bash
