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

## 3. The audit rules over an abstract item tree (stage 0, step 3; 2026-11-07)

`emusen_kit_core::audit` is `EmuSen_InterfaceSpec.md` §4's rules and `EmuSen_Qt.md` §8.1's two additions, written
once over a tree the kit defines and no toolkit: `Item` (an id, a name, a `Kind`, a rectangle in design pixels, whether
it is visible, hit-testable, operable or kept out of view, what it clips and scrolls and which edges fade, its words as
measured, the name a reader hears, the colours it paints, what a click and A each call, and whether the wheel moves
it). The Qt harness of §8.1 will fill such a tree from `objectName`, `mapToScene`, `Accessible.name`, the words of
`Text` and `grabImage`; here the rules are tested on hand-built trees. Every rule gives `Fault`s, one line each.

| Rule | Function | Source |
|---|---|---|
| The fit audit: past its panel or its nearest clipping or scrolling ancestor (a scroller excuses only what it scrolls to, up and down), words past their button or row, overlap of two things read or pressed neither inside the other by more than the slack, a scroller under two rows of 42, a drawn item sliced by a scrolling edge that does not fade, words cut with an ellipsis, past their box or at the foot, capitals under Inter's at 16 (11.64 design pixels, slack 0.036; no floor on the desktop), a title or footer cut; popups and hidden subtrees skipped | `fit` | Settings Reference §4.83.2, §4.83.7 |
| Pad reach: from the focus, the recorded edges reach every operable element not kept out of view | `pad_reach`, `reached` | InterfaceSpec §4.2 |
| Pointer reach: a click at the centre of each element the walk reaches hits it or one of its own children and calls what A calls; the wheel moves every scroller | `pointer_reach` | EmuSen_Qt.md §8.1 |
| No chrome: in the big-screen and card forms no menu bar or status bar is visible | `no_chrome` | §8.1 |
| Names a reader hears: every operable element has a non-blank one | `named` | InterfaceSpec §4.3 |
| Palette reach: every painted colour is declared; a theme's scene and everything under it exempt | `palette_reach` | InterfaceSpec §4.4 |

`check(root, &State)` runs them all for one recorded state: its panel, form, focus, edges, declared colours, slack (a
pixel, or a pixel and a half at a scale above one) and the capitals' floor.

**Where it differs from the C#.** WiseMan's `FitAudit.Check` walked live Avalonia controls and read their templates;
LunaP §204 moved it onto the kit's controls. This one reads only the tree it is given, as `InterfaceRules.cs` read a
`SpecDocument`, so the same rules hold a recording from any front end. The slack and the floor are arguments rather than
constants, and a words item carries its capitals' height already measured, since measuring a face's H is the harness's
job. The pointer rules are new: the C# front end had none, which is why its big-screen Home took no mouse (§0.2 of
`EmuSen_Qt.md`).

### 3.1 What it is held to

`tests/audit.rs`: a clean big-screen state passes every rule; then one fault planted per rule and found, with the
boundary beside it: past the panel, past a scroller's side but excused past its foot, within the slack; words past
their button; an overlap, a picture inside a control (not one), half a pixel (not one), a row's own words (not one); a
scroller of 80 (too little) and 84; a row sliced by a scroller's foot with and without a fade, and a picture past a
sideways scroller's side; an ellipsis, words past their box and at the foot, capitals of 11.0, 11.59 and 11.61 and the
desktop's no floor; a title and a footer cut; a popup and a hidden subtree skipped; the pad not reaching a button, an
unseen row excused, no focus; a scrim covering a row's centre, a row's own words over it (not covered), a click that
calls something other than A, a scroller the wheel does not move; a status bar on the big screen and on a card, and on
the desktop (allowed); a nameless and a blank-named control; a colour outside the palette, and the same under a scene.

Measured on 2026-11-07 with three seeded faults, each reverted: an item past its limit on one axis only passing
failed the past-its-panel test, the sideways scroller case and the title-and-footer test; the chrome rule checking the
desktop form instead of the big screen failed the chrome test; a blank name counting as heard failed the names test.

## 4. The drawn world as data (stage 0, step 4; 2026-11-07)

The spike found that the bodies and the sky are the scene graph's to draw (`EmuSen_Qt.md` §10.4), so
`emusen_kit_core::world` gives geometry, colours and times and never a pixel: `Shape`s (vertices, closed or open, a
`Paint` that is a colour or a gradient's stops, a stroke, an opacity), `Frame`s of a time, and the curves between them.
Everything is a pure function of its inputs and the host's clock; nothing in the module keeps time or allocates per
frame where a host can hand it a buffer (`star_points`). The random numbers are Mulberry32 (`SeededRandom`), as LunaP
§219.3 chose, held to an independent run of the published generator.

| Part | What it gives | Source |
|---|---|---|
| `WorldBody`, `geometry(body, hue, focused)` | a planet (a disc of radius 44 in a box of 100 lit by a three-stop radial gradient from 36%, 32%; craters, bands, a cap, a ring in two sizes whose back half is drawn before the disc and front half after, moonlets, a highlight) or six facets in four inks of the hue with an edge and two sparks (shards, a gem), back to front, with `size_of`, `colours`, `facet_inks` and `glow` | LunaP §222.1, §222.2 |
| `place_stars`, `twinkle`, `star_points`, the two skies | 220 stars from the seed in the draft's order, a 0.55 to 1 twinkle, and the gradients' stops | LunaP §221.1 |
| `SkyGlow`, `glow_stops`, `glow_bounds`, `wrap_hue` | one soft glow whose hue eases the shorter way round over Pan, fading in and out, on the host's clock | LunaP §221.2 |
| `Timeline::MOONRISE`, `Splash`, `Frame` | the draft's spans at pace 1.55 for 8 s; a splash placed from its seed; the frame for any time and size (stars, the moon's outline, halo, fill and crescent shadow, each planet's place on its orbit and spring into the row, glow and flare, the sweep's band, the row's line, the words' spacing and rise); `strike_times` for the bells | LunaP §219.2, §219.3, §222.6; EmuSen_Interface.md §22 |
| `motion::Glide`, `motion::spring`, `Curve` | a value moving over a span of the host's clock; the draft's damped spring; the easings the tokens name | LunaP §102.1, §219.5 |

