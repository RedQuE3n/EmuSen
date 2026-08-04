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

### Where the type lives

**`EmuSen.Galaxia`, not the core.** Galaxia is the config layer and a true leaf — no project references at all — and `PadButton` is the key type of two config files, so Galaxia already owned its persisted form. Putting the type itself there costs **no new assembly edges**: `EmuSen`, `EmuSen.Nehellania`, `EmuSen.Hotaru` and `EmuSen.Mistress` all referenced Galaxia already.

What that bought is §4: `EmuSen.Nehellania` could finally drop its reference to the core assembly and become a leaf, matching `EmuSen.Serenity`. (See `EmuSen_Audio_Sync.md` §7.1 for the audio half of the same move.) A new shared-contracts assembly was the alternative, and was rejected as heavier than the problem.

The SNES pad being a superset of the NES pad is why the union is currently just `SnesButton` renamed. That will stop being true — a Sega pad's C button, a PlayStation pad's second shoulder pair — and when a new member is added, it goes at the end and Venus's cast stays valid.

---

## 4. The stack above the core

Nothing below names a console:

| Layer | Type | Persists to |
| --- | --- | --- |
| `EmuSen.Nehellania/Input/GamepadBindingMap` | `Dictionary<PadButton, SDL.GamepadButton>` | `gamepadbindings.json` |
| `EmuSen.Nehellania/Input/GamepadManager` | polls SDL3, emits `PadButton` | — |
| `EmuSen.Mistress/Input/ControllerKeyMap` | `Dictionary<PadButton, Avalonia Key>` | `keybindings.json` |
| `EmuSen.Hotaru/Input/HotaruKeyMap` | `IReadOnlyDictionary<PadButton, Key>` | — (fixed defaults) |
| `EmuSen.Pharaoh/FrameRunner` | `Dictionary<(PadButton, int Controller), bool>` | — |

`EmuSen.Nehellania` now references **only** `EmuSen.Galaxia` (plus the SDL3 packages) and contains only the two gamepad files — audio output moved out to `EmuSen.Endymion`. `EmuSen.WiseMan/Common/LeafAssemblyTests.cs` pins that reference set so it cannot quietly regrow.

The two key maps stay separate on purpose — Hotaru has no settings UI to drive a rebind, so it carries fixed defaults that `EmuSen.WiseMan/Input/HotaruKeyMapTests.cs` asserts are table-equal to Mistress's defaults. Neither imports a core.

`GamepadBindingMap`'s defaults bind by physical button **position** (`SDL.GamepadButton.South`), not by printed label, so a Nintendo-layout and an Xbox-layout pad both put the same physical button under the same thumb. `EmuSen_Settings_Reference.md` §4.6 covers that and the pad-label lookup.

Both maps enforce that two console buttons never share one key: `Rebind` unbinds the previous owner. That check runs over the **whole** map, not just the displayed subset, so binding Select→S while an NES ROM is loaded still takes S away from X even though X is not on screen (§5).

---

## 5. The rebind window shows the loaded console's pad

`InputSettingsWindow` used to iterate `Enum.GetValues<PadButton>()` and build twelve rows unconditionally — so with an NES ROM loaded it offered to rebind X, Y, L and R, four buttons that core drops on the floor.

It now takes the button list as a constructor argument, defaulting to all twelve. `MainWindow` passes `_session.SupportedButtons`, which is the loaded core's list, or empty before a ROM is loaded — in which case the window falls back to all twelve, since with no core loaded there is no console to be specific about.

Consequences worth knowing:

- **Rows are built once, at window construction.** Loading a different ROM while the settings window is open does not re-lay it out. Closing and reopening it does.
- **A hidden button keeps its binding.** The maps are shared across cores and the window only filters what it *displays* — nothing is unbound by loading an NES ROM, and the SNES bindings are all still there when a `.sfc` is loaded next.
- **Conflict detection scans the displayed buttons only.** A clash between two hidden buttons is not reported, because neither is reachable on this console. Since `Rebind` already polices the whole map, the only way to get a clash at all is a hand-edited config file (`EmuSen_Settings_Reference.md` §4.5).

---

## 6. What this does not cover yet

- **Multitap, and ports beyond two.** `SetButton`'s `port` is an `int` rather than a two-valued enum specifically so this can grow, but no core models more than two controllers and no frontend offers to configure one.
- **Analog axes.** `GamepadManager` converts a stick to a d-pad (`AnalogStickAsDpad`, with a deadzone) and there is no analog value anywhere in the contract. A console with a genuinely analog stick needs a real addition here, not a fudge through `PadButton`.
- **Non-pad peripherals.** Light guns, mice, the SNES multitap's own protocol. All are console-specific hardware that would attach to the core, not to this interface.
- **Per-core default bindings.** Defaults are one table shared by every core. An NES ROM gets the SNES-shaped defaults for the eight buttons it has, which happens to be right (Z/X = B/A), and is luck rather than design.
