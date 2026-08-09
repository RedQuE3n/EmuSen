# EmuSen — Pegasus

Pegasus is the live shared notepad this project is developed through: two people
type into the same document at once, from different machines, and neither can
lose work. It is also the only F# in the repository, which §3 argues for rather
than assumes.

It is a leaf. Nothing in EmuSen references it, and it references only
`EmuSen.LunaP` (and through it Galaxia and Cauldron) plus `YDotNet`. If that ever
stops being true, §7 and §3 are the arguments to re-read first.

## 1. What Pegasus is for

A live notepad two people type into simultaneously while working. The governing
requirement, which arrived after an initial survey of existing tools and which
selects the entire architecture, is that **no party may lose information**.

Mature alternatives were surveyed before any code was written — CryptPad,
HedgeDoc, Etherpad, Rustpad, and Teamtype (the peer-to-peer editor-agnostic tool
formerly called Ethersync). The decision to build rather than adopt was taken
deliberately with that survey in hand. This section exists so that a later reader
does not mistake the build for ignorance of the alternatives.

## 2. Why a CRDT, and why not a simpler scheme

A last-writer-wins design loses data by construction: two people editing the same
region concurrently means one edit is discarded, and no amount of care in the
transport layer recovers it. Operational transformation preserves both edits but
requires a central authority to order operations, which reintroduces the single
point whose failure the requirement forbids.

A CRDT gives the property structurally. Every peer holds a complete replica,
concurrent edits merge deterministically, and a peer that has been offline
converges on reconnect without prompting anyone to choose a version. The
requirement is therefore not "use a CRDT because it is modern"; the requirement
is the CRDT restated.

Pegasus does not implement one. Writing a correct sequence CRDT is a research
project, and the failure mode of getting it subtly wrong is silent corruption of
the exact data the project exists to protect. It binds Yrs — the Rust
implementation of Yjs — through `YDotNet`.

## 3. Why F#, when F# was rejected on a sibling project

F# was evaluated on EmuSen's `F#ascent` branch and reverted in full. Those
objections were **boundary** objections and do not transfer to a standalone
codebase:

- The exhaustiveness benefit evaporated because C# gets no exhaustiveness
  checking over an F# union — a `switch` covering two of six cases compiles
  clean. Pegasus has no C# to be checked against; the wire protocol union is
  matched only from F#.
- `FSharp.Core` (2.4 MB) was pushed into every EmuSen frontend and the standalone
  shell. Pegasus has no host to burden.
- FsCheck gained nothing there because CsCheck matched it on generation and
  shrinking. Here FsCheck is the native choice, not a second testing story.

What F# is expected to earn, recorded so it can be checked rather than assumed:

1. `MailboxProcessor` as the answer to YDotNet's transaction discipline (§4.2).
2. Unions modelling the protocol with exhaustiveness that is actually enforced.
3. Property-based convergence testing as a first-class idiom.

If these do not materialise in practice, that is a finding to write down here,
not to paper over.

## 4. Phase 0 — dependency spike, run 2026-08-09

Nothing was built until the two young dependencies were proven on this machine.
Both spikes lived in a scratch directory and are not part of the repository; what
follows is what they established.

Environment: Fedora 44, .NET SDK 10.0.110.

### 4.1 YDotNet 0.6.0 — PASSED

`YDotNet` 0.6.0 and `YDotNet.Native.Linux` 0.6.0 restore on .NET 10 and the
`libyrs.so` native for `linux-x64` loads. Four claims were tested directly:

| Claim | Result |
|---|---|
| Divergent concurrent edits converge to identical text | PASS |
| Neither side's edit is lost in the merge | PASS |
| `ObserveUpdatesV1` yields a log that replays into an identical document | PASS |
| `StickyIndex` tracks a caret across remote edits | PASS |

The convergence test established a shared base, then had two replicas edit
without having seen each other, then exchanged state vectors and shipped only the
missing operations — 20 bytes one way, 18 the other. Both replicas ended
identical and contained both edits.

