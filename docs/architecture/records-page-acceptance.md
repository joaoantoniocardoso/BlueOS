# Records page device acceptance

Run on a vehicle before merging a change to the Records page, its player or the recording library
(`core/frontend/src/views/RecordsView.vue`, `components/records/`, `libs/mcap/`, `libs/recorder/`,
`core/services/recorder/`). Run every row of each section the change touches, not only the rows the change was
meant to fix: a bug hides in the sequence nobody re-ran. Report each id as PASS / FAIL / NEEDS USER with its
measurement.

- Arm only a SITL autopilot. On a real flight controller the armed rows are NEEDS USER.
- Name every file you create `acceptance-*` and delete it at the end; never change a recording you did not create.
- Chrome with `--disable-reading-from-canvas` cannot draw thumbnails: use a clean profile.

## Page behaviours (parity with draft 1, D-23)

| Id | Behaviour | Pass criterion |
|---|---|---|
| P1 | Views | List and cards views, search, State and date filters, the BlueOS menu entry; the layout and the sort survive a reload |
| P2 | Sort | Cards and list reorder the same way for every key, Duration included |
| P3 | Selection summary | Select all: the count, size, duration and Needs repair count match `du` and the `library` State |
| P4 | Delete | On a card and on a row, Delete asks first; Cancel keeps the file; Delete removes it and its cached thumbnail |
| P5 | Bulk actions | Bulk Delete and Repair submit one Job per file and show each outcome; bulk Download downloads each file |
| P6 | Card and row details | Duration, end time and tracks ("cam · N other topics", "no topics"); Needs repair reads "unknown"; the file being written ticks its duration every second and reads "recording..." |
| P7 | Thumbnails | A Ready file with no topics fetches none; one with unknown contents still does; at most two requests at once |
| P8 | Disabled actions | Repair, Delete and Download stay visible, disabled with a tooltip, when not allowed ("Wait until the file is finished before repairing" on the file being written) |
| P9 | Download | A Ready file downloads at once and writes nothing in the recordings folder; the file being written produces one `.snapshot-` copy that `mcap doctor` accepts; after Stop, the same row downloads directly; no label says "snapshot" |
| P10 | Export data | On a recording over 100 MB: the progress moves, no long task over 200 ms (`PerformanceObserver` `longtask`), Cancel stops it with no file, and the CSV opens in a spreadsheet |
| P11 | Recording without video | A recording with no video, or one that needs repair, opens in the player; a cut and Export data work; the playhead buttons are hidden |
| P12 | Repair | Progress shows the read offset, also when the page opens mid-repair; cancelling leaves the original byte-identical and no `.recover` file |
| P13 | Leave guard | The "Keep this page open" banner shows during an export, a download and a bulk repair until its last Job ends; a route change and a reload ask first; closing the player during an MP4 export asks first |
| P14 | Armed (SITL or user) | The player closes, browsing pauses, Download, Delete and Repair are disabled, the selection bar included; Stop recording stays usable |
| P15 | Dark theme | Previews and cards use the dark background |
| P16 | Older recorder | A library from an older recorder lists its files, every duration "unknown" |

## Two video streams

Stream A runs from `A0` to `A1`; stream B starts about 30 s after A and stops about 30 s before it.

### Setup

- Two MCM test streams, "acceptance cam a" (component 101) and "acceptance cam b" (component 102), H.264 ball 320×240
  at 30 fps. `COMMAND_LONG` `VIDEO_START_CAPTURE` / `VIDEO_STOP_CAPTURE` to `localhost:6040/mavlink` as 255/190.
- Live rows (`L*`) run on the always-on recording while the cameras are driven. Finished rows (`F*`, `V*`, `X*`) run
  on `acceptance-two-streams.mcap`, the live file's snapshot renamed after both cameras stopped.
- Drive Chrome over CDP: trusted mouse clicks and 10-step drags on `.timeline`, `Runtime.evaluate` probes,
  screenshots.

### Terms

- **Bs / Be:** B's coverage start / end (its lane): B's first frame to its last frame plus one frame interval when
  the recording has a message index, chunk-granular without one. **Bf / Bl:** B's first / last frame (`mcap cat`).
  **Bk:** B's first keyframe (B's buffered start once it opens). Same for A.
- **Probe:** `position`, `playing`, and per `<video>` `currentTime`, `readyState`, `paused`, `buffered`; the "Not
  available" count; any "Timestamps cannot be smaller…" text ("GOP error"); fps labels; `seeking` events per
  `<video>`.
