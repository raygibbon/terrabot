# Terrabot v0.2 architecture

The existing chat application and bounded Ollama transport remain in place.
Terri adds a pure personality simulation, layered terminal art, validated emotion
cues, and versioned local metadata. TerraBox and TerraGame were inspected directly
and neither framework was modified.

## Framework ownership

TerraTUI Application owns terminal lifecycle, deadlines, event routing and redraws.
Desktop/Group/Layout allocate header, character, transcript and input. Only input
is focusable, including after a resize hides and restores it. TextInput retains
its grapheme editing and paste handling. TerraBox owns Canvas clipping, true-colour
cells, Unicode widths, changed-row presentation, resize events and restoration.
Its FrameClock paces rendering without catch-up bursts.

Only `terragame-core` is needed. Its FixedStep drives 50 ms simulation steps with
at most twenty catch-up steps; its SplitMix64 Random supplies deterministic
weighted choices and blink timing. There is one creature and a handful of fixed
objects, so entity stores, world caches and a second application driver would add
no useful capability. Simulation ticks and RNG never depend on viewport dimensions
or draw calls. Long stalls drop excess time; they do not create a catch-up backlog.

## Module boundaries

- `pet`: traits, moods, activity priorities, cooldowns, bounded weighted selection,
  movement, blinking and typed interaction events. No I/O or provider access.
- `animation`: original palette-coded art; borrowed AnimationClip frame sequences;
  separate eyes, mouth, hands and feet; smooth pose transitions, squash/stretch,
  breathing, hopping, orb movement and at most six particles. A bounded logical
  Sprite grid converts pairs of pixels to TerraBox half-block cells. Indexed and
  background-cell character modes provide explicit fallbacks. Invalid animation
  settings pause only the character widget.
- `cues`: strict mood/gesture enums, numeric validation, JSON schema and dialogue
  profile. An LLM cannot supply code, arbitrary animation names or internal traits.
- `persistence`: bounded versioned metadata reads and atomic writes. No dialogue.
- `provider`: cancellable HTTP requests and a bounded 32-update channel. Streaming
  NDJSON is capped at 256 KiB per record and 128 KiB content per reply. Model lists
  cap at 1 MiB. A cue uses a separate non-streaming schema request, a ten-second
  deadline, a 64 KiB HTTP-body cap and a 2 KiB cue-document cap.
- `ui`: conversation, user events, worker lifetimes and layout. The system profile
  is added once per request, separately from retained user/assistant pairs.
- `demo`: a standalone TerraTUI client of the same character engine, without
  Ollama or state-file I/O. Timed runs report pacing after terminal restoration.

## Priorities and timing

Autonomous choices are the lowest priority. Conversation attention interrupts
idle behaviour; thinking and speaking hold attention until completion/cancellation.
A simulated denial has the highest cosmetic priority. Cues cannot override an
active conversation or a serious conversational tone. Success can celebrate;
ordinary completions settle into content attention. Text heuristics are conservative
and intentionally limited; they do not constitute sentiment understanding.

Traits remain finite within 0–1. Energy decreases slowly during active simulated
time and recovers during rest. There are no punitive mechanics. Configured base
weights combine with stable personality traits, preferred activities, boredom,
energy, cooldowns and tone constraints. Invalid persisted traits are rejected.
No autonomous decision can access files, invoke inference or execute actions.

Rendering defaults to 20 FPS, independently of the fixed simulation. Hidden or
disabled animation advances the bounded simulation once per second without idle
redraws; active model workers are drained at least every 50 ms in that mode.
Visible animation uses the earlier simulation/render deadline. Input and network
updates remain on the existing fair TerraTUI loop. No busy polling or unbounded
animation-event queue is introduced.

## Persistence and lifecycle

The application loads traits before terminal initialization and saves on clean
shutdown, after cancelling model and cue workers. Snapshot version 1 stores
traits, mood, preferred activities, counters, last interaction category and RNG
state. It contains no prompt, reply or transcript. A new session resets transient
poses and does not advance the simulation for time spent closed. Active tick and
session counts are modest progression metadata, not claims about real experiences.

Loads are capped at 16 KiB. A corrupt/future file stays untouched and disables
saving for that session. Saving uses create-new temporary files, mode 0600 on Unix,
file sync, rename and directory sync. Concurrent app sessions are last-writer-wins;
there is no multi-instance merge. Crashes may lose the current session's progress.
A failed save is reported after terminal restoration. The demo never saves.

Both worker types are cancelled by dropping their request futures and closing
bounded receivers. Shutdown joins the worker; runtime shutdown bounds waiting for
an OS DNS task. Redirects and proxy environment discovery are disabled. Terminal
Drop handles normal exits, I/O errors and unwinding; forced kills cannot run Drop.

## Future policy boundary

Tools, MCP, shell execution, Goose, OpenShell and credential brokering remain
absent. A denial animation is a demonstration, not a policy implementation.
The AI can propose actions, but only independent policy enforcement can authorise
execution. Future providers can reuse the domain-event boundary without owning
the terminal or directly mutating personality state.
