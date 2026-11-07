# EmuSen.Kit: the pure-Rust heart (`emusen-kit-core`)

The design is `EmuSen_Qt.md` §1.4: two crates and one QML module carry what was LunaP's. This page is the record of the
first crate, `emusen-kit-core`, at `Frontend/kit-core`: what it owns, where each value comes from, what it was held to,
and where it differs from the C# it replaces. The second crate, `emusen-kit-qt`, and the module's `.qml` files are stage
0's later work and will have sections here when they are built.

The crate names no Qt type, no runtime, no client crate and no console. It builds with cargo alone, outside the
Platform workspace, and a test (`tests/fence.rs`) fails if its lock file ever names a Qt binding, a runtime, client or
Platform crate. Its one dependency is `palette` (0.7, MIT OR Apache-2.0), for colour conversion and WCAG 2.1 contrast,
so neither is hand-rolled (the house rule of frameworks first).

## 1. The tokens (stage 0, step 1; 2026-11-07)

**Ownership.** Decided 2026-11-07: from this step the one owner of the interface's tokens is `emusen_kit_core::tokens`.
LunaP's C# copy (`Theme/Tokens/Dark.axaml`, `Light.axaml`, `LunaMetrics.cs`, `LunaMotion.cs`) keeps running Mistress
until stage 8 retires it, and is not edited for this; a change to a token is made here first.

### 1.1 Where the values come from

| Group | Source | In the crate |
|---|---|---|
| Colour, both columns | `EmuSen_Interface.md` §5.1 with its 2026-11-01 corrections (Muted, Border, Success, Error against SurfaceRaised) | `tokens::colour(Variant, ColourToken)` |
| Type, both forms | §5.2 | `tokens::type_scale(Form, text_scale)` |
| Spacing, both forms | §5.3 | `tokens::spacing(Form)` |
| Focus marks | §5.4 | `tokens::focus_marks(Form)` |
| Motion durations, curves, reduced motion, the idle breath | §5.5, LunaP §206.5 | `motion::MotionToken`, `motion::Curve`, `motion::breath_opacity` |
| Text scale 1 to 1.3, rounding to the half unit | LunaP §214.1 | `tokens::clamp_text_scale`, in `type_scale` |
| The section heading's 28 and the 5% safe area | LunaP §206.2, §206.4 | `tokens::header_size`, `tokens::safe_area_of` |

The values were read from both the design page and the C# on 2026-11-07 and agree everywhere the design names a token.
Alpha is written as the C# writes it: Scrim `#8C` (55%), CaptionSurface `#AA` (67%), the OnArt scrim's foot `#B8` (72%).

**Where the C# has more.** `Dark.axaml` and `Light.axaml` carry seven colours the design's table does not:
`HudSurface` (`#E61C1C1C`), `MeterText` (`#DCDCDC` / `#2A2A2A`), `SectionHeader` (`#9CDCFE` / `#0A5A96`), `Warning`
(`#D08770` / `#A34B1E`), and the meters' `Nominal`, `Busy` and `Hot` (`#32CD32`, `#FFD700`, `#FF4500` dark; `#1B7A1B`,
`#8A6300`, `#B32D12` light), with `Info` as Attention's older name. They are the desktop's heads-up plate, meter rows and
the older section headings, none of which the Qt screens have yet. They are not carried into the crate: a token joins it
when a Qt screen needs it, with its contrast measured then. This is a question for the tester (§1.4).

**What is not a token here.** The text scale and reduced motion are arguments, never statics: LunaP kept them as
process-wide switches (`LunaTokens.TextScale`, `LunaMotion.Reduced`) with a dispatcher-aware event; in the Qt design the
`Tokens` singleton owns them as properties, and this crate gives it pure functions of their values. So there is no
shared mutable state in the crate.

### 1.2 What it is held to

`tests/tokens.rs`, one test per source, each named for its section:

