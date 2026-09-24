# TODO

- Delete sessions/activities from the UI (there's no `delete_session` in `logic/sessions.rs` nor a delete button in `SessionModal`/`SessionList`).
- Paginate/virtualize the session list (`SessionList.tsx` loads every session with no pagination).
- System tray icon (sync now / open / quit without fully closing the app).
- Capture cadence and power in the `.FIT` parser (`RecordAccumulator` in `session_parser.rs` only accumulates HR, GPS, speed and altitude) and show them in `SessionModal.tsx`.
- Manually add strength sessions/exercises not imported from a `.FIT` file, including custom exercises outside Garmin's catalog (`ex_cat`/`ex_id` currently only ever come from imported series).
- Sync wellness data (sleep, stress, Body Battery, resting HR) in addition to activities — currently only `GARMIN/Activity` is read over MTP; wellness/monitoring files aren't imported at all.
- Add a user height setting and compute BMI alongside body metrics (weight, fat %, lean mass, water % are tracked, but there's no height field anywhere).
- Capture and show ambient temperature recorded during a session (the watch's internal sensor records it in the `.FIT` file, but it's not parsed anywhere).
- Compute and show total elevation gain/loss per session (altitude is already captured for the map profile, but never aggregated into a total ascent/descent figure).
