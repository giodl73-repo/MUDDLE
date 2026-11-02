# Native play and persistence boundaries

The adapter constructs an explicitly registered existing Rust host, calls
MuddleSession::for_host or decode/resume_for_host, and plays command attempts
through native MuddleCommand::parse and MuddleSession::play_turn. A shared native
client_snapshot supplies rooms, panels, hints, successful turn history, controls
and visual intent. The browser formats text and sends commands; it owns no locks,
resources, outcomes or game rules. Asset paths are inert data, never fetched.
Authored JSON mounts only the existing MuddleStaticHost: look and stored exits.

Each request reconstructs the host and replays the complete bounded sequence.
Failed attempts remain visible; native failed commands are not transcript turns.
No native commands are added or silently converted to successful outcomes.
The browser does not implement CLI quit/exit interception. Blank script lines
are skipped as explicitly described by the UI.

Native saves replay successful commands and check the final room, then import
the optional host checkpoint. A checkpoint can therefore replace replayed host
state; it is not a transcript proof. Saves have no host identity field or world
fingerprint. Explicitly select the matching host/world before import; acceptance
does not prove provenance, integrity, intended identity, or compatibility with
arbitrary future versions. Native host checkpoint validation remains authoritative.
Checkpoint save, replay-only save and original native transcript stay distinct.

The browser commits only a complete accepted result. Failed imports, resource
refusals, crashes, timeouts and cancellation retain the previous accepted session
and its exports. Candidate save/world editors are separate from active inputs.
Everything stays in memory; no automatic persistence, server, account or upload.
Download saves before closing. Original opened bytes remain separate from edited
text and explicit LF copies. Limits are browser admission, not native contracts.

Three built-ins are MUDDLE's existing mock/spike crates. This is not authoritative
BANISH/AMAZE game adoption, a product runtime migration, full Macroquad graphical
parity, a native filesystem/slot browser, or a dynamic host/plugin loader.