- **In step:** both `<video>` have `readyState` ≥ 2 and are unpaused, and `|A − B| ≤ 0.5 s`.
- **Held:** B is paused with `readyState` ≥ 2 (its first frame) and its `currentTime` does not advance, while A plays.
- **Clean:** no GOP error, no stream error overlay, at most 3 `seeking` events per `<video>` per user action, and A's
  `currentTime` advances monotonically while playing.

### Live recording (cameras started after the player opened)

| Id | Visible | Entry / action | Steps | Pass criterion |
|---|---|---|---|---|
| L1 | – | Open with no camera | Open the live file's player before any camera starts | "This recording has no video streams…", `tracks: []` |
| L2 | A | First camera starts after open | Start A; sample every 2 s | A listed and selected within one chunk duration + 2 s of the start command, without reopening; then, with no press, `playing`, A `readyState` ≥ 2 and advancing, `position` within one chunk of the coverage end |
| L3 | A, B | Second camera starts after open, A at the live edge | Start B at A0+30 s with A playing at the live edge; no touch for 60 s | B listed within one chunk duration + 2 s, own lane; A never pauses (clean); B held until A ≥ Bk, then in step within 2 s and for ≥ 60 s |
| L4 | A, B | Playback crosses Bs live, A ~20 s behind | Second player, paused on A before B starts; Play after B is listed so A crosses Bs by playback | B held until A ≥ Bk, then in step within 2 s and for ≥ 30 s; B never leads A by more than 0.5 s |
| L5 | A, B | Playback crosses B's end, live | Let L3's player play while B stops (A1−30 s) and past Be | A clean; B ends on its last frame, then "Not available" past Be; no GOP error; B's `seeking` count does not grow between Bl and Be |
| L6 | A, B | Live edge, both recording | Press "Skip to the latest" with both in range | Both in step within 5 s; `position` within one chunk of the end |
| L7 | A, B | Live edge after B stopped | Press "Skip to the latest" after B stopped, A still recording | A plays at the end; B "Not available"; clean |
| L8 | A, B | Open inside B's range | Open a new player while A and B both record | Both play near the live edge, in step within 5 s, clean |
| L9 | A, B | Pause / Play, live | Pause 5 s, then Play, inside B's range | Both paused while paused; in step within 2 s of Play |
| L10 | B | Hide A while live, inside B | Hide A in the Streams menu, then show it again after 10 s | B keeps playing (`position` advances from B); A rejoins in step within 5 s, no rewind |
| L11 | A | Hide B, show B again inside B | Hide B 10 s, show it again | B rejoins in step within 5 s, clean |
| L12 | A, B | Re-enter after B played and left, live | After B stopped and A passed Be, click back into B's range; let it play across Be again | In step within 2 s of the click; then as L5 |
| L13 | A, B | Lanes, live | Read `.timeline` after A appears, after B appears, after B stops | One lane, then two; B's lane inside A's; B's lane end stops growing after B stops |
| L14 | A, B | CSV defaults on a player that gained both cameras after it opened | Open Export data on the L3 player | Both video channels listed and unselected by default ("N−2 of N") |

### Finished recording (`acceptance-two-streams.mcap`)

| Id | Visible | Entry / action | Steps | Pass criterion |
|---|---|---|---|---|
| F1 | A, B | Open outside B's range | Fresh open | Opens at A's first keyframe and plays; B "Not available", B `readyState` 0, no error |
| F2 | A, B | Open outside, click inside (3×) | Fresh open, one trusted click mid-B; repeat with fresh opens | In step within 2 s, clean, 30 fps labels |
| F3 | A, B | Drag forward into B | Fresh open, drag from before Bs to mid-B, release | In step within 2 s, clean |
| F4 | A, B | Playback crosses Bs, fresh open | Fresh open, drag to Bs−15 s, release, no touch for 60 s after A ≥ Bk | B held until A ≥ Bk, then in step within 2 s and for ≥ 30 s |
| F5 | A, B | Playback crosses Bs after B played and left | Click mid-B, play 20 s, drag to Bs−15 s, play across | B's `currentTime` right after reopening is ≤ Bk + 1 s (no stale time); then as F4 |
| F6 | A, B | Playback crosses B's end | Click at Bl−15 s, play until A ≥ Be+10 s, no touch | A clean; B stops on its last frame, then "Not available" past Be; no GOP error; B's `seeking` count stable between Bl and Be |
| F7 | A, B | Drag forward out of B | From mid-B, drag to Be+20 s | A plays, clean; B "Not available" |
| F8 | A, B | Drag backward into B from after it | From Be+20 s, drag to mid-B | In step within 2 s, clean |
| F9 | A, B | Drags leaving and re-entering B (10) | Ten drags alternating between before Bs and inside B | Every re-entry in step within 2 s, clean |
| F10 | A, B | Re-enter by playback after B played to its end | After F6, drag to Bs−15 s, play across Bs | As F4 |
| F11 | A, B | Click between Bs and Bk | Click at (Bs+Bk)/2 | The playhead starts at the click; A plays clean; B held until A ≥ Bk, then in step within 2 s |
| F12 | A, B | Click between Bl and Be | Click at (Bl+Be)/2 | A plays clean; B shows a frame or "Not available", no GOP error, `seeking` count stable for 10 s |
| F13 | A, B | Pause / Play inside B | Pause 5 s, then Play | Both paused while paused, `|A − B|` ≤ 0.5 s; in step within 2 s of Play |
| F14 | A, B | Pause / Play while B is held | F4 setup; Pause before A reaches Bk, Play after 5 s | B stays held until A ≥ Bk, then in step within 2 s |
| F15 | A, B | Seek while paused inside B | Pause mid-B, click 30 s later inside B | Both play from the click in step within 2 s, clean |
| F16 | A, B | Play from the end of A | Drag to A's end, wait for A to end, press Play | Clean; once stopped, the playhead stays at A's end (not 0) and `playing` is false; Play starts again from a covered time |