The `ObserveUpdatesV1` result is what makes the append-only file format in
`Pegasus_Format.md` viable: the observer's payload is exactly what must be
appended, and replaying the log reconstructs the document.

`StickyIndex` was tested in both directions. A caret at offset 3 moved to 8 when
five characters were inserted before it, and stayed at 8 when three were appended
after it.

### 4.2 Two API contracts discovered the hard way

Both cost a spike iteration and both shape `Document.fs`:

- **`Doc.Text(name)` throws if any transaction is open.** It defines the root
  type; `Transaction.GetText(name)` only fetches, and returns `null` before the
  root has been defined. The handle must therefore be acquired once, outside any
  transaction, and reused. `DocumentActor` holds the `Doc` and its root `Text`
  together for this reason.
- **The library actively throws on overlapping transactions**, with
  `"Failed to open a transaction, probably because another transaction is still
  open."` This is stronger evidence for the single-owner actor than the
  second-hand thread-safety report that originally motivated it — the constraint
  is enforced, not merely advised.

### 4.3 A prediction retired: Awareness

The plan was written believing YDotNet did not support the Yjs Awareness
protocol, and specified building presence as a bespoke message type. **That was
wrong.** The `YDotNet.Protocol` namespace ships the entire protocol:

- `SyncStep1Message`, `SyncStep2Message`, `SyncUpdateMessage`
- `AwarenessMessage`, `QueryAwarenessMessage`, `AwarenessInformation`
- `Encoder` / `Decoder` with varint framing, and read/write extensions

Recorded as a retired prediction because the plan's risk register named this as
an unknown and the answer turned out better than assumed.

**Revised by §4.6.** The consequence first drawn here — that presence would ride
the standard protocol types and Pegasus would therefore speak `y-websocket` on
the wire — did not survive. The serialisation problem in §4.6 pushed the frame
layout to a bespoke binary encoding. What survives is the weaker and still useful
claim: the *payloads* Pegasus carries are ordinary Yjs updates and state vectors,
so a bridge to a `y-websocket` client remains a translation shim at the frame
boundary rather than a change to the document model.

### 4.4 Avalonia.FuncUI 2.0.0 — PASSED

FuncUI 2.0.0 shipped 2026-07-28 and was twelve days old when adopted, which was
the largest scheduled risk in the plan. It restores against Avalonia 12.1.0, and
a `Component` with `useState`, a `DockPanel`, a `TextBox` and a `TextBlock`
compiled on the first attempt.

More usefully, it renders under `Avalonia.Headless`: the spike showed a
`HostWindow`, walked the tree, set `TextBox.Text`, pumped the dispatcher, and saw
the sibling `TextBlock` update through component state. Pegasus can therefore
test its UI without putting a window on anyone's screen.

One mechanism note for the test suite: **FuncUI builds no XAML name scope**, so
`FindControl<T>(name)` throws `"Could not find parent name scope."` Tests reach
controls through `GetLogicalDescendants()` instead.

The plain-Avalonia fallback specified in the plan was not needed and is retired.

### 4.5 A defect found by the property test: colliding client ids

Yjs identifies every operation by `(clientId, clock)`. Two replicas sharing a
clientId therefore mint colliding operation identities, and the merge silently
keeps one side's work and discards the other's. This is precisely the failure
Pegasus exists to prevent, so it is worth stating how close it came to shipping.

`YDotNet`'s parameterless `Doc()` does not draw a client id with anything like
the entropy Yjs assumes. Measured directly:

    2000 documents created -> 16 distinct client ids
    id 48 seen 138 times, id 0 seen 137 times, id 54 seen 121 times

That is roughly six bits. With two peers the collision probability per pairing is
about one in sixteen.