- every colour of both columns, by hex;
- the 50 contrast pairs of §5.1 (Text, TextStrong, Muted, Attention, Success and Error at 4.5:1 and Accent and Border
  at 3:1 against Surface, SurfaceRaised and InputSurface, and OnAccent on Accent at 4.5:1, in both variants), measured
  with `palette`'s `Wcag21RelativeContrast`;
- the four corrections at the ratios §5.1 records (4.55, 3.04, 4.56, 4.53, within 0.02), and the value Muted replaced
  at 4.15;
- OnArt over a white picture under the scrim's foot at 9.3:1 (the scrim composited in sRGB, as a compositor paints
  8-bit layers);
- the type scales, spacing, focus marks, motion durations and curves, reduced motion's two rules, the breath's 2 s,
  1.6 s and 70%, the text scale's clamp and half-unit rounding (13.75 rounds to 14, away from zero, as .NET's
  `MidpointRounding.AwayFromZero` did), the header's 28 and the safe area at four screen sizes.

Measured on 2026-11-07 with three seeded faults, each reverted: Muted put back to `#86868A` failed the contrast
sweep, the corrections test and the dark column; Exit given a quadratic-out curve failed the motion table; the text
scale flooring rather than rounding failed the LunaP §214.1 test.

### 1.3 The surface

`emusen_kit_core::tokens`: `Form`, `Variant`, `ColourToken` (18 names), `Colour` (sRGB with straight alpha; `hex`,
`hsl`, `over`, `contrast`), `colour`, `contrast_pairs`, `on_art_over`, `TypeScale`, `Spacing`, `Insets`, `FocusMarks`,
`type_scale`, `spacing`, `focus_marks`, `header_size`, `safe_area_of`, `clamp_text_scale`, `MAX_TEXT_SCALE`,
`SAFE_AREA_FRACTION`, `TEXT_FLOOR`, `GRAPHIC_FLOOR`. `emusen_kit_core::motion`: `MotionToken`, `Curve`, the breath's
constants, `breathes`, `breath_opacity`. The versioning rule is `EmuSen_Qt.md` §5's R: a `cargo public-api` baseline
once the crate is consumed.

### 1.4 Open

- Whether the seven desktop colours above join the crate now or when a Qt screen needs them. Recommended: when needed.

## 2. Input (stage 0, step 2; 2026-11-07)

`emusen_kit_core::input` is the kit's whole knowledge of input, in four parts with one job each, and no clock: every
time is an argument, as `PadNavigator.Feed(held, now)` took its time, so a test states the numbers exactly.

### 2.1 The one event type

`InputEvent` is the pad, the pointer and the keyboard in one enumeration: `Pad(Press)`, `Hover(ElementId)`,
`Click { target, button }`, `Wheel { notches }` and `Key { key, shift }`. `intent(event)` applies `EmuSen_Qt.md` §1.7's
rules and gives an `Intent`: hovering is `Focus(element)` (the ring moves as the walk would); a primary click is
`Activate(element)`, the same view-model call as A there; a right click and the mouse's back button are `Press(Back)`;
the wheel is `Step(notches)`, one list row or carousel step per notch; Tab and Shift+Tab are `Walk(±1)`; the arrows,
Return, Escape, Menu, Select, Page Up and Down, Home and End are the presses their pad buttons would be. So the Qt layer
only translates `QMouseEvent`, `QWheelEvent`, `QKeyEvent` and the pad's queue into `InputEvent` and asks for the intent;
no screen interprets a device.

`ElementId` is the host's handle for an element the pointer rests on; the kit gives it no meaning.

### 2.2 The pad's grammar and its repeat tiers

`UiButton` is `PadNavigator`'s: what a pad asks of the interface, named by what it does (Up, Down, Left, Right, Accept,
Back, Menu, Options, PageUp, PageDown, First, Last, Guide, Search, Random, Screensaver). The physical pad is not here:
which bit is which button is Endymion's, and the pad family's map onto these roles is `PadRouter`'s in `emusen-kit-qt`.
The grammar keeps the player's A/B swap (`UiButton::swapped`) and the key a declined press becomes for a stock control
(`UiButton::key`: directions to arrows, Accept to Return, Back to Escape, Menu to Menu, Options to Select).

