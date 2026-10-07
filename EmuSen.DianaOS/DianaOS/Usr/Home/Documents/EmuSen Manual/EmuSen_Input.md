# EmuSen input — one pad type, every console

*This revision: initial. Written when `SnesButton` was pulled out of the frontends and replaced by `EmuSen.Cores.PadButton`, at the point Moon (NES) became the second real core.*

Companion docs: `EmuSen_Multicore.md` (how a ROM path picks a core at all), `EmuSen_Settings_Reference.md` §4 (the rebind window and how bindings persist), `Hardware/Nintendo/Venus - SNES/Venus_CPU.md` and `Hardware/Nintendo/Moon - NES/Moon_Memory.md` (what each console's controller hardware actually does with a button press).

---

## 1. Why this exists

`EmuSen.Nehellania` is described everywhere in this project as the core-agnostic input layer. It was not. It read:

```csharp
using EmuSen.Cores.Nintendo.Venus.Controllers;
...
public Dictionary<SnesButton, SDL.GamepadButton> ButtonToPad { get; private set; }
```

Every layer above the core was typed on `SnesButton`: `GamepadBindingMap`, `GamepadManager`, `HotaruKeyMap`, `ControllerKeyMap`, `InputSettingsWindow`, `FrameRunner`, the `--commands` script verbs. So did `gamepadbindings.json` and `keybindings.json` on disk. The generic-looking assembly boundary was cosmetic — the SNES core's namespace was imported by name into the input layer, the settings UI, and the headless test harness.

`ICore` originally *declined* to abstract input, with a comment saying so. That was a defensible call when there was one core: forcing the whole binding stack generic was a bigger job than that interface's purpose, and doing it speculatively would have been guessing at what a second console needed. The judgement was reversed the moment a second core existed and the guess stopped being a guess.

The cost of the reversal was small — see §3 for why, which was luck as much as design.

---

## 2. The contract

Two members on `ICore`, both defaulted so a core with no input modelled implements nothing:

```csharp
IReadOnlyList<PadButton> SupportedButtons => Array.Empty<PadButton>();
void SetButton(int port, PadButton button, bool pressed) { }
```

`PadButton` (`EmuSen.Galaxia/Input/PadButton.cs`) is the **union** of the buttons this project's consoles have, not the intersection:

```
B  Y  Select  Start  Up  Down  Left  Right  A  X  L  R
```

A union, because the alternative is a per-core button enum, and then every layer above the core is generic over a type parameter it cannot name — which is what the binding files, the rebind UI grid and the `--commands` `hold`/`tap` verbs would all have had to become. A flat union costs one thing: **a core must ignore a button it does not have.** Moon's `SetButton` maps through a `switch` that returns `NesButton?` and drops `null`:

```csharp
NesButton? mapped = button switch { PadButton.A => NesButton.A, ..., _ => null };
if (mapped is { } nesButton) SetButton(port, nesButton, pressed);
```

Explicitly *not* remapping X onto A or L onto B. A binding for a button the console lacks does nothing, which is the only behaviour a user can predict.

`SupportedButtons` exists so that a UI can show a real pad instead of the union. Venus returns all twelve; Moon returns the eight the NES pad has. §5 covers what consumes it.

**`port` is 0-based on this interface.** Venus's own `Input.SetButton` takes a 1-based port, so `VenusCore.SetButton` shifts by one on the way in. Moon indexes `Controller1`/`Controller2` directly. Callers that already thought in 1-based controller numbers (`FrameRunner.Hold`, the `tap2` verb) subtract one at the boundary rather than pushing the console's convention outward.

---

## 3. Why the names match `SnesButton` exactly

`PadButton`'s members are declared in the same order, with the same names, as `Venus/Input/Input.cs`'s `SnesButton`. That is deliberate and it bought two things:

**Nobody lost their bindings.** `EmuSen.Galaxia` persists enums **by name**, not by ordinal (see `EmuSen_Config_Reference.md`). An existing `keybindings.json` holding `{"Up": "Up", "B": "Z", ...}` deserialises into `Dictionary<PadButton, Key>` unchanged, because every key string still resolves. Retyping the whole binding stack needed **zero** config migration code and no version bump on either file. Had the names differed by even one member, this refactor would have shipped a migration step and a "your bindings were reset" notice.

**`VenusCore.SetButton` is a cast, not a table.**

```csharp
Bus?.Input.SetButton((Controllers.SnesButton)button, pressed, port + 1);
```

That cast is only correct while the two enums agree **ordinally**, which is a stronger condition than agreeing by name — reordering `PadButton` alone would silently send Start where Select was meant, with no compiler complaint and no exception, just wrong input. It is therefore pinned by a test rather than by this paragraph: `EmuSen.WiseMan/Cores/PadButtonTests.cs` asserts member-for-member that the two enums have the same names in the same positions and the same count. **If you add a button to `PadButton`, add it to the end, or that test will tell you what you broke.**

> **Update 2026-09-18:** four were added at the end (§7.1), and the test changed from "the two enums are equal" to
> what the cast actually needs: SNES's twelve are `PadButton`'s *first* twelve, in order. Venus now lists its twelve
> explicitly and drops anything past `R` before the cast — without that, a press of L2 indexed past Venus's bit table
> and threw, which `PadButtonTests.Venus_drops_a_button_past_its_twelve` demonstrates when the guard is removed.

### Where the type lives

**`EmuSen.Galaxia`, not the core.** Galaxia is the config layer and a true leaf — no project references at all — and `PadButton` is the key type of two config files, so Galaxia already owned its persisted form. Putting the type itself there costs **no new assembly edges**: `EmuSen`, `EmuSen.Nehellania`, `EmuSen.Hotaru` and `EmuSen.Mistress` all referenced Galaxia already.

What that bought is §4: `EmuSen.Nehellania` could finally drop its reference to the core assembly and become a leaf, matching `EmuSen.Serenity`. (See `EmuSen_Audio_Sync.md` §7.1 for the audio half of the same move.) A new shared-contracts assembly was the alternative, and was rejected as heavier than the problem.

The SNES pad being a superset of the NES pad is why the union is currently just `SnesButton` renamed. That will stop being true — a Sega pad's C button, a PlayStation pad's second shoulder pair — and when a new member is added, it goes at the end and Venus's cast stays valid.

---

## 4. The stack above the core

Nothing below names a console:

| Layer | Type | Persists to |
| --- | --- | --- |
| `EmuSen.Endymion/Input/GamepadBindingMap` | `Dictionary<PadButton, SDL.GamepadButton>` | `gamepadbindings.json` |
| `EmuSen.Endymion/Input/GamepadManager` | polls SDL3, emits `PadButton` | — |
| `EmuSen.Mistress/Input/ControllerKeyMap` | `Dictionary<PadButton, Avalonia Key>` | `keybindings.json` |
| `EmuSen.Hotaru/Input/HotaruKeyMap` | `IReadOnlyDictionary<PadButton, Key>` | — (fixed defaults) |
| `EmuSen.Pharaoh/FrameRunner` | `Dictionary<(PadButton, int Controller), bool>` | — |

**The gamepad files live in `EmuSen.Endymion` as of 2026-08-05.** They were `EmuSen.Nehellania`'s entire contents; that project was folded into Endymion and deleted, because the two wrapped the same two SDL3 packages and no consumer ever took one without the other (`EmuSen_Multicore.md` §9.2). Endymion references **only** `EmuSen.Galaxia` (plus SDL3), and `EmuSen.WiseMan/Common/LeafAssemblyTests.cs` pins that reference set so it cannot quietly regrow.

Nothing about the input stack itself changed in the fold — same three types, same namespace shape (`EmuSen.Endymion.Input`), same `gamepadbindings.json`. The rebind window and both frontends only changed a `using`.

The two key maps stay separate on purpose — Hotaru has no settings UI to drive a rebind, so it carries fixed defaults that `EmuSen.WiseMan/Input/HotaruKeyMapTests.cs` asserts are table-equal to Mistress's defaults. Neither imports a core.

`GamepadBindingMap`'s defaults bind by physical button **position** (`SDL.GamepadButton.South`), not by printed label, so a Nintendo-layout and an Xbox-layout pad both put the same physical button under the same thumb. `EmuSen_Settings_Reference.md` §4.6 covers that and the pad-label lookup.

Both maps enforce that two console buttons never share one key: `Rebind` unbinds the previous owner. That check runs over the **whole** map, not just the displayed subset, so binding Select→S while an NES ROM is loaded still takes S away from X even though X is not on screen (§5).

---

### 4.3 `Input/DefaultPadKeyMap`

The keyboard scheme both frontends start from — arrows for the d-pad, `Z`/`X`/`A`/`S` for B/A/Y/X, `Q`/`W` for the shoulders, `Enter`/`RightShift` for Start/Select — plus the `Key → PadButton` reverse lookup they both need.

It was spelled twice, in Hotaru's `HotaruKeyMap` and Mistress's `ControllerKeyMap`, with two copies of the reverse-lookup loop and a test asserting the two tables stayed equal — a problem being guarded rather than fixed.

`Bindings()` returns a **fresh dictionary per call**, not a shared readonly instance: Mistress rebinds into its copy, and a shared instance would leak one frontend's edits into the other.

**It lived in `EmuSen.LunaP` first, and moving it here is what let the toolkit leave the repository** (`EmuSen_LunaP.md` §19). LunaP was chosen originally for a reason that was true and beside the point: it was the one project referencing both Avalonia and Galaxia. But a general Avalonia toolkit has no business naming a console gamepad button, and this project already owns the mapping of physical input onto `PadButton` — `GamepadBindingMap` is the same subject from the SDL3 side. Holding it cost one `Avalonia` base package reference. Not `Avalonia.Desktop`, not the themes: nothing in this project draws anything, and `Avalonia.Input.Key` is the whole of what it needs.

Galaxia still cannot hold it, and that part of the original argument stands: Galaxia is a leaf with no references at all, because DianaOS references it and the core references DianaOS, so anything it depended on upward would close a cycle. It cannot name `Avalonia.Input.Key`.

## 5. The rebind window shows the loaded console's pad

`InputSettingsWindow` used to iterate `Enum.GetValues<PadButton>()` and build twelve rows unconditionally — so with an NES ROM loaded it offered to rebind X, Y, L and R, four buttons that core drops on the floor.

It filters to the console's own pad instead. The button list comes from `CoreCatalog.ButtonsFor(console)`, which reads each core's `PadButtons` **static** — `VenusCore.PadButtons` and `MoonCore.PadButtons`, the same arrays their instance `SupportedButtons` returns. It has to be static because the window lists every console's pad whether or not a ROM is loaded, and restating the lists in the catalog is how they would drift from what the core actually reads. `ConsoleBindingsTests` pins the two against each other.

## 5.1 One tab per console, and bindings that do not collide

Filtering alone was not enough. The maps used to be **shared** across cores: one `PadButton -> Key` dictionary, so NES `A` and SNES `A` were by definition the same key, and the window could only ever show the loaded console's slice of it.

The window is now a `TabControl`:

- **General** comes first, and holds everything that is not a console's pad — emulator hotkeys, the gamepad status/stick options, and the Player 2 mirror. These are genuinely global, which is why they are not repeated per console.
- **One tab per console after it**, ordered by `CoreCatalog.ConsolesInReleaseOrder` — grouped by manufacturer, then oldest console first, so NES precedes SNES. Adding a core adds a tab with no XAML change; `CoreDescriptor` carries `ConsoleName`, `Manufacturer` and `ReleaseYear` for exactly this.

Bindings are per console. `ControllerKeyBindings` and `GamepadBindings` each own one map per console and are the only things that touch the file; `ControllerKeyMap`/`GamepadBindingMap` no longer have `Load`/`Save` of their own. `keybindings.json` and `gamepadbindings.json` are now nested one level, keyed by console name (`EmuSen_Config_Reference.md` §3.6).

The console key is `CoreDescriptor.Console`, which is deliberately the same string as `ICore.CoreName` — `"NES"`, `"SNES"`. `MainWindow` assigns one straight to the other (`_activeConsole = _session.CoreName`), so a test pins that they agree.

Consequences worth knowing:

- **Two consoles may share a key, and that is not a conflict.** Both pads default to Z/X for B/A. Conflict detection runs per console; a clash is only reported between buttons *on the same tab*, or between a button and a hotkey.
- **Hotkeys stay global, so they still police every console.** Binding a hotkey to a key some console's button uses clears it on **all** of them, not just the visible tab — a hotkey has no console to be scoped to.
- **Rows are built once, at window construction.** Loading a different ROM while the settings window is open does not re-lay it out, and does not move the selected tab. Closing and reopening it does.
- **The window opens on the loaded console's tab**, or on General when no ROM is loaded, since then there is no console to prefer.
- **Which console is live follows the ROM.** `MainWindow` re-points `GamepadManager.Bindings` on every `LoadRom`, and Hotaru does the same in `SwapCore`. Before the first ROM, the first console in catalog order stands in.

---

## 6. What this does not cover yet

- ~~**Multitap, and ports beyond two.** `SetButton`'s `port` is an `int` rather than a two-valued enum specifically so this can grow, but no core models more than two controllers and no frontend offers to configure one.~~
  **Ports beyond two closed 2026-10-04 by §8**: the N64's four ports are played, every core says how many it has
  (§8.1), and Preferences configures which pad is which player. The multitap and the Four Score remain open (§8.10).
- ~~**Analog axes.** `GamepadManager` converts a stick to a d-pad (`AnalogStickAsDpad`, with a deadzone) and there is no analog value anywhere in the contract. A console with a genuinely analog stick needs a real addition here, not a fudge through `PadButton`.~~ **Closed 2026-09-18 by §7**, which made that real addition — `PadAxis` and `ICore.SetAxis` — and kept the stick out of `PadButton`, as this bullet asked.
- ~~**A second pad as player 2.** Since 2026-09-26 `GamepadManager` opens every connected pad and the interface reads
  them all (`EmuSen_Settings_Reference.md` §4.61), but a game reads only the first opened, player 1's, as it read the
  only pad before. Routing a second pad to port 1 is the work `EmuSen_BigPicture.md` §21.5's Q22 (b) left here: it
  reaches every game, and needs a rule for which pad is which player when pads come and go, which the frontend's
  "first opened" rule does not have to settle. `MirrorPlayer1ToPlayer2` (§5.1) is unrelated and unchanged.~~
  **Closed 2026-10-04 by §8.** The rule asked for is §8.2, its edge cases §8.3, and the route to a port §8.5. The
  mirror is unchanged in meaning; with a real player 2 it is or'd with player 2's pad rather than replacing it (§8.5).
- **Non-pad peripherals.** Light guns, mice, the SNES multitap's own protocol. All are console-specific hardware that would attach to the core, not to this interface.
- **Per-core default bindings.** Defaults are one table shared by every core. An NES ROM gets the SNES-shaped defaults for the eight buttons it has, which happens to be right (Z/X = B/A), and is luck rather than design.

---

## 7. The generic controller template

*Added 2026-09-18, when Mars (N64) became the first core with an analog stick, with dual-stick cores planned after it.*

**The pad every core is written against is a modern dual-analog controller**: the twelve SNES buttons, two more
shoulder pairs' worth of buttons, two sticks and two triggers. A core takes what its console has from that template
and ignores the rest — the rule §2 already set for buttons, now extended to axes.

### 7.1 It is libretro's RetroPad, which `PadButton` already half was

`PadButton`'s twelve members turned out to be in exactly the order of libretro's RetroPad — `B, Y, Select, Start, Up,
Down, Left, Right, A, X, L, R` are its joypad ids 0 to 11 — because both inherited the SNES pad. The RetroPad goes on
with **`L2, R2, L3, R3`** (12 to 15) and a left and a right analog stick (`libretro-common/include/libretro.h`,
`RETRO_DEVICE_ID_JOYPAD_*` and `RETRO_DEVICE_INDEX_ANALOG_LEFT/RIGHT`, read in the local RetroArch checkout). It is the
template every libretro core maps its console onto, dual-stick consoles included. So finishing it was a smaller and
better-anchored decision than designing a template: the four buttons are appended in RetroPad's order, and nothing
before them moves (§3).

| layer | type | what it holds |
| --- | --- | --- |
| buttons | `PadButton` | the RetroPad's sixteen: the SNES twelve, then `L2 R2 L3 R3` |
| analog | `PadAxis` | `LeftX LeftY RightX RightY LeftTrigger RightTrigger` — SDL3's six gamepad axes |
| bindable | `PadControl` | every button under its own name and number, then the eight stick directions |

**Axis values are normalised and console-free**: a stick runs −1 to 1 with **right and down positive**, a trigger 0 to
1 — the RetroPad's own convention (*"the Y axis being positive towards the bottom"*, `libretro.h`) and SDL's, so a value
passes from the pad to the core without a sign anyone has to remember. A console whose stick reports up as positive,
like the N64, turns it over in its own `SetAxis`.

> ~~"a stick runs −1 to 1 with right and **up** positive … Up is positive because that is what a console's stick
> reports."~~ **Retired the same day, before it shipped.** It generalised from the N64, whose stick does report up as
> positive, to consoles in general; the PlayStation's DualShock reports the other way. With no convention among
> consoles to follow, the template follows the RetroPad it is, and the core that disagrees does the flip.

### 7.2 The contract, and why the stick is not a button

`ICore` gains two members, both defaulted, so a digital-pad core implements nothing:

```csharp
IReadOnlyList<PadAxis> SupportedAxes => Array.Empty<PadAxis>();
void SetAxis(int port, PadAxis axis, double value) { }
```

§6 asked for exactly this — a real analog path, not a stick smuggled through `PadButton` — and the template keeps to
it: **a core only ever sees real buttons and real axes.** A keyboard still has to be able to push a stick, so the
*binding* layer, and only it, has a wider type. `PadControl` is `PadButton`'s sixteen under the same names and the same
numbers, followed by the eight stick directions (`LeftStickUp` … `RightStickRight`). A key bound to a direction pushes
that axis all the way; the two directions of one axis cancel and leave the stick's own reading; a key or pad button
holding L2 or R2 counts as that trigger pulled all the way. That rule is `PadControls.Resolve`, in Galaxia, so both
frontends share one copy and the tests reach it without a window.

The alternative — stick directions as `PadButton` members — would have made every core receive, and have to ignore,
"buttons" that are not buttons, which is the fudge §6 warned against, moved rather than avoided.

### 7.3 What the frontends do each frame

Both Mistress and Hotaru keep their keyboard state per `PadControl` and their gamepad state per `PadButton`. A button
goes to `SetButton` as before. **Every axis the loaded console reads is resolved and sent on every gamepad poll**, not
only on a change, because a stick moves continuously without crossing anything a change could be detected against; a
key press or release re-sends them as well.

- **The gamepad's sticks and triggers are fixed sources**: left stick → `LeftX/LeftY`, right stick → `RightX/RightY`,
  triggers → the trigger axes. A trigger past half its travel also counts as the L2 or R2 button, since SDL has no
  button for it. Stick clicks are ordinary bindable buttons, L3 and R3.
- **A console that reads the left stick as a stick stops getting it as a d-pad.** `AnalogStickAsDpad` still turns the
  left stick into d-pad presses for a digital console; `GamepadManager.LeftStickIsAnalog`, set whenever a ROM loads,
  switches that off for a console like the N64, where the stick and the d-pad are different things.
- **A small dead zone**, a tenth of the travel, keeps a pad at rest from drifting. It is separate from the d-pad
  threshold `StickDeadzone`, which answers a different question.
- **In the rebind window, a stick direction is a row** with a keyboard column like any button, and a gamepad column
  that names the stick and cannot be rebound — there is no pad *button* to bind it to (`EmuSen_Settings_Reference.md` §4).

### 7.4 Default keys, and files written before the template

The twelve new controls default to keys no existing default and no hotkey in either frontend holds: **E and R** for L2
and R2 (completing Q-W-E-R along the shoulders), **C and V** for L3 and R3, **I-J-K-L** for the left stick and
**T-F-G-H** for the right. A test pins that every control has a key and none collides with a hotkey.

**A binding file written before the template cannot mention the new controls**, and on load each console's map is
*replaced* by what the file holds — so without help, everyone who had ever saved their bindings would have no keys for
any of them. The absence cannot mean "unbound on purpose", because the file predates them. So a control from `L2` on
that the file does not mention **takes its default key, unless something in that map already uses the key** — a key
the player gave to something else stays theirs, and the new control waits to be bound. The gamepad file gets the same
rule for the new stick-click defaults. The one case this cannot tell apart is a file written *after* this change in
which a user deliberately cleared a new control; it gets its default back on the next load. That is the price of not
changing the file format, and it is recorded rather than engineered away.

### 7.5 What this does not cover

- **Per-console defaults.** Still one table for every console (§6). The N64 gets the stick on I-J-K-L and its C buttons
  on the right-stick keys, which is usable and not what a dedicated N64 layout would choose.
- **Analog input in scripts.** `FrameRunner`'s `hold`/`tap` verbs take `PadButton` names; a headless script cannot yet
  push an axis. Tests reach `SetAxis` directly.
- **Rebinding a gamepad axis.** Sticks and triggers come from their fixed sources; only buttons are rebindable on a pad.
- **Pressure-sensitive buttons**, rumble, and a second analog pair beyond what SDL calls a gamepad.

---

## 8. Local multiplayer: a pad for each player

*Added 2026-10-04. Until this section a game heard one pad, player 1's, however many were connected (§6, and
`EmuSen_Settings_Reference.md` §4.61). Here a second, third and fourth pad play as players 2, 3 and 4, up to as many
controller ports as the running core has.*