The pathology was then demonstrated rather than argued. Two replicas were forced
to share id 4242, each given a different edit, and synced in both directions:

    a="AAAA"  b="BBBB"   ->  after a full bidirectional sync:  a="AAAA" b="BBBB"

Each side kept only its own edit. No error was raised. `DocumentActor` therefore
always sets an explicit id, and three tests pin the behaviour, including one that
asserts the shared-id case still *diverges* -- if that test ever starts passing,
YDotNet changed and this section needs revisiting.

How it was found is worth recording: the hand-written convergence tests all
passed. The FsCheck property test failed intermittently, roughly one run in
twenty-five, which is what a one-in-sixteen collision looks like through a filter
of small cases. A hand-written suite would not have caught this.

### 4.6 System.Text.Json cannot serialise F# unions

`PeerId` and `NoteId` are single-case unions, and `JsonSerializer` throws
`NotSupportedException` on any F# union. Flat DTO records were tried first and
failed too: records nested in a module are not constructible by the deserialiser,
and `[<CLIMutable>]` did not rescue them.

The wire format is therefore binary -- a tag byte followed by
`BinaryWriter`-framed fields. This is smaller than JSON, has no dependency on a
serialiser's opinion of F#, and keeps the format entirely under our control. The
cost is that the frame layout is now something a human cannot read off the wire,
which is what `Pegasus_Sync.md` §3 exists to compensate for.

### 4.7 A second defect: client ids at or above 2^32 break delta sync

Fixing §4.5 by drawing ids uniformly below 2^53 -- the documented Yjs ceiling, so
a JavaScript peer can hold one exactly -- broke convergence *worse*, and in a way
that initially looked like a bug in our own mailbox.

It is not. With a client id at or above 2^32, `Transaction.StateDiffV1` ignores
the state vector it is given and returns the entire document:

    small ids (36, 22)        forB = 11 bytes of 19   forA = 11 bytes of 26   converged
    large ids (~10^15)        forB = 26 bytes of 26   forA = 41 bytes of 41   DIVERGED

The delta exactly equals the full state, which is the signature of a state vector
that failed to decode. Applying that full state then re-integrates operations the
receiver already held, producing a duplicated document: `"BASE-A"` and `"BASE-B"`
merged into `"BASE-BBASE-A"` rather than `"BASE-A-B"`.

The boundary was bisected and is exact:

    2^28, 2^29, 2^30, 2^31    converged
    2^32 - 1                  converged
    2^32 and above            DIVERGED

This is a 32-bit truncation somewhere in the binding's state-vector path.
`ClientId.ExclusiveMax` is therefore 2^32, giving 32 bits of entropy -- ample
against collision for a handful of peers, and far above the six bits the default
constructor supplies.

Two hypotheses were tested and rejected on the way, recorded so they are not
retried:

- **Native memory lifetime.** The suspicion was that byte arrays returned by
  YDotNet were views over memory freed when the transaction was disposed, and
  that the mailbox let them outlive it. Copying every array inside the
  transaction changed nothing.
- **Zeroed `DocOptions` fields.** Constructing `DocOptions(Id = ...)` might have
  silently defaulted the other options away from the library's intent. It does
  not: `DocOptions.Default` and `DocOptions(Id = 1)` agree field for field
  (`Encoding = Utf16`, `ShouldLoad = true`, the rest false/null), and building
  the options from `Default` reproduced the divergence identically.

The isolating step that mattered was reproducing the divergence with no
`MailboxProcessor` involved at all. Until that ran, the actor was the prime
suspect and the library was assumed correct.

---

## 5. Testing discipline

Everything is headless, including the UI. `Avalonia.Headless` renders a real
control tree without a display, so the window under test is the window that
ships — no window is ever opened on anyone's screen, which is the same rule
`EmuSen.WiseMan` holds for the cores.

The property test earned its place immediately. Both defects in §4.5 and §4.7
were found by randomised interleavings, not by the hand-written cases, which all
passed. §4.5's collision rate of roughly one in sixteen is exactly what an
intermittent property failure looks like through a filter of small examples.