`Grammar` is `PadNavigator` and `SceneRepeat` as one: `feed(held, now)` gives the presses of a tick, one as a button goes
down and, for the four directions and the two shoulders only, repeats while held. The repeats follow `RepeatTiers`: a
first delay, an interval, and an optional faster tier reached after a time held, whose switch may move several items at
once. Repeats are scheduled from when they fell due, not from the tick that noticed them, so a slow frame gives every
step the hold earned. The tiers are the measured ones:

| Tiers | Delay | Interval | Faster tier | Source |
|---|---|---|---|---|
| `OWN` | 400 ms | 80 ms | none | `EmuSen_Settings_Reference.md` §4.29 |
| `ESDE_LIST` | 500 ms | 114 ms | at 1.703 s a four-entry step, then every 15.9 ms | `EmuSen_BigPicture.md` §14.7 |
| `ESDE_CAROUSEL_FAST` | 500 ms | 180 ms | 80 ms from 1.58 s | §14.7 |
| `ESDE_CAROUSEL` | 500 ms | 200 ms | none | §14.7, also the grid's |

`menu_chord(held)` is true once as the guide button, or Options and Menu together, go down; `forget(held)` makes nothing
a press until it is let go, so the button that closed a menu does not reach what is under it (§4.29). `stick_direction`
turns a stick axis into a direction past 0.55, the C# front end's threshold.

### 2.3 The walk

`Walk` is the frame's focus walk of LunaP §202.4, with no control behind it: roles (navigation, content, detail,
actions, search), the frame's form (desktop, big screen, card), sections, a detail that is a page of its own on the big
screen or stands beside the content, a row's page, and actions with their pad buttons. `move_focus(±1)` steps the shown
roles in order and wraps; `press(button)` is ES-DE's grammar and says whether the walk used the press, so a host offers
it a button first and keeps what it declines:

1. the shoulders step the sections, wrapping, raising `SectionChosen` (a card has none to step);
2. A on the content, on the big screen, opens the detail's page and focuses it;
3. B climbs a level: a row's page closes, then the detail's page; the detail, actions or search give the focus to the
   content; on the desktop the content gives it to the navigation; at the top B is declined, the host's signal to close;
4. an action's own button invokes it, unless it is disabled, in which case the button is declined.

A change of form keeps the chosen section and the focused role, and closes the detail's page where the detail stands
beside. What the walk did comes back as `WalkEvent`s (`take_events`), so the host wires nothing into it.

**Where it differs from the C#.** `AdaptiveLayout` also owns the List card and an open cover, which take A and B ahead
of the walk; those are sheets in the Qt design (§1.7's "the active sheet or question" is offered a press before the
walk), so they are the host's, not the walk's. `HubLayout`, `GalleryLayout`, `PlanetLayout` and `PageLayout` each had a
`Move(dx, dy)` over their own rows and places; in Qt those are `ListView`, `GridView` and `PathView` answering the key
events a declined direction becomes, so no layout's move is ported.

### 2.4 What it is held to

`tests/input.rs`: every button presses once on the way down and only the six repeat; the own tiers at 400 then 80 ms;
the list's eleven steps at 114 ms, the four-entry step at 1.703 s and ten repeats of 15.9 ms in 159 ms; the carousel's
six steps at 180 ms then 80 ms from 1.58 s; the plain carousel's 200 ms; the chord once, and not for Start alone; forget;
the stick's 0.55; the swap and the declined press's key; every §1.7 rule for the pointer and the keyboard; and §202.4's
four rules, the walk and its wrap in both forms, a detail beside, a row's page, the swap before the grammar, a change of
form, and a layout with neither sections nor detail.

Measured on 2026-11-07 with three seeded faults, each reverted: the fast tier's switch moving one item rather than
four failed the list test; B on the desktop's content no longer climbing to the navigation failed the climb test; Tab
walking backwards failed the pointer-and-keyboard test.