The design separates three questions that the single-pad code never had to tell apart: **how many ports the game
has** (a fact about the core, §8.1), **which pad is which player** (a rule about devices that come and go, §8.2 and
§8.3), and **how a player's input reaches a port** (routing, §8.5). Each is answered once, below the frontends, so that
Mistress and Hotaru cannot disagree.

### 8.1 Ports per core

`ICore` gains `int ControllerPorts => 1`, defaulted like the rest of the input contract, and every core in the build
answers it. `EmuSen.Cores.ControllerPorts.Of(core)` is what a frontend asks; `ControllerPorts.ForConsole(console)` gives
the count with no game loaded, for the bindings window.

| Console | Engine | Ports | Source of the count | Players 2-4 reach the game |
|---|---|---|---|---|
| NES | Moon (C#) | 2 | `MoonCore.Ports` | player 2 |
| NES | MoonRT (Rust) | 2 | `MoonCore.Ports`, the shim | player 2 |
| SNES | Venus (C#) | 2 | `VenusCore.Ports` | player 2 |
| SNES | VenusRT (Rust, ABI v1) | 2 | machine info's `ports` (`EmuSen_CoreAPI.md` §6.4) | player 2 |
| Game Boy | Mercury (C#), MercuryRT (Rust) | 1 | the console | none, by the hardware |
| N64 | Mars (C#), MarsRT (Rust) | 4 | `MarsCore.Ports` | players 2, 3 and 4 |
| Genesis | Nephrite (Rust, ABI v1) | 2 to 8, by its two port settings | machine info's `ports`; with no game loaded, the console's pack (§8.11) | players 2 to 8 *(added 2026-10-07)* |

**A v1 engine is read from its descriptors, not from a table.** `CoreEngine` lives with the ABI host and does not
override the member; `ControllerPorts.Of` reads the highest port that machine info names a controller for, and, where
machine info names none, the ports its system's controllers list (`EmuSen_CoreAPI.md` §6.3). VenusRT's info names
ports 0 and 1, both `snes.pad`. A one-line `ControllerPorts` on `CoreEngine` itself would make the special case
unnecessary, and is left to the ABI host's owners.

**Three engines folded every port past the last onto the last.** The contract always said `port` is 0-based and an
`int` so that it could grow (§2, §6), but no frontend had sent anything past 1, and three cores relied on that:

- Moon chose `Controller1` for port 0 and `Controller2` for *any other* port;
- MoonRT's shim mapped port 0 to 0 and every other port to 1, mirroring Moon;
- Venus passed `port + 1` to its `Input`, which treats 1 as pad 1 and *any other* number as pad 2.

So a third player's presses would have landed on player 2's controller. This was predicted from reading the three
`SetButton`s, then demonstrated before it was changed: `ControllerPortTests.The_nes_second_pad_reads_port_1_alone`
(both engines) and `The_snes_second_pad_reads_port_1_alone` failed with the presses of ports 2 and 3 visible in the
game's own reads of `$4017` and `$421A`, and pass since each core drops a port it does not have. Mercury, MercuryRT,
Mars and MarsRT already dropped such ports. The ABI host's `NativeRtCore.PortFor` has the same clamp as its default;
both shims that derive from it override it, so nothing reaches it, but the default is worth changing where that file is
maintained. The router (§8.5) never sends past `ControllerPorts` either, so the fix is defence in depth rather than the
only guard.

### 8.2 Which pad is which player: the rule

Stated as a player would be told it:

1. **Pads take players 1, 2, 3, 4 in the order they connect.** The pads present when Mistress starts are seated in the
   order SDL lists them.
2. **A pad that is unplugged keeps its number.** Nobody moves up into it, and the other players keep theirs. A notice
   says "Controller disconnected: *name* (Player 2)", and the game is not paused.
3. **A pad that comes back gets its number back.** It is recognised by SDL's GUID for the device (vendor, product,
   version and bus, sixteen bytes) and by its device path.
4. **A new pad takes the lowest number that has no pad connected** — counting a number kept for a pad that is gone.
5. **The keyboard is player 1's** until the player gives it to another player (§8.3).
6. **Preferences ▸ Controllers shows which pad is which player and changes it** (§8.9); choosing a number that another
   pad holds trades the two.

`PlayerSlots` (Endymion) is the whole rule; `GamepadManager` seats a pad as it opens it and keeps the seat when it
closes it. A seat holds the closed `ConnectedPad`, whose GUID and path were read when it was opened, so a reservation
needs no copy of the device's identity.

**Matching, in order:** a seat reserved for a pad with the same GUID *and* path; else one reserved for the same GUID;
else the lowest seat without a connected pad. A path is the more specific of the two and is tried first; a GUID alone
still finds a pad that comes back through another USB port, which changes its path.

**Rule 4 is a choice between two readings of "reserved", and it was decided against the stricter one.** A strict
reservation — a new pad never takes a seat held for another — keeps a returning pad's number even when a stranger
connected meanwhile, but it fails the commoner case badly: a single player whose pad's battery dies and who picks up a
different pad would find that pad seated as player 2, which a one-player game does not hear, and would have to open
Preferences mid-game to play at all. The same failure follows a pad moved from Bluetooth to a cable, since the bus is
part of SDL's GUID. Under rule 4 the spare pad is player 1. What the strict rule would have protected — a returning pad
whose seat a newcomer took — is the rarer event, and its cost is a number one higher, which Preferences repairs. So the
reservation protects against renumbering, and against nobody's pad but the player's own.

RetroArch's documentation describes its default as assigning a newly connected device to the first user without one,
with an optional per-port reservation; ES-DE's user guide assigns no players at all (its "Input device settings" govern
only which pad steers the interface, §4.61). The rule here is RetroArch's default plus the keeping of a disconnected
pad's number.

**What is remembered, and for how long.** The seats last until Mistress quits; nothing is written about a pad, as §4.61
of the settings reference already decided. Remembering seats between sessions would belong in Mistress's SQLite
database (`EmuSen_Stack.md` §4) and is not done.

**Each pad is told its number** through `SDL_SetGamepadPlayerIndex`, which lights the player LED of a pad that has one
(a DualShock 4's light bar, a Switch Pro's lamps) and is ignored by the rest.

### 8.3 Edge cases, decided

- **Two identical pads.** They share a GUID; their device paths differ, and that is what tells them apart when both go
  and come back in either order (`PlayerSlotsTests.Two_identical_pads_are_told_apart_by_where_they_are_plugged_in`).
  Where SDL reports no path — its virtual joysticks have none, measured in `SdlVirtualPadsTests` — the lowest seat
  reserved for the model is taken, so two identical pathless pads that both drop may come back swapped. In Preferences
  two identical pads carry a number after the name, and pressing a button on one lights its row.
- **A pad that registers twice** (a Bluetooth pad seen both directly and through Steam Input's virtual pad): the twin
  is seated as the next player and every press would reach two ports. The remedy is the existing *First Controller*
  switch (`FirstControllerOnly`), which now quiets the game's other players as well: with it on the game hears player
  1's pad alone, which is exactly what it heard before this section, and the interface the first pad alone, as before.
  Detecting a twin automatically (same name within the same moment) was considered and rejected as a guess that would
  also catch two genuinely identical pads.
- **More pads than ports.** A pad past the console's ports is still seated (an SNES's third pad is player 3) and still
  steers the interface, but the game never hears it; Preferences says so on its row. Changing to a four-port game
  makes it heard without re-seating.
- **More pads than seats.** There are eight seats (`PlayerSlots.MaxPlayers`), room for the four-port N64 and the
  multitaps a later core might model. A ninth pad is no player and still steers the interface.
- **The keyboard.** `AppSettings.KeyboardPlayer`, default 1, is the player the keyboard plays as, beside that player's
  pad (their presses are or'd, as the keyboard and the one pad always were). One keyboard map per console is kept, not
  one per player; two players on one keyboard is not offered (§8.10). Hotaru has no setting and keeps the keyboard on
  player 1.
- **A pad unplugged mid-game.** Its buttons are let go on its port at the next poll, since a closed pad reads nothing
  held; the port stays plugged in while the seat is kept (§8.6). Nothing pauses: no existing setting asks for a pause
  on a disconnect, and ES-DE and RetroArch pause on none by default either.

### 8.4 Bindings per player

Every player has a pad map per console, and **a player whose map was never changed plays with player 1's**, so a
change to player 1's reaches every such player. Changing a player's map gives it its own, begun as a copy of player 1's
(`GamepadBindings.Own`), which is then independent; *Use Player 1's* drops it again.

**The file gains keys and renames none.** `gamepadbindings.json` keeps its console-keyed maps as player 1's, and a
player's own map is written beside them under `"SNES Player 2"`. A build from before players reads that key as a
console it does not know, keeps the map, and writes it back unchanged (`PlayerBindingsTests.A_players_key_is_kept_by_a_reader_that_knows_only_consoles`);
a file from before players has no such key, so every player plays with the console's map, which is what §7.4's rule
for files predating a change asks: absence means "as before". The flat file of §5.1 is inherited the same way.

The keyboard file is unchanged, for the reason in §8.3.

### 8.5 The route from a pad to a port

`PortRouter` (Endymion, beside `GamepadManager`, so it names no core: it calls the frontend's `SetButton`, `SetAxis`
and `SetControllerConnected`) replaces the two copies of the per-poll code that Mistress and Hotaru had. On each poll it
reads the pad of every player the game has a port for, through that player's bindings, and then for each port:

- a button goes to the core when it changes, as before; a port hears its player's pad, the keyboard if it is that
  player's, and on the second port player 1 as well while `MirrorPlayer1ToPlayer2` is on (§5.1 there; or'd with player
  2's own pad rather than replacing it);
- every axis goes on every poll, as §7.3 requires; with the mirror, the stick pushed furthest of the two;
- nothing is sent to a port the game does not have.

A key change re-sends from the pads as the last poll read them, as the frontends did, so a menu over the game that
keeps the pads from the game (§4.29 of the settings reference) still keeps them when a key is pressed. A new game resets
what was sent, since its core starts with nothing held.

### 8.6 A port with a controller in it

**The N64 tells an empty port from a full one, and its games look.** The joybus answers a status or state command only
for a controller that is present (`Mars_Serial.md` §3.1); both Mars engines started with port 0 present and ports 1-3
empty, and nothing could change that. Pressing buttons on port 2 would have reached a controller the game believes is
not there. `ICore.SetControllerConnected(port, connected)` is the addition, defaulted to nothing for the consoles whose
games cannot tell; Mars sets `Present`, and MarsRT gains `mars_machine_set_present`, an additive export the shim
tolerates the absence of.

**A port past the first holds a controller while its player has a pad seated — connected or kept — or the keyboard.**
Kept, so that a wireless pad dropping for a second does not pull the controller out from under the game; a game that
reacts to a removed controller sees one only when the seat is given up. Port 0 is never unplugged, so a one-player
session's machine is exactly as it was: the probe baselines of `Mars_Gpu.md` and the like, which run one player, are
unaffected by construction. The router sends the flags on every poll rather than on change, because a state loaded
carries the `Present` it was saved with (`Present` is part of `Controller`'s state in both engines).

### 8.7 Scripts and the shell

`FrameRunner.Hold` and `Release`, and the `--commands` `hold`/`release` verbs, already took a controller number. `tap`
and `tap2` gain `tap3` to `tap8` (`CommandsScriptRunner.TapController`). **The DianaOS shell has no input command**:
input reaches a core from a frontend's pads and keys or from Pharaoh's script verbs, and no shell command presses a
button, so there is nothing there to give a port. A `press` command would be the place, and would take the controller
as these verbs do.

### 8.8 Tests, and what was predicted

No physical pad is needed. Two harnesses stand in: `SimulatedPads`, the device layer's fake, now with a GUID (an MD5 of
the name unless set, so pads of one name share one, as one model does) and a path per pad; and **SDL's own virtual
joysticks** beneath the real `SdlPadDevices`, filtered to the ones the test attached, so that SDL's GUIDs and paths are
the measured ones.

| Test | What it shows |
|---|---|
| `PlayerSlotsTests` (12) | the seating rule, reservations, identical pads with and without paths, trading and *None*, the ninth pad, the first controller alone, per-player bindings, the player LED, a new device set |
| `PortRouterTests` (9) | each player's pad to its port, nothing past the last, unplugging mid-game, the keyboard's player, the mirror, axes per port, the plugged-in flags, a reset |
| `PlayerBindingsTests` (10) | player 1's map until changed, own maps, the file's added keys in both directions, files from before players, reset |
| `ControllerPortTests` (8) | every engine's count; port 1 reaching the NES's `$4017` and the SNES's `$421A` on both engines of each, ports 2-3 reaching nothing; the N64's four ports in Mars and, by state against Mars, in MarsRT |
| `SdlVirtualPadsTests` (1) | three SDL virtual pads seated 1-3, the identical pair sharing a GUID, player 2's press on port 1, a pad detached and a same-model pad attached taking its seat |
| `MultiplayerTests` (4) | through Mistress's own poll: a second pad on the SNES's second port, its hot-unplug and replug and notices, no pause; player 1's pad going and a spare taking seat 1; the keyboard as player 2; four pads on the N64's four ports in a big-screen session, and a fifth heard by none |
| `FrameRunnerTests` (+2) | the `tapN` verbs; `hold` on controller 2 reaching the SNES's second pad |
| `PlayerPreferencesTests` (4) | the rows' names and players, trading, None, the keyboard's player saved; a press lighting its row alone; a kept seat and Forget; the big-screen rows changed and lit by pad alone, and fitting |
| `PlayerBindingsWindowTests` (3) | each console's player count; a rebind for player 2 making its own map and leaving player 1's, the drawing lighting player 2's pad, Use Player 1's; the selector reached by pad |
| `WindowFitAuditTests` (+4, and the bindings window's existing 4 cases on every tab) | Preferences ▸ Controllers with four pads, two of one 80-character name and a kept seat, on the desktop and in a big screen, at 1280 by 800 and 1920 by 1200 |

**Predictions recorded with the tests, before they were run**, and how they fared:

- *Ports 2 and 3 land on controller 2 in Moon, MoonRT and Venus, and nowhere in the others* — confirmed (§8.1).
- *With no multitap the SNES's `$421C` reads 0* — **retired**: Venus returns `$42`, the open bus's last byte (the high
  byte of the address itself), with or without presses. The test now compares the register before and after rather
  than against zero; what a real console returns there is a separate question for the PPU's open-bus record, not this
  work.
- *A pad attached through SDL's virtual-joystick API is seen through the added event within one poll* — confirmed.
- *Two virtual pads of one vendor and product share a GUID and have no path* — confirmed (`ff00…` style GUIDs differ
  only by the product word).

Recorded before the first run of the fit audit of §8.9:

- *P1, P2: in a big-screen menu row the 80-character name is trimmed, and the audit reports it cut* — half right. It is
  trimmed (the picture shows it), but the audit does not report a menu row's trimmed label, which the menu draws with
  its own ellipsis. The consequence the audit missed is §8.9's numbering.
- *P3: on the desktop nothing is cut* — **retired**: the window was 1,080 pixels tall at 1280 by 800 (§8.9). The audit's
  readability floor and its rule against an unfaded scrolling edge are the big screen's (Q186) and are not applied to a
  desktop window, which keeps the desktop's text sizes and a scroll bar.
- *P4: the bindings window's player row fits* — **retired** (§8.9: the drawing's labels fell under the floor).

### 8.9 Where the player sees it

**Preferences ▸ Controllers** begins with the players (`PlayerPreferencesRows`): a *Keyboard* row choosing the
keyboard's player, then a row per connected pad — its name, and a choice of Player 1 to 8 or None — then a row per seat
kept for a pad that has gone ("Player 2: *name*, disconnected") with **Forget**. Choosing a player another pad has
trades the two (§8.2, rule 6). A row's hint says when its player is not heard: *None*, a player past the game's ports
("This game has 2 controller ports, so it does not hear player 3"), or First Controller being on. Rows come and go as
pads are plugged in and pulled out with the sheet open; the others keep their controls, so a focused choice stays
focused.

**Press a button to identify.** Any button held on a pad, or a trigger past half its travel, lights that pad's choice in
the theme's accent — a border on the desktop, the value's colour in a big-screen menu row — for as long as it is held,
as RetroArch's and ES-DE's documentation describe their "press a button" device lists. Two pads of one name are
numbered *in front* ("1 · 8BitDo …", "2 · 8BitDo …"): the first build numbered them at the end, and the fit audit's
picture at 1280 by 800 showed both long names trimmed before the number, so identical pads read identically in a menu
row. The light was the only way to tell them apart there, which is not enough for a player who cannot see the rows and
the pads at once.

**In a big-screen session** the same rows are ES-DE's menu rows, through `BigMenuForm` (§4.72.8 of the settings
reference): each pad is an option row that Left and Right step through the players, the kept seat an action row, so
the panel is used with a pad alone (`PlayerPreferencesTests.In_big_picture_the_pad_alone_changes_a_pads_player`).

**The Controller Bindings window** gains a *Player* selector at the right end of its tab strip, shown for a console with
more than one port and listing that console's players; **Use Player 1's** gives the player shown player 1's map again.
A line under the drawing says whose buttons are shown and whether they are the player's own, the keyboard column says
whose keyboard it is when that is not obvious ("Keyboard (Player 1)"), and the drawing and the tester light the
selected player's pad. The selector was first placed in a row above the drawing; the window fit audit then failed every
console tab with more than one port, at both sizes, because the drawing lost the row's height and its labels fell to
11.47 design pixels of capitals, under the 11.64 floor. The tab strip's right end was empty, and costs the drawing
nothing. With the pad, Up past the drawing's top row is the selector and Up again is the tab shown; Left and Right on
the selector step the players. `PadSettingsWindowTests`' audit of every control by pad found the selector reachable on
the SNES tab alone before that route was given, since Up from the other drawings' top rows reached the tab strip first.

**The desktop Preferences window** is now at most 720 pixels tall. It sized itself to its tallest tab, and with three
pads and a kept seat the Controllers tab made it 1,080 pixels, past a 1280 by 800 screen; its panes scroll.

### 8.10 What this does not cover

- **The SNES multitap and the NES Four Score.** Both are console hardware: a multitap answers on the second data line
  with its own protocol (`$421C`-`$421F` and the `$4201` I/O bit on the SNES; a signature byte after the 16th read on
  the NES). They belong in the cores, as §6 already said of the multitap, and would raise those consoles'
  `ControllerPorts` to 5 and 4. The seating and routing here already go to eight players, so adding either needs no
  change above the core.
- **Two players on one keyboard.** One keyboard map per console, given to one player. Per-player keyboard maps need
  conflict checking across players and a second set of defaults that no hotkey holds.
- **Seats remembered between sessions** (§8.2).
- **Hotaru's keyboard on another player**: Hotaru has no settings to move it.
- **Hotaru following a port count that changes mid-game** (§8.11).

### 8.11 A console whose ports its settings decide (2026-10-07)

**The Genesis is the first console whose port count is not a constant.** A Team Player or a 4 Way Play on a port
(`EmuSen_Settings_Reference.md` §4.101) makes four pads of it, so the console has two to eight players, and which it
has is a setting the player can change while a game runs. §8.1 assumed a count per console and §8.5 a count per game.
Three things follow, and none of them widens what §8 built: the seats, the bindings and the router already went to
`PlayerSlots.MaxPlayers`, eight.

- **With no game loaded**, the Controller Bindings window has no machine to ask. `ControllerPorts.ForConsole(console,
  stored)` asks the console's pack for each player's controller as the stored settings stand
  (`SystemPack.PlayerControllers`; §4.102 there), and falls back on §8.1's count for a console whose pack has no rule.
- **A running core's count is read on every poll**, not kept from the game's start. `ControllerPorts.Of` already read
  machine info; a v1 core raises `MACHINE_INFO` when its ports change and the host reads the descriptor again
  (`EmuSen_CoreAPI.md` §6.19), growing the buttons it holds for the core to the new ports.
- **`PortRouter.Resize(ports)`** is the router's part, called by Mistress's poll when the session's count differs
  from the router's. It is not §8.5's `Reset`, which forgets everything sent, as a new game requires. A resize keeps
  what the ports that stay were sent, so a button player 1 is holding is neither sent again nor dropped; and a port
  taken away is sent a release of each button it held, because a port's buttons are kept below the router while the
  port is gone, and would otherwise be found still down when the adapter is plugged in again.

**Why not `Reset`.** With `Reset` where `Resize` is, a button let go in the same poll as the count changed is never
released to the core, `Reset` having forgotten that it was sent; a button still held would be sent a second time,
which is `Reset`'s purpose (`PortRouterTests.A_reset_forgets_what_was_sent`). The first was predicted from reading
`Reset`, then shown: the first of the two router tests below, with its `Resize(5)` made a `Reset(5, …)`, fails with
the press sent and no release after it (measured 2026-10-07).

**Hotaru** resets its router when a game is loaded or swapped and not otherwise, so it plays the ports a game starts
with. Following a change there wants a test that drives Hotaru's game window with pads, which the harness does not
yet have; it is left until then rather than added unexercised.

**Coverage**: `PortRouterTests.Ports_added_while_the_game_runs_hear_their_players_and_the_rest_keep_what_they_were_sent`
and `A_port_taken_away_while_the_game_runs_lets_go_of_its_buttons`;
`MultiplayerTests.Four_pads_play_a_genesis_team_player_set_while_the_game_runs`, four pads plugged into Mistress, a
Team Player set on port 1 after the game has started, and players 1, 3 and 4's buttons found in the adapter's packet
as a test cartridge reads it.