`Caret.adjust` is a pure function for the same reason: the rule for where a
caret belongs after the buffer changed underneath it is arithmetic, and
arithmetic should not need a window to test.

---

## 6. The `.pegasus` note format

One note is one `.pegasus` file. The file is the authoritative replica of that
note on this machine; the `.md` beside it is a projection and is never read back.

### 6.1 Layout

    header   32 bytes
      0..3    magic  "PGSS"
      4       schema version (currently 1)
      5..7    reserved, zero
      8..23   note id, a GUID
      24..31  created-at, Unix milliseconds, little endian

    record   repeated to end of file
      0..3    payload length, uint32 little endian
      4..7    CRC-32 (IEEE) of the payload
      8..     payload: one Yjs update, exactly as ObserveUpdatesV1 delivered it

The schema version is in the header from the first release. This is deliberate
and cheap: the sibling EmuSen project has a documented case where a schema was
edited and silently did nothing to existing files that held real history, because
nothing in the file recorded which schema it was written under. A file that
cannot say what it is cannot be migrated safely.

### 6.2 Why append-only

The alternative is to serialise the whole document on every change. That is
simpler to write and strictly worse to crash inside: a process killed midway
through rewriting a file leaves neither the old contents nor the new.

Appending has the opposite failure mode. A crash mid-write leaves a trailing
record that is short, or whose CRC does not match. On the next open that record
is detected and dropped, and the file truncates to the last byte of the last
intact record. Nothing earlier is at risk, because nothing earlier was being
written. The dropped update is one edit — typically a few characters — and the
document is otherwise whole.

This is the property that lets the durability claim be stated plainly: a crash
costs at most the operation in flight.

### 6.3 Compaction

The log grows without bound, and replaying thousands of small updates at open is
wasteful. Compaction collapses it: the whole document is encoded as a single Yjs
update and written, with a fresh header, to `<name>.pegasus.compacting`. That
file is flushed to physical media, and only then is it renamed over the original.

`rename(2)` is atomic on Linux. A reader therefore sees either the complete old
file or the complete new one, never a blend, and a crash at any point during
compaction leaves a valid file. The temp file is the only thing that can be
orphaned, and an orphan is harmless.

Compaction is not a checkpoint that can be lost. It is a rewrite of information
already durably present.

### 6.4 What `Append` guarantees, and what it does not

`Append` writes the record and calls `Flush()`, which pushes the bytes to the
operating system. It does **not** call `fsync`.

The distinction matters and is worth being exact about:

- **Process crash** — the application is killed, `kill -9`, an unhandled
  exception. Nothing is lost. The bytes are already in the kernel's page cache
  and the OS writes them out regardless of what happened to the process.
- **Power loss or kernel panic** — the tail of the log can be lost, bounded by
  how long the page cache held it.

`Sync()` forces the tail to media and is called on close and when the session
goes idle, not per keystroke. Calling `fsync` on every character typed would make
the editor feel wrong to use, and would buy protection only against the narrower
of the two failures.

### 6.5 The Markdown projection

Beside `ideas.pegasus` sits `ideas.md`, containing the note's current text and
nothing else. It exists so the notes are readable by any editor, greppable, and
committable to a repository if someone wants that.

It is written through a temp file and a rename, so it is never observed
half-written.

It is **never read back**. This is the whole discipline: the moment a projection
becomes an input, editing it out of band silently diverges from the replica, and
the question "which of these two is right?" has no good answer. Pegasus answers
it by construction — the `.pegasus` file is right, always, and the `.md` is
regenerated from it.

If someone edits the `.md`, their changes are overwritten on the next keystroke.
That is a real sharp edge and it is the price of the guarantee.

### 6.6 What this format does not do

