# Round claims (backlog-order-2026-10-18)

One line per round. A session claims the next unclaimed round by appending its
line and pushing this branch **fast-forward only** (`git push origin claims`,
never `--force`). A rejected push means another session claimed first:
`git fetch origin claims && git reset --hard FETCH_HEAD`, then pick the next
unclaimed round. Owner decision 2026-10-18 ("Split, plus queue").

| Round | Session | Claimed (UTC) |
|---|---|---|
| 0 | session_01PJCSoqAo9bCWeVLpz61Wdy | 2026-10-18 |
| 1 | session_01PJCSoqAo9bCWeVLpz61Wdy | 2026-10-18 |
| 2 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-18T23:28 |
| 3 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-18T23:28 |
| 4 | session_01PJCSoqAo9bCWeVLpz61Wdy | 2026-10-19T06:50 |
| 5 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-19T06:52 |
| 7 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-19T07:25 (taken before 6: files disjoint from round 5 lanes; 6 is a sweep that should follow round 5) |
| ad hoc: card-overflow | session_01RDeEiq38KizbPJHTcaCvEU | 2026-10-19T08:35 (owner bug report: showcase cards clip their content; files: preview/src/main.rs Block* fns, preview/assets/main.css widget-card rules, a new playwright spec; row id 280) |
| ad hoc: docs-review | session_01U81eYPLNgreufRCunjx3nM | 2026-10-19T09:10 (owner docs review: preview/assets/main.css docs-shell + preview-card rules, preview/src/docs_nav.rs, tooltip arrow placement, preview/assets/avatars/*.svg; after the rounds-5+7 merge: component-page accordion in main.rs + build.rs, navbar menu icon, input_otp style.css, separator demo, persona rename; row ids 285-289) |

## New backlog row ids (reserved blocks, so two sessions never mint the same id)

| Ids | Session |
|---|---|
| 245-259 | session_01PJCSoqAo9bCWeVLpz61Wdy |
| 260-279 | session_01QoX93TwsKJ9GtMXWUXXfKw |
| 280-284 | session_01RDeEiq38KizbPJHTcaCvEU |
| 285-289 | session_01U81eYPLNgreufRCunjx3nM |