**Where it differs from the C#.** `LunaSplash` laid out the title and caption with Avalonia's text layout and reported
their boxes to the fit audit; text is Qt's here, so a `Frame` gives the words' size, letter spacing and baseline and the
Qt item measures them. The chime is not part of the `Timeline` (the C# record carried it): the sound is §5's, and a host
pairs `strike_times` with it. `WorldPaint` drew the ring's back as a whole ellipse under the disc; the geometry does the
same, so a scene graph may draw it as one closed stroke. The draft's `6.28` for a turn is kept as `DRAFT_TURN` rather
than tau, so the frames stay comparable with the seeded copy of the draft page LunaP §219.5 measured against.

### 4.1 What it is held to

`tests/world.rs`: Mulberry32 against an independent run for three seeds; the Moonrise timeline's every span, the pace,
the length, the star count, the seed and the fade; `draft_time`; thirteen bells at 1280 × 720 from 3.752 s to 5.548 s,
about 1.8 s apart, left to right, one planet ringing at the row's middle, no size no times; the spring's end at 0.998
and the cubic-out ease; the finished frame (every planet within 2 px of its slot on the row at 66%, the line the row's
width, the title's spacing closed to 10 and its baseline at 85%, the frame held after 8 s); the first frame dark; the
sweep at the row's middle half way with the middle planet flaring at 0.55 over 4.2 radii; a body's box; the sizes,
colours and glow; a plain planet's two shapes and the disc's gradient; the craters', bands', cap's, moonlets' and rings'
places, widths and opacities; shards and a gem inside their 100 × 160 box; the skies, the stars' ranges and their first
draws; the glow's stops, bounds, the shorter way round from 30 to 350 and its fade; a glide on the host's clock.

Measured on 2026-11-07 with three seeded faults, each reverted: the spring damping less (e^(-4v)) failed the spring
test, the finished frame and the mid-sweep frame; the sweep starting at the row rather than 60 units before it failed
the thirteen-bells test; the disc shrunk to radius 40 failed the planet geometry test. The crate was formatted with
rustfmt in this step (`rustfmt.toml`, 140 columns), which touched the earlier files' layout and nothing else.

## 5. The cues and the bells as sample synthesis (stage 0, step 5; 2026-11-07)

`emusen_kit_core::sound` is LunaP's `LunaSounds` as data and one renderer: `Chime::MOONRISE` (the G major pentatonic
from G4 as a root and a ladder of semitones, four inharmonic partials at 1, 2.76, 5.40 and 8.93 times the note with
amplitudes 1, 0.18, 0.06 and 0.02 and falls to a ten-thousandth at 0.55, 0.22, 0.11 and 0.06 s, an 18 ms rise, a 50 ms
tail, a seeded detune of up to ±0.2%, no echo, gain 0.2, a low-pass biquad at 2.4 kHz with a Q of 0.5 dB), `Bell` (a
rung, a start, a decay scale, a gain and a corner), `render_bells` (the draft's Web Audio graph: dry partials summed per
corner, a fed-back delay line when a chime asks for one, the biquad as Web Audio's specification gives it with its Q in
decibels, trimmed where the mix falls 60 dB under its loudest sample, written as 48 kHz stereo `f32`), `render` for the
splash (one bell per strike time in ladder order), and `Cue` with its nine designs of LunaP §220.2, `QUICK_SELECT` and
`bell_samples` for one rung. Nothing is recorded; every sound is made from these numbers.

**Where it differs from the C#.** `LunaSounds` cached each cue's bytes in a static dictionary; the crate keeps no
static and renders on request, and the kit's `Bells` item renders each cue once and keeps it (the house rule on
statics). Samples are `f32` frames rather than bytes, so no byte order is assumed. The chime is not part of the
timeline (§4). The first sample's placement rounds halves to even, as .NET's `Math.Round` did, so a render matches the
C#'s frame for frame given the same strikes.

### 5.1 What it is held to

`tests/sound.rs`: the ladder's thirteen pitches (392 to 1975.53 Hz) and D7 for a fourteenth; the chime's every number;
each cue's strikes (rungs, starts, the last Launch bell's longer decay, Refused's 900 Hz corner); the defaults (60%,
on, 12 dB under a game, only Saved and Loaded over gameplay); each cue's and the quick select's notes present at their
strike times, read by Goertzel over the ladder's thirteen pitches; two renders identical and an empty one silent; each
cue's loudest sample within 0.05 and 0.6; one bell at its rung; thirteen bells at 1280 × 720 lasting between 1.7 and
2.6 s from the first strike, each planet ringing its rung, at the cues' level.