No encryption at rest. The file sits in the user's own filesystem under whatever
protection that filesystem provides. Encrypting it would need a key, and a key
needs somewhere to live; that is a larger design than this project has taken on.

No per-author attribution in the file, though Yjs retains it internally and the
log would support extracting it later.

No cross-note transactions. Each note is independent, which is why the workspace
index (`Pegasus_Sync.md` §6) is itself just another note rather than a schema.

---

## 7. Why this is one assembly

Pegasus arrived as four projects — core, transport, application and tests — and
was collapsed to two on the way into EmuSen.

A project boundary has to buy somebody separability they are actually using.
Nothing outside Pegasus consumes its document model or its transport, and no
second frontend is planned over either. Four assemblies bought layering that only
Pegasus itself observed, and the layering survives as file order inside one
project: `Types` before `Codec` before `Crypto` before `Document` before `Store`
before `Workspace` before `Session` before the UI. F#'s compilation order makes
that ordering a compiler-enforced fact rather than a convention, which is most of
what the separate projects were providing.

This is the same reasoning that retired `EmuSen.Crystal` and `EmuSen.Nehellania`
on 2026-08-05, applied before the boundary was paid for rather than after.

`FSharp.Core` enters the solution here. That was the objection that sank the
`F#ascent` branch, and it is answered rather than waved away: Pegasus is a leaf,
so no frontend and no core links it, and the 2.4 MB lands only in this
executable's own publish. The objection was never to F# existing — it was to F#
being on every frontend's dependency path.

---

## 8. Built on LunaP

The window is a `LunaP.Windowing.ToolWindow`, the layout comes from
`LunaP.Fluent.Ui`, and the process starts through `LunaApp.Configure`. Pegasus
therefore inherits the shared theme, remembered window geometry, and the
bootstrap sequence.

That last one is not cosmetic. `LunaApp.Configure` ends with `UseX11()` on Linux
because **`UsePlatformDetect` does not pick X11 on a Wayland session**. Pegasus
was first written outside EmuSen with a hand-rolled
`UsePlatformDetect().WithInterFont().LogToTrace()`, which reproduced three
quarters of `LunaApp` and silently dropped the part that matters on the machine
it was being written on. A shared toolkit earns its keep exactly here: not in the
controls it supplies, but in the corrections already encoded in it.

The first draft also used `Avalonia.FuncUI`, a declarative F# layer over
Avalonia. It was dropped in the move. FuncUI is pleasant and it worked, but a
second UI idiom inside a repository that already has one is a cost paid by every
future reader, and nothing in a notepad needed what it added over `Ui.Row` and
`Ui.Dock`.

### 8.1 Two-way binding wants a re-entrancy guard

The note list and the editor both read from and write to the same state, and the
first version deadlocked the dispatcher on open. `refreshNotes` set
`SelectedIndex`, which raised `SelectionChanged`, whose handler refreshed the
editor, which refreshed the list. `Dispatcher.RunJobs` drains jobs queued while
it is draining, so it never returned.

The fix is two flags — `applying` for the editor, `syncingSelection` for the
list — and a narrower `pullText` that deliberately does **not** touch the note
list. The general rule, and it applies to any LunaP window doing two-way
binding: a control being rewritten from state must not be able to report that
rewrite back as a user action.

Worth recording that the headless UI test caught this and manual clicking would
not have: a human opening the window sees it hang and calls it slow, whereas the
test hangs a build.

---

## 9. Why the tests are a separate project

`EmuSen.WiseMan` is the repository's headless harness and would be the natural
home, but WiseMan is C# and these tests are F#. The convergence property is the
point of them, and CsCheck — which matched FsCheck well enough to help kill the
`F#ascent` branch — cannot express a generator over an F# document model without
the model being C# in the first place.

So `EmuSen.Pegasus.Tests` exists, and it is the second and last name Pegasus
spends. It contains no harness of its own: it is xunit and FsCheck over the same
public surface the application uses.

---

## 10. Sync: pairing, frames, and what the encryption is for

