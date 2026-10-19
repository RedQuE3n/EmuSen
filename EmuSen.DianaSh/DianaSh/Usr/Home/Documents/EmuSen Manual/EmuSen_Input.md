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

*Since 2026-10-16* `SnesButton` and its cast left with C# Venus (`VenusRT_Native.md` §68.4); the SNES pad is the pack's
`SnesSystem.PadButtons`, still `PadButton`'s first twelve, and the section below is the record of why. *Since
2026-10-19* that literal is gone too, and the pad is VenusRT's core info, in the same order (`EmuSen_Backend.md`
§18.10.12).

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

Galaxia still cannot hold it, and that part of the original argument stands: Galaxia is a leaf with no references at all, because DianaSh references it and the core references DianaSh, so anything it depended on upward would close a cycle. It cannot name `Avalonia.Input.Key`.

## 5. The rebind window shows the loaded console's pad

`InputSettingsWindow` used to iterate `Enum.GetValues<PadButton>()` and build twelve rows unconditionally — so with an NES ROM loaded it offered to rebind X, Y, L and R, four buttons that core drops on the floor.

It filters to the console's own pad instead. The button list comes from `CoreCatalog.ButtonsFor(console)`, which reads each core's `PadButtons` **static** — `VenusCore.PadButtons` and `MoonCore.PadButtons`, the same arrays their instance `SupportedButtons` returns. It has to be static because the window lists every console's pad whether or not a ROM is loaded, and restating the lists in the catalog is how they would drift from what the core actually reads. `ConsoleBindingsTests` pins the two against each other. *Since 2026-10-19* there is no static list for any console:
`ButtonsFor` reads the console's engine's core info from its sidecar, with no library loaded (`EmuSen_Backend.md`
§18.10.12).

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
| NES | MoonRT (Rust); Moon (C#) until it was deleted on 2026-10-16 | 2 | MoonRT's core info; `NesSystem.Ports`, the pack's, until 2026-10-19 (`EmuSen_Backend.md` §18.10.12) | player 2 |
| SNES | VenusRT (Rust); Venus (C#) until it was deleted on 2026-10-16 | 2 | VenusRT's core info; `SnesSystem.Ports`, the pack's, until 2026-10-19 (`EmuSen_Backend.md` §18.10.12) | player 2 |
| SNES | VenusRT (Rust, ABI v1) | 2 | machine info's `ports` (`EmuSen_CoreAPI.md` §6.4) | player 2 |
| Game Boy | MercuryRT (Rust); Mercury (C#) until it was deleted on 2026-10-16 | 1 | the console | none, by the hardware |
| N64 | Mars (C#), MarsRT (Rust) | 4 | `MarsCore.Ports` | players 2, 3 and 4 |
| Genesis | Nephrite (Rust, ABI v1) | 2 to 8, by its two port settings, and two more on a J-Cart | machine info's `ports`; with no game loaded, the console's pack (§8.11) | players 2 to 8 *(added 2026-10-07)* |

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
   order SDL lists them. *(2026-10-17: in a game, the first pad pressed then takes player 1 and the others follow in
   the order they are used; on a handheld an external pad is seated ahead of the built-in controls. Both are §8.13,
   and both leave this rule as the order before anyone presses.)*
2. **A pad that is unplugged keeps its number.** Nobody moves up into it, and the other players keep theirs. A notice
   says "Controller disconnected: *name* (Player 2)", and the game is not paused. *(Worded "*name* disconnected:
   player 2 is kept for it" since 2026-10-17, §8.13.5.)*
3. **A pad that comes back gets its number back.** It is recognised by SDL's GUID for the device (vendor, product,
   version and bus, sixteen bytes) and by its device path. *(2026-10-14: the GUID also carries a CRC of the device's
   name, which is what tells Steam Input's virtual pads apart, §8.12.4. Until that day the rule was never reached on
   Linux for a pad plugged in after start, because SDL never listed it, §8.12.)*
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
and `tap2` gain `tap3` to `tap8` (`CommandsScriptRunner.TapController`). **The DianaSh shell has no input command**:
input reaches a core from a frontend's pads and keys or from Pharaoh's script verbs, and no shell command presses a
button, so there is nothing there to give a port. A `press` command would be the place, and would take the controller
as these verbs do.

### 8.8 Tests, and what was predicted

No physical pad is needed. Two harnesses stand in *(and since 2026-10-14 a third, kernel pads made through `/dev/uinput`,
which found that neither of these reached the defect of §8.12)*: `SimulatedPads`, the device layer's fake, now with a GUID (an MD5 of
the name unless set, so pads of one name share one, as one model does) and a path per pad; and **SDL's own virtual
joysticks** beneath the real `SdlPadDevices`, filtered to the ones the test attached, so that SDL's GUIDs and paths are
the measured ones.

| Test | What it shows |
|---|---|
| `PlayerSlotsTests` (12) | the seating rule, reservations, identical pads with and without paths, trading and *None*, the ninth pad, the first controller alone, per-player bindings, the player LED, a new device set |
| `PortRouterTests` (9) | each player's pad to its port, nothing past the last, unplugging mid-game, the keyboard's player, the mirror, axes per port, the plugged-in flags, a reset |
| `PlayerBindingsTests` (10) | player 1's map until changed, own maps, the file's added keys in both directions, files from before players, reset |
| `ControllerPortTests` (8) | every engine's count; port 1 reaching the NES's `$4017` and the SNES's `$421A` on both engines of each, ports 2-3 reaching nothing; the N64's four ports in Mars and, by state against Mars, in MarsRT |
| `UinputHotplugTests` (13), `UinputMistressTests` (3) | *added 2026-10-14, opt-in:* kernel pads plugged, pulled and plugged back under the real SDL layer and under Mistress's window (§8.12.5) |
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
- *A pad attached through SDL's virtual-joystick API is seen through the added event within one poll* — confirmed. It
  did not hold for a kernel pad, which SDL never listed without its event pump (§8.12.2).
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
yet have; it is left until then rather than added unexercised. *Decided 2026-10-07: left, and on the Genesis's
follow-up list (`Nephrite_Native.md` §40.2) with the test named,
`HotaruMultiplayerTests.Four_pads_play_a_genesis_team_player_set_while_the_game_runs`.*

**Coverage**: `PortRouterTests.Ports_added_while_the_game_runs_hear_their_players_and_the_rest_keep_what_they_were_sent`
and `A_port_taken_away_while_the_game_runs_lets_go_of_its_buttons`;
`MultiplayerTests.Four_pads_play_a_genesis_team_player_set_while_the_game_runs`, four pads plugged into Mistress, a
Team Player set on port 1 after the game has started, and players 1, 3 and 4's buttons found in the adapter's packet
as a test cartridge reads it.

### 8.12 A pad that reconnects was never seen again: SDL's event pump (2026-10-14)

**The report.** On 2026-10-14 the tester reported that a controller which disconnects from the desktop or the handheld
and is connected again is not recognised by Mistress. Both machines were affected: the desktop (Fedora, KDE) and the
Legion Go S in SteamOS's Game Mode, where Mistress runs under Steam and gamescope and Steam Input presents virtual pads
of its own. The handheld's journal of 2026-10-12 showed the Bluetooth Xbox Wireless Controller reconnecting as a new
`input` device and Steam making its virtual "Microsoft X-Box 360 pad 0" and "pad 1" again.

§8.2 promised the opposite (rule 3), and every test of §8.8 passed. The tests had in common that no kernel device was
involved: `SimulatedPads` stands in for SDL altogether, and SDL's virtual joysticks are added to SDL's list by the call
that attaches them. What neither exercises is how SDL learns that a device was plugged in, and that is where the defect
was.

#### 8.12.1 The reproduction

**A kernel pad made through `/dev/uinput`.** The desktop account can write `/dev/uinput` (an access-control entry grants
it), so a test can make an input device the kernel, udev and SDL treat as a pad that was plugged in. `UinputPad`
(`EmuSen.WiseMan/Fixtures/UinputPad.cs`) makes one with an Xbox 360 pad's identity (USB, vendor `045e`, product `028e`,
version `0114`), its eleven buttons, two sticks, two triggers and the hat, through the uinput ioctls; it presses a
button by writing an event, and unplugs the pad by destroying it. udev tags the node for the seat's user, as it tags a
real pad. SDL accepts it as a gamepad from its own database and names it "Xbox 360 Controller", whatever the device's
own name, so the tests tell their pads by their event node rather than by name.

**Driven as Mistress drives it.** `UinputHotplugTests` runs a `GamepadManager` of each kind (the C# and the library's) on
the real `SdlPadDevices`, shown only the test's own pads, and ticks it as Mistress's pad timer does: `Poll` every 16 ms
on one thread, and nothing else. Before any change, under both managers:

- a pad made before the manager started was seen, pressed and heard; pulled, it was announced as disconnected within
  two polls; made again with the same identity on the same node, it was **never seen**, over four seconds of polls;
- a pad made after the manager started was never seen at all.

`UinputMistressTests` does the same under Mistress's own window and its real devices, in the desktop library, a
big-screen session and an SNES game: in each, the notice "Controller disconnected: Xbox 360 Controller (Player 1)"
appeared, and no reconnection followed. That is the report as the tester described it.

#### 8.12.2 The cause

A probe beside the tests asked SDL directly what it lists, under two ways of keeping it current. With
`SDL_UpdateGamepads` alone, which is what `SdlPadDevices.Update` called, SDL's list kept a pad that had been destroyed
and never gained one that had been made. With `SDL_PumpEvents`, both changes were in its list within a few polls.

SDL 3.4.2's source says why. On Linux, outside a container, SDL enumerates input devices through udev
(`ENUMERATION_LIBUDEV` in `src/joystick/linux/SDL_sysjoystick.c`). The Linux driver's `Detect`, which
`SDL_UpdateJoysticks` and therefore `SDL_UpdateGamepads` call, does nothing in that case but handle pending removals;
its comment reads "Polling will happen in the main event loop". The udev monitor is read only by `SDL_UDEV_Poll`, which
only `SDL_PumpEventMaintenance` calls, which only the event pump (`SDL_PumpEvents`, `SDL_PollEvent`, `SDL_WaitEvent`)
runs. Nothing in Mistress pumps SDL's events: Avalonia owns the window, and SDL is only an input and audio source here
(`EmuSen_Settings_Reference.md` §4.10). So no pad plugged in after start ever reached SDL's list. The manager's
one-a-second rescan asks that list (`SDL_GetGamepads`), and found nothing new in it either.

**Why a disconnect was still seen.** An open pad's node fails its next read with `ENODEV`; the driver marks the pad gone
and `Detect` removes it, which does not need udev. So a pull was announced at once and the return was not, which is
exactly the asymmetry in the report. It also explains why the pads present at start were always found: SDL scans
`/dev/input` once when its joystick subsystem starts.

**What the earlier tests could not see.** `SdlVirtualPadsTests` and the device parity test called `SDL_PumpEvents` before
each `Poll`, a step Mistress never took, and SDL's virtual joysticks need no detection anyway. §8.8's prediction "*a pad
attached through SDL's virtual-joystick API is seen through the added event within one poll*" was confirmed and is
kept, but it did not generalise to kernel devices, and nothing claimed that it did. The defect is in the layer below the
seating rule; `PlayerSlots` and the managers' bookkeeping were correct and are unchanged.

Where enumeration is not by udev (a Flatpak, Steam's runtime container, or `SDL_JOYSTICK_DISABLE_UDEV`), SDL watches
`/dev/input` with inotify inside `Detect`, and the old code would have seen a reconnection; on Windows and macOS the
drivers' own detection also runs in `Detect`. The defect was therefore Linux with udev, which both of the tester's
machines are.

#### 8.12.3 The fix

`SdlPadDevices.Update` calls `SDL_PumpEvents` where it called `SDL_UpdateGamepads`, and the library's `SdlPads.update`
does the same (`Platform/endymion/src/devices.rs`, with `SDL_PumpEvents` added to the functions it loads). The pump
updates the joysticks itself (`SDL_UpdateJoysticks`, under SDL's default `SDL_HINT_AUTO_UPDATE_JOYSTICKS`), so it
replaces the update rather than adding to it. The order of a poll is unchanged: pads that went are let go, the added
events and the rescan open new ones, and the pump at the end of the poll brings SDL's list and the pads' state up to
date, so a pad plugged in is opened at the next poll.

`SDL_PumpEvents` is to be called on SDL's main thread. SDL takes that to be the thread that started its events
subsystem, which in Mistress is the UI thread (the audio player starts SDL's audio, and with it the events subsystem, in
the window's constructor, and the gamepad's deferred start of §4.42 posts to the same thread); `Poll` runs on it. Hotaru
starts and polls its pad on its UI thread too.

The alternative considered was the hint `SDL_JOYSTICK_DISABLE_UDEV`, which moves enumeration to inotify inside
`Detect`. It would have worked, but it changes how SDL classifies devices for the whole process to avoid calling the
function SDL documents as the way its devices are kept current, and was not taken.

**The queue nobody reads.** The probe also showed a second defect of the same function. SDL queues an event for every
change of a pad's state, and nothing in the process takes them: `DevicesChanged` flushed only the added and removed
events. Each button change left four events in the queue (8,002 after 2,000 changes). At SDL's limit of 65,535 the
queue refuses every new event, the added events among them, and SDL formats an error for each refusal; with the queue
full, a second pad made and pumped was in SDL's list while `SDL_HasEvents` for the added event answered false. The
rescan still found such a pad within a second, so this did not cause the report, but the added event had stopped
working after a few minutes of play, and every poll walked 65,535 entries. `DevicesChanged` now flushes every joystick
and gamepad event (types `0x600` to `0x6FF`) after asking about the added and removed ones, in both halves; nothing in
the process reads the others.

#### 8.12.4 What "the same pad" is on the handheld

**What Steam Input presents.** In Game Mode Steam translates the physical controllers and presents each as a virtual
pad made through uinput, named "Microsoft X-Box 360 pad *N*" with Valve's vendor `28de` and product `11ff`, where *N* is
Steam's controller slot; SDL reads the slot from the name (`GetSteamVirtualGamepadSlot`). SDL hides such a pad unless
`SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD` is set (`SDL_ShouldIgnoreGamepad`), and that variable is how Steam tells
the programs it launches to use its virtual pads; the physical pads it translates are hidden from SDL through
`SDL_GAMECONTROLLER_IGNORE_DEVICES`. Both variables are Steam's behaviour as described for its games and were not read on
the handheld; step 6 below reads them.

**The slot is in the GUID.** SDL3 builds a pad's GUID from the bus, vendor, product and version and a CRC-16 of the
device's name (`SDL_CreateJoystickGUID`), so §8.2's description of the GUID as "vendor, product, version and bus" was
incomplete. Two of Steam's virtual pads differ only in the slot number in their names, and their GUIDs differ in that
CRC: measured on uinput pads of that identity, slot 0 is `030079f6de280000ff11000014010000` and slot 1
`0300b836de280000ff11000014010000`.

**Decided 2026-10-14: on the handheld, the same pad is the same Steam slot, and §8.2's rule needs no change for it.** A
reconnect makes the virtual pads again on new nodes, so the path is not kept and is not relied on; the GUID is kept as
long as Steam gives the controller its slot again, and §8.2's second match (a seat reserved for the same GUID) returns
the player. Where Steam gives the controller another slot, its GUID is new and rule 4 seats it in the lowest seat with
no pad connected, which, beside pads that stayed connected, is the seat it left. What the rule cannot do is return a pad
to its seat when Steam has both renumbered it and removed the pads below it at the same moment; no identity SDL reports
for the virtual pad survives that, short of the physical controller's own, which SDL exposes only through Steam's
virtual-gamepad information and which may itself change with each Bluetooth connection. That is not built: it would
widen `IPadDevices` for a case not yet seen, and the tester's check below will say whether it occurs.

**The pattern, reproduced on the desktop.** With `SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD=1` set in the C
environment the test reads, `UinputHotplugTests` makes Valve's two virtual pads, slots 0 and 1, seated as players 1 and
2; removes both; holds the first freed node with another device so that neither returns to its old node; and makes slot
1 again before slot 0. Each takes its own player, and each is heard as that player alone. A second case removes slot 1
and makes slot 2 beside slot 0: it is player 2.

**What the tester should try on the handheld**, with this build published there:

1. In Game Mode, with the Xbox Wireless Controller connected over Bluetooth, start Mistress and open Preferences ▸
   Controllers. Note which player the built-in controls are and which the Xbox controller is.
2. In the library, turn the Xbox controller off (hold its Xbox button). Expect "Controller disconnected: … (Player 2)",
   possibly with a disconnect and reconnect of the built-in controls as Steam makes both virtual pads again.
3. Turn it on. Expect, within a second or two of Steam showing it connected, "Controller connected: … (Player 2)", and
   the Xbox controller's d-pad moving the library.
4. Repeat in a big-screen session's themed view, and in a two-player game (an SNES game is enough), where the Xbox
   controller should be heard as player 2 after it returns and the built-in controls as player 1 throughout.
5. Quit Mistress, turn the Xbox controller off, start Mistress, and only then turn it on: it should be seen, as player 2
   beside the built-in controls.
6. If a returning controller gets another player number, run `journalctl -b --since "-5 min" | grep -i "x-box 360 pad"`
   and note the slot numbers in the names before and after: a slot that changed is the case this section leaves open.
   `tr '\0' '\n' < /proc/$(pidof EmuSen)/environ | grep SDL_GAMECONTROLLER` shows the two variables Steam gave
   Mistress. If the controller is not seen at all, the newest `crash_*.txt` in `~/Documents/Logs` and whether Steam
   Input is on for Mistress's shortcut are the first things to look at.

**Predictions for the handheld, recorded before it was tried:** the returning controller is seen in every screen;
Steam makes both virtual pads again with the slots they had, so both players come back by GUID; and the built-in
controls, if their virtual pad is made again, keep player 1.

#### 8.12.5 Tests

| Test | What it shows |
|---|---|
| `UinputHotplugTests` (13, opt-in) | under each manager: a pad present at start pulled and plugged back on the same node; a pad plugged in for the first time after start; a pad plugged back while another device holds its old node; two pads of one model with player 1's replugged elsewhere and player 2's heard as player 2 throughout; Steam's two virtual pads made again in the other order on new nodes; a Steam pad back in another slot. Each case checks the seats and that the pad's button is heard as its player alone. And the pump and the flush allocating nothing |
| `UinputMistressTests` (3, opt-in) | Mistress's window on its real devices, its pad timer's tick and its game poll driven by the test: a pad pulled and plugged back in the desktop library, a big-screen session and an SNES game, the notices naming Player 1 both ways, and then the pad's d-pad moving the library or its B reaching the game's port 1 |
| `SdlVirtualPadsTests`, `EndymionDeviceParityTests` | unchanged in what they check, but they no longer pump SDL's events beside the manager's poll, so they take the product's path |

The real-device tests make devices every program on the desk can see, so they run only with `EMUSEN_UINPUT_PADS=1`, and
are skipped with the reason where `/dev/uinput` cannot be written. Every one of the fifteen pad cases failed before the
fix and passes with it, measured on 2026-10-14: the two replug cases of `UinputHotplugTests` and the three window cases,
which begin with a pad present, failed on its return, and the other ten on the first pad made after start. No simulated test
was added: the defect was not in logic `SimulatedPads` can reach, and a simulated device that learns of a pad only on
`Update` would have passed before the fix too, since both managers always called it.

#### 8.12.6 Cost

Measured on the desktop on 2026-10-14 with one kernel pad open, best of five rounds of 20,000 calls: `SDL_PumpEvents`
took 1.84 µs where `SDL_UpdateGamepads` took 0.77 µs, and `DevicesChanged` 0.022 µs where it took 0.011 µs. Mistress
polls twice a frame while a game runs (the pad timer and the game's poll), so the pump adds about 2 µs a frame, a
hundredth of a per cent of a 60 Hz frame. Against the old code in the state a session reached, the poll is cheaper: with
a press between polls, the old pair took 69.1 µs a poll once the queue had filled (each `SDL_HasEvents` walking 65,535
entries) and the new pair 3.9 µs, both including the press written to the device. Neither call allocates:
`UinputHotplugTests.The_pump_and_the_flush_allocate_nothing` holds 1,000 polls with presses between them to zero bytes.

### 8.13 Who is player 1: the first pad pressed, and a handheld's own controls (2026-10-17)

#### 8.13.1 The report, and the defect demonstrated

**The report.** On 2026-10-17 the tester found that every PlayStation game on the Legion Go S, in SteamOS's Game Mode,
ignored the Xbox Wireless Controller, and found the reason: the controller was player 2. The handheld's built-in
controls had been seated first, so they were player 1, and a one-player game hears player 1. Nothing on the screen
said so: the pads present when Mistress starts are not announced (`EmuSen_Settings_Reference.md` §4.61), and the
Xbox pad, connected before Mistress started or reconnected beside controls that never went away, was never told
anything but its number in Preferences.

**Nothing was broken in the sense of §8.2.** Rule 1 seats pads in the order they connect, and the order on the
handheld is the order Steam makes its virtual pads, which is the order Steam opened the controllers: the built-in
controls at boot, the Xbox pad when it connects. The rule was followed. What it does not do is ask which pad the
player is holding. On a desktop the question rarely arises, because the pad plugged in is the pad in the hand. On a
handheld a second set of controls is always connected, and it is first.

**Demonstrated before anything changed.** `MultiplayerTests.The_pad_pressed_first_in_a_game_is_player_1_whichever_connected_first`
starts an SNES test cartridge from one pad, plugs in "Xbox Wireless Controller" beside it, presses Start on the Xbox
pad and reads the two pad words the game copies from `$4218` and `$421A`. On WiseMan at 6cc427b3, measured
2026-10-17, it failed with `(0, 4096)` where `(4096, 0)` was expected: the press reached port 2, and port 1 heard
nothing, which is the report. It passes with this section's rules. The test deliberately gives the first pad no
vendor or product, so it is not known as built-in: it shows the press rule working where §8.13.2's recognition cannot.

#### 8.13.2 How the built-in controls are recognised, and the evidence

**What was read on the handheld**, on 2026-10-17 with EmuSen closed, by read-only commands and a listing program
(`~/padprobe/padprobe.py`, SDL's `ForID` queries through `ctypes`, no pad opened):

- The kernel lists the built-in controls as **"Legion Go S", USB `1a86:e310`**, version `0043`, whose event node
  (`/dev/input/event2`) has mode `000`: SteamOS hides it from every program. Six HID interfaces of the same device
  are "wch.cn Legion Go S", and a seventh HID device, **"Legion Go S Controller", `28de:12ff`**, is the one Steam
  drives. Steam's two virtual pads are "Microsoft X-Box 360 pad 0" and "pad 1", `28de:11ff`, version `0001`; the
  Xbox pad is a Bluetooth device, `045e:0b13`.
- The system's SDL, 3.4.12, run outside Steam, lists the Xbox pad alone (through HIDAPI). The built-in controls are
  not in its list at all. With `SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD=1` it lists the two virtual pads as well,
  both named "Steam Virtual Gamepad", both `28de:11ff`, type `standard`, told apart only by their GUIDs
  (`030079f6de28…` and `0300b836de28…`, §8.12.4).
- Steam's controller log (`~/.steam/steam/logs/controller.txt`, 107,324 lines back to January) records the built-in
  controls as "Legion Go S Controller" `28de:12ff` on a `hidraw` node 269 times and as "Legion Go S" `1a86:e310`
  through SDL 49 times; after the boot of 02:08 that morning, as `28de:12ff`. The Xbox pads are `045e:0b13` and
  `045e:02e0`.

So **outside Steam's own bookkeeping, nothing a program sees under Steam Input tells the built-in controls from the
Xbox pad**: the same virtual vendor and product, the same type, the same mapping name, and a GUID that encodes only the
slot. The connection state is no help either: SDL 3.4.2's Linux joystick driver never assigns it (there is no
assignment of `connection_state` in `src/joystick/linux/SDL_sysjoystick.c`), so for a virtual pad, which is an evdev
device made through uinput, it is SDL's unknown for every slot.

**Steam's bookkeeping does reach SDL, through a file.** Steam passes each program it launches the environment variable
`SteamVirtualGamepadInfo`, the path of a file with one section per virtual pad. The client libraries on the handheld
(`steamclient.so`, both widths) contain the variable's name and the format strings `[slot %d]`, `name=%s`,
`VID=0x%.4x`, `PID=0x%.4x` and `type=%s`. SDL 3.4.2 reads exactly that format
(`src/joystick/SDL_steam_virtual_gamepad.c`), keys it by the slot it reads from the virtual pad's name, and then
answers `SDL_GetGamepadVendor`, `SDL_GetGamepadProduct`, `SDL_GetGamepadType` and `SDL_GetJoystickName` with the
section's values, that is, with the controller behind the virtual pad (`SDL_GetJoystickVendor` and its relatives in
`SDL_joystick.c`). The listing program confirmed the mechanism on the handheld's own SDL: with a file of one section
naming slot 0 as `1a86:e310`, slot 0's vendor, product, type and joystick name became the file's.

**The rule built on it.** `BuiltInControls` (Endymion) is a table of the vendor and product pairs that are a
handheld's own controls, and `ConnectedPad.IsBuiltIn` asks it with what SDL reports for the pad. Each entry has its
evidence:

| Vendor:product | What it is | Evidence |
|---|---|---|
| `1a86:e310` | the Legion Go S's controls, as the kernel and SDL name them | `/proc/bus/input/devices` and Steam's log on the tester's handheld, 2026-10-17 |
| `28de:12ff` | the same controls as SteamOS presents them to Steam ("Legion Go S Controller") | the `hidraw` device's `HID_ID` and Steam's log, same day |
| `28de:1205` | the Steam Deck's controls | SDL 3's built-in mapping `03000000de2800000512…,Steam Deck Controller` in the shipped `libSDL3.so`; not measured on a Deck |

Under Steam Input a virtual pad is known as built-in when Steam's file names the built-in controls behind it, and
outside Steam (the desktop session, or SDL listing the device itself) when SDL reports the device's own identity. The
table holds no other handheld: the ROG Ally, the Legion Go and the rest are absent until one is measured, and on them
the setting of §8.13.3 does nothing, while the press rule still applies.

**Where this cannot work, said plainly.** If Steam does not write the file (a Steam that predates it, Steam Input off
for the shortcut so that SDL sees raw devices it may not be able to open, or a program launched outside Steam in Game
Mode), every virtual pad reports `28de:11ff` and none is built-in. The handheld setting then has nothing to act on.
That is why the press rule, which needs no recognition at all, is the primary fix and the handheld setting the
convenience; neither depends on the other.

**The file is read after the pad, sometimes.** SDL reads Steam's file when its joystick subsystem starts and again,
at most every three seconds, when the file's time changes. A controller connected mid-session is a new virtual pad
made by Steam at about the moment Steam rewrites the file, so SDL may open the pad before it reads the new section.
`ConnectedPad.Vendor` and `Product` are therefore read while the pad is open, not once at opening, and kept when it
closes, as its name and type always were; the pad's row and the notices know it once SDL does. The seat it took
when it opened is not revisited. Measured on the desktop with uinput pads of Steam's identity, the section was read
before the pads opened in every run. A second detail of the same code matters only to tests: SDL keeps the time of
the file it last read, and when it last looked, across a stop of its joysticks, so a second file written within the
same second or the same three seconds is not read. `UinputHotplugTests` dates each file it writes apart and waits out
the three seconds between its runs.

**The names.** Under Steam Input SDL names every virtual pad by its mapping, "Steam Virtual Gamepad", so the notices
and the Preferences rows named both pads alike. SDL's `SDL_GetGamepadName` would return the controller's name only
when the pad has a Steam handle, and Steam's file carries none (no `handle=` format in the client libraries). For a pad
whose GUID is Steam's virtual pad's and whose reported vendor and product are not `28de:11ff`, which is to say a
virtual pad Steam's file describes, `SdlPadDevices.Name` and the library's `SdlPads::name` now return
`SDL_GetJoystickName`, the controller's name from the file. Measured with uinput pads and a file in Steam's format
before and after: "Steam Virtual Gamepad" twice, then "Legion Go S Controller" and "Xbox Wireless Controller".

#### 8.13.3 The rules

Stated as a player would be told them:

1. **In a game, the first pad a button is pressed on is player 1.** The next pad pressed is player 2, and so on. A
   press is any button held, or a trigger past half its travel, as the Preferences rows light a pad
   (`ConnectedPad.AnyPressed`, one definition for both). A pad that becomes player 1 this way trades seats with the pad
   that was player 1, by §8.2 rule 6's trade, so nobody is left without a seat. A button still held from the press
   that started the game is not a press in the game: that pad claims once it has let go and pressed again (§8.13.8).
2. **The players then hold for the rest of the game.** A pad that has claimed its player keeps it. A restart
   (Restart Game) keeps the players; a new game, or the same game started again from the library, asks again.
3. **A player is changed mid-game only on purpose:** *Swap Players 1 and 2* in the pad menu's Game Settings, or a
   choice in Preferences ▸ Controllers. Either fixes every seat for the rest of the game, so a later press moves
   nobody.
4. **On a handheld, an external pad goes ahead of the built-in controls** (`ExternalControllerFirst`, on by default).
   A pad that is not built-in, seated after a built-in pad, trades seats with the lowest built-in pad before it, so
   the built-in controls move to the next player. It applies as pads connect, at start and later, and when the
   setting is turned on; it does nothing where no built-in controls are connected, and never takes a seat a press
   already claimed in the running game.
5. **The notice names the player first:** "Xbox Wireless Controller: player 1". When a pad lands on a player other
   than 1 while player 1 has no controller, or while player 1 is built-in controls nobody has pressed in the game, it
   says so and what to do: "…: player 2, and player 1 is the built-in controls. Press a button on it first in a game
   to take player 1." (or, with the press rule off or player 1 already claimed, "Swap Players under Game Settings makes
   it player 1.").

**One owner.** `GamepadManager` (Endymion), which already owned the seats, owns both rules
(`GamepadManager.Claims.cs`): `ClaimPlayersByPress`, `ExternalControllerFirst`, `BeginClaims`, `ClaimPressed`,
`SwapPlayers` and the `Claimed` event. `PortRouter.PollPads` asks for the claims before it reads the players, so the
press that claims player 1 is heard as player 1 in the same poll. The frontends only say when a game starts
(`BeginClaims`, not on a restart) and pass the two settings; Mistress and Hotaru cannot disagree. The rules are C# on
both halves: they are built from `PlayerSlots.Move`, which both halves already implement and the parity tests hold
alike, so the library's interface gained only the pad's vendor and product (§8.13.6). Both are off in a
`GamepadManager` until a frontend turns them on, which keeps every test of the seating rule and the routing as it was.

**Decided, and why.**

- *A slot's bindings belong to the slot.* A pad that becomes player 1 plays with player 1's map (§8.4). The
  alternative, bindings following the pad, would make "player 2's map" mean whichever pad was pressed second, and
  §8.4's file is keyed by player.
- *Nothing sticks when pads trade mid-game.* The router sends a button to a port when the port's state changes
  (§8.5), and a port's state is read from whichever pad is that player's, so on the poll of a claim or a swap each port
  is sent the difference between the old pad's buttons and the new pad's. A button held on the pad that moved away is
  released on its old port in that same poll.
- *Two pads pressed in one poll* take players in the order they connected (the order of `GamepadManager.Pads`). Polls
  are 16 ms apart; a tie is a coin toss to the players, and the connection order is the only order both halves agree
  on without a clock.
- *First Controller* (`FirstControllerOnly`) keeps its meaning for the game, "the game hears player 1 alone", and the
  press decides who player 1 is. For the twin of a pad that registers twice (§8.3) this is the better outcome: the twin
  is pressed in the same poll, connected later, and so becomes player 2, whom the game does not hear. The interface's
  "first controller" is still the first pad opened, as ES-DE defines it; the two can differ, and on a handheld with
  the handheld setting on they do not.
- *Mirror Player 1 to Player 2* is routing below the seats and is unchanged: it mirrors whoever player 1 is.
- *The keyboard* is not a pad and claims nothing; it stays the player `KeyboardPlayer` names, beside that player's pad.
- *A pad set to None* in Preferences stays out of the game when pressed: None is a choice, and a press does not undo
  a choice. A ninth pad, which has no seat, likewise.
- *A pad that reconnects* returns to its seat by §8.2 rule 3, so a claimed player 1 that drops and comes back is
  player 1 again, and a pad that claimed nothing meanwhile does not move up. While it is gone, its seat is kept, and
  rule 4's reading (a new pad takes the lowest seat with no pad connected) still lets a spare pad take it.
- *The press rule is a setting* (`ClaimPlayersByPress`, on by default), because a player who arranges the seats in
  Preferences before a game would otherwise have them rearranged by whoever presses first. Turned off, §8.2's order
  holds and only an explicit choice changes it.
- *The handheld setting defaults on* and has no effect without built-in controls, so it needs no detection of "a
  handheld" beyond the pads themselves; Preferences says when no built-in controls are connected.
- *The swap trades players 1 and 2* rather than offering a list. The report is a two-pad problem, and a pad menu row
  that names two players is one press; any other arrangement is Preferences' rows, which the menu opens from the row
  below it. The row is shown while player 2 has a pad, connected or kept. It is in Game Settings, beside Players &
  Controllers, rather than on the menu's first page, which already has ten rows (§4.69.8 of the settings reference).
- *Claims last until the next game starts*, not until the game ends; after a game they no longer matter except that
  the handheld setting will not displace a pad that claimed player 1 in the last game, until the next one starts.

**Rejected:** remembering the player's preferred controller between sessions by its GUID or name. It was the fallback
proposed for the case where the built-in controls cannot be told apart, and the press rule makes it unnecessary for
the report; it would also need §8.2's "nothing is remembered about a pad" reopened, with a place in Mistress's
database. It remains the way to go if a player wants a pad to be player 1 before any press, and is a question for the
tester rather than built.

#### 8.13.4 What the player sees

**Preferences ▸ Controllers** gains two rows after the players: *Player 1* ("The first controller pressed"), the press
rule's switch, and *External Controller* ("Ahead of the built-in controls"), the handheld setting's, whose hint adds
that it does not apply while no built-in controls are connected. The pad menu's **Game Settings** begins with **Swap
Players 1 and 2** while player 2 has a pad; choosing it closes the menu, trades the two and shows "*name*: player 1 ·
*name*: player 2". The settings are in `EmuSen_Settings_Reference.md` §4.119.

#### 8.13.5 The notices

| When | Words |
|---|---|
| a pad connects as player 1 | "Xbox Wireless Controller: player 1" |
| a pad connects as player 2 or later | "*name*: player 2" |
| … while player 1 has no controller, or is built-in controls not pressed in the game | "*name*: player 2, and player 1 is the built-in controls. Press a button on it first in a game to take player 1." (or "…has no controller…"; "Swap Players under Game Settings makes it player 1." where the press rule is off or player 1 is claimed) |
| a pad connects with every seat taken | "*name*: no player" |
| a press moves a pad to another player | "*name*: player 1" |
| a pad disconnects | "*name* disconnected: player 2 is kept for it" |
| Swap Players | "*name*: player 1 · *name*: player 2" |

The words before were "Controller connected: *name* (Player 2)" (§8.2). The player's number now comes right after the
name, which is what the tester asked for, and the notice of a claim is new: it is the only notice a player gets for
pads that were present when Mistress started, since those are not announced.

#### 8.13.6 The library's part: a pad's vendor and product

The pad's vendor and product reach `ConnectedPad` on both halves: `IPadDevices.VendorProduct` in the C#, the
`PadDevices::vendor_product` trait method in `Platform/endymion` (with `SDL_GetGamepadVendor`, `SDL_GetGamepadProduct`
and `SDL_GetJoystickName` added to the functions it loads), and in `emusen_platform.h` the numbers
`EMUSEN_ENDYMION_PADS_VENDOR_PRODUCT`, `EMUSEN_ENDYMION_SIM_PAD_VENDOR_PRODUCT` and
`EMUSEN_ENDYMION_DEVICE_VENDOR_PRODUCT`, the vendor in the high sixteen bits and the product in the low. The interface's
version is 12 (`EmuSen_RustPlatform.md` §3.2: matched exactly). A simulated pad has a vendor and a product a test sets,
and the device parity tests compare both halves' readings of them.

#### 8.13.7 Tests

| Test | What it shows |
|---|---|
| `MultiplayerTests.The_pad_pressed_first_in_a_game_is_player_1_whichever_connected_first` | the report: failed on 6cc427b3 (`(0, 4096)`), passes; the first pad not known as built-in |
| `MultiplayerTests.On_a_handheld_the_notice_names_the_player_an_external_pad_took` (3 cases) | through Mistress's poll: the handheld setting on seats the Xbox pad as player 1 as it connects; off, the notice's two remedies, and the game's ports with the press rule on and off |
| `PlayerClaimsTests` (16 cases) | recognition by vendor and product; both halves reading them at opening and keeping them; the built-in pad first and the external pad's first press player 1; the rule off; the handheld setting on and off, at start and hot-plugged, turned on later, and never over a claimed player; a claimed pad reconnecting; two pads pressed together; the swap and the claims it ends until the next game; one pad cannot be swapped; None and First Controller; the keyboard; a button held as the game starts claiming nothing until pressed again (§8.13.8) |
| `PadMenuTests.Swap_players_under_game_settings_trades_players_1_and_2` (3 cases) | the row reached by pad in a desktop session and a big screen at 1280 by 800 and 1920 by 1200, the page fit-audited, the trade and its notice |
| `UinputHotplugTests.Steams_virtual_pads_report_the_controllers_behind_them_and_the_external_one_is_player_1` (2, opt-in) | under each manager on the real SDL layer: Steam's two virtual pads made through uinput with a file in Steam's format; the vendors, products and names of the controllers behind them; the Xbox pad player 1 and the built-in controls player 2, each heard as its player alone |
| `ControllersTests.The_controller_settings_persist` | the two switches saved, reaching the pads on the next poll, and their defaults |
| `EndymionDeviceParityTests` | the vendor and product added to the simulated pads' and the managers' compared states |
| `PlayerBindingsWindowTests.Each_console_offers_as_many_players_as_it_has_ports` | corrected, not changed in behaviour: it had failed on WiseMan since the Sega CD was offered on 2026-10-16, its expected list one console short |

The existing multiplayer cases about the seating order and the routing (`Player_1s_pad_going…`, the N64's four pads,
the J-Cart and the Team Player) run with the press rule off, since they press other players' pads before player 1's
and are about §8.2 and §8.5, not about this section; their expectations are unchanged.

**Cost.** Measured on the desktop on 2026-10-17, best of five rounds of 100,000 calls in a Release build:
`ClaimPressed` with one pad still unclaimed (the case of a player who never presses the built-in controls, every poll
of the game) took 0.70 µs through the library and 0.27 µs in the C#, and allocated nothing. Once every seated pad has
claimed, it returns after one comparison per pad. Mistress claims once per game poll, so the cost is under a
microsecond a frame.

**Predictions recorded before the tests were first run**, and how they fared:

- *The Mistress-level test fails on WiseMan with the press on port 2* — confirmed (`(0, 4096)`).
- *Settings set in a test's `appsettings.json` reach the window* — **retired** on the first run: both new settings
  were dropped on loading, because the platform library's model of `AppSettings` (`Platform/galaxia/src/models.rs`)
  lists every property and drops a key it does not know. They were added there, which is the step any new setting of
  that file needs.
- *SDL reports the controller behind a virtual pad from a file in Steam's format* — confirmed on the desktop with
  uinput pads, and on the handheld's SDL with the listing program.
- *The virtual pads keep SDL's mapping name* — confirmed ("Steam Virtual Gamepad" for both), which led to the names
  of §8.13.2.

**What the tester should check on the handheld**, with this build published there:

1. In Game Mode, with the Xbox Wireless Controller on, start Mistress. Open Preferences ▸ Controllers: the built-in
   controls and the Xbox pad should be named as such, not "Steam Virtual Gamepad", and the Xbox pad should be player 1.
2. Start a PlayStation game and press a button on the Xbox pad: the game should respond.
3. Turn *External Controller* off, quit to the library, turn the Xbox pad off and on: the notice should say it is
   player 2 behind the built-in controls; in the next game, its first press should make it player 1, with a notice.
4. With two pads in a game, open the pad menu, Game Settings, Swap Players 1 and 2.
5. If the names in step 1 are still "Steam Virtual Gamepad", Steam did not give Mistress its file:
   `tr '\0' '\n' < /proc/$(pidof EmuSen)/environ | grep SteamVirtualGamepadInfo` shows whether the variable is set,
   and the file it names shows the sections. The press rule works either way.

#### 8.13.8 The press that started the game claimed player 1 (2026-10-18)

**The symptom.** `On_a_handheld_the_notice_names_the_player_an_external_pad_took` failed 4 of 4 runs on 69974969 on
2026-10-17 and 2 of 3 on a later branch, and passed on rerun on 2026-10-18; `The_pad_pressed_first_in_a_game_is_player_1…`
failed once. The failures had three faces: the Xbox pad's notice "…: player 2" where the handheld setting should have
made it player 1; the bare "Xbox Wireless Controller: player 2" where the notice should have said how to take player
1; and `(0, 4096)` where `(4096, 0)` was expected, the original report's own signature.

**The cause.** All three are what §8.13.3's rules give once seat 1 has been claimed before the test's pads arrive,
and something did claim it: the A press that launched the game from the library. The launch goes through
`MainWindow.RunSession`, which starts the game's 60 Hz `DispatcherTimer` calling `PollGamepad`. The test's helper
pressed A, ran the interface's tick and its dispatcher jobs, then let A go. If the game's timer came due within those
jobs, which it does when the launch takes longer than one frame interval to reach them, `PollGamepad` ran with A
still held, `ClaimPressed` read the launching pad as pressed and counted seat 1 claimed. After that
`PlayerOneClaimed` was true, so `ConnectedNotice` gave the plain seat, `SeatAhead` started its search above seat 1,
and the Xbox pad's later press took seat 2. Nothing in the claim logic reads a clock; the timing was the dispatcher's,
and only decided whether a game poll fell inside the launching press.

**Demonstrated before anything changed.** A diagnostic case read `GamepadManager._claimed` after the helper's launch
(it should be 0), and each of the three cases was looped in one test host, 200 runs each, measured 2026-10-18 on the
desktop:

| Setting | launch claimed seat 1 | handheld, setting off | handheld, setting on | first press |
|---|---|---|---|---|
| alone | 10 of 200 | 4 of 200 | 14 of 200 | 2 of 200 |
| sixteen busy loops on the sixteen threads | 106 of 200 | 100 of 200 | 52 of 200 | 138 of 200 |
| beside a second test host running the other Mistress tests | 8 of 200 | 0 of 200 | 0 of 200 | 68 of 200 |

Every failure carried one of the three messages above. With the timing forced, by the helper holding A across one
`PollGamepad` call before letting go, all four failed 50 of 50 runs, and so did the three `MultiplayerTests` cases in an
ordinary run.

**A product defect, not only a test's.** A real press lasts around a tenth of a second and the game polls every 16 ms,
so for a game that loads within the press, the pad that launched it held a button through the game's first polls and
claimed player 1, every time; for a game that loads after the press is over, it did not. On the handheld with the
external pad first, launching with the built-in controls' A moved them to player 1 and the Xbox pad to player 2, the
report of §8.13.1 recreated by the launch, and whether it happened depended on the load time. §4.29 of the settings
reference already says the button that closed a menu is not the game's; the launch is the same case, and rule 1 says
a press *in* a game.

**The fix**, in the rule's owner, `GamepadManager.Claims.cs`, so Mistress and Hotaru both have it: `BeginClaims`
notes each open pad that holds a button as the game starts, and `ClaimPressed` passes over such a pad until it is seen
with nothing held, after which its next press claims as any other. A pad that connects later, or one that pressed
nothing at the start, is unaffected; the game still hears the held button as its seat's, as before (whether the game
should hear the launching press at all is §4.29's question and is not changed here). The list is allocated once and
holds a handful of pads at most, so the poll allocates nothing. No `MainWindow` file changed.

**The tests now hold the press.** `MultiplayerTests`' launch helper holds A across one game poll, as a real press
does, so the cases run the same path every time instead of whichever path the timer chose.
`PlayerClaimsTests.A_button_held_as_the_game_starts_claims_nothing_until_it_is_pressed_again` is the defect at the
manager: the built-in controls holding A at the start, the Xbox pad player 1 by the handheld setting, two polls with
no claim, then a fresh press on the built-in controls claiming player 1. It failed before the fix with the claim made
on the first poll.

**After the fix**, the same four cases looped 200 runs each in each setting, with the launch now holding A across a game
poll, measured 2026-10-18: 0 of 800 failed alone, 0 of 800 under the sixteen busy loops, 0 of 800 beside the second
test host, 0 of 800 in one test host beside every other Mistress test (2,255 more, all passing, 54 skipped by their
own opt-in), and 0 of 800 under the busy loops with `EMUSEN_ENDYMION_NATIVE=0`, the C# manager in place of the
library's. The claims are C# over either manager, so the two halves were expected to agree, and did.

#### 8.13.9 The game heard the press that launched it (2026-10-18)

§8.13.8 left one question open: whether the game should hear the launching press at all. The tester decided on
2026-10-18 that it should not. In real use the A that launches a game from the library is still held for about a
tenth of a second into the game's first polls, and a game that loads within that time read it as its own: a menu
opened, a title skipped or a jump made by a press the player meant for the library. §4.29 of the settings reference
already keeps the press that closed a menu from the game; the launch is the same case.

**Demonstrated before the change.** `MultiplayerTests.The_press_that_launched_a_game_is_not_the_game_s_until_it_is_let_go`
launches a pad-reading SNES image with A held across a game poll, as `MultiplayerTests`' launch helper has done since
§8.13.8, seats by connection so the claim rule of §8.13.8 is out of the way, and polls three more times with A still
down. Before the change the game's pad word read `0x8000` (the SNES B bit, where the bottom face button lands) where
`0` was expected.

**The rule**, in `MainWindow`'s input path, where the menu's rule already lives: `RunSession`, which every launch,
restart and console start goes through, sets `_padQuiet`, the flag a closing menu sets, and `PadBelongsToTheInterface`
keeps every button from the game until no button is held, then clears it. The game's input still goes through the
frontend API's input calls as before; what changed is only that nothing is sent while the flag stands. A launch made
with nothing held clears the flag at the first poll, so a game started from a menu or the keyboard is unaffected.
The rule is Mistress's, as §4.29's is: Hotaru launches from a prompt, with no press to carry over.

**After the change** the same test reads `0` through the three polls and after the release, and the B bit once A is
pressed again.

**One test helper changed with it.** `MultiplayerTests`' launch helper let A go and then, in two Genesis cases, pressed
other pads' buttons before any game poll ran, which no player's hands can do inside 16 ms: the quiet saw buttons held
at every poll and the two cases read no input (`Four_pads_play_a_genesis_team_player_set…` and
`…j_cart_game…`). The helper now runs the game's poll once after letting go, as the 60 Hz timer would, and both pass.
Measured 2026-10-18 with the change: the pad, multiplayer, launch, emulation menu and player claims classes with the
Shaders recordings, 250 passed and 1 skipped by its own opt-in.
