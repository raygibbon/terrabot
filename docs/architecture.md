# Terrabot v0.1 architecture

`main` validates CLI/configuration before entering the terminal. `config` parses
a single TOML document, supplies defaults, validates every supported field and
rejects all execution enable flags. `ui` owns application state and delegates
terminal lifecycle to `terratui::Application`. TerraBox's terminal Drop restores
raw mode and the alternate screen on normal return, errors and unwinding.
Ctrl+C is a raw key handled by the application. Forced termination is outside
that lifecycle.

## Existing framework reuse

The implementation was based on the local TerraBox sources, ENGINE.md,
TERRATUI.md, architecture.md, the widget gallery and cell animation example.
Both crates are direct sibling Cargo path dependencies; neither is modified.

TerraTUI Desktop, Group and Layout splits allocate the header, Terry, transcript
and input. Input is the group's only focusable child, so it regains focus after
being clipped out at very small sizes. TextInput supplies grapheme-aware editing and bracketed paste handling;
a thin wrapper imposes an input size cap. Custom transcript and Terry widgets
draw through clipped TerraBox Canvas objects. TerraBox owns true-colour output,
Unicode width, input parsing, SIGWINCH resize events and changed-row presentation.
Frames are rebuilt only when the application requests redraw; rendering never
clears the physical screen every animation frame.

Terry is a procedural 36×36 pixel design packed into 36×18 terminal cells with
`▀` foreground/background halves. The compact version samples the same geometry
at half resolution. Chassis, visor, limbs, eyes, mouth, orb and particles are
independent features driven by elapsed wall time and eight explicit states.
SecurityBlocked is reserved and rendered/tested but no security events exist yet.
TerraBox FrameClock schedules TerraTUI one-shot Deadline timers without catch-up
bursts. Disabled or hidden animation has no idle timer; an active request alone polls at
50 ms. Visible animated mode drains model updates at the configured frame rate.

## Transport, memory and cancellation

The provider module uses Ollama's documented `/api/tags` and streaming
`/api/chat` endpoints:
https://docs.ollama.com/api/tags and https://docs.ollama.com/api/chat.
One worker thread owns a single-thread Tokio runtime and reqwest client. It
never owns UI state. A bounded 32-message Tokio channel carries updates; the UI
drains at most 32 per tick. TerraTUI's unbounded EventSender is deliberately not
used for stream data. Backpressure applies across network reads and sends.

A bounded NDJSON decoder handles arbitrary byte and UTF-8 boundaries, terminal
completion frames, server errors and premature EOF. Frames are capped at 256 KiB,
model lists at 1 MiB, and generated content at 128 KiB. HTTP redirects and proxy
environment discovery are disabled. Requests carry no tools and no credentials.

Dropping Worker sends a cancellation signal, closes the update receiver and
joins the worker. A Tokio select surrounds the whole request including channel
backpressure and network waits. Cancellation drops the request future/socket;
it does not wait for model output. Shutdown, error and panic unwinding drop the
worker. Server-side GPU cleanup is controlled by Ollama.

Only successfully completed user/assistant pairs enter future request history.
The transcript separately retains visible partial/error messages. Oldest complete
pairs are trimmed before request construction. No history is written to disk.

## Extension boundary

The UI consumes Ready/Text/Done/Error domain events rather than HTTP frames.
Future providers can implement that boundary without owning the terminal.
Execution, policy decisions and credential brokering are absent. Future agent
integrations must keep proposals distinct from independently authorized actions:
**The AI can propose actions, but only independent policy enforcement can
authorise execution.** Reserved visual states are not authorization mechanisms.