### 10.1 Topology

One peer hosts and the other joins. This describes who opens the socket and
nothing else — it is not a client/server split, and the host is not authoritative.
Both sides hold a complete replica and both persist it (`Pegasus_Format.md`), so
the host disappearing costs the joiner nothing but liveness.

An always-on relay is deferred, not rejected. The session abstraction is shaped so
that a relay is simply a peer that never disconnects; adding one should not
require the protocol to learn a new role.

### 10.2 Pairing

The host displays an address and a join code such as `7-lantern-quartz`. The
joiner types both. The code is drawn from a 32-word list with a leading digit,
giving `9 x 32 x 32` ≈ 9,216 combinations.

That is small, and deliberately so — it is a short-lived code read aloud across a
room or over a phone, and §5 explains what it is actually protecting against. The
words were chosen to be unambiguous when spoken.

The code is case- and whitespace-insensitive, because it will be retyped by hand.

### 10.3 Frame format

A frame is a tag byte followed by a payload:

    0  Hello       peer id, display name, colour  (length-prefixed UTF-8 strings)
    1  SyncStep1   raw Yjs state vector
    2  SyncStep2   raw Yjs update answering a state vector
    3  Update      raw Yjs update
    4  Awareness   peer, then caret and anchor as int32
    5  Bye         no payload

Strings use `BinaryWriter`'s 7-bit-encoded length prefix. Multi-byte integers are
little endian.

The encoding is bespoke rather than JSON because `System.Text.Json` cannot
serialise F# unions and `PeerId` is one; the full account is in
`Pegasus_Design.md` §4.6. The payloads of tags 1–3 are ordinary Yjs bytes, which
keeps a future bridge to a `y-websocket` client a translation at the frame
boundary rather than a change to the document model.

### 10.4 Exchange

On connect, each side sends `Hello`, then `SyncStep1` carrying its state vector.
On receiving `SyncStep1`, a peer replies with `SyncStep2` containing exactly the
operations the other lacks. Thereafter each local edit is broadcast as `Update`.

Because the payloads are Yjs updates, the exchange is idempotent and
order-independent: a duplicated or late `Update` merges to the same document. The
protocol needs no acknowledgements and no sequence numbers, and a reconnect is
just another `SyncStep1`.

`Awareness` is sent on caret movement and is not persisted. Presence is
disposable by nature; a stale cursor is noise, not data.

### 10.5 What the encryption is and is not

Every frame is sealed with AES-256-GCM under a key derived from the join code by
PBKDF2-HMAC-SHA256, 210,000 iterations, with a fixed salt. Each frame carries a
fresh random 12-byte nonce, so no counter has to survive a reconnect. The
handshake is an HMAC challenge/response that proves both sides derived the same
key without putting it on the wire.

**The salt is fixed, and that is a real weakness.** Both peers must derive the
same key from the code alone, with no round trip to agree on a random salt. A
fixed salt means an attacker can precompute against the 9,216-code space. The
iteration count raises the cost of doing so but does not change the shape of the
problem.

So the honest statement of what this buys:

- A machine that stumbles onto the listening port cannot read the notes or inject
  edits without the code.
- Frames cannot be tampered with undetected in transit, because GCM authenticates
  them.

And what it does not buy:

- It is not protection against someone who can watch the pairing happen.
- It is not protection against an adversary willing to spend real compute on a
  9,216-entry keyspace.
- There is no forward secrecy. A recorded session is readable by anyone who later
  learns the code.

This is a pre-shared key for a notepad two people run across a LAN or a private
network. Anyone wanting the stronger property should tunnel Pegasus over
something that provides it — WireGuard or Tailscale — rather than trusting this
layer to be more than it is. Frames are capped at 64 MiB so a hostile length
cannot drive an unbounded allocation before authentication has a chance to fail.

### 10.6 The workspace index

