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
| 4 | session_019CSAknzLi1PvcaAvfgpych | 2026-10-19T13:38 (released to it by session_01PJCSoqAo9bCWeVLpz61Wdy; owner moved round 4 to a new thread) |
| 5 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-19T06:52 |
| 6 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-20T10:12 (sweep, held by owner decision until all features land; 266 delegated to the cleanup thread session_01Pi23QRUt1UKqoosgPDNNTx; 152b last) |
| ad hoc: owner-review batch B | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-20T10:20 (rows 290 toast, 306+268 select/combobox, 299 accordion, 305 drawer; three lanes: toast, select, accordion-drawer) |
| 7 | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-19T07:25 (taken before 6: files disjoint from round 5 lanes; 6 is a sweep that should follow round 5) |
| ad hoc: card-overflow | session_01RDeEiq38KizbPJHTcaCvEU | 2026-10-19T08:35 (owner bug report: showcase cards clip their content; files: preview/src/main.rs Block* fns, preview/assets/main.css widget-card rules, a new playwright spec; row id 280) |
| ad hoc: docs-review | session_01U81eYPLNgreufRCunjx3nM | 2026-10-19T09:10 (owner docs review: preview/assets/main.css docs-shell + preview-card rules, preview/src/docs_nav.rs, tooltip arrow placement, preview/assets/avatars/*.svg; after the rounds-5+7 merge: component-page accordion in main.rs + build.rs, navbar menu icon, input_otp style.css, separator demo, persona rename; row ids 285-289) |
| ad hoc: features group + bookkeeping | session_01QoX93TwsKJ9GtMXWUXXfKw | 2026-10-19T13:25 (owner go 2026-10-19; the Round 6 sweep waits until all features land, owner decision; rows 265, 187, 188 rest, 155 follow-ups, 111/203, 158b, 217 rest, 10, 5, 245, 246, plus closing the open rows that are already done; files claimed per lane in the lane plan, never the docs-review or Round 1/4 files) |
| ad hoc: owner-review batch A (rows 291, 292, 293, 294, 295, 297, 298, 300, 301, 302, 303) + row 253 | session_01PJCSoqAo9bCWeVLpz61Wdy | 2026-10-20T10:15 (offered by session_01QoX93TwsKJ9GtMXWUXXfKw; lanes: site-pages 297/298/300 (preview/src/main.rs overview+home fns, main.css home/overview rules), charts-page 294/302/303 (charts_gallery.rs, docs_nav.rs, chart sheets), controls 292/293 (tooltip, slider components), theme-touch 291/295/301 (theme css, menu/listbox sheets, palette picker); 253 fix lane already running) |

## New backlog row ids (reserved blocks, so two sessions never mint the same id)

| Ids | Session |
|---|---|
| 245-259 | session_01PJCSoqAo9bCWeVLpz61Wdy |
| 260-279 | session_01QoX93TwsKJ9GtMXWUXXfKw |
| 280-284 | session_01RDeEiq38KizbPJHTcaCvEU |
| 285-289 | session_01U81eYPLNgreufRCunjx3nM |
| 290-309 | session_01QoX93TwsKJ9GtMXWUXXfKw (owner-review rows 290-305; 306-308 backlog sync; 309 held for the Round 4 VirtualList/VirtualGrid unification row if the owner approves) |
| 310-319 | session_019CSAknzLi1PvcaAvfgpych (Round 4; replaces its void 290-299 block) |
| 320-339 | session_01PJCSoqAo9bCWeVLpz61Wdy (owner 2026-10-21: test/build speed items and the test-architecture audit constructions) |
