---
id: '22'
title: Explore publishable shell names ending in -sh for Rush rename
slug: explore-publishable-shell-names-ending-in-sh-for-r
status: open
priority: 2
created_at: '2026-04-24T07:20:30.979017Z'
updated_at: '2026-04-24T07:46:44.399919Z'
notes: |-
  ---
  2026-04-24T07:23:38.673155+00:00
  New hard constraint from user: every candidate name should support an acronym/backronym, not just sound good. Future naming exploration should include explicit expansions and evaluate whether the expansion feels natural versus forced.

  ---
  2026-04-24T07:34:07.439408+00:00
  Naming direction shifted: keep a silly/self-aware attitude, but move away from the literal word 'agentic' in backronyms. Future options should preserve shell legitimacy and playful tone without centering 'agentic' explicitly.

  ---
  2026-04-24T07:41:53.976506+00:00
  User surfaced more playful backronym directions and a concrete constraint: `mash` is already taken as a crate, so it should be treated as blocked for publication despite being a strong naming skeleton. User likes silly/self-aware expansions like 'Mildly Usable Shell', 'Sorta Usable Shell', and still considers 'Modern Agentic Shell' tonally appealing.

  ---
  2026-04-24T07:45:08.275853+00:00
  User preference update from naming exploration:
  - Likes the ironic `_ Usable Shell` family.
  - `push` / 'Probably Usable Shell' is fun but blocked by existing crate.
  - `lush` is also blocked by existing crate.
  - `sush` is acceptable but pronunciation is awkward.
  - User likes `tush` ('Totally Usable Shell') as a hilarious apparently available option.
  - User also proposed `aush` ('Actually Usable Shell').
  Future exploration should emphasize pronounceability, crate collision reality, and the ironic-understatement 'usable shell' frame.

  ---
  2026-04-24T07:46:44.399915+00:00
  User reaction: `aush` is a strong favorite so far. Positive associations: sounds close to an exclamation ('ah, shit') without being explicit, memorable when spoken, and 'Actually Usable Shell' matches current perceived product reality. Follow-up implication from user: benchmark posture may need strengthening so the name's claim is evidenced rather than just vibes.
labels:
- naming
- branding
- rush
- shell
verify: |-
  python3 - <<'PY'
  print('naming exploration captured in mana or response')
  PY
kind: job
---

Context:
- User likes the product feel of 'Rush' but expects the public/published name will likely need to change.
- Current direction: explore shell names that end in 'sh'.
- Goal is naming exploration, not final trademark clearance yet.

Needs:
- Generate options that preserve some of Rush's feel: speed, momentum, command-line, sharpness, daily-driver shell identity.
- Prefer names that naturally end in `sh`.
- Distinguish between names that are product-brand viable vs crate/binary/package viable.
- Flag obvious risks: genericity, collision likelihood, pronunciation awkwardness, unclear meaning.

Output shape:
- Shortlist grouped by vibe/theme
- Quick commentary on strengths/risks
- Candidate directions for deeper validation later (search/trademark/GitHub/crates)

Constraints:
- Exploration only for now
- Do not claim availability without checking later