A workspace is a directory of `.pegasus` notes plus `_index.pegasus`. The index is
an ordinary Pegasus document holding a map of note id to display name.

This is the design's one piece of genuine economy. Note creation and rename are
concurrent operations on shared state and therefore need conflict resolution —
and because the index is just another document, they get the same CRDT, the same
file format, the same crash recovery and the same sync path as note text. There
is no second mechanism to write, test, or reason about.

Deletion tombstones the index entry and leaves the file on disk. Nothing the user
authored is removed by the application.

---

## 11. The blank window, and what "agnostic" has to mean to be worth anything

### 11.1 It shipped rendering nothing, and the suite was green

The first published build opened a white rectangle. Every control existed, the
window sized correctly, the socket layer worked, and 53 tests passed.

The cause is one missing line. Every EmuSen `App` includes
`avares://EmuSen.LunaP/Theme/LunaTheme.axaml`, which is `FluentTheme` plus the
shared palette; `Mistress` does it in `App.axaml` and `Hotaru` the same way.
Pegasus, ported from a standalone repo where it had added `FluentTheme` by hand,
overrode `Initialize` **not at all** after the move. Without a control theme,
`TextBox` and `ListBox` have no `Template`, and a control with no template
occupies layout and draws nothing.

**The suite missed it for a reason worth stating.** `LunaTheme.axaml` carries a
comment predicting exactly this — *"the one WiseMan's TestAppBuilder includes
too — a headless render pass that misses the theme silently asserts over
untemplated controls."* The UI tests built their headless `Application` with a
bare `FluentTheme()` instead of LunaP's theme. Every assertion walked the
**logical** tree, which is fully populated whether or not anything is
templated, so the tests were structurally incapable of seeing the defect and
were also loading a different theme than the application. Both halves were
wrong in the same direction.

Two changes, and the second matters more than the first:

- `Shell.applyTheme` is now the single place the style is loaded, and both the
  application and the tests call it. They can no longer diverge.
- A guard asserts every `TemplatedControl` in the window has a `Template` after
  measure and arrange, and it was **made to go red on purpose**: with the theme
  removed it fails naming `TextBox, ListBox`, which is the shipped symptom in
  words. §3.53's rule about guard tests applies here exactly — the green was
  worth nothing until the red existed.

The general lesson for any LunaP consumer: a headless test that queries the
logical tree proves the tree, not the rendering. If what can break is *whether
anything is drawn*, the assertion has to be about templates or pixels.

### 11.2 What agnostic means here

Pegasus is a notepad built on a windowing toolkit. It is not part of the
emulator, and `EmuSen.LunaP` is intended to be published on its own as a general
Avalonia toolkit. Both statements are only true while Pegasus depends on LunaP
and on nothing else in this repository.

That is now a test rather than an intention: `Pegasus references the toolkit and
nothing else of EmuSen` reflects over the assembly's own references and fails
naming anything `EmuSen.*` other than `EmuSen.LunaP`. It reads direct references
deliberately — LunaP's own dependencies on Galaxia and Cauldron are LunaP's
business and will be settled when it is packaged, not Pegasus's.

Agnostic also means the three RIDs the project publishes are real targets rather
than aspirations, and one defect was found by taking that seriously:
`defaultWorkspaceRoot` was `~/.local/share/pegasus/workspace`, built from a
literal `.local/share`. That is a Linux convention, wrong on Windows and
unidiomatic on macOS, in a project whose `out/` has carried `osx-arm64`,
`osx-x64` and `win-x64` since before Pegasus existed. It resolves through
`SpecialFolder.LocalApplicationData` now.

Changing that path could have stranded an existing workspace, so it does not: an
existing directory at the old location keeps being used. This mirrors
`ConfigStore`'s own Directory / PreviousDirectory / LegacyDirectory order rather
than inventing a second migration idiom — and it is the same rule the ROM
library gets, which is that data a user authored is read, never quietly
abandoned.