### Stream visibility (finished recording)

| Id | Visible | Entry / action | Steps | Pass criterion |
|---|---|---|---|---|
| V1 | B | A hidden, Play from outside B | Fresh open, hide A, press Play | Jumps to B's range and plays B; `position` advances; clean |
| V2 | B | A hidden, click inside B | Click mid-B | B plays from the click; `position` follows B; clean |
| V3 | B | A hidden, playback crosses B's end | Click at Bl−10 s, let it play | B stops at its end and the playhead stays there (not 0); no GOP error; `playing` false once nothing advances |
| V4 | A | B hidden, play across all of B's range | Hide B, drag to Bs−10 s, play past Bs | A clean; no `<video>` for B; one lane |
| V5 | A, B | Show B while playing inside B | From V4, show B mid-B while playing | B in step within 3 s, clean |
| V6 | A, B | Show B while paused inside B | Pause mid-B with B hidden, show B | B shows the frame at the playhead and stays paused; `playing` false; `|A − B|` ≤ 0.5 s |
| V7 | B | Hide A while both play inside B | Hide A, wait 10 s, show A | B keeps playing, `position` follows B; A rejoins in step within 3 s |
| V8 | B | Hide A while playing outside B | Play at Bs−30 s, hide A | The playhead moves into B's range and plays B, or `playing` turns false |
| V9 | – | Both hidden, then both shown | Hide both, show both inside B | "Select at least one stream to play", plain lane; both rejoin in step within 3 s |
| V10 | mixed | Timeline lanes | Both / A hidden / B hidden / both hidden / both shown | Two lanes (B inside A); one lane per visible stream; a plain track when none; the playhead on the lanes |

### Export (finished recording)

| Id | Visible | Cut | Pass criterion |
|---|---|---|---|
| X1 | A, B | 50 s inside B | Two MP4s, each 50 s ± 0.2 s, decode fully (`ffprobe`, `ffmpeg -f null`) |
| X2 | A, B | 20 s before Bs | One MP4 (A, 20 s ± 0.2 s); the notice names B as holding no keyframe in the cut |
| X3 | A, B | Bk−20 s to Bk+20 s | Two MP4s; A ≈ 40 s; B from its first keyframe, ≈ 20 s ± 1 s; both decode fully |
| X4 | A, B | Bl−20 s to Bl+20 s | Two MP4s; A ≈ 40 s; B ≈ 20 s ± 1 s, ending at Bl; both decode fully |
| X5 | A, B | 20 s after Be | One MP4 (A); the notice names B |
| X6 | A | 40 s inside B, B hidden | One MP4 (A) |
| X7 | A, B | – | CSV defaults: the video channels are unselected ("N−2 of N") |

### Accepted, not failures

- Without a message index, coverage is chunk-granular: the lane and the "Not available" overlay may start up to one
  chunk before Bf and end up to one chunk after Bl. A stream waiting for the leader shows its first frame, paused.
- Exported MP4s carry up to one GOP of pre-roll frames at time 0; `ffmpeg -f null` may print "non monotonically
  increasing dts" warnings for them. These are warnings, not decode errors.
- While a CSV export runs, its dialog blocks the player's close paths; leaving the page still asks first.
