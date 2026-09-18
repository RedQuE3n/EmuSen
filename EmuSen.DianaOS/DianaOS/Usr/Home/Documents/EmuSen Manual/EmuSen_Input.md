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

- **Multitap, and ports beyond two.** `SetButton`'s `port` is an `int` rather than a two-valued enum specifically so this can grow, but no core models more than two controllers and no frontend offers to configure one.
- ~~**Analog axes.** `GamepadManager` converts a stick to a d-pad (`AnalogStickAsDpad`, with a deadzone) and there is no analog value anywhere in the contract. A console with a genuinely analog stick needs a real addition here, not a fudge through `PadButton`.~~ **Closed 2026-09-18 by §7**, which made that real addition — `PadAxis` and `ICore.SetAxis` — and kept the stick out of `PadButton`, as this bullet asked.
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
the user gave to something else stays theirs, and the new control waits to be bound. The gamepad file gets the same
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
