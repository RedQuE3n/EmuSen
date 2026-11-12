# EmuSen.Kit: the pure-Rust heart (`emusen-kit-core`)

The design is `EmuSen_Qt.md` §1.4: two crates and one QML module carry what was LunaP's. This page is the record of the
first crate, `emusen-kit-core`, at `Frontend/kit-core`: what it owns, where each value comes from, what it was held to,
and where it differs from the C# it replaces. The second crate, `emusen-kit-qt`, is §7, and the module's `.qml` files
join it step by step.

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
| The raised shadow (added 2026-11-09, §7.55) | §5.4 | `tokens::raised_shadow(Form)` |
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
`Shadow`, `type_scale`, `spacing`, `focus_marks`, `raised_shadow`, `header_size`, `safe_area_of`, `clamp_text_scale`,
`MAX_TEXT_SCALE`, `SAFE_AREA_FRACTION`, `TEXT_FLOOR`, `GRAPHIC_FLOOR`. `emusen_kit_core::motion`: `MotionToken`, `Curve`, the breath's
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
The grammar keeps the player's swap (`UiButton::swapped`: Accept and Back trade places, and since 2026-11-08 Search and
Screensaver, the North and West buttons, as the C# front end and ES-DE trade them) and the key a declined press becomes for a stock control
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
| The fit audit: past its panel or its nearest clipping or scrolling ancestor (a scroller excuses what it scrolls to, along the axis it scrolls), words past their button or row, overlap of two things read or pressed neither inside the other by more than the slack, a scroller under two rows of 42, a drawn item sliced by a scrolling edge that does not fade, words cut with an ellipsis, past their box or at the foot, capitals under Inter's at 16 (11.64 design pixels, slack 0.036; no floor on the desktop), a title or footer cut; popups and hidden subtrees skipped | `fit` | Settings Reference §4.83.2, §4.83.7 |
| Pad reach: from the focus, the recorded edges reach every operable element not kept out of view | `pad_reach`, `reached` | InterfaceSpec §4.2 |
| Pointer reach: a click at the centre of each element the walk reaches hits it or one of its own children and calls what A calls; the wheel moves every scroller | `pointer_reach` | EmuSen_Qt.md §8.1 |
| No chrome: in the big-screen and card forms no menu bar or status bar is visible | `no_chrome` | §8.1 |
| Names a reader hears: every operable element has a non-blank one | `named` | InterfaceSpec §4.3 |
| Palette reach: every painted colour is declared; a theme's scene and everything under it exempt | `palette_reach` | InterfaceSpec §4.4 |
| Contrast: words drawn over what is behind them reach 4.5:1, or 3:1 for large type (24 px, or 18.66 px bold), over all but a spare tenth of what is behind them (since 2026-11-08, §7.10); words crossing a scroller's faded edge are not held (since 2026-11-09, §7.35) | `contrast` | WCAG 2.1, 1.4.3 |

`check(root, &State)` runs them all for one recorded state: its panel, form, focus, edges, declared colours, slack (a
pixel, or a pixel and a half at a scale above one) and the capitals' floor.

**Where it differs from the C#.** WiseMan's `FitAudit.Check` walked live Avalonia controls and read their templates;
LunaP §204 moved it onto the kit's controls. This one reads only the tree it is given, as `InterfaceRules.cs` read a
`SpecDocument`, so the same rules hold a recording from any front end. The slack and the floor are arguments rather than
constants, and a words item carries its capitals' height already measured, since measuring a face's H is the harness's
job. The pointer rules are new: the C# front end had none, which is why its big-screen Home took no mouse (§0.2 of
`EmuSen_Qt.md`). A scroller excuses what lies beyond it along either axis it scrolls, where `FitAudit.Check` excused only up and
down, since a pad never scrolled sideways there (decided 2026-11-08: the Qt themes' strips scroll sideways under the pad,
Horizon's first). What crosses a scroller's edge is still held by the edge rule, which accepts only a faded edge. A
flickable is a scroller along an axis when its content and its margins together are longer than it, since Qt scrolls over
the margins too; the harness counted the content alone until 2026-11-08, when Horizon's strip (1,220 of content, margins
of 64, 1,280 wide) was read as not scrolling and its last console, resting 4 units past the window under the faded
edge, was called past its panel.

### 3.1 What it is held to

`tests/audit.rs`: a clean big-screen state passes every rule; then one fault planted per rule and found, with the
boundary beside it: past the panel, past a scroller's side but excused past its foot, within the slack; words past
their button; an overlap, a picture inside a control (not one), half a pixel (not one), a row's own words (not one); a
scroller of 80 (too little) and 84; a row sliced by a scroller's foot with and without a fade, a picture sliced by a
sideways scroller's side, and a window-wide sideways strip excusing what lies beyond it, sliced at its faded edges and
cut at an unfaded one; an ellipsis, words past their box and at the foot, capitals of 11.0, 11.59 and 11.61 and the
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
than tau, so the frames stay comparable with the seeded copy of the draft page LunaP §219.5 measured against. Since
2026-11-08 a frame with no size has no flare, a planet carries the draft's inks and focus, and the row's line its width
(§7.10).

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

Measured on 2026-11-07 with three seeded faults, each reverted: the ladder climbing ten semitones an octave failed
the ladder test; the chime's gain at 0.02 failed the chime's constants and the thirteen bells' level; every bell
rendered a rung high failed the notes-present, one-bell and thirteen-bells tests, which is what the Goertzel reading
is for.

## 6. What stage 0 still owes

This crate is the first of stage 0's items (`EmuSen_Qt.md` §7). Not here, by design: `emusen-kit-qt` (the cxx-qt
items over these decisions), the `EmuSen` style, the two client crates, the first models, the pad bridge, the test
harness that fills §3's tree from a Qt item tree, the spec writer and the AppImage. Not carried from LunaP, and said
so above: the hub, gallery, planet and page layouts' own arrangement and `Move(dx, dy)`, which are Qt Quick's layouts
and views in the design; the List card and the open cover, which are sheets; text layout, which is Qt's.

## 7. The Qt half: `emusen-kit-qt` (stage 0, step 6; 2026-11-07)

`Frontend/kit-qt` is the cxx-qt 0.10.0 binding that puts kit-core's decisions on Qt (`EmuSen_Qt.md` §1.4): a library
crate with `links = "emusen-kit-qt"` that exports its cxx-qt interface, so the gallery and the harness, which link it,
register its QML module by running `CxxQtBuilder` themselves. Its dependencies are `cxx`, `cxx-qt`, `cxx-qt-lib`
(`qt_full`) and `emusen-kit-core` by path; nothing of the runtime or the client crates. It lives in a cargo workspace,
`Frontend/Cargo.toml`, which excludes `kit-core` on purpose: the workspace would take over kit-core's lock file, and
kit-core's fence test reads that file. Every decision stays kit-core's; the crate translates Qt's types, events and
numbers to and from it, and a reader audits it by checking each translation against the kit-core function it calls.

The QML module is `EmuSen.Kit` 1.0, registered by `build.rs`, its `.qml` files in the resource system under
`qrc:/qt/qml/EmuSen/Kit/`. Its surface is the `.qmltypes` of §7.6 (rule Q).

### 7.1 The enumerations (`kinds`)

Each mirror is a `Q_ENUM_NS` in a namespace element of its own, since a QML namespace resolves a key by name alone and
`Back` is a pad button and a cue, `Search` a pad button and a role: QML writes `Kit.Pad.Accept`, `Kit.Forms.BigScreen`,
`Kit.Roles.Content`, `Kit.Variants.Dark`, `Kit.Repeat.EsdeList`, `Kit.Bodies.Planet`, `Kit.Rings.Big` and `Kit.Cues.Back`
under the module's alias (§7.8): `PadButton` is `UiButton` in its order (the conversion goes through `UiButton::index`
and `UiButton::ALL`, and `tests/kinds.rs` names every pair so a reordering on either side fails); `Form` is the frame's
`WalkForm` with its three values, and the tokens read Card as the big screen's column; `Role` is the walk's `Role` plus
`NoRole` for no focus; `Variant` and `Tiers` are the tokens' variant and the grammar's four measured repeat sets.

### 7.2 `Tokens`

A QML singleton with four inputs, `form`, `variant`, `textScale` and `reducedMotion`, and 71 read-only properties, one
per token, recomputed by `column::Column::compute` from kit-core whenever an input changes: the 18 colours as `QColor`
(`surface`, `surfaceRaised`, `inputSurface`, `voidFill`, `scrim`, `captionSurface`, `border`, `rule`, `text`,
`textStrong`, `muted`, `accent`, `overAccent`, `success`, `attention`, `error`, `overArt`, `overArtScrim`); the type
scale as pixel sizes (`fontDisplay` to `fontGlyph`) and `headerSize`; the spacing (`base`, `rowHeight`,
`controlHeight`, `paneGutter`, `paneGap`, `cardWidth`, `titleRow`, `helpBar`, `tileWidth`, `tileHeight`, `tileGap`,
`rowRadius`, `tileRadius`) and `safeArea` as `QMarginsF`, with `safeAreaOf(width, height)` for a screen's own; the
focus marks (`ringThickness`, `ringOffset`, `barWidth`, `tileScale`, `hoverBorder`); the eight motion tokens as
milliseconds with reduced motion applied (`instant`, `quick`, `enter`, `exit`, `pan`, `reel`, `fade`, `poster`), the
fade of each (`quickFade` … `posterFade`; `fade` itself is the Fade token's fade, 120 ms under reduced motion) and each
token's curve as `QEasingCurve::Type`'s number so QML binds `easing.type` to it (Linear 0, InQuad 1, OutQuad 2, OutCubic
6); and the breath (`breathDelay`, `breathPeriod`, `breathLow`, `breathes`). The properties are read-only on purpose:
a theme overrides token values through §3.4's mechanism, never by writing the singleton.

Three tokens are renamed from the design's table: OnAccent, OnArt and OnArtScrim are `overAccent`, `overArt` and
`overArtScrim`. In QML a property named `onX` reads as a signal handler, and cxx-qt's connector for `accent`'s change
signal is `on_accent_changed`, the same name a property `on_accent` would give its own signal; the build failed on
exactly that collision on 2026-11-07. Font families are not here: kit-core has no such token (§1.3), and the one owner
rule says one joins kit-core first (§7.7).

**A theme's colours (2026-11-08).** `Tokens` holds a table of colour overrides, each for one variant, set from Rust
(`set_overrides`, not a QML property) and consulted before the kit's column whenever the column is computed, so a switch of
variant takes the theme's colour for that variant. The table belongs to one engine's singleton: the program sets it on a
theme's engine only, so the program's own engine, which draws every card, keeps the kit's colours (`EmuSen_Qt.md` §3.5.1).
A token's `Tokens` name is kit-core's `ColourToken::property`, which theme-check reads too; a theme's colour is parsed by
`Colour::parse`.

### 7.3 `FrameController`, and the three ways in

One `FrameController` per `Frame`, over one kit-core `Walk`. Its writable properties feed the walk (`form`, `hasDetail`,
`detailBeside`, `navigationShown`, `actionsShown`, `searchShown`, `sections` as a `QStringList`, `selectedSection`,
`sheetOpen`); its read-only ones mirror it after every call (`focusedRole`, `detailOpen`, `pageOpen`); its signals are
the walk's events (`sectionChosen`, `focusMoved`, `detailOpened`, `detailClosed`, `pageClosed`, `actionInvoked`) plus
two of its own, `sheetPressed` and `backDeclined`. Its invokables are the walk's verbs (`moveFocus`, `focusRole`,
`stepSection`, `openDetail`, `closeDetail`, `openPage`, `closePage`, `setAction`, `clearActions`, `press`) and the three
ways in:

- **The pad.** `offer(button)` is §1.7's order, as the pure function `frame::route` states it and `tests/route.rs`
  holds it: an open sheet or question first (`sheetOpen`, and the press comes back as `sheetPressed` for the sheet's
  own handler), then the walk, then, declined, the focused control as the key `UiButton::key` names (`send_key`, the one
  C++ function of this step: a `QKeyEvent` press and release sent to `QGuiApplication::focusWindow()`, which
  Qt Quick delivers to the active focus item). B declined at the top also raises `backDeclined`, the host's signal to
  close. The player's A/B swap is applied once, in `PadRouter` (§7.4); the walk's own `set_swap_accept_back` stays off,
  so the fact has one owner.
- **The pointer.** `Walkable.qml`, the module's first component, is the §1.7 declaration made once: an `Item` with a
  `HoverHandler` calling `controller.hoverAt(item, hovered, at)` (only the pointer's own move focuses, §7.39, §7.108; `hover(item, hovered)`, unguarded, stays for any other caller) and one `TapHandler` for the left, right and back buttons
  calling `controller.click(item, button)`. Rust maps Qt's button to kit-core's `PointerButton` (`qt_keys`), asks
  `intent` and acts: `Focus` gives the item the active focus (`QQuickItem::forceActiveFocus`, called through cxx-qt's
  `extern "C++Qt"` without glue); `Activate` focuses the item and then offers Accept through the same `offer`, so a
  click and A take one path and reach the same call, the `activated` signal `Walkable` raises on Return or Enter;
  `Press(Back)` offers B. No screen interprets a device, and nothing is per screen.
- **The keyboard.** `key(key, modifiers)` takes what the focused control declined and propagated up to the frame:
  Backtab is Tab with Shift, Enter is Return, and the rest is `qt_keys`' table against kit-core's `Key`; `intent` then
  gives `Walk(±1)` for Tab or the press a key means, which goes to the sheet or the walk and never back out as a key,
  so a key cannot loop.

### 7.4 `PadRouter`

A QML singleton whose queue tests push `PadEvent`s into, as Endymion's reader does (§7.15): `push
(button, down)` queues, `pump()` applies the queue on the router's own clock (an `Instant` from construction; no
static) and `pumpAt(ms)` on a stated one, so a test writes the exact times kit-core's tests write. Each pump runs
`router::Router`: the queue sets the held buttons, `Grammar::menu_chord` is checked (once, and then `forget`, so the
button that opened a menu reaches nothing under it, raised as `menuChord`), and `Grammar::feed` gives the tick's
presses, each swapped by `swapAcceptBack` and raised as `pressed(button, repeat, jump)`. `tiers` chooses the repeat set.
A `Frame` consumes presses through a `Connections` on the singleton, enabled while it holds the active focus, calling
`controller.offer`; a `Timer` in `Frame` pumps while a frame is shown (step 8). Both arrive with `Frame.qml`.

### 7.5 The drawn items: `Body`, `Sky`, `Moonrise` and `Bells` (stage 0, step 7; 2026-11-07)

**Drawn by Qt Quick Shapes, not by a geometry node of our own.** `EmuSen_Qt.md` §10.4 said the bodies and the sky
should be the scene graph's, a `QSGGeometryNode` or a `ShaderEffect`. Both were weighed against §8.1's harness, which
runs on the software backend: that backend draws no custom geometry node and no shader effect, so a gallery grabbed
under it would show no bodies at all, and a geometry node also means hand-written C++ for the node, its material and its
vertex layout. Qt Quick Shapes is the scene graph's own vector item: on a GPU its geometry
renderer builds exactly those geometry nodes, and under the software backend its own renderer paints the same paths, so
one `.qml` draws the same picture on both. kit-core's `Shape` (an outline, closed or open, a solid or radial fill, a
stroke, an opacity) maps onto `ShapePath` with a `PathPolyline` one to one, which is the design's intent: the world as
data, the framework drawing it. So the kit's own part is a model, not a renderer.

- **`BodyShapes` and `Body.qml`.** `BodyShapes` is a list model over `world::rows`, one row per kit-core shape of
  `geometry(body, hue, focused)` in the body's unit box (100 by 100, or 100 by 160), rebuilt (a model reset) when
  `shape`, `hue`, `focused`, `craters`, `bands`, `cap`, `ring` or `moons` change; `unitWidth` and `unitHeight` are
  read-only. A row's roles are `outline` (a `QPolygonF`, the closed outline with its first point repeated), `fillInk`,
  `hasGradient` with `gradientX`, `gradientY`, `gradientRadius` and four stops `stop0At`/`stop0Ink` to `stop3At`/`stop3Ink`,
  `strokeInk`, `strokeSize` and `inkOpacity`. `Body.qml` is an `Item` with one `Shape` per row from a `Repeater`, each
  with one `ShapePath`, a `RadialGradient` bound to the four stops and switched in by `hasGradient`, round caps and
  joins, and a `Scale` transform from the unit box to the item's size, so a resize (the focused tile growing from 58 to
  84) costs no model work; its `implicitWidth` and `implicitHeight` are the unit box. Two limits, stated: a gradient
  carries four stops at most and a shorter one repeats its last, because QML cannot repeat a `GradientStop` without
  logic (kit-core's bodies use three, the glow four); and a linear paint, which no body uses, is drawn as its first
  stop's colour.
- **`StarField` and `Sky.qml`.** `StarField` is a list model of the stars placed once by kit-core from `seed` and
  `count` (Moonrise's by default, so the splash and the screens share one sky), with `moonriseSky` choosing the
  gradient's stops (`stop0At`/`stop0Ink` to `stop2…`), `width`, `height` and `starsOpacity`, and a glow driven by
  `glowHue` (negative for none), `glowX` and `glowY` as shares, whose eased state (`SkyGlow`) is read as `glowVisible`,
  `glowCenterX/Y`, `glowRadiusX/Y` and four stops. `tick(seconds)` is the host's clock: `star_points` fills one buffer
  (no allocation per frame) and every row's roles are announced at once. `Sky.qml` (its paint one clipped `backdrop` item since §7.10) is a `Rectangle` with the three-stop
  gradient, a `Repeater` of rounded `Rectangle`s for the stars (a disc of a pixel or two needs no path), one `Shape`
  for the glow (a circle with the radial gradient, scaled to the ellipse by a `Scale` transform, since QML's radial
  gradient is circular) and a `FrameAnimation` calling `tick` while `animating`. *[own, because the per-tick update of
  220 rows is a cost to measure: 220 opacity bindings a frame; if it shows on the handheld, the stars become one
  `QSGGeometryNode` behind a GPU-only path with the software backend keeping the rectangles.]*
  **Still when nothing moves** (2026-11-08, `EmuSen_Qt.md` §9.3.2). A screen's stars hold still, each at its twinkle for
  time 0, as LunaP's `LunaSky` draws them (LunaP §221.2; `SkyState::tick` ignores the clock unless the sky is
  Moonrise's, and a tick of a screen's sky announces the glow alone, not the 220 rows). `glowSettled` (read-only, on
  `StarField` and `Sky`) is kit-core's `SkyGlow::is_settled` at the clock's time, and the `FrameAnimation` runs only
  while it is false, so the clock runs while the glow eases after a focus move and stops when it is at rest. The clock is
  the sum of the animation's `frameTime`s, not its `elapsedTime`, because `elapsedTime` starts again from zero at each
  start (measured with Qt 6.11.2: frame 0 after a restart reads 0.016 s), which would put the clock before the glide it
  is easing. Moonrise's own `Sky` stays `animating: false` with the splash's time bound in, so its stars twinkle for the
  splash's length and stop with it. Rule Q's registry was reviewed and copied: one addition, `StarField.glowSettled`, declared last so no existing property's index moves.
- **`MoonriseModel` and `Moonrise.qml`.** The model holds the splash's bodies (`addBody(name, hue, shape, craters,
  bands, cap, ring, moons)`, `clearBodies`), places `Splash` from `Timeline::MOONRISE` at the first tick after a change,
  and on `tick(seconds)` reads `Splash::frame` into read-only properties (the moon's centre, radius, halo, outline
  drawn, fill and shadow; the sweep's band; the line; the words' sizes, spacings and baselines; `starsOpacity`,
  `unit`, `finished`) and the planets into rows (`planetX/Y`, `planetRadius`, `planetOpacity`, the glow and flare, the
  body's box and its kind, hue and features), with a model reset only when a planet appears. The nine colours of the
  draft are constant properties. `Moonrise.qml` lays the frame out: a `Sky` with the model's seed and star count, the
  halo and crescent shadow as rounded rectangles, the lit disc as a `Shape` with a radial gradient, the outline as a
  `PathAngleArc` whose sweep is `360 × outlineDrawn`, a `Repeater` of worlds each with its glow, `Body` and flare, the
  sweep band as a horizontal `Gradient`, the line, and two `Text`s for `title` and `caption`, set by the host (the kit
  owns no words), placed by their baselines. A `FrameAnimation` ticks until `finished`.
- **`Bells`.** A QObject over `BellCache`: `prepare(cue)` renders a cue through kit-core the first time and keeps it,
  `prepareAll`, `durationMs`, `peak`, `quickSelectFrames`; `renders` counts renders so a test sees the cache hold,
  `rate` is 48 000. Playback through Endymion's interface player is `BellPlayer`'s (§7.15); the pairing of Moonrise's
  `strike_times` with the chime is owed (§7.7).

**Two cxx-qt facts found here.** A `#[qenum]` named `Roles` in three bridges clashes: cxx-qt emits `using Roles =
enum class X::Roles` at namespace scope in each header and the module's type registration includes them all, so each
model's roles have their own name (`BodyRole`, `StarRole`, `PlanetRole`). And an `extern "C++Qt"` base type may be
declared once per crate: cxx-qt generates an upcast shim named for the type (`rust::cxxqt1::…QAbstractListModel_upcastPtr`),
so a second bridge declaring `QAbstractListModel` fails the link with a duplicate symbol; `src/base.rs` declares it and
the models alias it (`type QAbstractListModel = crate::base::ffi::QAbstractListModel`).

**Held to** (`tests/world.rs`, `stars.rs`, `bells.rs`, 10 tests): a planet with every feature gives twelve rows in
kit-core's order with the opacities kept and closed outlines closed once; the disc's radial gradient at kit-core's
centre, radius and stops, padded to four; solid, stroked and bare shapes; shards and a gem inside their tall box; the
stars are Moonrise's draws, fill the screen and vanish at no opacity; the two skies; the glow comes in over Pan to the
strength, at the radii, clear at its edge, and goes with no hue; a cue renders once, durations are stereo frames at
48 kHz, the cues mirror kit-core's. Measured on 2026-11-07 with four seeded faults, each reverted: `pad_stops`
repeating the first stop failed the disc test; a closed outline left open failed the rows test; the two skies swapped
failed the skies test; a cache that rendered on every ask failed the render-once test. `qmllint` passes the four `.qml`
files against the built module (`Body.qml` and `Sky.qml` carry `pragma ComponentBehavior: Bound`, as their delegates
read the outer model). No C++ was added: the count stays at 23 lines.

### 7.8 The `EmuSen` style and the kit's controls (stage 0, step 8; 2026-11-07)

**The style** is `EmuSen_Qt.md` §1.5's: a directory of control files, `Frontend/kit-qt/style`, each composing its
`QtQuick.Templates` type and painting in `Tokens`, with Basic as the fallback for a control not written. It travels in
the kit's binary as a resource module named `EmuSen` at `qrc:/qt/qml/EmuSen/`, beside `EmuSen/Kit`, with a hand-written
`qmldir` (`style/EmuSen.qrc`): Qt 6 resolves a custom style by importing a module named for the style, so
`QQuickStyle::setStyle("EmuSen")` before the engine loads is `import EmuSen` (measured 2026-11-07: a module
`QtQuick.Controls.EmuSen` was not found, with "module EmuSen is not installed"; the `QtQuick.Controls.*` names are the
built-in styles'). cxx-qt 0.10 registers one QML module per crate, so a second `QmlModule` was not open, and a resource
module is the plainer form anyway, since the style has no Rust type. Twenty-one controls are written: `Button`, `CheckBox`, `RadioButton`, `Switch`, `Slider`, `ComboBox`,
`TextField`, `Label`, `ItemDelegate`, `ScrollBar`, `TabBar`, `TabButton`, `ProgressBar`, `ToolTip`, `Menu`,
`MenuItem`, `MenuSeparator`, `Popup`, `Dialog`, `DialogButtonBox` (because `Dialog` instantiates it, and a Basic one
would paint Basic's colours into a sweep) and `Pane`. The rules every file keeps, held by `tests/style.rs`: it imports
`QtQuick.Templates as T` and `EmuSen.Kit`, it paints only `Tokens.*` (no `palette.*`, nothing from Basic), its sizes are
the tokens' (`controlHeight`, `rowHeight`, `base`, `rowRadius`, `tileRadius`, `paneGutter`, the type scale), and the
focus is marked as §5.4 says: the ring (`ringThickness` in the accent) on `activeFocus`, the hover border
(`hoverBorder`), the focus bar (`barWidth`) at a row's left. **`activeFocus`, not `visualFocus`**: Qt shows the latter
only for keyboard focus reasons, and a pad moves the focus with `forceActiveFocus`, which is "other"; the design makes
every input an equal, so the ring follows the focus however it got there. The marks a reader would see drawn
(a check, a radio dot, a chevron) are plain rectangles and discs for now; drawn glyphs are owed (§7.7). `IconLabel`,
`CheckLabel` and `PlaceholderText` come from `QtQuick.Controls.impl`, as every Qt style's files do, so no text or icon
layout is written here.

**The kit's controls**, in `Frontend/kit-qt/qml`, each a `.qml` of layout and handler lines only:

- `Frame`: a `FocusScope` with its own `FrameController` (exposed as `controller`, its `form` as `form`) and four
  slots, `navigation`, `content` (the default), `detail` and `actions`, plus `help`, the help bar's model. The forms
  are three arrangements of the same panes bound to `form`: the **desktop** has the navigation at the left
  (`navigationWidth`), the content beside it, the detail beside the content when `detailBeside`, the actions row at the
  foot and no help bar; the **big screen** keeps the safe area (`Tokens.safeAreaX/Y`), the navigation as a row of
  `titleRow` at the top, the content under it, the detail as a page over the content when `detailOpen` (or beside), the
  actions above the help bar, and the help bar flush on the foot (`EmuSen_Interface.md` §24.7); the **card** form puts
  the scrim over everything and the content in a column of `cardWidth`. The pad reaches it through a `Connections` on
  `PadRouter`, enabled while the frame holds the active focus, whose one line is `controller.offer(button)`; a `Timer`
  pumps the router on the wall clock while `PadRouter.clockRunning` (a test turns it off and pumps at stated times);
  and `Keys.onPressed` hands the keys its controls declined to `controller.key`, which decides and reports acceptance.
  The desktop chrome (menu bar, status bar) is the desktop form's and not here (§7.7).

**The kit is always imported as `Kit`** (decided 2026-11-07): every `.qml` file writes `import EmuSen.Kit as Kit` and
`Kit.Frame`, `Kit.Tokens`, `Kit.Pad.Accept`, as Kirigami's convention has it (`import org.kde.kirigami as Kirigami`),
because Qt Quick Controls has a `Frame` too and theme authors will import it unqualified out of habit, and with two
unqualified imports whichever comes last silently shadows the other. `qml-rules` reports an unqualified `import
EmuSen.Kit` (§7.9); the kit's own files, the style and the gallery follow it, and the theme API's pages will repeat it.
- `HelpBar`: a row of entries from a model with `button` and `label` roles, each a flat `Button` whose press is
  `controller.offer(button)`, so a pointer has every action the bar names (§1.7); a ring stands where the glyph will be.
- `MenuRow` and `WholeRow`: `Walkable`s of `rowHeight`, so a click is A there and hovering focuses them; the row's
  surface raises on focus or when `current`, the focus bar shows at the left, the hover border on hover
  (`Walkable.hovered`). `MenuRow` has `label` and `value` words (`Accessible.name` is the label); `WholeRow` gives its
  whole width to its content (a slider, a switch with its words).
- `MenuPanel`: a raised panel with a `ListView` of `MenuRow`s (`model` with `label` and `value`, or a `delegate` of the
  host's), the current row holding the focus so Up and Down are the view's own, Return the row's `activated`, the
  wheel and a `ScrollBar` the view's; it raises `activated(index)`.
- `Card`: a raised column of `cardWidth` with the tile radius, its content inside the pane gutter.
- `Sheet`: a modal `Popup` that sets `controller.sheetOpen` while open, so the controller offers presses to it first:
  B comes back as `sheetDismissed` and closes it, every other press as `sheetPressed(button)` and as the key the
  sheet's focused control reads (`frame::sheet_route`, held by `tests/route.rs`). Its enter and exit are fades of
  the tokens' durations and curves.
- `Question`: a `Sheet` with `title`, `message` and `choices` (a list of words from the host) as `Button`s in a
  horizontal `ListView`, so Left and Right are the view's own and A is the current button's; `chosen(index)`.
- `Carousel`: a `PathView` on a straight path with `pathItemCount` from its width and `itemWidth`, snapping one item
  at a time over `Tokens.reel`; a `WheelHandler` steps one item per notch, Left and Right step by key, and a swipe is
  the view's own flick.

Not here, as the step said and §7.7 lists: `Keyboard`, `RatingPicker`, `DatePicker`, `NormalizedCanvas`, the glyphs.

**Held to** (`tests/style.rs`, three tests; `tests/route.rs` has the sheet rule): the qmldir lists every control file
once with its version, the `.qrc` carries the qmldir and every file, every file composes a template in the tokens and
reads no palette. `qmllint` passes the 34 kit and style files against the built module and the style directory, with
the project's import paths. Measured on 2026-11-07 with four seeded faults, each reverted: a sheet that never dismissed
failed the sheet-rule test; `Slider` dropped from the style's qmldir failed the qmldir test; `Frame.qml` dropped from
the kit module failed rule Q's qmldir comparison; a misspelt token in `Button.qml` (`surfaceRaise`) was caught by
`qmllint` as a missing property. Tests in the crate: 34. No C++ was added (23 lines).

### 7.9 The harness `emusen-qt-test` and the checker `qml-rules` (stage 0, step 9; 2026-11-07)

**The harness** is `EmuSen_Qt.md` §8.1's first tier and §8.3's shape, at `Frontend/qt-test`: a library crate the kit's
gallery and later the screens' tests link, with one `QGuiApplication` per process (so one test function per test
binary, the suite serial, as §8.3 says). `Harness::new` sets `QT_QPA_PLATFORM=offscreen`, `QT_QUICK_BACKEND=software`
and the EmuSen style before Qt starts, and `QT_FORCE_STDERR_LOGGING=1`, because Fedora's Qt sends its warnings to
journald when stderr is not a terminal (measured 2026-11-07: a scene that failed to load printed nothing in a test run
while the journal held "module EmuSen is not installed"; a harness that swallows Qt's words is useless). `load(url, w, h)`
loads a scene whose root is a `Window`, taking the root from the engine's `objectCreated`, shows it exposed and active
at the size, and gives a `Scene` with the verbs: `tree`, `click`, `right_click`, `key`, `wheel`, `focus`, `focus_item`,
`grab`, `property` and `pad`, plus `record_edges` and `audit`.

Added 2026-11-08 for scenes drawn with Vulkan (`EmuSen_Qt.md` §1.8.5): `load_prepared` hands the made window to the test
before it is shown, as a host that adopts a device prepares it (§7.16); `compositor::Compositor` starts `kwin_wayland
--virtual` on a socket of the test's own and points Qt's Wayland platform at it, so such a window is real and drawn yet
nothing appears on the desk; `virtual_pad` holds SDL's virtual joysticks, moved from the gallery's `pads` test so the
gallery and the program's window share one copy; `pad_chord_only` reads whether the router gives the chord alone.

- **The item tree becomes kit-core's.** `harness_tree` (C++) walks the window's items once and writes one JSON object
  per item: its type (Qt names a QML component's meta-object after it, `MenuRow_QMLTYPE_3`, so the type is the class
  name before `_QMLTYPE_`), its scene rectangle, visibility with the effective opacity, clip, whether it is hit-testable
  (it accepts a mouse button or holds a pointer handler), operable (a control with a focus policy, or a walkable with
  `activeFocusOnTab`, and not a focus scope whose content holds the focus), the words of a `Text` with `truncated`, the
  content size and the font's cap height, a flickable's scrolling axes and `interactive`, the edges it declares it fades
  (`fadeTop` … `fadeRight` properties, the protocol a scroller follows), its spoken name (`Accessible.name`, then its
  `text`, then its `placeholderText`) and the colours it paints (`color`, a drawn border's colour, a shape path's fill
  and stroke). `tree.rs` reads that into kit-core's `Item`s: the kind by what the dump says (a popup item, a menu bar,
  a scroller only when it scrolls, words, a picture, a row of the kit's or Qt's, a control), `unseen` for an operable
  item outside its nearest clipping ancestor (a row built out of view), the spoken name of a nameless operable item from
  the first words under it (as Avalonia's peers resolved it, InterfaceSpec §4.3), `Body`, `Sky` and `Moonrise` as
  scenes, and clear colours dropped. So every rule is kit-core's; the harness re-implements none.
- **Pad reach is measured, not declared.** `record_edges` focuses each operable item in turn and presses Up, Down, Left
  and Right through `PadRouter`'s queue on the harness's own clock (`clockRunning` off), reading where the focus lands;
  `audit` hands kit-core those edges with the panel, the form, the focus, the variant's eighteen colours and the slack.
- **The pad goes in where a pad goes in.** `pad(button)` pushes a down and an up into the router's queue and pumps at
  stated times; the router is reached as the engine's singleton, downcast to the kit's `PadRouter` by cxx-qt's `Upcast`.
- **C++**: `cpp/harness.cpp` and `harness.h`, the tree dump and QTest's verbs (`mouseClick`, `keyClick`, a wheel event,
  `qWait`, `qWaitForWindowExposed`, `grabWindow`, a property read, the singleton). QTest's mouse, keyboard and
  window helpers are declared only under `QT_GUI_LIB`, which CMake defines and cc-rs does not, so the file defines it.
  Since 2026-11-08 also a count of the window's `frameSwapped` (kept as a dynamic property on the window, connected at
  the first read) and a list of the animations running under the items (`QQuickFrameAnimation` and
  `QQuickAbstractAnimation` with `running` true, each named by its nearest named item), which `Scene::frames`,
  `Scene::running` and `Scene::assert_still` read (`EmuSen_Qt.md` §9.3.2). A view's own highlight animations are
  private to Qt and not in the list; the frame count still sees them.

**What the harness found in the kit** on 2026-11-07, each fixed in the kit, not the test: a help-bar button took the
focus on a click, so the row it was meant to act on lost it (`focusPolicy: Qt.NoFocus`); a sheet took the focus out of
the frame's scope, so the frame's `Connections` went quiet while a sheet was open (enabled while `sheetOpen` too); a row
given the focus by a hover or a forced focus did not become the list's current row, so Down then jumped back (the panel's
row makes itself current on focus); a list cut rows at an edge that did not fade (the panel fades its scroll edges and
declares them); a scroll bar lay over the rows' right ends (it stands in the panel's margin); Qt's buttons ignore Return,
their key being Space, so A did nothing on a stock button (the style's buttons take Return and Enter as a click or a
toggle); a `TabBar`'s focus stopped at the bar (its list takes the focus and the current tab holds it); and `Walkable` is
now a `FocusScope`, so a row whose content asks for the focus hands it on. Two findings are owed to stage 1 (§7.7): a
`TextField` and a `ComboBox` take Up and Down themselves, so a vertical walk cannot pass them, and the four audited sizes
are the gallery's (step 10).

**`qml-rules`** is `EmuSen_Qt.md` §1.3's checker, a pure-Rust library at `Frontend/qml-rules` exposed as the third
command of the `emusen.dev` DianaSh module (`Platform/emusen-dev-module`, with its man page `qml-rules(1)`), per the
2026-10-26 rule that every tool is a DianaSh command. It reads `.qml` text with comments stripped and reports, one a line
as `file:line rule words` with a last line `ok` or `N findings`: a `function` other than a `Connections` handler of one
call; `Qt.callLater`; an array or object literal; words the player would read outside the bindings that take a name
(`objectName`, `font.family`, `source`, `fragmentShader`, `vertexShader`, `property`, `properties`, `sequence`, `Accessible.role`, `placeholderText`) and
colour literals; `Component.onCompleted` beyond taking the focus; a `ListModel` with rows; an import outside the allowed
list, or `EmuSen.Kit` imported without `as Kit` (§7.8). `--fixture` reads test scenes and the gallery as fixtures,
whose words, rows and literals stand in for models. `tests/rules.rs` holds every rule and the spike's three deliberate
breaks (`EmuSen_Qt.md` §10.3); `qt-test/tests/lint.rs` runs the checker and `qmllint`, with the project's import paths,
over every file of the kit, the style and the scenes on every test run.

**Held to**: `tests/menu.rs` (a menu scene: the panel and its eight rows in the tree, Down moves the focus and A
activates, a click at a row's centre focuses and activates it the same way, the help bar's Open acts on the focused row,
then the gate at 1280 × 800 from recorded edges, and the wheel), `tests/sheet.rs` (F1 opens the question, B dismisses it
choosing nothing, A chooses the current button), `tests/style.rs` (eleven styled controls present, the gate passes over
them, a grab is written), `tests/lint.rs`; `qml-rules/tests/rules.rs` (4); the dev module's two tests. Measured on
2026-11-07 with four seeded faults, each reverted: every item marked hit-testable in the dump put the panel's fade over
its rows and failed the menu scene's pointer reach; a colour read with its channels reversed failed the style scene's
palette reach; the unqualified import allowed failed the rules test; the frame deaf while a sheet is open failed the
sheet test. Two faults seeded first were not observable and are recorded as such: nothing hit-testable in the dump
changes nothing for rows, since kit-core's `operable()` makes an item hit-testable itself; and `unseen` never set
changes nothing in the menu scene, whose out-of-view rows are visible items the walk explores anyway.

### 7.10 The gallery and stage 0's gate (stage 0, step 10; 2026-11-08)

**The gallery** is `Frontend/kit-gallery`: the QML module `EmuSen.Gallery` (one file, `qml/Gallery.qml`) and a program,
`kit-gallery`, that opens it in a window with the EmuSen style; the gate loads the same scene headlessly through the
harness. One `Frame` holds three sections, which the walk steps with the shoulders (`controller.sections`); the
navigation shows them as a row of `TabButton`s on the big screen and a column on the desktop, pointer targets with no
focus of their own, as the help bar's buttons are, since the pad reaches sections by the shoulders and never by a
direction (§2.3).

- **Menu**: a `MenuPanel` of eight rows that scrolls; the screens' `Sky` with its glow behind four `Body`s (a planet
  with every feature, focused; a plain planet; shards; a gem); a `Carousel` of eight `Card`s, each with a world. The
  panel's Right reaches the carousel and the carousel's Up the panel, a path the fixture declares with `KeyNavigation`.
- **Controls**: eight `WholeRow`s holding every styled control (a `Button`, a highlighted one, `CheckBox`, `RadioButton`,
  `Switch`, `Slider`, `TabBar`, and a `ComboBox` beside a `TextField`), a `ProgressBar` and a `Label`.
- **Moonrise**: the splash over the whole window at its real size, as a player sees it. The frame stays under it at no
  opacity, so it keeps the focus and the shoulders still turn the page.

The `Question` opens on F1; the `HelpBar` and `NormalizedCanvas` frame every section. `NormalizedCanvas` is new here:
the big screen's 800 design units scaled to the window's height (`EmuSen_Interface.md` §24.7), inactive on the desktop.
The gallery's words, rows and lists are a fixture's (`qml-rules --fixture`). Not shown: `Bells`, which is sound (its
render and cache are held by `tests/bells.rs`), and the parts owed by §7.8.

**The gate** is `tests/gate.rs`, one test of about 40 s on the desktop. It runs `qml-rules` and `qmllint` over the
gallery, then visits 90 states: both variants, three forms, three sections, and the five sizes of the harness's
`SIZES` (1280 × 800, 1920 × 1200, 1920 × 1080, 1280 × 720, 3840 × 2160; the second, the handheld's own panel, added
2026-11-08, §7.13). In each state it focuses the first operable element, reads
the tree, sweeps the style (Controls), grabs, records the edges, and gives kit-core's `audit::check` the tree the edges
were recorded on: the fit audit, pad reach, pointer reach, no chrome, names, palette reach and contrast. A 73rd state is the
question open at 1280 × 800, audited alone as a player sees a sheet. The style sweep samples one grab at each styled
control's right end, inside its background, and requires a token of the variant *[own, because `EmuSen_Qt.md` §8.3's
`TestCase` is JavaScript, which §1.3 keeps out of QML, and the harness already holds the grab]*. A state with faults
writes `<state>.faults.txt` beside its grab: the faults, the tree's legend and the recorded edges. Two checks of the
kit's own promises run beside the rules: in the Menu section nothing under the carousel rests outside its box (within
the audit's slack of a pixel and a half); a list that scrolls has taken the height it was given (in the first ancestor
with 48 design pixels of room below the list, something stands below it and across it, or the list was cut short with
empty space under its fade); and the finished splash shows all 220 of its stars. **Measured on 2026-11-08:
73 states, no fault, 41.0 s and 41.7 s.** The grabs are written to `~/.cache/emusen/tmp/kit-gallery/` for a human look.

**Contrast, measured.** For every visible, enabled text the dump gives its colour with its opacity folded into the
alpha, its size and weight, and where its words are laid out: inside the paddings, the text's own width and height
placed by its alignment (an item's `boundingRect` ignores the paddings, so a check box's label box took in the
indicator). The harness then hides every text (opacity 0, given back afterwards, which keeps QML's bindings), grabs
once and samples each text's box on a grid a few pixels apart, up to 16 by 6 points: what is behind the words as drawn.
Kit-core composites the ink over each sample and computes WCAG 2.1's ratio; the tenth of samples that may fall short
lets a star or a rule's edge pass and catches words half over their own colour. A disabled control's words are exempt,
as WCAG exempts an inactive component's.

**What the gate found in the kit**, each fixed where it lived and no rule loosened:

1. kit-core: `Splash::frame` at no size gave a NaN flare (0/0 where the sweep met a planet at a unit of 0). A hidden
   splash ticks at 0 × 0, and the dump wrote the NaN as JSON's null. The flare is now 0 with no size, as
   `strike_times` already gives no times; `tests/world.rs` holds a 0 × 0 frame.
2. kit-core: the Moonrise caption at the draft's 15 units fell under the big screen's floor (§3: the capitals of 16
   design pixels of the desktop face, 11.64) at every size: in the system face its capitals measured 0.68 of its size
   (7.14 pixels at 10.5), so 10.2 pixels at 1280 × 720. It was raised to 18 units, then set back the same day: the
   draft is the splash's oracle (decided 2026-11-08 by the tester), so the caption is the draft's 15 units and the
   audit names it, `SMALL_TEXT_EXCEPTIONS` in `audit/fit.rs`, the one exception to the floor, with its reason.
3. kit-qt: a finished `Moonrise` did not follow a resize, since its frame was read only on the clock's tick, which
   stops when the splash finishes. The model now lays the frame out again at the shown time when its size changes.
4. kit-qt: `Carousel` spaced its items by the path's length over `pathItemCount` (132 units for items 204 units wide,
   so the cards overlapped) and drew items past its box over its neighbours. Items now sit one `itemWidth` apart about
   the centre, and the view clips. It is also an element the frame walks to: `activeFocusOnTab`, §1.7's
   `HoverHandler` and `TapHandler`, and a required `controller`, so the gate holds it to pad and pointer reach.
5. kit-qt: `WholeRow` was `rowHeight` tall whatever it held; on the desktop a 34-pixel control in a 24-pixel row
   overlapped the next row. It is now the taller of the two.
6. kit-qt: the sky's glow is 1.125 times the sky's height and spilled over its neighbours. The sky's own paint is now
   one clipped item that declares `backdrop`.
7. `NormalizedCanvas`: at the first layout, before the window had a height, a unit of 0 made the section bar's
   `contentX` NaN for good. The unit is 1 until the parent has a height.
8. Three layout faults seen in the grabs on 2026-11-08, each fixed where it lived. The gallery fixed its menu panel at
   five rows, so the list scrolled and faded with a third of the page empty under it; the Menu page is a `RowLayout`
   whose panel fills the height (`Layout.fillHeight`), and the fade shows only when rows overflow. The desktop form
   had no margin, its content flush against the window's top edge; the frame's edge is now the pane gutter there,
   the content starting after the navigation's right edge. The slider's handle was `textStrong`, black in the light
   variant; it is `overAccent`, as the switch's knob.
9. kit-qt: the card form had no surface: its content stood on the Scrim over whatever was behind, and "A label in the
   tokens" in the light variant was dark grey on black. The frame now draws the card's raised surface (`surfaceRaised`,
   the tile radius) a pane gutter outside the content area, which is `EmuSen_Qt.md`'s card over the theme.
10. kit-qt: Moonrise showed no stars. Kit-core draws no stars at no opacity, so the star model's rows went from 0 to 220
   with only `dataChanged`, and the view never made them; the splash begins at no opacity, the screens' sky at 1, which
   is why only the splash lost them. The model now resets when the stars appear or vanish.
11. kit-qt and kit-core: the Moonrise shadow was a whole disc drawn over the halo, and the halo a flat translucent disc.
    Compared with the approved draft (`~/.cache/emusen/tmp/moonrise/reference-8s.html`, its finished frame) and the C#
    `LunaSplash` side by side, both clip the shadow to the moon and draw the halo as a radial gradient, flat out to 0.6
    radii and clear at 3.2. Kit-core's `Moon` now gives the shadow as it is drawn (`shadow_outline`, the outline of the
    two discs' overlap, into the host's buffer), the halo's inner radius, the face's two-circle gradient (from a tenth of
    a radius at 0.35 and 0.3 radii up and left, to 1.1 radii) and the outline's width (2.2 units); Moonrise draws them
    in the draft's order, the outline before the face. `MoonriseModel`'s `shadowX`, `shadowY` and `shadowRadius` gave
    way to `shadowOutline`; rule Q's registry was reviewed and copied (before 1.0 the comparison is strict equality).
    The comparison is in `~/.cache/emusen/tmp/kit-gallery/compare/`: the draft, the C#, and Qt on the GPU and on the
    software backend at 1280 × 720, the draft's 13 hues and words.

**Moonrise against the C# reference**, measured on 2026-11-08. The C# `LunaSplash` render of the finished frame
(`Testing/moonrise/LunaSplash-1280x720.png`, 1280 × 720, the draft's 13 consoles and words) is the draft's picture
star for star (258 pixels of 921,600 differ by more than 2%, all glyph and circle edges). Five more differences made
Qt's frame match it:

- **The stars.** The splash's sky drew its stars as the screens' do (a radius of 0.7 at the height over 800) on its own
  clock. They now follow `LunaSplash`: 1.4 at the height over 720 (`SPLASH_STAR_SIZE`), twinkling on the draft's time,
  which the splash hands its sky (`MoonriseModel.draftTime` to `Sky.time`), so the finished frame holds still as the
  C#'s does. Placement was already the same seeded draws. A star is an antialiased disc (a `Shape`), as the canvas draws
  it: the software backend snapped a sub-pixel rounded `Rectangle` to whole pixels.
- **The faces.** The title is Barlow Condensed Light and the caption Barlow Medium, the C#'s own files (SIL Open Font
  License 1.1, `kit-qt/fonts/`, the licence `licenses/Barlow-OFL-1.1.txt` as for the C#, SHA-256 of the title's face beginning `2c37e1e6`, as LunaP §206.3
  records), loaded from the kit's resource `fonts/Fonts.qrc` by `FontLoader`.
- **The planets.** A plain planet (no world from the host) is the draft's disc: a two-circle gradient from
  hsl(hue, 70%, 78%) at a tenth of the radius about a focus 0.4 radii up and left, to hsl(hue, 55%, 38%) at the radius
  (`PlanetFrame`'s `plain`, `light`, `rim`, `focus`, `focus_radius`); a host's world is still a `Body`.
- **The row's line** is a stroke one unit wide centred on the row (`Line.width`), drawn over the planets as the draft
  draws it; it was a 1-pixel rectangle with its top on the row, below them.
- **The caption** is back at the draft's 15 units (above).

The gate checks it: at 1280 × 720, the big screen, dark, Qt's finished Moonrise is compared with the reference pixel by
pixel (`Scene::difference`). A pixel matches when the other picture has a pixel within 2 pixels and 24 levels on every
channel, either way round; inside the boxes of the title, the caption and the moon (4 pixels out), 128 levels. Up to 4
pixels may differ. The tolerance is antialiasing's: the reference was drawn by Skia with subpixel (LCD) text, colour
fringes no grayscale renderer reproduces, and its own coverage of a stroke's edge; Qt's software backend draws the same
edges under a pixel apart, and a glyph or the moon out of place still differs by far more than 128 levels. Measured:
0 pixels differ. The differing pixels are marked in `moonrise-difference.png` beside the grabs; the comparison
pictures, with the difference magnified eight times, are in `~/.cache/emusen/tmp/kit-gallery/compare/`. What it cannot
see: a star dimmer than 24 levels over the sky.

**What the harness read wrongly**, fixed in the harness. Each is a translation of Qt's tree; the rules are unchanged.

- Words were measured in their item's units (`contentWidth`, `contentHeight`, the cap height) while boxes were in the
  scene's, so under the canvas's scale every text at 1280 × 720 ran past its box and every text at 3840 × 2160 read as
  too small. The dump multiplies them by the item's scale in the scene.
- A turned Text was held to its scene box, the bounding box of its turned footprint, while its words were measured along its
  own axes, so Constellation's RECENT chart label, turned -90°, ran past a box 23 wide at 1280 × 800 though its own box held
  it whole (measured 2026-11-08). The dump now measures words by the scene's scale along the item's own axes and, for a
  turned item, records its own box (`own`, width and height unturned in the scene's units); the word rules hold the words
  to that box, and the placement rules (clip, panel, overlap, pointer) keep the scene box. Held to by `qt-test`'s
  `tests/turned.rs`: a turned word that fits its own box passes and one longer than its own box is still caught; seeded,
  the scene box in place of the own box failed the first.
- A `TextField` is a `QQuickTextInput`, not a `QQuickControl`, so no text field was operable or held to pad reach. A
  text input or edit with `activeFocusOnTab` now is.
- A focus scope was a container, not operable, whenever any child held its focus. A `PathView` gives its current card
  the focus, so the carousel stopped being operable once walked. It is a container only when that child is operable.
- A focus that lands inside an element (the carousel's current card) is reported as the walkable element holding it
  (`Tree::walkable_of`).
- An item wholly outside a clip that scrolls (a list's rows built out of view, a carousel's cards past its box) is not
  shown: Qt draws none of it and delivers no pointer to it. A `PathView` scrolls when it holds more items than it shows.
  Outside a clip that does not scroll, an operable item stays shown and `unseen`, as before, so a misplaced control is
  still past its limit.
- An item that declares `backdrop`, and everything under it, is a panel to the rules: the sky is behind the screen,
  not a picture a player reads, and the worlds pass over its glow by design.
- `set_property` takes a dotted path (`controller.selectedSection`) through `QQmlProperty`. New verbs: `colours_at`
  (one grab sampled at many points), `audit_item` (one item's subtree, as an open sheet), `set_variant` and
  `set_token_form` (the `Tokens` singleton, reached as the router is).

**The GPU against the software backend**, measured on 2026-11-08: the harness's own `Harness` run with
`QT_QUICK_BACKEND=rhi` and `QSG_RHI_BACKEND=opengl` on the offscreen platform renders on the RX 6800 (radeonsi, Mesa
26.2.4) through GLX, with no window on the desktop. Two differences showed:

- The software backend ignores a clip for Qt Quick Shapes (a red `Shape` at x = 150 inside a clipping `Item` 100 wide
  was red at (175, 145)); the GPU clips. (Fixed for `Body` and `PadGlyph` on 2026-11-08, §7.34.) So the carousel's worlds past its box, hidden on the GPU, were drawn over the
  menu's rows in the gate's grabs. The kit no longer depends on the clip: the carousel shows as many whole items as fit,
  an odd number about the centre (`pathItemCount`), so none rests outside its box on either renderer; the clip stays
  for the half items passing its edges during a step. The gate holds the promise (above).
- The stars, rounded rectangles under a pixel across, were drawn by the software rasterizer and nearly all lost on the
  GPU, so the grabs showed a sky the player did not see. They are now antialiased, which draws a sub-pixel star as a
  faint dot on both, as the draft's canvas does.

Compared over the whole frame at 12% tolerance, the two backends then differ only in the edges of glyphs and the help
bar's circles (0.14% of the pixels at 1280 × 800). The probe's offscreen GLX surface keeps the size it was created
with, so a state is grabbed in a window made at its own size; the gate stays on the software backend, which needs no
GPU.

**qml-rules' fence.** The back end's `emusen.dev` module depends on `Frontend/qml-rules`, so qml-rules has left the
Frontend workspace and keeps its own lock file, as kit-core does. `qml-rules/tests/fence.rs` holds its tree to no
EmuSen crate and nothing named for Qt or cxx, and its manifest to no path dependency and no workspace inheritance.

**A duplicate accepted.** `dupes` finds the opening of the list models' bridges alike (`moonrise.rs` and `sky.rs`, 114
tokens; `body.rs` under the threshold): each bridge declares the cxx-qt-lib types its model names, with the header
that declares each. A type can be aliased from another bridge, as `QAbstractListModel` is from `base.rs`, but its
header is still included in every bridge that names it, so the block would repeat either way. `dupes.accept` records
the pair with that reason.

**Held to**: the gate; `tests/world.rs`'s 0 × 0 frame in kit-core; the fence (2); the harness's own scenes (`menu`,
`sheet`, `style`, `lint`), which pass unchanged under the corrected reading. Measured on 2026-11-08 with five seeded
faults, each reverted: the Moonrise model not laying out again on a resize failed the gate (the words of the finished
splash past the window at 1280 × 720 and too small at 1920 × 1080); the carousel's old path failed the gate (cards past
its clip, cut at an edge with no fade); the dump's cap height in the item's units failed the gate (too small to read at
1920 × 1080); a path dependency in qml-rules' manifest failed both fence tests; the flare without the size guard failed
the 0 × 0 frame. That last test first asked at 1280 × 0, where the planets stand apart from the sweep and the division
gives an infinity, not a NaN, so the fault went unseen; it asks at 0 × 0 now, the size the gallery met.

Measured on 2026-11-08 with four more seeded faults, each reverted: the card form's surface not drawn failed the gate on
contrast in the light card form at all four sizes; the carousel's old `pathItemCount` (as many as cover its width, plus
two) failed the gate with cards resting outside it; the star model not reset failed the gate's count of the splash's
stars; the shadow drawn as its whole disc failed kit-core's test that the shadow stays inside the moon. Kit-core's
`tests/audit.rs` holds the contrast rule itself: 4.5:1 and the large type's 3:1 at 24 px and 18.66 px bold, the ink
composited over what is behind it, the spare tenth, and no ink nothing to judge. The gallery's menu panel fixed at five rows again
failed the gate with a list cut short in every form, variant and size. The first star moved 12.8 pixels to the right
failed the gate's comparison with the reference at 7 pixels, against 4 spare and 0 when clean.

### 7.11 The spec writer (stage 0, the rest, step 1; 2026-11-08)

`emusen-qt-test`'s `spec` module writes a shown state in `EmuSen_InterfaceSpec.md` §2's `spec 1` format, so that a Qt
screen is held to the C# recording of the screen it replaces by the C# side's own reader, comparator and rules
(`EmuSen_Qt.md` §8.2) *[own, because the format is EmuSen's own and Qt has no writer for it]*. Its one job is
translation: it takes kit-core's audit `Item` tree as `tree.rs` built it, the raw dump beside it and the walk, and
writes text. It decides none of the facts the rules read; each is the decision kit-core's audit already made, so a
recording and the audit cannot disagree about the state they describe.

**The header.** `state`, `size`, `kind` (the caller's: the gallery writes `menu` on the big screen and the card, `sheet`
on the desktop), `root` (the root item's QML type and `objectName`), `unit`, `slack` and `floor`, `frame` (the window),
`case` and `palette` (the variant's token colours, each once, as `#RRGGBBAA`). `unit`, `slack` and `floor` come from
`harness::Measures::of(form, height)`, the same value the audit's `State` is now built from. The banner, a comment every
reader skips, names the Qt front end.

**The tree.** Every shown item under the root, depth first, numbered from 0. An item kit-core does not show (Qt's
`visible` false, at no opacity, or wholly outside a clip that scrolls, §7.10) is left out with what is under it, and a
backdrop is one line, as the rules read it. Each key, from where:

| Key | From |
|---|---|
| `role` | the role Qt's accessibility gives the item (`QAccessible::queryAccessibleInterface`, what a screen reader is told), in the format's words, which are Avalonia's control types (`text`, `button`, `listitem`, `scrollbar`, …); a Qt role with no counterpart is `none` |
| `name`, `text`, `heard` | `objectName`; a text's words; the spoken name kit-core's item carries (§7.9) |
| `at`, `seen` | the scene rectangle; the part no clipping ancestor hides, where that differs |
| `op`, `unseen`, `off` | kit-core's operable and unseen; a disabled item |
| `drawn`, `nohit` | kit-core's `is_drawn`; not hit-testable |
| `limit`, `by` | on drawn items, kit-core's limit: the nearest ancestor that clips or scrolls (`scroll` when it scrolls down, else `clip`), or the window (`panel`) |
| `owner`, `in` | words whose parent is a control or a row, as kit-core's rule reads them; the scrollers above |
| `overflow`, `viewport`, `fades`, `vfixed` | a scroller's content height past its own, its rectangle, the edges it declares it fades, and no vertical scrolling |
| `layout`, `box`, `ellipsis`, `size`, `caps`, `face` | a text's laid-out size and its item's, in scene pixels; truncation; the font's size and capitals in design pixels; the family |
| `paints` | the colours kit-core reads, except under a scene (`Body`, `Sky`, `Moonrise`, a backdrop), which kit-core exempts from palette reach while the format exempts only whole scenes |
| `allow` | the reason of `SMALL_TEXT_EXCEPTIONS` for the item it names |

**The focus and the walk.** `Scene::walk` keeps every press's outcome (`Landing::At`, `Same`, `Lost`), and
`record_edges` is now the moves among them, so the audit and the recording read one walk. A press from or onto an item
not written (a row built out of view) names it by its `path:` of child indices, the format's reference for a control it
did not record. A focus that lands on an item made after the tree was read (a carousel's card scrolled in) is the
walkable element holding it, found through a fresh tree by its nearest ancestor the first one holds.

**The walk puts back what each press moved**, as `EmuSen_InterfaceSpec.md` §3's walk puts back lists' selections and
scroll offsets: a recording must not depend on the states visited before it. After every press the harness reads back
and restores what the tree recorded of two kinds. The selections, values and scrolls Qt Quick's own classes declare
(`currentIndex`, `value`, `contentX`, `contentY`, and a path's `offset`, each only where a `QQuick` class declares it
as a number and it is writable, so a binding is written only where a press already broke it) are written back where they differ;
a path's offset is not written, since it follows its current index by the view's own animation, and writing it
mid-move was measured to leave the carousel between cards; the views are settled instead (below). A focus scope's
remembered item is given back with `setFocus`, which moves no focus outside the scope. Found on 2026-11-08: the dark
and the light Controls state at 1280 × 800 differed by six focus edges, because the first state's walk had pressed
Right on the tab bar (changing the tab) and Right from the combo box (leaving its row remembering the text field), and
every later state began there; it showed at one size only because that state is visited first. Found the same day: Qt Quick's
`RectangularShadow` declares a writable `offset` too, a vector, and was taken for a path view whose `moving` reads as
nothing, so the settle never ended on a screen with a shadowed cover; the properties are kept only when Qt types them as
numbers. A recording rests before its picture (`Scene::rest`): every
running animation loops without end, a design's resting motion, which the harness's list of running animations marks
`looping` when the animation or a group it runs in repeats forever; with nothing running, no frame swaps for 100 ms. It
fails past 5 s, naming what still runs.

**A state is read only when its views are at rest, as Qt declares it** (`Scene::settle`). Every flickable and path
view in the tree must be neither `moving` nor `flicking`, and a path view that holds its current item in its highlight
range (`StrictlyEnforceRange`) must stand at the offset that item rests at, `fmod(count - currentIndex, count)`, the
target Qt's own step to an index moves to. The harness polls those declarations every 10 ms and fails the state past
5 s, so an unsettled state is refused rather than read. `resize` settles after `QQuickTest::qWaitForPolish` (Qt's word
that no item is still to be laid out) where it slept 20 ms, and the put-back settles after its writes. The gallery gate
reads each state after a resize and so after a settle; its fixed 60 ms wait is gone.

The defect that showed it, measured on 2026-11-08. The release gate failed about two runs in three with
`big-Dark-menu-3840x2160: rests outside the carousel 341: 350, Card… at 2950,875 486x648`, the same card at the same
rectangle, at 4K only, while the debug gate passed. The mechanism: the walk's Left and Right on the carousel each start
a Reel step (400 ms) towards a new current index, and **Qt declares no movement during such a step**: a probe sampling
the carousel every 10 ms after a press read `moving=false flicking=false` throughout, while the offset went from 8 to 7
over 380 ms (`~/.cache/emusen/tmp/qt-stage0/probe-carousel-moving.log`). The walk left the carousel stepping back to its
index, and the next state was read about 110 ms later with the offset a few thousandths of an item short: logged before
each big-screen Menu read in a release run, 0.0029 at 1920 × 1080, 0.0072 at 1280 × 720 and 0.0032 at 3840 × 2160, the
run that failed. A thousandth of an item is 0.55 pixels at 4K, where an item is 551 pixels, and 0.28 at 1920 × 1080, so
only at 4K did the card at the carousel's edge stand past its box by more than the audit's 1.5-pixel slack; the release
build's steady timing left the same residue each time, hence the same rectangle. With the settle, the same log read an
offset of 0 at every state. Without it (the put-back and the settle taken out), three release runs failed once, with
that line exactly.

Measured on 2026-11-08 with the settle: **10 release runs of the gate, each beside a release build of the kit in a
target folder of its own with the compiler cache off, at a load average of 10.2 to 13.2 on 16 threads: 10 passed**,
41 to 48 s each (`~/.cache/emusen/tmp/qt-stage0/runs/`). The check still catches a real spill: the carousel's old
`pathItemCount` (as many as cover its width, plus two) seeded again failed the release gate with 192 `rests outside the
carousel` faults, the first `big-Dark-menu-1280x800: … Card… at 1229,324 180x240`.

Seen in every run since step 1, and not part of this defect: when the question opened, Qt logged `RangeError: Maximum
call stack size exceeded` twenty times from `Sheet.qml` and `Frame.qml`, and the gate passed over it. §7.14 found and
fixed it, and made every logged warning fail the tests.

**Not written, and why.** `focus` (focusable): the dump has one fact, `operable`, and writing it twice would be an
invention. `sideways`, `live`, `element`, `volatile`, `part`, `title`, `footer`, `walk` and `entered`: nothing in the
kit has them yet, and the walk starts from every operable item, so it is never truncated. The grammar's presses after
the walk are owed to stage 1, where each screen's test says what its presses observe.

**Two toolkits' facts, read side by side.** A Qt `Text` is not hit-testable where Avalonia's `TextBlock` is, so C#'s
overlap rule, which reads drawn items without `nohit`, passes over words that kit-core's overlap rule still judges;
the Qt slack is kit-core's 1.5 above a unit of 1, where the C# recordings at 1920 × 1200 write 2; and `layout` and `box`
are scene pixels, where the C# writes the control's own. None changes a comparison (`InterfaceSpec.Compare` reads names,
roles, words, rectangles and the focus graph), and each is the toolkit's fact rather than one bent to match.

**The C++** of this step: four facts in the dump (enabled, the item's scale in
the scene, the face, a flickable's content height); the role read from Qt's accessibility instead of the attached
`Accessible.role`, which only items that declare one carry (every `Text` read `none` through it); and the put-back's
reads and writes (what a press may move, a focus scope's remembered item, an item found by its id, a number read and
written, the focus given back). Switching accessibility on (`QAccessible::setActive`) was tried first and changed no
recorded byte, so it is not done. A first put-back that read the whole tree after every press took the gate from 44 s to
103 s; reading only the recorded properties keeps it at 50 s.

**Proved with the C# side's own code**, measured on 2026-11-08. The gate writes each of its 73 states as `<state>.spec`
beside its grab in `~/.cache/emusen/tmp/kit-gallery/`. A program outside the tree
(`~/.cache/emusen/tmp/qt-stage0/spec-proof/`) compiled WiseMan's `Fixtures/InterfaceSpec.cs` and
`Fixtures/InterfaceRules.cs` unchanged, with `FitAudit.DesktopCaps` (0.7273, the one value the rules read from the
Avalonia-bound `FitAudit.cs`, and kit-core's `DESKTOP_CAPS`) stated beside them. For every recording it read the file
with `SpecDocument.Parse`, wrote it back with `SpecDocument.Write` and compared the bytes after the banner, ran
`InterfaceRules.All`, and ran `InterfaceSpec.Compare` against `big-Dark-menu-1280x800`. Its output (`all.txt` there):

- 73 of 73 written back byte for byte;
- the C# rules (fit, pad reach, names, palette reach) found **0 faults in all 73**, as kit-core's audit had;
- the comparator, which reads no colours, found **0 differences** between the dark and the light recording of every
  state, all 36 pairs (six in one pair before the put-back, above); against `big-Dark-menu-1280x800` it named 19 in
  the same state at 1920 × 1080, rectangles measured in the expected state's design pixels, for example `navigation: at
  86.4,54,1747.2,135, recorded 64,40,1152,100, 617.6 design px apart`, and 95 in the Controls section, beginning
  `missing: MenuPanel panel`.

Two runs of the gate wrote the 73 recordings identically but for one line in each of the eight desktop Menu states:
the width of the section list's content item, Qt's own estimate, 1278.35 in one run and 1277.83 in the other. It is an
unnamed item the comparator does not read; a byte-for-byte hold of a recording would have to leave it out.

Recordings are change detectors only (`EmuSen_Qt.md` §8.2). The gallery's are written for the comparison and a reader's
look, not committed and not held byte for byte: the gallery has no C# screen to be compared with, and its own pictures
are held by the gate.

**Held to**: `qt-test/tests/spec.rs`, one test: numbers and quoting as C#'s `Num` and `Quote` write them; the menu scene
recorded, its header in the format's order, the ids in order with no level jumped, each line's facts in key order, the
panel by name, the rows operable and named, a row's words owned by the row with their capitals, face and limit, the
focus on the first row, Down reaching the second, Up staying (`same`), every press an edge line. The harness's scenes
(`menu`, `sheet`, `style`, `lint`) and the gate pass unchanged over the walk. Measured on 2026-11-08 with three seeded
faults, each reverted: words owned by their outermost ancestor rather than their parent failed `spec.rs` (no owner on the
first row's words); a press that kept the focus recorded as lost failed it (`edge 8 up same` missing); the facts written
in reverse order failed its key order, and, before that check existed, the C# round trip (`written back DIFFERENTLY`,
the first difference at line 14). The put-back is held by `spec.rs` (a second walk records the same text) and
`style.rs` (after the walk the tab, the slider's value, the combo box's choice and its row's remembered focus are as
before). Three more seeded faults, each reverted: no put-back failed `spec.rs`'s second walk; selections never written
failed `style.rs` (the tab and the choice at 1, not 0; the menu scene's list puts itself back when a row takes the focus,
so `spec.rs` could not see it); the remembered focus not given back failed `style.rs` (the combo box's focus false).

**A delegate made again stands for the one the tree read** (2026-11-09). theme-audit's `many` state, a console of 9,000
games, found every theme's rows unreachable at every size after the first, and a plain `ListView` of 9,000 walkable rows
the same (`qt-test`'s `LongListScene.qml`). The cause was the walk's bookkeeping, not the views: the tree names items by
their `QObject`'s address, and a view deletes a delegate scrolled out of it (past its cache) and makes another when it
scrolls back. At 1280 × 800 the walk's Down from the last row it had built scrolled the list by a row, the first row's
delegate was deleted, and the put-back's scroll made a new one. At the next size the tree's first row, the focus, was that
new delegate, read last; by the time the walk reached it, Down from the last row had deleted and remade it again, so
focusing it by its old address found nothing and the walk skipped it silently: no edge from the focus, every row
unreachable (measured: 15 rows, 56 presses recorded, none from `row.0`). A freed address can also be taken by the next
delegate made, so the old number could name the wrong row. And a focus scope's remembered item was given back by an
address that no longer existed, which is how a theme's screen came to hold the focus on the list's content item.

The walk now keeps, beside the tree, the object standing for each of its items (`walk_by`'s `live`). Each read of the
tree marks the items it reads (a dynamic property, `harnessRead`); after a put-back, when the window holds an item with
no mark (`harness_made_since`), or the focus had landed on no item the walk knew, the tree is read again and each of the
first reading's items is found in it: the same object if its type, objectName and box are still the same, else the
unclaimed item that has them (`Tree::found_in`, to half a pixel, the put-back having scrolled the views back). The
put-back then runs again through those objects, so a scope's focus is given back to the row made again. Landings are
read through the same objects, and a focus on no element of the tree (a row the list made as it scrolled) lands on none,
recorded `none`, where it had been read as the list's content item and held to the pointer rule. `Scene::focus` finds a
focused item made again in the tree's item's place the same way.

Held to by `qt-test/tests/longlist.rs`: at each of the five sizes the first row holds the focus, the walk reaches every
row on the screen, a click reaches each element the walk reaches, and the walk puts the focus back on the first row. Before the change it failed at 1920 × 1200 with nine rows
unreachable. The S6 probe (that list in a theme package, `theme-audit --states many`) went from five sizes of reach
findings in each variant to none, and from 46 s to 19 s. Cost, measured 2026-11-09 at a load of 7 to 8 on 16 threads:
the gallery's gate 67.0 and 68.7 s against 60.9 s with the re-reading taken out, nearly all of it the carousel, whose
every step makes a card. Two seeded faults, each reverted: the re-reading taken out failed `longlist.rs` at 1920 × 1200
(nine rows unreachable); a landing on no element read as the list's content item failed it at 1280 × 800 (the content
item covered, held to the pointer rule).

**An element that slides is still itself** (2026-11-09). The program's acceptance test failed on the reference theme's
Choose a console once the re-reading above was in: the pad reached neither the shelves nor two consoles at 1280 × 800 and
one console at every other size, and at 1920 × 1200 the focused console's engine line was not shown. The orbits' consoles
are a `Repeater`'s, never made again, but each press turns the orbits (a `Behavior` on each console's angle, the reel's
length), and the focused console's covers are made again for each focus, so the walk read the tree again while the
consoles were still sliding back. `found_in` kept an object as itself only while its box was unchanged, so a console
mid-slide stood for nothing, the walk focused nothing for it and recorded no press from it, and a scope's focus was given
back to nothing. An object still alive and still of its type and objectName is now itself wherever it has moved
(`found_in`, `same_as`); the box is asked only of an item made again in another's place. Held to by `qt-test/tests/shift.rs`
over `ShiftScene.qml`: six walkables in a row, the focused one growing over 300 ms so the rest slide, and marks made again
for each focus; every Right lands on the next and the pad reaches all six. The box asked of the same object, seeded, failed
it (no press recorded from tile 1); the acceptance test passed with the change on WiseMan c0a6358ab.

### 7.12 The pad's glyphs (stage 0, the rest, step 2; 2026-11-08)

The help bar's circles were empty until this step. Each entry now draws the glyph of the control that gives its press,
in the pad's family, following the design's word: **drawn shapes and a letter from the interface's type**, not a font
of symbols (LunaP.md §103.1, whose outlined set this is; `EmuSen_Interface.md` §5.2 sets the letters' size, Pad glyph,
22 design units on the big screen and 13 pixels on the desktop, the token `fontGlyph`). Every shape is plain geometry
and none is a vendor's artwork or colour.

**kit-core** owns the decisions, in two places.

- `input::PadControl` names a pad's controls by place, as SDL does (South is the bottom face button), and
  `UiButton::control(swap)` says which control gives each press: Accept on South and Back on East, traded by the
  player's swap through `swapped`, the router's one rule; Menu on Start, Options on Select, the pages on the shoulders,
  First and Last on the triggers, Search on North, Screensaver on West, Random on a stick's click, the table of the C#
  front end (`MainWindow.Controllers.cs`). The pad bridge (step 4) reads the same table the other way.
- `glyph` draws a control of a family as marks in a unit box (a polyline, filled or stroked at a share of the box) and
  the letter the family prints, with the name a reader hears. `PadFamily` is `Generic`, `Xbox`, `PlayStation` or
  `Nintendo`; `Generic`, the pad whose printing is not known, is the RetroPad's face by position: four dots in a
  diamond with the meant one filled, the shoulder's own half filled, a pill with a play mark for Start and a square for
  Select. The lettered families differ at a glance: a ring round the letter (Xbox), the letter cut out of a disc
  (Nintendo).

**kit-qt** draws them: `PadGlyphMarks`, a list model of the marks in pixels at the glyph's size, and `PadGlyph.qml`,
one `Shape` a mark, the letter a `Text` centred on its ink (`TextMetrics`' tight box and `FontMetrics`' ascent), or,
where a mark is `cut`, a `PathText` in that mark's own path with the odd-even fill, so the letter is a hole showing
what lies behind, as Qt Quick Shapes draws it, with no paint of ours. `Families` mirrors `PadFamily` (§7.1).
`HelpBar` takes `family` (the frame passes `padFamily`), the glyph takes the router's `swapAcceptBack`, and each
glyph sits inside its entry's button, so a click on it is the entry's press (§1.7).

**The help bar on a pad of each family**, at 1280 × 800, big screen, dark: the generic face (dots, the pill with a play
mark), Xbox (A and B in rings, the three-line ring), PlayStation (a cross and a circle in rings, the three-line pill) and
Nintendo (B and A cut out of discs, the plus ring), Choose on South and Back on East in each, as the table gives them.
The gate writes the four as `helpbar-<family>.png` beside its grabs, and `emusen-qt-test`'s `glyphs` test writes every
press in every family, sixteen by four, to `~/.cache/emusen/tmp/qt-test/glyphs.png`; both were looked at on
2026-11-08.

**No glyph is empty.** `Scene::ink_inside` samples one grab on a grid within 0.35 of a glyph's size from its centre and
counts the points that differ by more than 24 levels from the colour most of them show: an empty ring, filled or not,
shows one colour there, and every glyph of the set covers at least two points *[own, because the check is the
harness's own pixel reading, as §7.10's sweep is]*. The gate holds every help-bar glyph shown in every state to two
points or more, three glyphs where the bar is shown and none where it is not, and the four families' bars the same way.

**Held to**: kit-core's `tests/glyph.rs` (3: every control of every family draws a mark inside its box, closed where
filled, one cut mark at most and never without a letter, and a name; the face letters and names by position, the
generic dot at its own place; every press its own control, the swap trading Accept and Back only); kit-qt's `kinds`
(the families pair by pair); `emusen-qt-test`'s `glyphs` (64 glyphs, each named, Accept's name on each family's South,
none empty, nothing logged) and `menu` (a click on the Open entry's glyph is that entry's press); the gate. Rule Q's
registry gained `Families` and `PadGlyphMarks` and the qmldir `PadGlyph`; reviewed and copied, additions only.
Measured on 2026-11-08 with three seeded faults, each reverted: the model giving no marks failed the gate
(`help-bar glyph glyph-Choose draws nothing inside its ring (0 points)`); the cut letter never given failed `glyphs`
(Nintendo's B, A, X and Y, 0 points each: a disc with nothing cut shows one colour); the swap not applied failed
kit-core's `each_press_has_its_control_and_the_swap_…` (Accept on South with the swap on).

**The swap trades North and West too** (decided 2026-11-08 by the tester). The C# table traded North and West with the
swap as well as South and East, as ES-DE's does, while kit-core's `UiButton::swapped` traded Accept and Back only, so
with the swap on Search stayed on North. `swapped` now trades Search and Screensaver as well, so the player's existing
setting means the same on both front ends; the router and the glyphs read the one rule. Held by kit-core's `input`
test (Search and Screensaver trade, Menu stays) and `glyph` test (Search on West with the swap on). Seeded on
2026-11-08: the new pair left out of `swapped` failed both.

### 7.13 The gallery as an AppImage (stage 0, the rest, step 3; 2026-11-08)

The gallery program is bundled by the route `EmuSen_Qt.md` §9.2 names: `linuxdeploy` with its Qt plugin lays out the
AppDir, `appimagetool` packs it. **`cargo xtask appimage --out DIR`** is the build host's command for it (`xtask/src/
appimage.rs`, beside the host's other commands of `EmuSen_RustPlatform.md` §28.11, as `EmuSen_Qt.md` §1.9 places the
front end's build): it checks the four pins of `xtask/appimage-tools.tsv` (§9.2's table) against their sha256 and
refuses to run on any difference, builds `kit-gallery` in release, runs linuxdeploy (`QMAKE` the system's `qmake6`,
`QML_SOURCES_PATHS` the gallery's, the kit's and the style's QML so the Qt plugin's `qmlimportscanner` finds their
imports, `EXTRA_PLATFORM_PLUGINS` adding `libqwayland.so` and `libqoffscreen.so` to `xcb`, `EXTRA_QT_MODULES=
waylandcompositor`, which the plugin's README names for Wayland and which brings the client's shell, decoration and
graphics plugins, `NO_STRIP=1` and `PATCHELF` as §9.2 explains), then appimagetool. *[own, because the steps are the
tools' own and the command only orders them and checks the pins, as the host's other commands do]*

**The program.** `kit-gallery` sets `QSG_RHI_BACKEND=vulkan` unless the environment names another backend (§9.2, §1.8),
and its window is shown by its scene (`visible: true`). `--grab FILE` saves the window's first swapped frame of the
window's own size and ends (below, *The handheld's panel*):
`QQuickWindow::grabWindow`, its `frameSwapped` signal and `QCoreApplication::quit` are bound in Rust through cxx-qt's
`extern "C++Qt"`, and `QImage::save` through a type of the crate's own over the same C++ `QImage` (`grab::Picture`),
since cxx binds no method on another crate's type; no hand-written C++. On Qt's offscreen platform the scene graph has
no Vulkan surface and draws with its software backend, so an offscreen grab is a software render, as the harness's
are; the bundle's Vulkan path is the handheld's windowed run.

**The bundle**, measured on 2026-11-08: 88,127,992 bytes; the AppDir 248 MB with 174 libraries, the `xcb`, `wayland` and
`offscreen` platforms and Wayland's client plugins. The Qt plugin's scan bundles more QML than the program imports
(Particles, Pdf, Scene3D, VirtualKeyboard, every Controls style), as its import scanner follows `QtQuick.Controls`;
trimming it is owed to the release build (§9.2's container). It reports `Missing qml module: EmuSen.Kit`, which is
expected: the kit's and the gallery's modules are compiled into the program's resources, not files the scanner can find.

**Run from a clean copy.** The AppImage was copied alone into an empty folder and run with `env -i` (only `HOME`,
`PATH=/usr/bin:/bin` and `TMPDIR` set), `QT_QPA_PLATFORM=offscreen` and `LD_DEBUG=libs`: it printed `grab …: saved` and
exited 0. Of the libraries initialised, every Qt one (15) and every QML plugin's came from the bundle's mount, and none
from `/usr/lib64`; ICU 77 came from the bundle; what came from the system was glibc, the GL dispatch libraries, X11,
fontconfig, freetype, harfbuzz and the compression libraries, AppImage's exclusion list. Its frame against the same
program run from the build tree on the system's Qt: **0 pixels differ** (ImageMagick `compare -metric AE`). Against the
gate's own grab of the same state (`big-Dark-menu-1280x800.png`): 205 pixels differ exactly and none at a 10% fuzz, the
stars, which twinkle on the clock. The bundle is not copied to the handheld here; that run is another's.

**Held to**: the host's `appimage` test (the four pins read, a tool that is not its release refused with its URL, a
missing one told where it comes from), and the run above. Measured on 2026-11-08 with three seeded faults, each
reverted: the sha256 comparison skipped failed that test; a pin changed by one digit made `cargo xtask appimage` refuse
before building (`appimagetool-x86_64.AppImage is not the pinned 1.9.1`); `PATCHELF` unset made it refuse (`PATCHELF
names no patchelf`). The bundle built with linuxdeploy's own patchelf, before the fix, crashed at start, as §9.2 records.

**The handheld's panel, and the grab's timing** (2026-11-08). The handheld is a Lenovo Legion Go S; its panel is a BOE
NS080WUM-LX1, 8 inches, 1920 × 1200 (`gamescopectl info` and `xdpyinfo` agree). 1280 × 800, the gallery's own size and
the gate's first, is a Steam Deck–class size, kept as a fit size, as every window fits at 1280 × 800 and 1920 × 1200;
the gate runs at 1920 × 1200 too since this date. On the handheld under gamescope (Vulkan, `DISPLAY=:0`, on the
charger) the program's X window was 1920 × 1200 (`xwininfo`), and `--grab` saved a 1920 × 1200 frame whose content was
laid out at 1280 × 800 in the top-left corner, the void (black) over the rest.

What was ruled out on the desktop, 2026-11-08:
- *The layout.* `tests/fill.rs` shows the gallery at its own 1280 × 800, then gives it 1920 × 1200 through the platform
  (on the offscreen platform `QWindow::resize` reaches Qt through `QWindowSystemInterface::handleGeometryChange`, the
  call a platform makes for a window manager's resize) and samples six points near the edges and corners: none shows
  the void. Seeded: the canvas's scale read once (`unit: 1`) failed it, at the bottom-right corner. The test also found
  the same gap in the harness: about one run in four saw the void, since the offscreen platform delivers the new
  geometry as a queued event and `Scene::resize` waited for polish before the size had arrived. `harness_resize` now
  waits (`QTest::qWaitFor`, two seconds at most) until the content item has the size, then for polish; eight runs in
  eight then filled. The gate, at its five sizes, visits 91 states and passes (53 s).
- *The ratio, the Vulkan path and a resize before the first frame.* The program on this desktop's Xwayland under KWin,
  with Vulkan: at the session's ratio of 1.25 (a 1600 × 1000 frame) and at 1.875 (`QT_SCALE_FACTOR=1.5`, a window
  KWin clamped to 2400 × 1348 before its first frame), each grab filled to within four pixels of its edges; the
  software backend offscreen at a ratio of 1.5 filled 1920 × 1200 likewise.

**The mechanism, as its picture states it.** A frame of 1920 × 1200 pixels drawn at 1280 × 800 is a frame whose size
is not the window's size at its ratio as Qt knew it. Qt's Vulkan swapchain takes its size from the surface, which is
the X window gamescope had made 1920 × 1200, while Qt Quick drew at the size Qt last heard, 1280 × 800 at a ratio of 1:
gamescope's resize had not yet reached Qt, and `--grab` took the first frame swapped, inside that gap. **So `--grab`
now takes the first frame of the window's own size** (`grab::settled`: the frame's pixels are the window's size at its
ratio, within a pixel), asks for another frame (`QQuickWindow::update`) while a frame of another size comes, saves what
it has after 300 frames, and prints both sizes: `grab FILE: saved, frame 1600x1000, window 1280x800 at ratio 1.25,
after 0 frames of another size` on this desktop's Xwayland, `frame 1280x800, window 1280x800 at ratio 1` offscreen.
Held by the `grab` unit test (the window's own size at ratios 1, 1.5 and 1.25 as Qt rounds them; the swapchain's size
before Qt is told, and a frame from before a resize, refused). Seeded: the ratio left out of `settled` failed it.

Predictions for the handheld's next run, to be measured there: (1) the line names one frame or more of another size,
then a 1920 × 1200 frame of a 1920 × 1200 window, and the picture fills it: the defect was the grab's timing, and a
window on the panel was never wrong for longer than gamescope's resize took to arrive; (2) the line names 300 frames
and a 1280 × 800 window: Qt never hears gamescope's size, a fault between gamescope and Qt's `xcb` platform, and the
fix becomes the program's (the big-screen form asks for the screen's size) — not built until measured.

### 7.14 A test fails on any warning Qt logs; the sheet's key loop (2026-11-08)

**Every warning fails.** The harness installs a Qt message handler when it starts (`qInstallMessageHandler`, which
cxx-qt-lib does not bind and which takes a plain function with no context, so its list is the C++ file's own, read and
cleared by `harness_messages`) that keeps every warning, critical and fatal message, QML's errors among them, and still
prints them as before. `Harness::assert_quiet` fails on any message that is not one of `QT_INTERNAL`'s, each named with
the reason it is Qt's own; the list is empty, since every message seen so far was ours. Every harness test (`menu`,
`sheet`, `style`, `spec`) and the gate end with it.

It found two at once, both ours:

1. **The sheet's key loop.** With a question open, the frame's `Connections` offer each pad press to the sheet, which
   announces it and sends the key it means to the focused control (§7.3). When the focused control was outside the
   sheet and declined the key (the gate's walk focuses every operable item of the window, the rows behind the modal
   question among them), the key propagated up to the `Frame`, whose `Keys.onPressed` gave it to
   `FrameController::key`, which, the sheet being open, offered it to the sheet again, which sent the key again, to the
   same control: a recursion that ended only when the JavaScript stack ran out, twenty times in a gate run. §7.3 already
   stated the rule the code broke, that a key goes to the sheet or the walk "and never back out as a key, so a key cannot
   loop"; the sheet's branch of `key` sent it back out. Now `sheet_take` dismisses or announces and returns the key, and
   only the pad's path (`offer`) sends it; the keyboard's path never does. `sheet.rs` holds it: a question open, the
   frame given the focus, Right pressed, and nothing logged.
2. **An empty shadow.** Moonrise's `shadowOutline` began as an invalid `QVariant`, so its `PathPolyline` logged `path
   of type 0 not supported` once at every start. It begins as an empty polygon.

Measured on 2026-11-08 with two seeded faults, each reverted: the keyboard's path sending the key again failed the
gate (`Qt logged 20 warnings or errors`, the `RangeError` from `Sheet.qml` and `Frame.qml`) and `sheet.rs` (2 of them);
the shadow beginning invalid failed the gate (`PathPolyline: path of type 0 not supported`). With both fixed, the
harness tests, the debug gate and the release gate pass with nothing logged.

**A message at the process's exit, known and harmless** (proved 2026-11-08). The acceptance test's binary, and any harness
binary that shows a picture decoded at a `sourceSize`, prints `QObject::~QObject: Timers cannot be stopped from another
thread` after its test result. The object is Qt Quick's process-wide pixmap cache, a global static in libQt6Quick: when a
small picture is released (a cover's delegate, or every picture as the engine is dropped) the cache keeps it and starts a
30 s expiry timer on the thread that released it, which is libtest's test thread; the cache is destroyed at process exit
on the main thread, with the timer still running. Shown with a probe on the software backend: an `Image` with a
`sourceSize` printed it and one without did not (its full-size picture is over the cache's limit and freed at once); an
`Image` with `cache: false` did not; a logged exit trace put the destructor in libQt6Quick's exit handlers right after
the image plugins'; and the cache purged (`QQuickPixmap::purgeCache`) with 31.5 s waited before the test returned
removed it, the timer having expired. Unloading the image plugins changed nothing. It cannot be stopped from the harness
(the cache is not exported), and running each Qt test on the process's main thread (`harness = false` in every test
file) was weighed and declined on 2026-11-08 as churn for a message. It cannot fail a test: `assert_quiet` reads the
messages kept while the test runs, and this one is printed after the test function has returned.

### 7.15 Endymion's pads and the bells: `emusen-qt-endymion` (stage 0, the rest, step 4; 2026-11-08)

`EmuSen_Qt.md` §1.7 as built. Qt 6 has no gamepad module, so the pad is Endymion's, read through the platform client
(`EmuSen_Qt.md` §1.1.2) on a thread of its own and put on the GUI thread through `qt_thread().queue`, into the
`PadRouter` of §7.4; the bells of §7.5 play through Endymion's interface player.

**Where the pad's family lives** (decided 2026-11-08). Which printing a pad carries is a fact about the pad, read from
SDL's `SDL_GamepadType` and, where SDL says Standard or Unknown, the pad's name (the Legion Go S and a Steam Deck print an
Xbox layout). Two homes were weighed: kit-core reading what Endymion reports (the SDL type's number and the name), or
Endymion reporting the family. kit-core stands without EmuSen and names no SDL number, and the C# front end, which reads
pads through the same platform library, could not have read kit-core's copy, so the C# table would have stayed a second
owner. So Endymion owns it: `emusen_endymion::family` holds the table moved from Mistress's `PadFamilies.cs`, which is
deleted, and `emusen_endymion_pads_get(…, EMUSEN_ENDYMION_PADS_FAMILY)` answers it, an extension of the existing call
(platform interface 41, `EmuSen_RustPlatform.md` §48). Mistress's help bar and the Qt reader both read that one answer;
kit-core's `PadFamily` stays the kit's drawing of a family, and the reader pairs the header's four numbers with it by
name (`reading::family_of`).

**The crate.** `Frontend/qt-endymion`, `emusen-qt-endymion`, the QML module `EmuSen.Endymion` 1.0, a member of the Qt
workspace. Its dependencies are the workspace's cxx-qt crates, kit-core and kit-qt (the presses, controls, thresholds,
cues and the enumerations Qt has them as) and the platform client by path; kit-qt still names no client, so the kit
stands alone and only this crate joins it to Endymion. Four parts, one job each:

- `reading.rs`, **one poll as the interface reads it**, plain Rust over the client's `Pads`: the devices polled, the
  changes taken, the devices brought up to date (`update`, SDL's events pumped, every tick as the C# poll ends), then
  every pad the interface reads (`FRONTEND_COUNT`) read control by control. A control's press is kit-core's
  `PadControl::press`, `UiButton::control` read the other way (either stick's click is Random, as the C# read them),
  added to kit-core for this step; the player's swap is not applied here, since the router applies it once (§7.3). The
  left stick is the d-pad through kit-core's `stick_direction` (0.55), and a trigger is held past
  `TRIGGER_THRESHOLD` (0.5, the C#'s), now kit-core's beside the stick's. The pads read are merged (a button is held
  while any pad holds it, so a second pad's press and release of a held direction changes nothing), and a tick gives
  the buttons that went down or up since the last, in the kit's order. The pad that went from holding nothing to
  holding something is the last used; the help bar's family is its family while it is read, else the first pad's, else
  the generic face. A pad pulled out lets go of what it held. *[own, because Qt has no gamepad reading and Endymion
  answers raw controls, which the interface's grammar must read; `sdl_button` names SDL's button numbers, which
  Endymion's calls take, once.]*
- `reader.rs`, **the thread**: `Reader::spawn(open, only, deliver)` opens the pads on the thread, then reads every 16 ms
  (Mistress's pad timer, `EmuSen_Input.md` §8.12) and hands each non-empty tick to the sink whole; a failure is said
  once until a poll succeeds; dropping the reader stops and joins the thread, and a sink that refuses stops it too.
  **Coalescing per axis** is the poll itself: an axis is read once a tick and only its direction's crossing leaves the
  thread, so however many events SDL saw between two ticks, the GUI thread receives at most one change per direction
  and one closure per tick, and none when nothing changed.
- `pad_reader.rs`, **`PadReader`**: writable `platformLibrary` (the path, as `LibraryModel` takes it: a QML element
  cannot be handed a Rust value, §1.2.1 of `EmuSen_Qt.md`) and `onlyPads`; read-only `family` (kit-qt's `Families`),
  `connected` and `problem`; one signal, `button(PadButton, down)`. Naming the library lends SDL (`libSDL3.so.0`, once
  a process) and starts the reader, whose ticks come back only through `qt_thread().queue`, where each press is emitted
  in order and the properties published. `onlyPads` is SDL's pad ids, separated by spaces, read alone, as
  `emusen_endymion_pads_show_only` is a test's: it keeps a test's virtual pads from opening a pad on the desk.
- `bell_player.rs`, **`BellPlayer`**: writable `platformLibrary` and `volume` (kit-core's `DEFAULT_VOLUME` until set),
  read-only `ready` and `problem`, and `play(cue)`. The player is made off the GUI thread once the library is named,
  every cue rendered by kit-core once and remembered by Endymion's interface player under its name (48 kHz stereo
  32-bit floats, little-endian, as the player keeps them), and comes back through the queue; `play` replaces the cue
  still playing, as the player does. kit-qt's `Bells` stays the render cache a test inspects.

**Into the router.** A host declares the reader in its scene and joins its signal to the router in one line, so the
real pad's presses enter the same queue tests push into (§7.4), and binds the help bar's family to the reader's:

    PadReader { id: pads; onButton: (button, down) => Kit.PadRouter.push(button, down) }
    property int padFamily: pads.family

The gallery does this (`Gallery.qml`, with `platformLibrary`, `onlyPads` and `padsConnected` on its root for the
host). The gallery program names no platform library yet, so a human look reads no pad until a host does (§7.7).

**Held to.** No C++ was added. Measured on 2026-11-08 on the desktop, against the platform library built from the same
commit:
- Endymion's `family` (3 unit tests): every SDL 3.4 type, Unknown to GameCube, as the C# table classified them, SDL's
  type before a name that says otherwise, a newer type read by its name; the names of the C# test (Legion Go S, Steam
  Deck, Steam's virtual pad, DualSense, Switch Pro, an 8BitDo pad, none); the header's numbers.
- The platform client's simulated-pads test: a pad of no known type or name is generic, SDL's PS5 type PlayStation, a
  Standard pad named "Nintendo Switch Pro Controller" Nintendo, and the family kept once the pad closes.
- kit-core's `glyph` test: every press's control, swapped or not, read back by `press` and the router's swap gives the
  press again; either stick's click is Random.
- `reading` (3 tests, 3.1 s, Endymion's simulated pads): each of SDL's fifteen buttons and the six axes gives its
  press and lets go, the stick and the triggers short of their thresholds give none, two presses come in the kit's
  order; two pads merged, the family following the pad last used, the last used pulled out while holding Accept
  (Accept up, the family back to the first pad, one pad read), one plugged in while reading and used, and none left
  (the generic face); the thread hands on the first tick, a press and its release, sends nothing after it drops, and
  says a failed opening once.
- `bells` (3.1 s, SDL's dummy audio driver, nothing reaches the speakers): each of the nine cues played under its name
  queues its samples on the player's stream. `qml`: `BellPlayer` made through the queue in a harness scene, every cue
  played with no failure said, nothing logged, and the scene keeps the QML rules.
- The gallery, `kit-gallery`'s `pads` (11.9 s): an SDL virtual pad (SDL's own virtual joystick, attached in-process
  with an Xbox 360's vendor and product, as WiseMan's `SdlVirtualPadsTests` attaches them; `onlyPads` names it alone)
  read by the gallery's `PadReader` puts the Xbox face on the help bar, then walks the gallery, every operable element
  in four directions with what each press moved put back, each press held until the router holds it and let go until
  it lets go; the landings equal the same walk made through the router's queue, as the gate makes it. A second pad
  (a DualShock 4's vendor and product) plugged in while the gallery runs is read and leaves the family on the pad last
  used until a click of its stick, when the PlayStation face shows; pulled out, the family returns to the Xbox pad, and
  with both gone, to the generic face. Nothing logged. The harness's walk takes its press as a parameter for this
  (`walk_by`), and `pad_held` and `run_pad_clock` read and restart the router's clock. `pad_down` and `pad_up` (added 2026-11-08) press a
  button and leave it held until let go, each edge pumped on the harness's clock, for a view whose own clock repeats a
  held direction (the ES-DE painter, EmuSen_Qt.md §4.8.5). *[own, because SDL's virtual
  joysticks are SDL's test interface and no SDL binding crate is in the tree: the test declares
  `SDL_VirtualJoystickDesc` and five calls from SDL_joystick.h itself.]*

Four seeded faults, each reverted: the family taken from the first pad read, ignoring the pad last used, failed
`reading`'s merge test (no change of family to the Nintendo pad); the stick's vertical read inverted failed the
controls test (Down for the stick pushed up); the platform library answering the SDL type for the family failed the
platform client's test (6 for PlayStation's 2); the player remembering eight of the nine cues failed `bells` (Refused
queued nothing). The last was first seeded against the harness test alone and passed it, since the interface player
takes a name it was not given as a path and answers success: the play's proof is the queued bytes, read through the
client's `InterfacePlayer::queued`, added for it.

### 7.16 `GamePicture` and the adopted device (stage 1, the game picture; 2026-11-08)

`EmuSen_Qt.md` §1.8 as built in the kit; the protocol, its proof and the session behind it are there (§1.8.1 to §1.8.4).
The kit's part knows no runtime, no client and no Serenity: it shows a Vulkan image someone else drew, on the scene
graph's own device, and says when each frame was swapped.

- **`GamePicture`** (`cpp/game_picture.h`, `.cpp`), an item of `EmuSen.Kit` 1.0, registered from its `QML_ELEMENT` by the
  module's own type registration. Two properties: `feed`, the address of a host's `EmuSenPictureFeed` (a number, since
  QML carries no native pointer and a QML-made element cannot be handed a Rust value), and `live`, which keeps the item
  redrawn at every swap (so a host sets it only while its game runs unpaused: set always, every swap asks for the
  next and the window never rests, `EmuSen_Qt.md` §9.3.2). At the item's sync, on the render thread with the GUI thread held, it asks the feed to draw for
  its size in pixels and wraps the image the feed names with `QNativeInterface::QSGVulkanTexture::fromNative` in a
  `QSGSimpleTextureNode`, made again only when the image or its size changes: no copy and no upload. Each
  `frameSwapped`, on the render thread, it hands the steady clock's nanoseconds to the feed and, while `live`, asks
  for the next frame. What the render thread reads of the two properties is what the last sync read.
  *[own, because cxx-qt has no scene-graph bindings (`EmuSen_Qt.md` §11, risk 4): an item's `updatePaintNode` and a
  native texture are C++ only.]*
- **The feed** is a plain C struct, `EmuSenPictureFeed { context, draw, swapped }`, and `EmuSenPicture { image, width,
  height, format, layout }`, mirrored in Rust by `emusen_kit_qt::picture::{Feed, Picture}` with a layout test.
  `draw` answers 1 with a picture, anything else for none, and is the host's only chance to submit to the queue it
  shares with the scene graph.
- **`kit_adopt_vulkan`** (the same files, called from Rust as `picture::adopt`): a window's scene graph put on a host's
  device before its first show: a `QVulkanInstance` adopting the host's `VkInstance` (`setVkInstance`, with the
  extensions it was made with and API 1.2), owned by a child object of the window so that it goes after the window's
  scene graph, and `QQuickGraphicsDevice::fromDeviceObjects` with the physical device, the device, the queue's family
  and index 0. *[own, because cxx-qt-lib binds neither `QVulkanInstance` nor `QQuickGraphicsDevice`.]*

**Held to**: the layout test of the two structs (`picture::tests`), the registry (`GamePicture` and its two properties
added to `Testing/qmltypes/EmuSen.Kit-1.0.qmltypes`), and `EmuSen_Qt.md` §1.8.1's runs in a real window. 197 lines of
C++ were added (`game_picture.h` 64, `game_picture.cpp` 133, blank lines and includes counted), the reasons above.

### 7.17 The pad drives the game: one reader, two sides (stage 1, the play; 2026-11-08)

`EmuSen_Qt.md` §1.8.5 as built in `emusen-qt-endymion` and the kit. Pads have one owner (Endymion) and one reader (§7.15's
thread): each tick reads the pads for the interface, as before, and for a running game, through each player's bindings.

- **`game.rs`**, plain Rust, two parts. `GamePads` is the hand-over between the reader's thread and a game's frame thread:
  what the game wants (its console and game, for the bindings; its ports; the axes it reads, by PadAxis), whether it owns
  the pad, and the newest read (a mask by PadButton and the read axes for each player, and the players seated), taken by
  the frame thread with `clone_from` into a read it keeps, so no tick or frame allocates. `GameReading` is the reader's
  half: when the wants change it asks Endymion for every player's bound buttons (`bindings_bound`, platform interface 43,
  `EmuSen_RustPlatform.md` §50) and sets the left stick to be a stick where the game reads LeftX; each tick it seats the
  open pads in the order they were opened (player 1 alone where the first controller only is read) and lights their
  numbers, then reads each seated player's buttons with Endymion's `pads_pressed` (the stick for the d-pad, a trigger
  past half, else the bound button) and the wanted axes. *[own, because Endymion has no seat table outside the C#
  `PlayerSlots`; the seats the player arranges and keeps (`emusen_endymion_slots_*`) come with the Pause Card's players
  page.]*
- **Who hears the pad.** Nothing is handed over while the game does not own the pad (paused, or a menu over it). Given
  back, the game hears nothing until every button is let go, so the press that closed a menu is not the game's (Mistress's
  `PadBelongsToTheInterface`).
- **`PadReader.gamePads`**, a constant number naming the reader's `GamePads`, which a `SessionModel` is named by
  (`pads`), as `GamePicture` is named its feed: a QML element is handed no Rust value. The model takes a reference of its
  own (`GamePads::at`), so the hand-over outlives a reader dropped first.
- **`PadRouter.chordOnly`** (kit-qt): while on, a pump gives the menu chord alone, and a button held then is the grammar's
  only once let go and pressed again, so neither the walk nor a repeat sees game input and nothing held over the game
  presses when the interface has the pad back. The kit knows no game: a host binds it (`Play.qml`: the game running and
  not paused), and answers the chord (`SessionModel.togglePause` until the Pause Card exists).

**Held to** (measured on 2026-11-08, the platform library from the same commit): kit-qt's `router` test
`kit_7_17_…` (Down held over the game gives no press or repeat; the chord reaches the program; Down still held when the
interface has the pad presses nothing until let go and pressed again); `emusen-qt-models`' `input` test (Endymion's
simulated pad, the reader's game half and `GameInput` on the test core, whose RAM byte 3 counts up each frame port 0's
first button is held: nothing heard while the game does not own the pad; a press held as it takes the pad not heard; A,
bound by the console's map, heard on port 0 for 10 frames; another button not heard as A); Endymion's unit test of the
binding rule (§50 of the platform page); the platform client's binding test (player 2's own, player 3 with player 1's,
a player out of range refused). Four seeded faults, each reverted: the quiet rule dropped (caught by `input`: the held
press was heard), a player's own map ignored (Endymion's test: player 2 read player 1's A), the router letting a press
held over the game through (`router`: the walk saw it), a button told to port 1 (`input`: A never counted). No C++ was
added.

### 7.18 Worlds as lines: Moonrise's and the tiles' bodies from the host's data (2026-11-08)

Decided 2026-11-08 by the tester, from `EmuSen_Interface.md` §24.6: each console is drawn by its maker in Choose a console
and in Moonrise, so the program's splash draws the bodies, the approved draft remaining the oracle for everything else
in it (sky, stars, moon, halo, title, layout). The kit's splash took a host's bodies only through the `addBody`
invokable, which a view cannot call in a loop under `EmuSen_Qt.md` §1.3, and a `Body` had no palette, though kit-core's
`WorldBody` has carried one since LunaP §222.1. A minor addition of the kit's surface closes both: a world as one line
of plain text, the form of a declarative input.

- **kit-core** `world::line`: `world_line(hue, body)` writes and `parse_world` reads `HUE SHAPE CRATERS BANDS CAP RING
  MOONS [#LIGHT #MIDDLE #RIM]` (shapes `planet`, `shards`, `gem`; rings `none`, `subtle`, `big`; moons at most 2; the
  palette three colours or none, a world without one drawn from its hue); `parse_worlds` reads one a line and leaves out a
  line it cannot read. The grammar has one owner, here; a host writes lines with `world_line`, never by hand.
- **kit-qt** `MoonriseModel.worlds` (and `Moonrise.worlds`): the lines, each a planet of the row, a plain one (no palette,
  no feature, a planet) drawn as the draft's lit disc as before, any other as a `Body`; each planet's row gained the role
  `inks`, the palette as three words. `BodyShapes` and `Body` gained `inks` (the palette as three `#RRGGBB` words; empty
  is the hue's own) and `world` (a whole line, which, when it reads, stands for the hue, the shape and the features). The
  name is `inks`, not `palette`, because every `QQuickItem` has a `palette` and QML warns that the member overrides it.
- **The registry** (rule Q, §7.6) gained `BodyShapes.inks`, `BodyShapes.world`, `MoonriseModel.worlds` and the role
  enumeration's `Palette`, additions only; reviewed and copied.

The host's side is `EmuSen_Qt.md` §7.1: `LibraryModel` writes each console's line from the systems table's look (the one
owner of which console is which body), as the role `world` of a console's row and as `splashWorlds`, the systems with an
engine in the table's order. **Held to** (measured 2026-11-08): kit-core's `world` test (the NES's line with its palette
written and read back; a capped, ringed planet with two moons and a shard cluster read from a text with an unreadable
line left out; a palette of two colours and an unknown shape refused); kit-qt's `world` test (three words a palette,
two or none not; the palette, not the hue, paints the disc); the registry test. Seeded on 2026-11-08 and reverted: the
palette dropped in `parse_world` failed kit-core's test.

**The bodies against Mistress's splash** (measured 2026-11-08). The draft's picture has plain planets only, so a second
C# reference holds the bodies: `Testing/moonrise/LunaSplash-bodies-1280x720.png`, Mistress's own splash at 1280 × 720
on its finished frame, with the courts canvas's eleven consoles (NES, Super Nintendo, Nintendo 64, Game Boy, Game Boy
Color, Master System, Genesis, Sega CD, PlayStation, PlayStation 2, Dreamcast) in the systems table's order, each the
body `StartupSplash.Bodies` gives it, and the caption "11 CONSOLES". WiseMan's `MoonrisePictureTool` writes it (with
`EMUSEN_BIGPICTURE_PNG=1`, to `~/.cache/emusen/tmp/moonrise/raw/`, then copied beside the draft's): a themed session
whose start-up work never ends holds the splash, as `StartupSplashTests` does, and the tool sets the eleven on it. The
list is stated rather than read from the build's engines because the engines a runtime offers follow the cores found
beside its platform library, which differ between WiseMan's output (eight cores) and the Qt tests' platform library
(none built beside it); a pixel comparison needs one list on both sides. Which systems have an engine is held by the
library view's test instead (§7.1 of `EmuSen_Qt.md`).

`emusen-qt`'s `tests/splash.rs` holds the program to it: the program's window at 1280 × 720, its Moonrise given the
eleven consoles' lines (`library::world_line_of` over the runtime's systems) and the caption from `words.rs`, skipped to
its finished frame by a pad press and held up, is compared pixel by pixel. The comparison is `emusen-qt-test`'s
`moonrise::against`, moved there from the gallery's gate so the gate and the program share one tolerance (§7.10's: 2
pixels and 24 levels, 128 inside the words' and the moon's boxes). Against the draft 4 pixels may differ; against the
bodies 16, since Skia and Qt's curve renderer antialias the shards' and gems' straight edges differently (10 pixels, each
within 30 levels of the tolerance, on the Dreamcast's and the Genesis's shards); a wrong body differs at hundreds. The
caption's box is loose enough to let a digit through, so the test also holds its words. Seeded on 2026-11-08, each
reverted: the palette dropped by `world_of` failed it at 407 pixels; the moonlets dropped at 30; the caption counting one
too many passed the pictures and failed the words, which is why the words are held.

### 7.19 Rows of bodies and a screen's declared actions (stage 1; 2026-11-08)

**Decided 2026-11-08: two additive extensions of the kit, for stage 1's screens.** Both were weighed as `EmuSen_Qt.md`
§1.3 asks (an extension, a module, or a component of its own): each is an extension, a new type and a new property on
existing surfaces, since the job is the kit's and neither gives a component a second job.

**The planet rows' layout and walk** are the kit's: the C# drawing of Choose a console was LunaP's (`LunaP.md` §222, as
`EmuSen_Interface.md` §24.6 records). `kit-core`'s `layout::planet_layout(width, leading, bodies, geometry)` lays out a
leading row of wide tiles across the margins and then the bodies in centred rows of fixed tiles: as few rows as fit
(`capacity`, (width − 2 × margin + gap) / (tile + gap)), as even as they divide; every tile gets its box, its row and its
four neighbours, Left and Right within its row and stopping at the ends, Up and Down the tile of the next row whose centre
is nearest, the first of two at the same distance. `PlanetGeometry::COURTS` is the courts canvas's measures (margin 64,
gap 18, leading tiles 72 high at 100, tiles 176 × 190 from 186 every 200, the orbit 52 into a tile, the focused tile
lifted 6). It names no console: the host hands it counts. `kit-qt`'s `PlanetLayout` puts it on Qt as lists a screen's
tiles bind to by index (`xs`, `ys`, `widths`, `heights`, `left`, `right`, `up`, `down`, `orbits`, `lift`), each a
QList inside a `QVariant`, since `qmllint` cannot name cxx-qt's list aliases (`QList_f64`) and failed every binding on
them. The screen's tiles bind `KeyNavigation` to the neighbours, so the walk between rows is data and Qt's own. A
body's size on a tile is kit-core's `size_of` (LunaP §222.2), now `Body.tileWidth` and `Body.tileHeight`, so a screen
writes no shape's size.

**A screen's actions as data.** A frame's X and Y (and any other button the walk does not answer itself) are declared
as text, one action a line, `key Button` with the button as `UiButton` names it (`kit-core`'s `WalkAction::from_lines`;
`UiButton::named`). `FrameController.actionLines` takes the text and replaces the walk's actions with it; `Frame.qml`
binds it to its help model's `actionLines` where the model has one, so one declaration feeds both the help bar's
entries and the walk, and a screen writes no handler and no `setAction` call. `emusen-qt-models`' `HelpModel` gives
each screen's lines from the same table as its entries (`help.rs`, `EmuSen_Qt.md` §7.1). The pressed action comes back
as `actionInvoked(key)`; a screen passes the key on in one call.

**A press passed on as a key is not walked twice** (found 2026-11-08 with the program's screens). A press the walk
declines goes to the focused control as its key (§7.3); when the control declines that key too, it reaches the frame's
`Keys.onPressed`, which walked it again. For B that raised `backDeclined` twice, and the program went back two screens
for one press. `offer` now marks the key it sends, and `key` declines a key while it is being passed on. The C# frame
had no second path, since its keys and the pad met in one handler.

**Moonrise under reduced motion, and the keys that skip it.** `MoonriseModel.still`, which the kit's `Moonrise` binds
to `Tokens.reducedMotion`, draws the splash's last frame from the start while its clock runs on, and `finished` waits
for the clock to reach the length, as `EmuSen_Interface.md` §22 and Mistress hold it. `qt_keys::press_of` is the
press a Qt key means in the kit's map (kit-core's `intent` over `kit_key`), none for Tab or any other key;
`MoonriseModel.takeKey` skips on such a key and keeps every key from the screens behind, since a Tab left unaccepted
walked Qt's focus chain to the screen under the splash.

**The registry** (rule Q, §7.6) gained `PlanetLayout`, `BodyShapes.tileWidth` and `tileHeight`,
`FrameController.actionLines`, and `MoonriseModel.still` and `takeKey` (each placed last, so the other properties keep
their indexes); nothing was removed.

**Held to** (`kit-core/tests/layout.rs`, 3, and `input.rs`'s action lines, measured 2026-11-08): the courts canvas's
eleven bodies under three places (the places at 64, 454 and 844, 372 wide; six tiles from 67, five from 164; the orbits
at 238 and 438); the walk's neighbours, a tie going to the first, the ends stopping; capacity at three widths and seven
bodies dividing four and three; the lines read with their buttons by name, a line naming no button skipped. The program's
`tests/screens.rs` holds the rest through the harness (`EmuSen_Qt.md` §7.1). Seeded faults, each reverted: a tie going to
the last failed the walk test; the passed key's mark dropped failed the program's screens test at B from the grid.

### 7.20 A theme's own colours, the help bar's face, and paired glyphs (stage 1; 2026-11-08)

Three extensions, each optional, found while the program's screens were brought to the accepted render (EmuSen_Qt.md
§7.1, step 5). None changes what an existing screen draws.

- **`Kit.Scene`**, a plain `Item` whose content is a theme's own drawing in its own colours: a console's hue on its
  engine's name and tag, the crescent's ink beside the wordmark. The gate's palette reach treats it as it treats `Body`,
  `Sky` and `Moonrise` (`emusen_qt_test::tree::SCENES`): its colours are the author's. Contrast still holds its words to
  4.5:1. A theme that paints a hue outside `Scene` fails the palette, as before.
- **`HelpBar.labelFont` and `entrySpacing`**, aliased by `Frame` as `helpFont` and `helpSpacing`, so a theme sets its
  labels' face and the gap between entries (the reference theme: Barlow at 20, entries 40 apart, as the accepted render
  has them). **`HelpBar.glyphGap`**, aliased as `helpGlyphGap`, is the gap from an entry's glyph to its label: the kit's
  default stays 8 (`base` × 2), and the reference theme sets 10, the `IconTextSpacing` Mistress's help bands use. An entry's width is its content's: the style's button no longer pads a short entry ("Play") to its own
  minimum, which pushed every later entry right.
- **A paired entry.** An entry whose model gives a `pair` role draws a second glyph after its first: the shoulders, LB
  and RB, as one entry ("Console"), as the accepted render draws them. A model without the role draws one glyph.
  `HelpModel`'s `pair` comes from `help::pair_of`, which names the right shoulder beside the left and nothing else.
- **The glyph's letter is drawn as a path** (`PathText`), as the cut letter already was, so the gate reads a pad glyph as
  one picture. Drawn as a `Text`, the Xbox family's lettered rings, which the own look's screens show for a generic pad,
  failed contrast at 1.00:1, the letter sampled over its own ring's ink.

### 7.21 A menu row's marks and rules, and a help bar a screen draws itself (stage 1, the Pause Card; 2026-11-08)

Three extensions, each optional and off by default, found while the Pause Card was brought to Mistress's render
(`EmuSen_Qt.md` §7.1, step 7). Each is an extension of an existing control rather than a type of the program's own, since
LunaP's `MenuPanel` drew all three for every card, and the Home card and the List card of stage 2 draw the same rows.
None changes what an existing screen draws: the gallery's gate passes unchanged.

- **`MenuRow.steps`, `opens`, `ruled`, `sectionEnd`, `inset` and `labelFont`.** A stepped value stands between ◂ and ▸
  (Save State's slot, the speed), a row that turns to a page of its own ends in ▸ (Game Options, Leave), each mark a
  `Shape` triangle in Muted 12 units from the value, a step's 8 × 11 units and an opening row's 9 × 13, as Mistress's
  render measures them (measured 2026-11-08 within a pixel at 1280 × 800). `ruled` draws a
  one-unit rule in `Rule` along the row's foot, and `sectionEnd` draws it in `Border`, the lighter rule before a new
  section (Mistress's `SectionRule` is `#707076`, six levels from `Border` in blue: the token is the one owner, and the
  difference is named in `EmuSen_Qt.md` §7.1). `inset` is the words' distance from the row's edges (the kit's default
  stays 16; the card sets 24) and `labelFont` the face of the label and the value.
- **`HelpBar.ruled`**: the rule along the bar's top, on by default; off where the host draws the band itself.
- **`Frame.helpShown`**: the frame's own help bar on the foot, on by default. The Pause Card's band is 98 units high, lies
  under the panel and its shadow, and slides down on its own, so the card draws a `HelpBar` of its own over the frame's
  controller and help model while the frame keeps the model for its declared actions (§7.19).

- **A fourth face**, `fonts/BarlowCondensed-Regular.ttf` (LunaP's own file, SHA-256 beginning `b4bb48af`; the Barlow
  licence and notice already cover it): the face of LunaP's card titles (`AdaptiveLayout`'s `TitleSize` 32 in
  `CondensedFile`). The card's title in Medium came out 265 units wide where Mistress's is 255; in Regular, 254.

**Held to**: the gallery's gate (4) and the kit's 39 tests pass unchanged; `qt-app/tests/pause.rs` reads the marks' rows
through the card. No C++ was added.

### 7.22 The docked keyboard (stage 2, the forms; 2026-11-08)

Owed by §7.8 and decided by `EmuSen_Interface.md` §18.1: one docked keyboard serves every text row (search, Edit
Details, the Shell, a tool's search). Weighed as `EmuSen_Qt.md` §1.3 asks, it is an extension of the kit: a new type and
a new model, no second job for any existing one. *[own, because Qt Virtual Keyboard is GPL-only in the open edition and
docks nowhere, §1.4.]*

- **The rules are kit-core's** (`keyboard.rs`, no Qt). A layout is plain text: its name, then one row a line, keys apart
  by spaces with a width after a colon (`Space:5`), the special keys named `Erase`, `Space`, `Shift`, `Next` and `Done`.
  `LETTERS` and `SYMBOLS` are LunaP's `KeyboardLayout.Search` and `Symbols`, the layouts Mistress's search shows; a host
  may hand its own through `KeyboardModel.layouts` (layouts apart by a blank line). A key's box is `DOCKED.key_box`:
  keys 72 × 48 units, 8 apart, each row centred, the first row 16 units under a 1-unit rule at the band's top, the band
  304 units high for five rows (Mistress's band is 398 to 702 with its keys 15 above its foot). The walk is LunaP's
  `OnScreenKeyboard.Move`: across a row it wraps; up and down it goes to the key whose middle is nearest, the first of two
  as near, and stops at the top and the foot. Characters go in at the caret, upper-cased while shifted, the shift letting
  go after one letter; Next shows the next layout and is labelled with its name; Erase shows as `⌫`. **B erases the
  character before the caret, and with nothing to erase closes the keyboard, not keeping**; Done closes it keeping the
  text.
- **`KeyboardModel`** (kit-qt, `keyboard.rs`) is the keys as a list model (roles `label`, `keyX` from the band's centre,
  `keyY`, `keyWidth`, `keyHeight`), with `text`, `caret`, `current`, `shifted` and `bandHeight`; `key(key, modifiers,
  text)` takes what the band's item is given (the arrows walk, Return types, Escape is B, Backspace erases, printable
  text from a physical keyboard goes in at the caret), `act` takes a frame's declared actions (`space`, `done`, `erase`),
  `point` and `click` the pointer's, `setText` a field's own edits; `closed(kept)` reports the close.
- **`Keyboard.qml`** draws the band on Surface under its Border rule, each key on SurfaceRaised with the row radius, the
  current key ringed in the accent 2 units wide with its word in TextStrong while the band holds the focus (LunaP's
  docked keyboard draws no ring once the focus has left it). The pad reaches it as any control: the frame's walk declines
  the directions, A and B, which arrive as keys (§7.3). Y and X are a screen's declared actions (`HelpModel`'s `Typing`
  entries: `space Search`, `done Screensaver`), passed to `act` in one call; the shoulders and Start stay the screen's.
  Hovering a key makes it current; a click types it. A `TextField` shows the text by binding `text` and `cursorPosition`
  to the model and hands its own edits back through `setText`.
- **B is the focused control's before it is the host's.** Before this step the frame raised `backDeclined` as soon as
  its walk declined B, and then sent Escape to the focused control too, so a screen would have gone back while the
  keyboard erased. `send_key` now reports whether an item took the press (`QKeyEvent::isAccepted`, with no focus item
  counting as none), and `backDeclined` is raised only when nothing took it. An open combo box's list closes on B the
  same way without the screen going back.

**Held to**: kit-core's `tests/keyboard.rs` (5: Mistress's boxes on a 1280 band from `GamesSearch-1280x800.spec`, the
labels through the shift and the layouts, the walk's wrap, nearest key and ends, B and Done, a layout read from lines);
qt-test's `keyboard.rs` (the band at 0,398 1280 × 304 and the keys at Mistress's boxes; Down and A type `q` into the
field, Y a space, B erases; a click types its key and makes it current; B twice more empties the field and the third
closes without keeping and without the screen's B; X keeps; the gate at the five sizes; still once settled; nothing
logged). Measured on 2026-11-08.

### 7.23 Notices (stage 2, the forms; 2026-11-08)

`EmuSen_Interface.md` §18.6. An extension of the kit, as §7.22: kit-core's rule, a model, and a layer.

- **The rule** (kit-core `notice.rs`): one notice shown at a time, a second replacing the first and its hold starting
  again (`HOLD_MS`, 2,000 ms). Over running gameplay only a notice raised as fit for it shows (Saved, Loaded, the
  screenshot); any other waits, a later one replacing the one waiting, and is shown when gameplay stops, which is when
  a card or a screen comes up. A notice unfit for gameplay that is showing as gameplay starts goes back to waiting.
- **`NoticeModel`** (kit-qt) puts the shown notice on Qt (`words`, `attention`, `shown`, `holdMs`) and takes
  `gameRunning` from the host; `raise(words, attention, overGame)` and `expire()`. **How a model raises one**: the model
  that knows (a session saving a state, a tool approving a core) emits a signal with its words, and the window passes it
  on to `raise` in one handler line; the kit knows no words and no games.
- **`NoticeLayer.qml`** draws it at the top right inside the safe area (its right edge at 1216 and its top at 40 on the
  1280 × 800 canvas): SurfaceRaised, the tile radius, a 1-unit Border, padding 14 × 24, the status glyph in Success (a
  check) or Attention (a triangle) and the words in Body TextStrong. It fades and slides in over Enter and out over Exit
  (nothing moves under reduced motion; the fades keep their 120 ms), and a `Timer` of `holdMs`, restarted on each
  `raised`, ends it. The big-screen and card forms have no status bar (`EmuSen_Qt.md` §1.7), so a status line there is a
  notice. The shadow of §5.4 is drawn on the GPU (§7.27); the software backend cannot draw it (`MultiEffect` is an
  empty box there, `EmuSen_Qt.md` §7.1 step 6). *Not drawn yet*: the desktop's status-bar line, which is stage 6's.

**Held to**: kit-core's `tests/notice.rs` (2); qt-test's `notice.rs` (the card at the top right; a second replacing the
first with its hold started again; gone after 2 s; over gameplay a notice of the game's own shown and another waiting
until gameplay stops; still once gone; nothing logged). Measured on 2026-11-08.

### 7.24 The question form, and the vertical walk past a field (stage 2, the forms; 2026-11-08)

**The question** is `EmuSen_Interface.md` §15.6's form and §18.3's Resume question, drawn as Mistress's question card
(`CoresPlan-1280x800.spec`, LunaP's `ManagerLayout` plan sheet in its big-screen form). `Question.qml` stays a `Sheet`, so
the controller offers it every press first (§7.3), and gains its form: the popup covers the window, its content on a
`NormalizedCanvas` (so the card is in design units at every size; the desktop form, stage 6's dialog, draws it
unscaled); a cover in `overArtScrim` (#000000B8, the value of Mistress's cover) from the top to the help bar, which stays
undimmed; the card 720 wide on SurfaceRaised with the Border edge and the tile radius, centred over the cover, its words
33 units inside (padding 32 and the border), as tall as its content up to the cover's height less 100; the title in Title
(the reference theme sets Barlow Condensed Regular, the card titles' face, §7.21) in Text; then `message` and the
`sentences` in Body, 8 apart, and the `caption` in Caption Muted (the setting that stops the asking, a hash), scrolling
between the title and the answers when they are long; then the answers as `MenuRow`s 654 × 54, 4 apart, 12 under the
words and 33 above the card's foot, **the first focused** (the safe answer, `No` or `Continue`: a hurried A never does
what cannot be undone). A chooses the focused answer, a click chooses the one under it, and **B gives `backIndex`**
(`No`'s index, or -1 to go back without answering, as the Resume question's B goes back to the library). Its own help bar
is `help`, a host's model (`HelpModel`'s `Question`, A Choose · B No, or `QuestionBack`, A Choose · B Back). The words
are a model's: `emusen-qt-models`' `question.rs` writes `confirm(title, sentences)` (No first, B No) and `resume(game)`
(§18.3: "Continue where you left off?", one sentence, Continue and Start again, the caption naming Settings ▸ EmuSen ▸
Saves and states ▸ Resume ◂ Ask each time ▸).

`MenuRow.overRaised` (an additive property, off by default) is the answers' look: a row on a raised card is clear at
rest and Surface when focused, as Mistress's question draws its focused answer, with the accent bar.

**The vertical walk past a field or a combo box** (owed by §7.9 and §7.7, decided with the docked keyboard): the row walks
first, and the control takes the keys only once chosen. Measured on 2026-11-08 with Qt 6.11.2, a one-line `TextField`
already declines Up and Down (with the caret mid-word, Up and Down reached the list), so it is unchanged; the combo box
took them and changed its value. The style's `ComboBox` now hands Up and Down, while its list is closed, to its
ancestors as Qt's own propagation would and never acts on them (`KeyPass.toAncestors`, a singleton over `pass_key` in
`kit_glue.cpp`, which offers the key to each ancestor in turn until one accepts it), so a `ListView` of rows moves past
it and its value stays; its open list keeps Up and Down; A (Return or Enter) opens a closed list. A declared path (`KeyNavigation` on the control, as the gallery's fixtures have) still wins,
since its handler runs first.

**Held to**: qt-test's `sheet.rs` (the card at 280, 720 wide, centred over a cover that stops at the help bar at 742, the
title 33 units inside and 39 high, the answers 654 × 54 four apart and 33 above the foot, No focused; B answering No;
Down and A choosing Yes; a click choosing Yes; a key declined behind it not looping, §7.14; the gate at the five sizes
with it open; still; nothing logged) and `walk.rs` (Down passing a field and a combo box in a list with their caret and
value unmoved, Up passing both and leaving the field to the row above with its caret unmoved, A opening the combo box's
list); qt-models' `question.rs` (3). Measured on 2026-11-08.

**Not done, named:** with a combo box's list open, the focus leaves the frame for the list's popup, so the frame's
`Connections` stop offering the pad's presses until the list closes; the pointer and the keyboard work. A popup in the
frame's sheet rule (as `Sheet` does) is the fix; owed to stage 5, whose settings pages are the first with combo boxes.

### 7.25 The pad router sleeps; the night help band (stage 2; 2026-11-08)

- **The router's clock runs only while it has work** (owed by `EmuSen_Qt.md` §9.3.2, "Seen, not changed"). Each
  `Frame`, `Game.qml` and the splash ran an 8 ms `Timer` pumping `PadRouter` while shown, waking the GUI thread 125
  times a second on a still screen. `Router::pending` is true while an event waits or a button is held (its repeats and
  the chord are timed by the clock); `PadRouter.pending` follows it after every push and pump, and the three timers run
  only while it is true. A push starts the clock and the pump that applies the last release stops it. Repeats are
  unchanged: a held button keeps the clock running. Held to by `kit-qt/tests/router.rs` (idle, a waiting event, a held
  button, let go) and by `idle.rs`, `pause.rs` and the gallery's `pads.rs`, which drive the router through it.
- **The help band is glass at night** (the tester, 2026-11-08: on Home the band read opaque where Mistress's shows the
  sky through it). Mistress draws a full screen's band from LunaP's `HelpBand`, which at night is the input surface at
  0.72 with its Border rule at 0.72, flush on the foot. `HelpBar.glass` draws that, and `Frame` sets it on the big screen
  whenever the frame draws no surface of its own (`surfaceDrawn` off, the night screens with their sky). The fill is a
  translucent colour, `Qt.alpha(inputSurface, 0.72)`, never `transparent` with an overlay: measured on Vulkan on
  2026-11-08, a root `Rectangle` in `transparent` left the strip at alpha 184 with no sky under it (the trap the Pause
  Card met, `EmuSen_Qt.md` §7.1 step 7), and the translucent colour blends, the band reading (15, 17, 28) against
  Mistress's (15, 17, 27) at 1920 × 1200. kit-core's palette reach now takes a declared opaque token at any alpha, since
  a token through a colour's alpha is the token as it is through an item's opacity (`tests/audit.rs`).

### 7.26 The Guide chords, the shoulders on a card, and a property's signal in the harness (stage 2; 2026-11-08)

- **The Guide chords** (`EmuSen_BigPicture.md` §48.7, Mistress's `GuideChordReader`). kit-core's `GuideChords`
  (`input/guide.rs`) reads the held buttons each tick: Guide held with Up, Down or the right shoulder going down is
  `With(button)`, the first of `GuideChords::BUTTONS` to go down winning the tick; a button already down when Guide went
  down is no chord; Guide let go with nothing chorded is `Alone`; `reset` forgets a Guide held across a menu. The kit
  says only which button was chorded. What each means (save, load, a screenshot) is the program's, in
  `emusen-qt-models`' `pause::guide_chord`.
- **The router reads them while a game owns the pad.** With `set_guide_chords` on and `chord_only`, Guide alone opens the
  menu as it is let go (`Pumped.menu_chord`), a chord is `Pumped.guide_chord` and opens nothing, and Options with Menu
  open the menu at once only while Guide is up, as Mistress's `MainWindow.Pad` has it. Away from a game, or with the
  chords off, the reader is reset each pump and Guide opens the menu as it goes down, as before.
- **`PadRouter` gains `guideChords` and `guideChord(button)`**, reviewed under rule Q: an extension, both declared last
  (property index 5 after `pending`; the signal after `isHeld`), so no existing index moves. The committed registry was
  diffed before it was copied: 16 lines added, none changed beyond line numbers. `Game.qml` binds the property and
  hands the signal to `SessionModel.guideChord`.
- **`guideHeld`, and the game kept from the pad while Guide is held** (decided by the tester on 2026-11-08, as Mistress's
  `_padQuiet` keeps it). `Router::guide_held` is true while Guide is held with the chords read over a game, and
  `PadRouter.guideHeld` follows it after every pump and every change of `guideChords` or `chordOnly`: a read-only
  property, the seventh, declared last under rule Q (the registry's diff again additions only). The program binds it to
  `PadReader.gameMuted`, and the pads' hand-over (`GamePads::set_muted`) gives the game no button while it is on and,
  after it, none until every button is let go, as after a menu. So Guide with Up saves the state the game was in, not
  one with Up pressed.
- **A card's shoulders and triggers reach its row.** `UiButton::key` gave no key for PageUp, PageDown, First and Last,
  though `Key::button` mapped PageUp, PageDown, Home and End to them, so a press the walk declined on a card (whose
  form has no sections to step) was dropped. They now become PageUp, PageDown, Home and End for the focused control,
  as every other declined press becomes its key. On the big screen and the desktop the walk still takes the shoulders
  for its sections first. `MenuCard` hands a key its row did not take to its host (`keyed`), and the Pause Card's
  model strides the reel with it (`EmuSen_Qt.md` §7.5).
- **The harness counts a property's change signal.** `Scene::watch(object, property)` puts a `QSignalSpy` on the
  property's notify signal and `notified` reads how many times it fired since. Since Qt 6.9 `QSignalSpy` is no
  QObject, so a child object of the watched one holds it, to live and be found with it. This answers the seeded fault
  that no test caught in stage 2's Speed step (`EmuSen_Qt.md` §7.3): a property whose value is right but whose change is
  never announced. *C++ part*: `harness_watch` and `harness_notified` in `qt-test/cpp/harness.cpp` [own, because QtTest's
  spy is C++ and cxx-qt reaches no signal by its meta-method].
- Held to by `kit-core/tests/input.rs` (Mistress's cases of the reader, and the four keys), `kit-qt/tests/router.rs`
  (a chord, Guide alone on release, the two middle buttons, the chords off, `guide_held` while Guide is held and only
  with the chords on), `qmltypes.rs`, and `emusen-qt-models`' `input` over the test core and a simulated pad (A held
  while muted is not heard, nor after the mute until let go, and is heard pressed again). Seeded faults, each reverted:
  the hand-over ignoring the mute (`input`); `guide_held` with the chords off (`router`).

### 7.27 A shoulder a screen declares; the pointer resting; the notice's shadow (stage 2, the loose ends; 2026-11-08)

- **The shoulders without sections.** The walk stepped its sections on LB and RB in every form but the card's, and
  with no sections it declined them, so a screen could not declare a shoulder (`EmuSen_Qt.md` §7.6: the search's Move
  to grid). Now a walk with no sections offers the shoulders to the declared actions like any other button; a walk
  with sections keeps them. An extension under rule Q: no surface changes, and a screen that declares no shoulder sees
  no difference. Held to kit-core's `tests/input.rs` (both shoulders reaching a declared `toGrid` with no sections; a
  walk with sections still stepping them past a declared shoulder).
- **The pointer resting.** `Scene::hover` (qt-test, `harness_hover`, Qt Test's `mouseMove`) moves the pointer to a point
  and leaves it there, nothing pressed, so a test reaches what a hover does without a click's A. *C++ part*: one
  function in `cpp/harness.cpp` [own, because the harness's pointer is Qt Test's, which has no Rust binding]. Found by
  it on 2026-11-08: the pointer left where a click had been lands on a cover after a resize to 3840 × 2160, which took
  the focus there (`EmuSen_Qt.md` §7.6's search follows the focus).
- **The notice's shadow** (§7.23 left it undrawn): §5.4's one shadow, offset 0,8, blur 24, black at 50%
  (`shadowOpacity`, `shadowRadius`), cast by a copy of the card's shape through `MultiEffect`, as the cards cast theirs,
  fading and sliding with the card. The software backend still draws none. Mistress's own notice (LunaP's
  `NoticeLayer`, a pill on HudSurface) draws no shadow; the design's (`EmuSen_Interface.md` §18.6, "the shadow") is
  kept (decided 2026-11-08 by the tester), a difference from Mistress named in `EmuSen_Qt.md` §7.6. Held to `qt-app`'s `acceptance.rs` on Vulkan.

### 7.28 The kit's extensions for the ES-DE painter: a release, a key through the queue, the filled glyphs, a help bar outside a frame (stage 4, K; 2026-11-08)

The ES-DE view (`EmuSen_Qt.md` §4) takes the router directly, as `Game.qml` does, and draws a theme's `helpsystem`
element in the theme's style. Each piece below is an extension of an existing surface under rule Q, declared after
what was there; none changes what an existing screen draws, and the gallery's gate passes unchanged (73 states).

- **`PadRouter.released(button)`.** ES-DE fades its metadata back in as a held direction is let go, and a held list
  stops at its end. `Router` raises a release at the pump that applies it, once, and only for a button whose press it
  raised (`raised`, cleared on release and by the menu chord), with the player's swap as the press had it. A button
  held over a game, or one the menu chord took, was never pressed and is never released. The pad's releases are
  raised before the pump's presses, so a release and a press at one pump leave the press last.
- **`PadRouter.pushKey(key, modifiers)`**, an invokable returning whether the key is known: the kit's key map
  (`qt_keys::press_of`, the one owner of what a key means) made a press through the router's queue, so the view's
  `Keys.onPressed` is one line and a key and a pad share the pump's order. A key the map does not know (Tab, letters)
  is declined, false. A key is a tap: at the pump it is raised as a press and then its release, after the pad's
  presses, never swapped (Return is Accept whatever the swap, as a key is not the pad's button), and nothing while a
  game owns the pad (`chordOnly`). A key held down repeats at the keyboard's own rate, each repeat a tap; ES-DE's
  measured tiers apply to the pad only (a named difference, §7.7).
- **The filled set** (`glyph::GlyphStyle::Filled`, `glyph/filled.rs`): every control of every family one solid shape
  with its letter, symbol or mark cut out, as ES-DE's help draws its buttons and LunaP's `PadGlyphStyle.Filled` drew
  them for Mistress (`EmuSen_BigPicture.md` §40.1): the face buttons a disc 0.425 of the box (22 pixels beside a
  26-pixel label), lettered by the family or, on PlayStation, its shape cut as the band between two outlines; the
  shoulders and triggers their bodies with the family's name cut out; Start and Select a disc (a pill on PlayStation)
  with the family's mark cut; the d-pad a solid plus with an arrow cut into the tip of the arm the press moves along;
  a stick's click a disc with a ring cut, Guide a disc with a hole. A cut letter is bold (`Letter.bold`), so it reads
  at 22 pixels. **An unknown pad is lettered as an Xbox pad in the filled set** (`glyph::lettered`), ES-DE's default
  type, as Mistress did; the outlined set keeps the positional face. `Mark` gained `holes`, closed polygons cut out of
  a filled mark odd-even, so a ring is two of them and no two holes overlap. kit-qt: `Kit.GlyphStyles` mirrors the
  style; `PadGlyphMarks` gained `style`, the `holes` role and `letterBold`; `PadGlyph.style`, its holes drawn by a
  `PathMultiline` in the mark's own `ShapePath`.
- **`HelpBar`** gains `pressed(button)`, raised by every entry's click; `controller` is optional (null by default), and
  `Frame` connects its bar's `pressed` to `controller.offer` instead of passing it, so a press is offered once. Further:
  `glyphStyle`, `labelColor`, `iconColor` (the glyphs' ink), `entryScale` (an entry's glyph, gap and label at that share
  of their size), `horizontalPadding` and `verticalPadding`, and `fitted`, a box that hugs its entries (each entry as
  high as its content, by a `Binding` that leaves the style's height on the band) in place of the band on the foot.
  The background's colour and corner radius are the bar's own `color` and `radius`, so no property was added for them.
  An entry whose model gives an `icon` role (a theme's `customButtonIcon`) draws that picture in the glyph's place.
- **Added 2026-11-08 (stage 4, E3):** `HelpBar.glyphSize`, a glyph's side in pixels before `entryScale` (0, the default,
  is the glyph's own size), so the ES-DE painter sizes its icons from a theme's `fontSize` as ES-DE does
  (EmuSen_Qt.md §4.8.5). `PadGlyph`'s letter now grows with its glyph: its pixel size is the marks' `size` times
  `letterScale` over `sideShare` (1.4, the glyph's own side over the glyph type's size), where it had stayed at the
  token's size, so a glyph drawn larger (by `glyphSize` or `entryScale`) kept a small letter in a large disc. At the own
  size nothing drawn changes.
- **Corrected 2026-11-08 (E3):** the first form of that growth read the item's own `width` and `height` inside the
  letter's binding, and `PadGlyphMarks` announced its letter and `letterScale` again on every change of `size`; Qt 6
  evaluates a pending `width` binding when it is read, so the letter's binding was told of a change while it ran, and
  Qt logged "Binding loop detected" on every glyph of qt-app's help (its `program` test, CPU tier, red on it: the
  seeded fault). The letter now reads only the marks, and a change of `size` resets the marks' outlines alone, since the
  letter does not depend on it.
- **Changed 2026-11-08 (E3): `HelpBar` is an `Item` whose backing `Rectangle` is drawn only when `color` has any
  alpha.** A clear `Rectangle` as the bar's root wrote alpha 0 over its strip on Vulkan (Qt 6.11.2, RX 6800), and the
  view beneath it was not drawn there, as on the menu card (EmuSen_Qt.md §7.1, step 7): Art Book Next's help in the
  ES-DE painter showed a see-through strip at every size after the first (EmuSen_Qt.md §4.8.5). `color` (with the glass
  and Surface defaults) and `radius` are kept as properties of the bar; a Rectangle's `border` and `gradient` are no
  longer the bar's, and no host set them.

**The filled help bar beside Mistress's**, at 1280 × 800: `emusen-qt-test`'s `helpbar` writes
`~/.cache/emusen/tmp/qt-test/helpbar-filled.png` (the ES-DE gamelist's ten entries in the four families, white on
`#202020`, at §40.1's sizes, and a fitted bar with a picture), and `side-by-side-helpbar.png` beside it pairs each bar
with `MenusFollowupPictureTool`'s `synthetic-1280x800-gamelist-<family>` (§40.9). Looked at on 2026-11-08: the same
letters by family and by position (Nintendo's B on South), the same discs, pills and cut shapes, the same order and
spacing within a few pixels. Differences seen, none a defect of the set: Mistress's d-pad for Choose shows both arrows
of its axis where one press draws one; the fixture's labels sit in a taller line box (the kit's face, not ES-DE's), so
its box is 47 pixels high against 34; the glyphs' letters are a little lighter than LunaP's widened ones.

**Held to**: kit-core's `tests/glyph.rs` (every control of every family in the filled set one closed solid shape in its
box, a cut letter (bold) or holes and never both, every hole inside its shape, no two controls of a family alike but
the two sticks; the unknown pad lettered as Xbox in the filled set only; the outlined set unchanged); kit-qt's `router`
(a release once after its press, swapped as it was, none for a button held over a game or taken by the chord; Return
and Enter Accept and Escape Back with the swap on, Tab and a letter declined, no key while a game owns the pad),
`kinds` (the styles pair by pair) and `qmltypes`; `emusen-qt-test`'s `glyphs` (the filled set's 64 beside the outlined
64, Accept's name on each family's South, every filled glyph showing its cut), `helpbar` (a click raises its press once
with no controller, the fitted box round its entries and paddings, the picture drawn and the glyph hidden, every
filled glyph of the four bars cut) and `menu` (the frame's bar still offers its press); the gallery's gate. Rule Q's
registry gained `GlyphStyles`, `PadGlyphMarks.style` and `letterBold` and the `Holes` role, and `PadRouter.released`
and `pushKey`, each declared last; the diff was read before it was copied, additions only. Measured on 2026-11-08 with
four seeded faults, each reverted: a release raised without its press failed `router` (held over a game, released);
the key's press swapped failed `router` (Return as Back); the holes not drawn failed `glyphs` (18 filled glyphs with
nothing cut out); the frame's bar not offering its press failed `menu` (the help bar's Open reached no row).

**A declined entry (2026-11-08, for Horizon's disabled look).** An entry whose model carries `enabled: false` is drawn at
38% and is not pressed; an entry without the role is enabled, so every bar already drawn is unchanged. The role is read
as `model.enabled`, never as a delegate's required property, which would override `Item.enabled`. `EmuSen.Theme`'s
`Help.entries(kind, {accept: false})` gives A's entry so (`EmuSen_Qt.md` §3.2). Proof: `helpbar` (Back at 0.38, Play
whole, a click on Back raising nothing).

### 7.29 A cover tile and a rounded picture (stage 3, the reference theme as a package; 2026-11-08)

The reference theme drew a game's cover with its own `Cover.qml`, `RoundPicture.qml` and `GameCard.qml`. A theme may not
reach the program's files, and every theme that draws covers needs the same three rules, so they moved into the kit as
two types, added last to the registry's qmldir (rule Q, additions only), and `EmuSen.Theme` re-exports them by name.

- **`RoundPicture`**: a picture fitted whole (`Image.PreserveAspectFit`, aligned as asked) or cut to fill its box
  (`crop`), drawn as a `Shape` whose path is a rounded rectangle filled with the `Image`'s own texture; on the software
  backend, which has no texture fills, the `Image` itself with plain corners. It is decoded at the size it is drawn
  (`pixelScale`, canvas units to pixels, times the screen's ratio).
- **`CoverTile`**: a game's cover whole at its own shape on its box's foot (or its right, `alignRight`), the generated
  card in the console's hue with the title where there is no picture or it fails to read, the favourite's mark at the
  picture's top right in the accent, the ring at 7 units about the painted picture, and the lift of a lit tile (1.06
  about its centre, 6 units up) unless `lifts` is off. The kit owns no mark: `mark` is the favourite's path in a 24-unit
  box, given by the host (`Marks.heart` of `EmuSen.Models`) or the theme (`Marks.heart` of `EmuSen.Theme`), both the
  models' `marks::HEART`.

Proof: the program's card pane and every cover of the reference package are these types, so the stage 1 and stage 2
recordings (`qt-app/tests/cards.rs`, the acceptance test) hold them to the accepted renders unchanged.

**The generated card's title** (2026-11-08) is drawn in `overArt` and the kit's Barlow (`Barlow-Regular`, the body face,
loaded by the tile). It was `textStrong` in the system face: near-black on the card's hsl(h, 45%, 32%) in the light
variant, and not the design's face in either. White on the card's head, its lightest end, is 4.75:1 at
the lightest hue and up to 12.0:1 at the darkest (computed over the 360 hues), and more toward the foot, where the title
stands. Drawn smaller than its box, the title keeps its size on the screen (decided 2026-11-09, over dropping the title,
so a card always names its game): `drawnScale`, the tile's own `scale` unless a theme that scales an ancestor sets it,
divides the title's size when under 1. Orrery draws a far cover at 0.8, where the title's capitals measured 11.31 at
1280 × 720 against the floor's 11.64. `theme-qt/tests/qml.rs` holds the title's ink and face in both variants, and its
capitals on the screen the same for a plain tile, a tile scaled 0.8 and a tile in an item scaled 0.8 that says so. Re-recorded on the RX 6800 on
2026-11-09 (the acceptance test and `cards.rs`, both passing before and after): of 100 pictures only the Pause Card's
Change Disc page changed for the card, whose disc has no picture (its title now Barlow and white, at every size); Home
and the Saved page changed only in their clocks' minutes.

### 7.30 Orbits: groups of bodies on nested half-ellipses (stage 3, the reference theme's Choose a console; 2026-11-08)

**Decided 2026-11-08: an extension of the kit, a new type beside `PlanetLayout`.** The reference theme's Choose a console
became the orbits (`EmuSen_Qt.md` §3.4.2). Its layout depends on the whole catalogue: the makers sorted by how many consoles
each has, which three are shown, how far each long orbit has turned, and where Up goes. A theme cannot work that out:
§1.3 allows no loop in QML, and `EmuSen.Theme` gives no model of one maker's consoles. Writing it as bindings would have
put about 300 lines of loop-free arithmetic in a screen. The form weighed was an extension, since laying bodies out is
the kit's job (§7.19) and needs no knowledge of consoles: the kit is given a group key for each row and nothing else.

**`kit-core`'s `orbits::orbit_layout(width, groups, leading, focus, kept, geometry)`** is pure Rust.
- **Groups.** It groups the bodies by key, ordering the groups by first appearance and keeping each group's members in
  the list's order, so an interleaved table works as a sorted one. It sorts the groups by size, ascending and stable.
- **Slots.** Slot s is the half-ellipse about (W/2, 0) with radii 260 + 180 s by 220 + 150 s. The window on it is the
  angles where a body's centre stays below y 178 (197 on slot 0, decided 2026-11-08, so a focused body's ring clears
  the leading row at 140) and at least 100 inside each side; L is the window's arc length at the
  mean radius. Capacity is ⌊L / 150⌋ + 1.
- **Which groups are shown.** One group takes slot 1, or slot 2 when slot 1 cannot hold it whole. Two groups take slots 1
  and 2. Three or more take three consecutive slots from a base. The first base centres the focused group where it can;
  later the previous base (`kept`) holds until the focus leaves the shown groups, and then moves by the least that shows
  it again.
- **A group longer than its window turns.** The window holds g + 1 bodies, g = ⌊L / 150⌋. The focus stands at the
  ⌊g/2⌋-th of them and the set is centred in the window: off = clamp(s_focus − ⌊g/2⌋ · 150 − (L − 150 g)/2, 0,
  (n − 1) · 150 − L), and world i sits at angle θmax − (150 i − off) / r. For an odd capacity this is the canvas's
  centred focus; for an even one (the inner slot's 2, the middle's 6, the outer's 10 at 16:9) it shows the whole
  capacity, where a centred focus showed one fewer. A group that fits is centred on π/2.
- **Labels and ellipses.** A group's label is right-aligned at the point 60 units inward from its window's end, never
  further right than W − 64 and never above y 152. Ellipses sit half a pitch past each end that has more bodies beyond
  it, wherever the arc there is below y 150.
- **The walk.** It runs over the leading items, then the bodies. Left and Right move along a group. Up and Down go to the
  shown body nearest in x on the adjacent group, else that group's first body. Up from the innermost goes to the nearest
  leading item, and Down from a leading item to the nearest shown body. Each orbit's `reach` is where a jump to it lands:
  its shown body nearest the focus in x.

`OrbitGeometry::REFERENCE` holds the canvas's measures. The function names no console.

**`kit-qt`'s `OrbitLayout`** reads its input from any list model: `model`, `role` (the group key, read as text) and
`labelRole` (each orbit's label, taken from its first row). It reads again when its `read()` is called, which a screen
does on the model's `modelReset` and `dataChanged`. A screen also sets `width`, `leading` and `current` (the walk's index
of the focus), and passes each leading item's centre through `lead(i, x)`.

It hands back lists that bind by index:
- per body: `angles`, `orbitOf`, `shown`;
- per orbit: `radiiX`, `radiiY`, `orbitShown`, `sizes`, `turning`, `labels`, `labelRights`, `labelTops`, and `beforeXs`,
  `beforeYs`, `afterXs`, `afterYs` (NaN where no ellipsis is drawn);
- `centre`, `orbitCount`, `focusedOrbit`, and `sections` and `section` for the frame's shoulders.

A screen draws a body at (centre + rx cos θ, ry sin θ), with θ and the radii animated by the screen's own `Behavior`s. The
body therefore travels along its arc when an orbit turns, and its radii change when the orbits scroll.

The walk is steps, not neighbour lists. `stepLeft()`, `stepRight()`, `stepUp()` and `stepDown()`, and `jump(key)` for a
section the frame's shoulder chose, each do three things in order: make the target the current focus, lay out again so
the target is shown, and only then emit `jumped(index)`. The screen gives that index the focus. KeyNavigation could not
do this:
1. A target may be waiting on an orbit off screen, invisible until laid out, and Qt's KeyNavigation passes over an
   invisible item.
2. A binding to `Repeater.itemAt` is not evaluated again when the Repeater makes its items. Measured on 2026-11-08: a
   place's `KeyNavigation.down` was still null once its index had arrived, so Down from a place did nothing.

The model is read through `QAbstractItemModel`'s own `rowCount`, `index`, `data` and `roleNames`, with no C++ of its
own. They are declared once, in `kit-qt/src/item_model.rs`, for every layout that reads a model (`OrbitLayout`,
`FigureLayout`). The two layouts' own declarations each emitted the type's upcast shim and would not link side by side
(§7.6). The roles are read as text by the one reader there, `item_model::role_texts`. A model without the group role
gives every row the empty key, so its rows make one group, as an empty maker does.

**Proof** (measured 2026-11-08). `kit-core/tests/orbits.rs`, nine tests:
- the slots' windows and capacities at 1280 (2, 6, 9) and at 1422.2 (the outer slot holds 10);
- a focused world on the inner orbit with its ring at or below y 140, and every resting one's top at or below 142, at
  both widths of the five sizes over four catalogues;
- the canvas's eleven consoles at the canvas's points, within 0.01;
- the labels at (753.72, 152), (971.26, 152) and (1125.76, 219.84);
- the walk, ties going to the first;
- one console at the middle apex, seven on the outer slot, none at all;
- 120 consoles turning, with 18 shown and the outer orbit's ellipses at (1215.30, 193.87);
- forty under one maker: the twenty-first of them at the outer apex, with ellipses at both ends;
- the base kept and moved;
- an interleaved table;
- the jump targets.

`kit-qt/tests/qmltypes.rs` holds the registry, which only gained the type. `theme-qt`'s scene reads a group count from
the fed library.

### 7.31 Groups as figures on a chart (stage 3, Constellation; 2026-11-08)

Constellation's Choose a console draws each maker as a figure of stars, laid out by count. The layout is a shelf packer
with sums per row and a walk to the nearest body of the next line, which a theme's QML cannot hold (§1.3 of
`EmuSen_Qt.md`: no function, no arrays) and `Library.consoles` gives no list of groups to iterate. Decided 2026-11-08:
it is the kit's, beside `PlanetLayout`, and `EmuSen.Theme` re-exports it by name.

**`kit-core`'s `figures::figure_layout(width, keys, geometry)`** takes each item's group key in the table's order and
groups them by first appearance. A figure's box is its label zone, its bodies at the pitch (the pair's pitch for a group
of two) and a label's width; a group too long for the width wraps into lines of as many as fit, at least two. The packer
puts the leading box of three pointers first in the flow, places boxes on shelves of the safe width (gap 64), gives a
figure of several lines a row group of its own, shared only with the pointers waiting alone when both fit, and moves the
pointers to close their row. A row of several boxes is justified to the margins; a lone box is centred. The full tier
holds while it needs at most two lines; past that the compact tier is laid out, and it scrolls when taller than the chart
(468 units). Bodies alternate raised and lowered about each line's centre, the phase following the box's place in its
row. The walk: Left and Right along a line; Up and Down to the node of the adjacent line nearest in x, the first of two
at the same distance; the apex above its two pointers, which sit between the line's last item and each other, and Right from the apex to
the lower right pointer. The canvas has no Right from the apex; theme-audit's 120-console sweep, which walks along each row and
back, never reached Recently played without it (measured 2026-11-08). For the same sweep, Pointers standing alone in their row go
Down to the ends of the next line, the lower left to its first node and the lower right to its last; from Recently played the
nearest node in x lay mid-line, and a sweep entering there missed every console to its right (Game Boy and Super Nintendo of
the 120). These two are the layout's only departures from the canvas. Each item
also gets its group and the item drawn before it in its figure (a wrapped line goes on from the last body above).
`FigureGeometry::CHART` is the Figures canvas's measures (full: body 72, pitch 110 or 130, amplitude 20, label 140,
row pitch 232, zone 100; compact: 56, 96 or 112, 16, 124, 168, 90; chart 96 to 564; the pointers 360 wide, their stars
at (180, -60), (80, 36) and (280, 36)). It names no console.

**`kit-qt`'s `FigureLayout`** reads the role a screen names from any list model, as text (a number's digits), lays it out
at `width`, and gives the lists by index (`xs`, `ys`, `raised`, `left`, `right`, `up`, `down` over the items and then the
three pointers at `count` to `count + 2`; `prior` and `groupOf` over the items; `figureXs`, `figureYs` and `firsts` over
the groups), the tier's `body`, `labelWidth` and `zone`, `compact`, `contentHeight` and `scrolls`. A screen calls
`read()` when the model resets. Added last to the registry (rule Q, §7.6); nothing was removed.

**Held to** (`kit-core/tests/figures.rs`, 6, measured 2026-11-08): the canvas's own packer, run in a browser over its
states, gave every node's place, raised side, line and four neighbours, every figure's box and the chart's tier, lines,
height and scrolling, kept as `tests/figures/*.tsv`; the layout matches all of them at W 1280 and 1422.2 for eleven
consoles of three makers, one console, none, thirty of twelve makers, a hundred and twenty of twelve and a maker of forty
among eleven others; an interleaved table groups by first appearance. `theme-qt`'s harness scene reads the consoles'
groups on the list's reset. Seeded faults, each reverted: the pointers moved to open their row (three tests failed), a
tie going to the last (the forty-maker's walk), the compact tier taken past one line (the eleven).

### 7.32 Groups as wheels over a dial (stage 3, Orrery; 2026-11-08)

Orrery's Choose a console, Wheelwork (`qt-s3/final/orrery.md` §3), stands one maker's wheel at the left with its
consoles on the spokes' ends, the focused one always at the top, and runs a dial along the foot: the screen's places,
then one small wheel per maker. The dial needs one stop per group and a walk across two kinds of node, which a theme's QML
cannot hold (§1.3 of `EmuSen_Qt.md`: no function, no arrays), and the walk must not change as the wheel turns, so
the reach audit sees one graph. Decided 2026-11-08: the layout is the kit's, beside `FigureLayout`, and `EmuSen.Theme`
re-exports it by name.

**`kit-core`'s `wheels::wheels(width, keys, places, geometry)`** groups the items by first appearance and gives each its
group and its place in it, each group's first item and size, the dial's stops (the `places` first, then one per group)
with their centres from x 64 at the pitch, the dial's width, its shown width (W − 128) and whether it scrolls. Nodes are
the items in table order, then the stops. **The walk**: an item's Left and Right are its group's previous and next item
and past a wheel's ends the walk runs on into the previous group's last and the next group's first, so Right held from
the first world hears every console in maker order (the canvas stops at a wheel's ends, which leaves theme-audit's sweep,
Right and Left along a row with Down between, only the first maker's worlds; changed 2026-11-08 so the sweep reaches all 120);
its Down is its group's stop; a stop's Left and Right run along the
dial, a group's stop goes Up to the group's first item, a place to the first group's first (the canvas's places go
Up nowhere, which leaves the audit's sweep, starting on a place, no world to hear; changed 2026-11-09); the shoulders go to the previous and
next group's stop from any node (from a place, RB to the first group's). **`wheels::turn(layout, current, geometry)`**
is the wheel as the focus stands: a focused item puts its group's wheel at its place; a group's stop turns that wheel to
its first item; a place dims the first group's wheel (the spec's "mechanism waiting"); each item of the wheel gets its
distance from the top, shown within the reach of three either way, so slot (d + 8) mod 8 holds it and the rest wait
behind the rim, counted as `hidden` for the three dots; the stop lit for the focus; and the camera, which centres that
stop and is clamped to the dial (the canvas's `want`, `off`). `WheelGeometry::WHEELWORK` is the canvas's measures (eight
slots, reach 3, the dial from x 64 to a margin of 64, pitch 136). The wheel's centre, radius and the slots' angles are
the theme's drawing: the layout gives `step` (45°), and a world's angle is 90° − step × d, so a turn animates the angle
and the worlds travel the rim. It names no console.

**`kit-qt`'s `WheelLayout`** reads the role a screen names from any list model as text, through
`item_model::role_texts`, which `FigureLayout` now shares (one reader of a model's role for every layout). Inputs:
`width`, `places`, `model`, `role`, `weightRole` (a numeric role summed per group, so a maker's stop can say its games)
and `current` (a node, -1 none). On `laidOut`, which follows the rows, the width or the
places only: `count`, `groups`, `reach`, `step`, `dialWidth`, `view`, `scrolls`, and the lists `groupOf`, `placeIn`,
`firsts`, `sizes`, `totals`, `ranks` (each group's rank by size, the fewest first, for rings filled from the inside, as
Orrery's splash draws its orbits), `dialXs` and the walk's `left`, `right`, `up`, `down`, `previous`, `next` over the items and then the
stops. On `turned`, which follows `current`: `wheel` (-1 none), `dimmed`, `hidden`, `lit`, `camera`, and the lists
`turns`, `onWheel` and `shown` over the items. A screen calls `read()` when the model resets. Added last to the registry
(rule Q, §7.6); nothing was removed.

**Held to** (`kit-core/tests/wheels.rs`, 6, measured 2026-11-08), the canvas's rules over its states: the eleven of three
makers focused on the second fill slots 7, 0, 1, 2 and 3 with six stops at 132 … 812 and no camera; one console is one
world and four stops; none is no wheel and three places; thirty of twelve makers is a fifteen-stop dial that scrolls,
the camera 0 at the first maker, centred at a middle one and clamped at the last at W 1280 and 1422.2; a hundred and
twenty of twelve show five of ten at the second world and seven of ten at the sixth, the last maker eleven presses of RB
away; the forty-maker's mid-point fills every slot but the rim's with 33 hidden, and its last world stands at the top
with nothing after it, and Right from the first world runs through all 120; an interleaved table groups by first
appearance and steps by group, not by row, running on in maker order. From the first
focus the walk reaches every node of every catalogue. `theme-qt`'s harness scene reads the consoles' groups on the
list's reset and turns to the first console. Seeded faults, each reverted: the reach taken as two (three tests failed:
the eleven, the hundred and twenty, the forty-maker), the camera centred on the first stop whatever the focus (the thirty).

### 7.33 Overlap on turned shapes, and a stack the kit lays out (stage 3, Orrery; 2026-11-09)

Orrery's orbits tilt each cover by the ellipse's normal and, by the spec, lay a neighbour partly under the apex. The fit
rule's overlap measured every item by its scene box, the axis-aligned box about it, so two tilted covers whose shapes
clear each other were faults, and an overlap the design intends was one too. Decided 2026-11-09: the rule measures the
true shape, and the kit declares the stacks whose members may overlap; bending the design's steps around the measure
was refused.

**Turned shapes.** The harness records a turned item's four corners in the scene (`corners`, where an edge of it is not
level), and `Item::shape` is those corners or the scene box's. Two items overlap when their shapes share more than the
slack along every separating axis (the edges' normals of both, the separating axis theorem); two items neither turned
keep the box rule exactly.

**Stacks.** `StackMember`, a kit-qt object with no properties, marks the item it is placed in as one member of a stack;
the stack is that item's parent. The harness finds it by its class and records, for the member and everything inside
it, the stack, the member, the member's layer (its z, then its place among its siblings, the order Qt draws them in),
whether the member holds the focus and whether the item is the member itself (`Stacking`), and for words the four
corners of their drawn glyphs (`inked`), since a card's title fills its card while its glyphs sit at its foot. The contrast
rule samples what lies behind turned words over those same corners, so a title on a tilted card is read against the
card, not the sky beside it (measured 2026-11-09: Orrery's tilted card titles had read 1.2:1 against the day paper). Members of one stack may lie
over each other; two items of different members still fault when the lower one is words whose glyphs the upper one
covers (a caption may not be hidden)
or is a focused member (its ring may not be covered), and anything of a stack against anything outside it is held as
before. `EmuSen.Theme` does not export `StackMember`, so only the kit's own types declare a stack: today `OrbitSlot`, a
`Walkable` that places a cover of a list on an ellipse about the list's centre by its distance from `turning`, tilted
by the normal, scaled by distance and lifted 12 when lit (Orrery's Home and Games, `qt-s3/final/orrery.md` §1.2), its
covers stacked by nearness to the apex. `OrbitSlot` is re-exported by name (rule Q, additions only).

**Held to** (measured 2026-11-09): `kit-core/tests/stack.rs`, two bars turned 45° whose scene boxes cross by over 100
units but whose shapes lie 10 apart pass, and 5 of 20 shared is a fault; three covers over each other in their order
pass, and so does a caption whose box reaches under the apex while its glyphs clear it; a stack's cover over a label outside it, a caption under its neighbour, a focused member under another and two
stacks' covers over each other are each caught. `qt-test/tests/stack.rs`: the harness records a turned Text's corners,
two turned lines whose boxes cross are not an overlap, and a `StackMember`'s item and its words carry one stack. Seeded
faults, each reverted: captions allowed under a neighbour (the stack test), stacks not told apart (the stack test),
turned items measured by their boxes (the turned test).

### 7.34 A body and a glyph cut at a clipping edge on the software backend (2026-11-08, remade 2026-11-09)

**The defect.** On the software backend a Qt Quick `Shape` is painted past a clipping ancestor, while a `Text` beside it is
cut (first seen in §7.10, where the carousel was made not to depend on the clip; seen again in Meridian's recordings, a rail
row cut by its edge showing its world above the rail). Proved on 2026-11-08 by `qt-test/tests/clip.rs` over
`ClipScene.qml`: four rows of a `Body`, a `PadGlyph` and words in a `Flickable` 160 high that clips, grabbed at rest,
scrolled by 32 and back, sampling every 4 pixels above and below the Flickable. On the unchanged kit the body was drawn
past the edges at 123 of 462 points at rest and 224 scrolled, the glyph at 73 of 396 and 133; the words at none. A row
wholly outside the Flickable was drawn whole, and a row the scroll had moved was drawn at both places.

**The cause** (found 2026-11-09). A Shape on the software backend is a `QSGRenderNode`, painted by Qt's
`QQuickShapeSoftwareRenderNode`, whose `render()` sets the painter's clip only when the clip region it is given is not
empty; and the renderer gives a render node its dirty region, cut to its clip, as that region
(`qquickshapesoftwarerenderer.cpp`, `qsgsoftwarerenderablenode.cpp` of Qt 6.11, read 2026-11-09). A Shape whose bounds lie
wholly past its clip has an empty region, so it is painted with no clip at all, whole. A probe of the same day: a circle
of 80 in a clipping item of 60, 20 inside it, was cut (no red past the clip); at 70, wholly past it, 4,872 pixels of it
were drawn below. One partly inside is cut correctly. The node's bounds are its paths' bounds, widened by the stroke or a
unit, not its item's: a circle centred on its item's top edge, the item wholly past the clip, was cut correctly.

**The fix.** On the software backend alone, each Shape of `Body` and `PadGlyph` is not drawn while its paths' bounds,
`Shape.boundingRect` (which holds the stroke's ink), meet no clipping ancestor's area: the kit's `InsideClips`, an item
the size of those bounds inside the Shape, tells it so (`inside`), measured again whenever any ancestor moves, scrolls or
scales, which `QQuickItem::ItemObservesViewport` makes Qt report as `ItemTransformHasChanged`. The bounds lie within the
node's own, so where they meet the clip the node's region is not empty and Qt cuts it; where they do not, nothing of the
Shape's ink lies inside the clip. C++ glue, since QML hears of no ancestor's move (`EmuSen_Qt.md` §1.4). The GPU is
unchanged.

**The layer it replaces, and why it went.** From 2026-11-08 each was drawn through a layer on the software backend
instead, which is clipped, but which is not the same picture: the layer's pixmap is rasterised on its own pixel grid
and drawn at the item's place, snapped to the window's pixels, and it is sized to the item's size rounded, then scaled
to it. Measured 2026-11-09 with a circle of 46 drawn plainly and through a layer: at a whole-pixel place and size 1
pixel differed, by 3 levels; at a place of (100.37, 60.25) 42 pixels, by up to 130 levels; at a size of 46.3, 79
pixels, by up to 235; `layer.smooth` changed nothing. Moonrise's bodies stand at such places and sizes, and
`qt-app/tests/splash.rs` read them 202 pixels off Mistress's splash, where with the layer off they pass; the landing
check had not run that test. So the layer was wrong, not the reference; the reference stands.

**What it is held to.** `qt-test/tests/clip.rs` over `ClipScene.qml`: four rows of a `Body`, a `PadGlyph` and words in a
`Flickable` 160 high that clips, grabbed at rest, scrolled by 32 and back, sampling every 4 pixels above and below it.
Before either fix the body was drawn past the edges at 123 of 462 points at rest and 224 scrolled, the glyph at 73 of
396 and 133, the words at none; after, none, and the first row's ring past its body's box is drawn. And
`qt-test/tests/phase.rs` over `PhaseScene.qml`: a ringed body with two moons at (30.37, 20.25), 46.3 square, and a glyph
at a fractional place too, uncut inside a clipping `Flickable`, are the same picture pixel for pixel as the same under no
clip, and the body as its shapes drawn as plain Shapes by Qt with nothing of the kit's around them. Seeded faults,
2026-11-09, each reverted: the layer back on `Body` on the software backend, caught by the phase test (1,671 of 5,328
pixels off the plain shapes); `InsideClips` always inside, caught by the clip test (the body past the edges at 123
points at rest). The grabs are `clip-<state>-<backend>.png` and `phase-software.png`, written to `EMUSEN_QT_COMPARE`.

### 7.35 A fade to transparency on every backend, and which backends draw each effect (2026-11-08, remade 2026-11-09)

**`EdgeFade`** is the kit's one fade, re-exported by `EmuSen.Theme` by name. A theme places the scroller inside it, and it
fades what it holds to transparency over `length` (7 base units unless set) at each edge it is told to fade
(`fadeTop` … `fadeRight`, bound to the scroller's own declarations, which the audit reads): each pixel keeps the share
`clamp(d / length, 0, 1)` of its colour and alpha, `d` its distance from the faded edge, the shares of two faded edges
multiplied at a corner, so what lies behind shows through, a picture as well as a fill. Its first form (2026-11-08)
ramped a colour the theme named over the scroller: exact over one flat colour and wrong over anything else; the theme API
changed before release (`CHANGELOG-theme-api.md`). One API, the backend chosen inside the kit, and two ways of drawing.

**On the GPU, a shader of the kit's own.** `EdgeFade` draws what it holds into a layer and the layer through a
`ShaderEffect` over `shaders/edge_fade.frag`, baked by Qt's `qsb` in the kit's `build.rs` into
`qrc:/qt/qml/EmuSen/Kit/shaders/edge_fade.frag.qsb`, as `esde-qt` bakes its picture's shader (`EmuSen_Qt.md` §4.8). The
shader multiplies the layer by the four ramps, given as fractions of the item's size. `MultiEffect`'s mask was measured
first, since its documentation promises soft masks (doc.qt.io/qt-6/qml-qtquick-effects-multieffect.html, read 2026-11-09:
"The alpha channel of the source item is used for masking"; `maskSpreadAtMin` "softens the transition from the
transparent mask pixels towards opaque mask pixels by adding interpolated values between them"). A probe on the RX 6800 on
2026-11-09, Vulkan and OpenGL in a `kwin_wayland --virtual` of its own, put a red block over stripes through seven forms
of it: a ramp item behind `layer.enabled`, a gradient `Rectangle` as Horizon's strip had it, a `ShaderEffectSource` of
the ramp, and a `MultiEffect` item of its own, with the thresholds and spreads at their defaults, at 1, and at both ends.
None masked anything: the red stayed whole to the edge, where the mask's alpha was 0.05. The kit's shader in the same
probe drew (255, 35, 35) at 86% of red over white, the value the software path gives. Qt's own effect is used where it
works; here it does not, so the kit carries 20 lines of GLSL.

**On the software backend, a painter inside the layer.** Qt's software adaptation renders no shader effect ("ShaderEffect
components in QtQuick 2 cannot be rendered by the Software adaptation", doc.qt.io/qt-6/qtquick-visualcanvas-adaptations-software.html,
read 2026-11-08), but it renders a layer and a `QSGRenderNode`, whose `render()` paints with the renderer's `QPainter`
(doc.qt.io/qt-6/qsgrendernode.html, read 2026-11-09: "The software backend exposes its QPainter and saves and restores
before and after invoking render()"). So the layer holds, last over what it fades, the kit's `EdgeMask`: a render node
that fills each faded edge's band with a `QLinearGradient` from no alpha at the edge to whole alpha `length` in, under
`QPainter::CompositionMode_DestinationIn`, which multiplies what the layer holds by the gradient's alpha. It is drawn
only on the software backend and only while an edge fades. C++ glue, since cxx-qt declares no render node
(`EmuSen_Qt.md` §1.4).

**A Qt defect the software path steps around.** A software layer whose content holds any opaque node (an opaque
`Rectangle`, an image without alpha) is drawn into a pixmap with no alpha channel, so what it leaves transparent comes out
black. Qt 6.11's renderer asks whether its opaque nodes cover the background with `QRegion::contains(const QRect &)`,
which is true when the region merely overlaps the rectangle (measured 2026-11-09: a region of 320 × 10 "contains"
320 × 160), so any opaque node makes the layer opaque, and the pixmap (`QPixmap(size)`, the screen's format, `RGB32`
offscreen) is filled with transparency, the only thing that gives it an alpha channel, only when the layer is not opaque
(`qsgabstractsoftwarerenderer.cpp`, `qsgsoftwarepixmaprenderer.cpp`, `qsgsoftwarelayer.cpp`, read 2026-11-09). Measured
with a probe the same day: a layer holding an opaque rectangle 10 high of 160 drew black where it held nothing, and one
holding only translucent content drew the stripes behind. `layer.format` cannot help: the software layer's `setFormat`
does nothing. A node is not opaque when its opacity is under 1, and a pixmap keeps its alpha channel for as long as the
layer keeps its size; so `EdgeFade` holds what it fades at opacity 0.999 until `EdgeMask` reports (`primed`) that the
pixmap it paints on has an alpha channel, and at 1 after. `primed` falls back to false at once, before the frame is drawn,
when the mask's size, visibility, window or pixel ratio changes, each of which makes the layer anew. The cost is the
first frame of a new layer drawn at 0.999: at most one level in 255 from the settled frame (measured 2026-11-09 over
gradients, words and a picture: 43,760 pixels off by one, none by more), never black. Two nearer forms were tried and
dropped: filling the pixmap with transparency after the frame left one frame black at every first show and resize; a
render node placed before the content to do the same within the frame is culled by the renderer wherever opaque content
covers it. A layer inside what `EdgeFade` holds is not primed by it.

**What the audit makes of a fade.** Words crossing a scroller's faded edge are fading out there, so the contrast rule
does not hold them, as the edge rule accepts what a faded edge cuts (§3; `kit-core` `fit::fading_out`). Found by Horizon's
audit on 2026-11-09: the titles of the generated cards past the strip's right edge, faded toward the land, read 1.75 to
2.12:1 where nothing faded them before.

| Effect | GPU | Software backend |
|---|---|---|
| `MultiEffect` (shadow, blur, colourize) | drawn | nothing (no shader effects) |
| `MultiEffect`'s mask | nothing (measured above) | nothing |
| `EdgeFade` | the kit's shader over a layer | `EdgeMask` in a layer |
| `RoundPicture` | a texture-filled rounded `Shape` | the `Image` with plain corners (§7.29) |
| `Body`, `PadGlyph` | Shapes | Shapes, each not drawn while wholly past its clip (§7.34) |
| The notice's shadow (§7.23) | drawn | none |

**What it is held to.** `qt-test/tests/fade.rs` over `FadeScene.qml`: rows of the accent with gaps, in a clipping
`Flickable` inside an `EdgeFade` fading its foot and its right side by 40, over vertical stripes, light and dark, 5
pixels each. Down a light column and a dark one, each of the last 28 pixels is the row's colour and the stripe's own
mixed by the ramp, `row × k + stripe × (1 − k)` with `k = (200 − y − 0.5) / 40`, within 6 levels; its distance from the
stripe never grows toward the edge, no two neighbours differ by more than 12 levels, and the last pixel is the stripe's
within 6; three points of the right side's ramp the same; a gap between rows shows the stripes within 2, and a row above
the fade its own colour within 2. Then the first frame drawn after the layer is made anew, the fade turned off and on and
then the fade resized, each read from one grab with no frame between (`Scene::set_property_now`): the gaps the stripes
and a row its colour, within 2. Software by default; with `EMUSEN_QT_GPU` it draws with Vulkan in a compositor of the
test's own (`kwin_wayland --virtual`, as `theme-audit --gpu`). It passes on both (2026-11-09, the RX 6800), and the two
backends' grabs are the same picture (`magick compare -fuzz 2%`: 0 of 120,000 pixels differ).

Seeded faults, measured 2026-11-09, each reverted, each failing the test: on the software backend a hard mask (the
ramp a step at its middle), a fade to a colour (the ramp painted over rather than multiplied), the software path off
(`EdgeMask` never shown), the content never held under whole (the layer black between the rows), and the prime kept
through a resize (the resized first frame black between the rows); on the GPU a hard mask (`step`) and a fade to a colour
(`mix` toward the backdrop's colour). The audit's new exception: `kit-core`'s
`kit_7_35_words_crossing_a_faded_edge_are_fading_out_and_not_held_to_contrast`, and the same words across an edge that
does not fade, or inside the strip, still held.

**The layer goes off only once the event queue is clear** (found 2026-11-09). `theme-audit --gpu` over Meridian
faulted twice in `QQuickItem::geometryChange` under `QQuickWindow::resizeEvent`, at the resize to 3840 × 2160, where an
`EdgeFade` in the hidden Recent band had its `fadeRight` fall as the window grew. The cause is a use after free inside
Qt. A layer's `ShaderEffectSource` listens to its item's geometry (`QQuickShaderEffectSource::setSourceItem` adds it as a
`Geometry` change listener), and `QQuickItemLayer::deactivate` deletes it at once when `layer.enabled` goes false
(`delete m_effectSource`, the effect item the same). `QQuickItem::geometryChange` tells its listeners from a copy of
their list (`notifyChangeListeners`, "intentional copy (QTBUG-54732)"), so a listener deleted while the list is being told
is still called (`qquickitem.cpp`, `qquickitem_p.h`, `qquickshadereffectsource.cpp`, Qt 6.11.2, read 2026-11-09). An edge
bound to the scroller's own size falls exactly there: the fade's new size reaches `held` and the scroller through their
anchors, which come first in the fade's list, the scroller's width notifies, the edge's binding falls, `fading` with it,
and the layer is deleted; the fade's list then reaches the dead source. Backtraced on 2026-11-09 on both backends: the
deactivation under `QQuickFlickable::geometryChange` under the fade's `fillChanged`, and the fault in the fade's own
`geometryChange` three anchors below `resizeEvent`, as the audit's. Hidden or shown makes no difference; whether the freed
memory is reused by then does, which is why it faulted in the audit and passed in small scenes. So `EdgeFade` turns its
layer on at once and off through a `Binding` with `delayed: true` (Qt's own way to write a value once the event queue is
clear): `layer.enabled` is `fading || held.layered`. Until then the layer draws with no edge fading, the same picture. A
theme needs no gating of its edges on visibility. The warnings Horizon's survey logged on 2026-11-09 at placing (`Cannot
read property 'x' of null`, lines 17 to 21) are not this: they came from the first form's `x: root.source.x`, built at
4dc9678bd under Horizon's files for the second form, which sets no `source`; the second form reads no such property.

`qt-test/tests/fade_resize.rs` over `FadeResizeScene.qml` holds it: an `EdgeFade` in a hidden band, its right edge bound to
its scroller's `contentWidth > width` (2000), the window grown to 3840 × 2160 and back twice, then the same shown, and
nothing logged. The test sets glibc's `M_PERTURB` (`mallopt`), so freed memory is overwritten and a call into it faults on
every run; without it the unfixed kit passed on both backends. Measured 2026-11-09: the unfixed kit faults in that test on
the software backend and on Vulkan (the RX 6800); the fixed one passes on both, three runs on software, two on Vulkan, and
`fade.rs` still passes on both. Seeded faults, each reverted, each a fault in the test: `layer.enabled: root.fading` as
before, and the `Binding` not delayed.

### 7.36 A held clock, a held press and drawn covers in the harness (stage 3, theme-audit's states; 2026-11-09)

theme-audit records a theme's splash at a stated time and a screen with A held (`man theme-check`, STATES), so the harness
(`emusen-qt-test`) gained three verbs, each the framework's own mechanism.

- **`Scene::hold_clock(Some(ms))`** replaces Qt's animation driver with one whose `elapsed()` is a number the harness sets
  (`QAnimationDriver`, the documented way to drive Qt's animation timer; `cpp/harness.cpp`'s `HeldClock`). The first hold
  installs it at 0; each later one moves it to `ms` 16 ms at a time, calling `advance()` and processing events at each step,
  as a display's frames would, so a `Behavior` or a `FrameAnimation` that a step starts runs from its own time rather than
  being jumped over. `None` uninstalls it, which leaves Qt's default timer driver in place, not a threaded render loop's own:
  a caller holds the clock last. Every `QAbstractAnimation` and `FrameAnimation` is held; a `Timer` is not. Under a held clock
  nothing settles, so `rest()` must not be called (a running animation never ends); the audit waits a frame instead.
  The clock is the `Harness`'s, a field holding the C++ object (`harness_clock_hold` gives it, `harness_clock_move` moves it,
  dropping it uninstalls it), not a static of `harness.cpp` (changed 2026-11-09: the house rules forbid a mutable static used
  as a wire). The harness is the owner because the driver Qt replaces is its application's (`QUnifiedTimer` is the
  thread's), and one harness is one application; the field is declared before the application, so it is let go first. A
  second hold moves the one clock rather than installing another: seeded, a fresh clock at every hold failed
  `qt_3_7_a_splash_is_held_at_1_3_and_5_s_and_judged_at_each` on Qt's own words, "animation driver already installed" and
  "trying to uninstall a driver that is not installed".
  The same two lines came from Horizon's GPU run on 2026-11-09, once a hold, from another cause: a threaded render loop
  (Qt Quick's on Vulkan, or the software one under `QSG_RENDER_LOOP=threaded`) installs its own animation driver, so Qt
  refused the held clock (the first line), every timed state was read on the render loop's clock instead, and letting the
  clock go uninstalled a driver Qt had never taken (the second); offscreen under a threaded loop the run then crashed.
  theme-audit, the program that holds the clock, now sets `QSG_RENDER_LOOP=basic` before Qt starts, whatever the
  environment asks, so the held clock is the one driver installed, once a hold, and let go once. The harness itself sets no
  render loop: its other users (the program's Vulkan acceptance test) run on Qt's choice. Held to by theme-audit's
  `qt_3_7_a_held_clock_logs_nothing_where_the_environment_asks_for_a_threaded_render_loop`, the splash fixture's three
  timed states under `QSG_RENDER_LOOP=threaded`: their findings alone, no warning line and nothing on standard error. Seeded,
  the setting taken out failed it on both lines.
- **`Scene::key_hold(key, down)`**, `QTest::keyPress` or `keyRelease` alone: a press held, as a theme's pressed look reads it
  (`Keys.onPressed` without the release).
- **`covers::draw`**, moved from the program's tests (`qt-app/tests/common`), which re-export it: covers drawn by one Qt Quick
  window resized to each box and grabbed, a flat colour, a white rim and the words. theme-audit draws the synthetic library's
  covers with it, so the two keep one drawing.

Proof, measured 2026-11-09: theme-audit's `state-splash` fixture, a wordmark past the right edge from 2.5 s to 3.5 s, is
found at 3 s alone and its pictures at 1 s and 5 s are equal and differ from 3 s's (`qt_3_7_a_splash_is_held_at_1_3_and_5_s_and_judged_at_each`);
`state-pressed`'s row grown by a held Return overlaps its neighbour in the pressed state alone.

### 7.37 A on a walkable: the element's own handler hears it first (2026-11-09)

A `Walkable` answered Return and Enter with `Keys.onReturnPressed` and `Keys.onEnterPressed`. Qt's `Keys` treats a key
with a handler of its own as accepted before the handler runs, and emits the general `pressed` only for a key still not
accepted after it (Qt's `QQuickKeysAttached::keyPressed`), so a theme's `Keys.onPressed` written on a `Walkable` itself
never heard A; one written on a focused child did. Found by theme-audit's `pressed` state on 2026-11-09: Meridian's rail
entry widens its bar to 6 while A is held (`RailEntry.qml`, `Keys.onPressed` on the entry, which passes the key on) and
never did.

The walkable's Return handler now emits the element's own `pressed` first, with the key not accepted, so every
`Keys.onPressed` on the element hears A; if none of them takes it (accepts it), the walkable activates. Either way the
key ends accepted, so A never reaches the items around the element: the frame answers a Return that reaches it as the
pad's A, which on the big screen opens the content's detail (kit-core's walk), and Meridian's handler, written to let
"the press still reach Walkable's own handler", sets the key not accepted. Enter is sent through the same handler. A
key the walkable does not act on is not accepted and goes on as before. The handler is three statements of QML, not
logic moved into a model: it is the order of two of Qt's own signals.

Held to by `qt-test/tests/walkable.rs` over `WalkableKeysScene.qml`, with Return, Enter and the pad's A: a plain
walkable activates once a press; one whose `Keys.onPressed` passes A on hears it and activates once; one whose handler
takes A does not activate; none lets A reach the item around them, which still hears a key no one acts on. Two seeded
faults, each reverted: the old handlers failed it (the passing handler heard nothing); the key left as the theme's
handler left it failed it (A reached the item around, twice).

### 7.38 A declined place, and words mid-fade, under the contrast rule (theme-audit; 2026-11-09)

**A declined place.** The harness has exempted a disabled control's words since §7.10 (`enabled` false), as WCAG 2.2's
1.4.3 exempts "text or images of text that are part of an inactive user interface component" (https://www.w3.org/TR/WCAG22/,
read 2026-11-09). A place the host declines is not disabled in Qt's sense: the pad still walks to it, so a theme draws it
dim (every theme at 38%) with `enabled` true, and theme-audit's `disabled` state held its words to 4.5:1 (S6's survey:
Meridian, Constellation and Orrery 20 lines each, the Scraper's name and line at 1.8 to 3.2:1). Decided 2026-11-09: such a
place is exempt, and nothing else is. theme-audit finds the declined places by the name a reader hears (`place_name`, the
state's keys), and exempts the words under each only when it is drawn apart from its offered look: the place's box is read
on a grid (`grid`, about 10,000 points, a design unit apart or less), declined and then offered through the theme's own
surface (`Library.show_declined`, which changes the rows in place, so the delegates and the focus on the place stay), and
some point must change by 3:1 (kit-core's `STATE_CHANGE`, `changed_share`), the ratio WCAG 2.2's 2.4.13 asks "between
the same pixels in the focused and unfocused states". A place drawn as if offered keeps the rule and is a finding of its
own. 2.4.13's area, "a 2 CSS pixel thick perimeter of the unfocused component", is not asked, and why: words dimmed to 38%
change about 2% of a 480 × 56 row, where the ring is 7.9%, so the area would name every theme's dim look as no look; it is
written for a focus indicator, which must be found, not for a state that is read on the place itself. Any changed point
is weak against a design that changes one dot; the grid's spacing and the two pictures differing in nothing else (the
same focus, the views at rest) keep stray pixels out.

**Words mid-fade.** A held clock (§7.36) reads the splash at 1, 3 and 5 s, and S6 found every theme's caption at 5 s
(2.05 to 2.30:1) mid-fade. Contrast is judged at rest, and the harness can read honestly whether a text is moving, not
where it will rest: a `Behavior`'s target value is Qt's private, and a theme's own time property drives the rest. So a
text is skipped while its opacity changes: the clock is held a frame (16 ms) before the stated time, the tree read, then
held at the time and read again, and a text whose ink, its colour with its opacity, differs between the two is not judged
in that state (`changing`, `rested` in theme-audit). The same text at rest at any other time is judged.

Held to by theme-audit's tests over two fixtures, each with its seeded faults reverted. `state-declined`: Home's places
as rows, the declined row's words at 38% and the last row's (Power, offered) at 38%; in the default and the `disabled`
state, at every size and variant, Power alone is found. The declined picture compared with itself (nothing changes)
failed it (the Scraper drawn as if offered and found low); every text exempted failed it (Power no longer found).
`state-fade`: a caption fading to rest at 0.3 between 0.5 and 1.5 s, a title fading in to full between 4.5 and 5.5 s;
only the caption, at 3 s, at 5 s and in the default, is found. No text skipped failed it (the caption found at 1 s); every
text skipped failed it (the caption not found at 3 s).

### 7.39 A walkable focused inside a view becomes its current item; the pointer focuses by moving (2026-11-09)

A `ListView` or `GridView` moves its keys from its `currentIndex`, not from the item holding the focus. The pointer
focuses a walkable directly (§7.3, `Walkable`'s `HoverHandler`), and so does theme-audit's walk, so a cell hovered in a
theme's grid held the focus while the view's current item stayed where it was, and Right went on from that one. Found by
theme-audit's `many` state on 2026-11-09 in every theme (`EmuSen_Qt.md` §3.7.2): each grid's cells unreached past the first
row's first, every cell's Right landing on cell 1 and Down on cell 5. The kit's `MenuPanel` had made its row current on
focus since §7.8, and Orrery's orbit cover did the same for itself; the themes' grids did not.

**The walkable makes itself current.** Decided 2026-11-09: the kit owns this once, an extension of `Walkable`, so every
theme's grid and strip has it without a line of its own. When a walkable takes the active focus it calls the singleton
`ViewCurrent.take(item)` (`current.rs`), which looks up the item's parents for the first standing directly in an item
view's `contentItem` (the delegate: the walkable itself, or an item it stands in, as Horizon's cover stands in its slot)
and sets the view's `currentIndex` to `indexAt` that delegate's centre. A view is an item view when Qt's own class says so
(`QObject::inherits("QQuickItemView")`, the base of `ListView` and `GridView`), so a `PathView` or a `Flickable` with a
`currentIndex` is never moved; the kit's `Carousel` is itself the walkable. A walkable in no item view, or over no delegate
(a header), changes nothing, and nor does the view's own current item: a view gives the focus to the delegate it makes
current before placing it, so its box says nothing yet (a strip that made its eighth item at x 0 read it as item 0, and
Right from item 6 landed on item 0). What the view does with its current item follows unchanged: a `highlightRangeMode`
turns the strip or the orbit to it, as Orrery's line already chose. `MenuPanel`'s own line is removed. *C++ part*: one
function in `cpp/kit_glue.cpp`, `make_current` [own, because an item's parents, `inherits` and `QMetaObject::invokeMethod`
for `indexAt` have no Rust binding in cxx-qt-lib, and the kit's QML holds no functions (`qml-rules`)].

**The pointer focuses by moving.** A view that turns to its current item moves other items under a pointer that stays
still, and Qt delivers a hover to each item that comes under it. With the walkable making itself current, the strip
then ran on: hovering a strip's fifth item turned the strip, the seventh came under the pointer and took the focus, the
strip turned again, and it stopped at its end (measured on 2026-11-09 in `GridScene.qml`: the focus at item 7 of 8 within
300 ms). The same hover moved the focus off the item a key had chosen whenever a key turned the strip under a resting
pointer. `FrameController` now keeps where the pointer was last seen over a walkable (`hoverAt(item, hovered, at)`, beside the unguarded `hover(item, hovered)` it keeps, and
`pointerMoved(at)`, the scene position from the `HoverHandler`'s point; Qt sets the point before it reports `hovered`),
and an element coming under the pointer at that same place takes no focus. A pointer that moves gives the focus as before. §7.108
replaces the controller's saved position with a fact of the window.

**The walk settles a turning view.** theme-audit's walk focuses each element and puts back what the press moved, then
waits for the views to rest (`Scene::settle`). Qt's `moving` reports a flick or a drag, not a view turning to its current
item, so the walk read Horizon's strip mid-turn after the put-back, lost its first cover (found by its box, which had not
yet come back) and recorded no press from it: every cover unreached at most sizes. `harness_turning` [own, in
`qt-test/cpp/harness.cpp`, for the same reason] reads a view as turning while its highlight follows its current item and
stands off it, until three frames pass without the highlight moving (a theme may place its delegates itself, as Orrery's
orbit does, where the highlight rests off them); the settle waits on it.

Held to by `qt-test/tests/grid.rs` over `GridScene.qml`: a grid of ten walkable cells, five a row, and a strip of thirty
that turns its current item into a band, whose walkables stand inside each delegate's `FocusScope`. The pointer rests on
a cell or an item and the pad presses: Right from cell 7 lands on 8 (`currentIndex` 8), Down from cell 3 on 8, Left from
cell 9 on 8; Right from the strip's item 4 on 5, Left from item 3 on 2 after the strip turned; each hovered item keeps
the focus while the strip turns under the pointer; Right six times from item 2 lands on item 8, through items the strip
made. Then the walk over the same scene: every strip item's Right lands on the next, and the last read's on none; the walk
puts the strip back; a settle after focusing item 5 leaves the strip at rest. Five seeded faults, each reverted and each
failing it: the walkable's line removed (Right from cell 7 on cell 1); the view's own current item not excepted (Right six
times landed on item 1); the walkable's own delegate alone looked at (Right from item 4 on item 1); the pointer's place
not kept (hovering item 4 left the focus on item 8); the settle not waiting on a turn (contentX 121 when settled, 470 half
a second later). theme-audit's `many` over the five themes on 2026-11-09 (the reference package and Meridian from this
tree, Horizon, Constellation and Orrery from their branches' copies kept for the survey): no reach finding left, where
each had 70 or 80; with the walkable's line removed, Meridian's 70 came back.

### 7.40 The settings frame (stage 5, piece A; 2026-11-09)

Screen 6 of `EmuSen_Interface.md` §4, as LunaP's `SettingsLayout` drew it for the C# Settings (`EmuSen_BigPicture.md`
§52, LunaP §213): the strip of sections at the top, rows of a label and a value down the left, and a pane at the right
with one slot for a preview over the focused row's sentence. Weighed as `EmuSen_Qt.md` §1.3 asks, it is an extension of
the kit, a new type over the existing `Frame`: no existing type takes a second job, and the frame knows nothing of
settings. A host hands it rows as a list model and is told what the player did; which setting a row is, and what a step
or a choice means, stay the host's (`SettingsModel` of stage 5's piece D).

- **The measures are kit-core's** (`settings::frame`, `SETTINGS`), derived from the big screen's spacing where the two
  agree: the strip at the safe area's top (64, 40), as high as a control (44), names 8 apart with 16 of padding; the rows
  and the pane from 108 (the strip, then the pane gutter of 24); the pane 416 × 578 with an inset of 24, its slot 368
  wide, 12 below the slot and 8 between lines; the pane 64 from the right edge and the rows filling what is left before
  a gap of 32, so 704 wide at 1280 and 846.2 at 1422.2 (1920 × 1080 and 1280 × 720 on the canvas of 800); ten whole
  rows of 54 beside the pane. These are LunaP's constants and Mistress's recorded boxes (`SettingsSound-1280x800.spec`,
  `SettingsDisplay-1280x800.spec`), held to them by `tests/settings.rs`.
- **The walk is kit-core's** (`settings::rows`, `RowWalk`): Up and Down move the focus and stop at the ends; the window
  of ten moves only when the focus leaves it, a row at a time (LunaP's `KeepRowShown`); the wheel moves the window to a
  whole row and leaves the focus; Left and Right step a value (`Steps`) or a slider (§7.42) and are kept on any other
  row; A chooses a row, which is the host's to act on, open or ask about; a row a host marks `held` (a value the engine
  holds, `EmuSen_BigPicture.md` §52.5 item 2) takes neither; the shoulders step the strip's sections, wrapping, while no
  page is open under the section, and are declined, so the host's, under a trail (the console page's LB and RB, §52.4);
  B, Start and the rest are declined to the frame. A row's kind is `Kit.RowKinds` (`Action`, `Steps`, `Opens`, `Asks`,
  `Slider`, `Choice`), a number in the model's `kind` role.
- **`SettingsLayout`** (kit-qt `settings.rs`) puts the measures and the walk on Qt: the measures as constant
  properties, `paneLeft` and `rowsWidth` from the canvas's `width`, the walk's `current`, `first`, `marked`, `count` and
  the focused row's `sentence`, `sections`, `section` and `trailShown` from the frame; it reads the roles `kind`, `held`,
  `marked`, `sentence`, `minimum`, `maximum`, `step` and `amount` from any list model through `item_model::role_texts`,
  and raises `stepped`, `slid`, `chosen` and `sectionChosen`.
- **`SettingsFrame.qml`** is a `Frame` with its navigation hidden, so the frame's controller, router, keys, pointer and
  help bar are the ones every screen has. The strip is Qt Quick Controls' `TabButton`s in the style (§7.8), each as wide
  as its upper-cased name in Caption with its padding; it draws the `trail` instead while pages are open, the last
  chosen. The rows are a `ListView` with a `DelegateChooser` on `kind`: a `MenuRow` with its ◂ ▸ or ▸ marks (§7.21), a
  `ChoiceRow` (§7.41) or a `SliderRow` (§7.42). A press the frame's walk declines reaches the focused row as a key and
  goes up to the rows' scope, which hands it to the walk; hovering a row focuses it, so the walk follows Qt's focus, and
  a click is A there (§7.3). The view's own current index is cleared, so it never moves the focus to a row of its own
  choosing as its model changes; the walk reads the model after the view has its rows, and again whenever the model
  says its rows changed. The pane is SurfaceRaised with the tile radius; its slot (`preview`) holds what the host puts
  there, a picture of 368 × 276, a meter of 16 or the two variants of 160 as LunaP drew them, and its `lines` slot what
  the host adds under the sentence.
- **Found and fixed on the way: a `MenuRow`'s marks past a clip.** On the software backend a `Shape` wholly past its clip
  is painted unclipped (§7.34), so the eleventh row of a list showed its ◂ ▸ marks 26 units below the window
  (2026-11-09, `settings-frame-Dark-1280x800.png`, the label clipped and the marks not). The marks are now held to their
  clip as `Body`'s shapes are, through `InsideClips`; no other drawing of a `MenuRow` changes.

**Where it differs from Mistress**, each named: the chosen name's underline is the style's 4 units (`barWidth`), where
`SectionTab` drew 3; the names are set in the style's face, since `Tokens` has no family yet (§7.7), so DISPLAY's tab is
about 111 units wide where Mistress's is 104; the help bar is the kit's, 58 on the foot (§4 of `EmuSen_Interface.md`),
where Mistress's band is 98 from 702; a held row's value is drawn as any other, and an `Asks` row draws no `…`, both
waiting for the host's words in piece D. The desktop form (a source list, rows of 48, search) is stage 6's, and the
card's pages are piece D's on this frame.

**Held to**: kit-core's `tests/settings.rs` (12: the measures at 1280 and at 1422.2, the ends, the window, Left and
Right by kind, a held row, the page of choices, a long page's mark in view, the declined presses, the kinds' numbers,
the shoulders, a slider's grid); kit-gallery's `settings_gate.rs` (the frame with a picture, a meter and the two
variants, the page of choices and the page of sliders, at 1280 × 800, 1920 × 1200, 1920 × 1080 and 1280 × 720, dark and
the variants light: fit, pad reach, pointer reach, no chrome, names, palette, contrast; the strip, rows and pane at
Mistress's boxes at both 16:10 sizes; no mark drawn below the window) and `settings_walk.rs` (the pad through the
router's queue and the pointer: the window by one at the eleventh row and the list's offset with it, a value stepped, a
page's row chosen, a held row neither chosen nor stepped, the pane's sentence, the shoulders wrapping, B to the host;
resting, clicking, the wheel to a whole row, a tab, a right click as B). The gallery's states are `SettingsGallery.qml`,
a scene of the gallery module beside `Gallery.qml`, with its models as a host would hand them. Rule Q: `SettingsLayout`
and `RowKinds` are additions to the registry and `SettingsFrame`, `ChoiceRow` and `SliderRow` to the `qmldir`; nothing
was removed or changed (`CHANGELOG-kit.md`). The harness gained `Scene::drag` (QTest's press, eight moves and release,
§7.9). Measured on 2026-11-09 with six seeded faults, each reverted and each caught: the window moving one row late
(kit-core's window test), A on a choice leaving the mark (the page of choices test), a slider's step taken from the floor
rather than the nearest step (the grid test), the rows' box one row too tall (the gate, Mistress's boxes), the `MenuRow`
fix undone (the gate, a mark below the window at 748,674), and a slider's pad step not raised (the walk test).

### 7.41 The page of choices (stage 5, piece A; 2026-11-09)

A page for one row whose value is one of a few, such as a console's Engine (`EmuSen_BigPicture.md` §52.4; the same
value as Cores' Make Default), drawn on the settings frame under the trail of the pages above it. Its rows are of kind
`Choice` and one of them carries the model's `marked`: the page opens with the focus on the choice in force, and in its
window on a long page; A, or a click, chooses the focused one, the mark moving to it at once and `chosen` raised for the
host, whose rows, read again, are the mark's owner; B is the frame's, so the host closes the page keeping what was
chosen (Mistress's line: "A chooses; B keeps what was chosen", a host's words in the `lines` slot). `ChoiceRow.qml` is a
`MenuRow` whose words stand 32 from its edges, the choice in force marked by a dot of 10 in Accent centred in that
inset, and heard as checked by a reader. Mistress marked it with a `●` before the value's words; the dot is a drawn
mark, as the style's other marks are (§7.8).

**Held to**: kit-core's page of choices and long page tests, the gallery's `choices` state through the gate, and the walk
test (opened on Venus (C#), the shoulders left to the host, A on VenusRT moving the one mark to the first row, a click on
Snes9x moving it again).

### 7.42 The slider row (stage 5, piece A; 2026-11-09)

A row with a slider, for a shader's levers and a crop's edges (`EmuSen_Interface.md` §16.2, `EmuSen_BigPicture.md` §52.5
item 1). Mistress stepped these as values and typed them under A; the slider row keeps both, Left and Right through the
walk and A for the host's prompt, and adds the pointer's drag.

- **The step is kit-core's** (`settings::slider`, `SliderRange`): Left and Right move the amount by the row's `step`,
  on the grid of steps counted from the range's `minimum` (an amount off the grid steps onto the nearest grid point
  first) and inside the range, a range given high to low taken low to high; a row with no step takes a hundredth of
  its range (`STEPS_WITHOUT_A_STEP`); at an end, or on a held row, the press is kept and nothing is raised. The walk
  keeps the amount it gave until the host's rows are read again, so two quick presses take two steps.
- **`SliderRow.qml`** is a `WholeRow`: the label over 40% of its width in Row, Qt Quick Controls' `Slider` in the style
  (§7.8) filling the middle, and the host's words for the value in Muted, 104 wide and right-aligned so the slider does
  not move as they change. The slider takes no focus of its own: the row holds it, and the pad's Left and Right are the
  walk's. The slider's `stepSize` is the row's step with `snapMode` always, so a drag snaps to the same grid; a drag
  raises `slid` with the amount, and the frame raises it with the row. A held row's slider is disabled, and neither the
  pad nor a drag moves it.

**Held to**: kit-core's slider tests (Left and Right without a step, the grid, the maximum held, an amount off the grid, a
held slider, a range given high to low, no number); the gallery's `slider` state through the gate, and the walk test (two
Rights and a Left by 0.5, a drag of the second row to its end giving 25 and its words `25.0%`, the held row left as it
was by a drag and a press, though the pointer focused it).

### 7.43 The tool screen (stage 5, piece B; 2026-11-09)

The tool screen is the big-screen form of `ManagerLayout` that `EmuSen_Interface.md` §15.2 specifies and §15.9 lists as
the kit's to gain: a header row with the tool's facts, a section strip with the search box at its right, a columned list
whose columns stay inside its width, the apply bar with its meter under the list, and the pane spanning the list and the
bar. It takes everything it shows from models and knows no tool; Cores, Packages, Library, the Scraper and the Cheats
database are each one source behind stage 5's `ToolModel` (pieces F, L and M).

**Where its places come from.** `emusen_kit_core::tool` places the parts from the screen's size in design units and the
big screen's spacing tokens (safe area, row height, help bar). Its measures are those of Mistress's recording
`Cores-1280x800.spec`, which is the oracle for these screens (`EmuSen_Qt.md` §7, stage 5): the header 48 at the safe
area's top; the strip 44 at 60 below it, the search 240 wide at its right; the list from 120 below the safe area's top,
0.6 of the width (`AdaptiveLayout.ContentShare`), in as many whole rows of 54 as leave room for the bar of 110 (an inset
of 18, a line of words, then the controls) above the content's foot; the pane 16 beside the list, from the list's top to
the bar's foot. The content's foot is the safe area's foot less a help bar's height, 702 at 800 high, where both C#
recordings end it. The frame's own band has stood flush on the foot, 742 to 800, since 2026-11-05 (§7.1 of
`EmuSen_Qt.md`), so 40 units lie between the bar and the band; moving the foot to the band would add a ninth row and
part from the recording, which is a question for the tester (§7.45). At 1280 × 800 the places are the recording's to the
unit: header 64,40 1152 × 48, strip 64,100 912 × 44, search 976,100 240 × 44, list 64,160 691 × 432 (eight rows), bar
64,592 691 × 110, pane 771,160 445 × 542. At 16:9 the design is 1,422.2 wide and the extra width goes to the list and the
pane in their shares, as §15.2 says.

**The pane spanning the list and the bar** is the layout's rule, not a screen's arrangement: the pane's top is the list's,
its foot the bar's, and the bar stands on the list's foot with the list's width, so the apply bar belongs to the list.
§15.9 item 2 is this.

**FitsWidth** (`emusen_kit_core::table`, LunaP §204's `LunaTable.FitsWidth`). A column takes its width one of two ways: a
content column the width of its widest words, rounded up to whole units; a fill column its least, then a share by weight
of what the content columns and every fill's least leave. Where the room cannot give the fills their least, the content
columns give up the difference in proportion to their widths; where even that cannot fit, the leasts are scaled into the
room. The widths never sum past the room, which is the table less its lead (the mark column, 24), the gaps (24) and the
trail (24), so the last column ends at or before the table's right less the trail and no table scrolls sideways. The
Cores recording's three columns come out to the unit: Core (fill) 297 at 88, Console 144 at 409, Status 154 at 577 in a
list of 691 from 64. A cell whose words are wider than its column is elided, which the fit audit counts as a fault, so a
host gives a fill column a least that holds its longest name (§15.2 measured `Mupen64Plus-Next` at 190) and says the rest
in the pane, as the C# Cores tool does.

**On Qt.** `ToolLayout` puts the places on Qt as rectangles a screen binds to, recomputed when the size changes.
`ColumnFit` reads a columns model (roles `key`, `fill`, `least`) and a rows model through `item_model::role_texts`, the one
reader of a model's role the kit's layouts share, measures each content column's words in the cells' own font, and hands
back each column's `xs` and `widths`; a screen calls `measure()` when its rows change (the screen's `Connections` on the
model's reset, insert, remove and data signals and the list's count do so). The measuring is hand-written C++
(`cpp/text_width.*`, `QFontMetricsF::horizontalAdvance`), since cxx-qt-lib binds `QFont` but no font metrics, and the
alternative, measuring in QML, would need a loop over the model, which `qml-rules` forbids. `Tones` is the colour a tool's
words take by what they say (`Text`, `Muted`, `Strong`, `Accent`, `Success`, `Attention`, `Error`), so a model hands a
tone and never a colour; `ToneText` draws words in their tone's token.

`ToolScreen.qml` is a `Frame` whose places are laid over the whole frame: its parts are declared in an item reparented to
the frame's root, so they stand in screen coordinates as the recording places them while staying inside the frame's focus
scope, where the frame's walk, `PadRouter` and help bar serve them. The frame's content area, offset by the safe area and
its pane gap, could not hold the header at 40 and the list at 160 without a change to `Frame`, and the frame's surface is
not this piece's to change. Its properties: `title`; `facts` (roles `words`, `tone`), right-aligned in Caption with a
drawn dot between them, since a separator in QML would be words in a view; `rows` and `columns` (the columns' roles also
`trailing` for words aligned right and `toneKey`, the row's role holding the cell's tone); `nameRole`, the role a reader
hears for a row; `searchText` and `searchHint`; `paneTitle`, `paneCaption` and `sentences` (roles `words`, `tone`) in a
scrolling pane with faded edges; `summary` and `summaryTone`, `progress` (below zero no job runs), `applyLabel`,
`applyEnabled`, `meter` and `meterWords`; the signals `activated(index)` (A or a click on a row) and `applied()`. The
strip is `SectionStrip`: the frame's own `sections` and `selectedSection`, so the shoulders step them, drawn as tabs in
Caption upper case, the chosen one strong over the accent bar, only whole names shown, a click choosing one. The apply
control is an `ActionButton` with the Start glyph (`Pad.Menu`), the shared action button of §7.44.

**The walk.** The list holds the focus; Up from its first row reaches the search, Down from its last row and Right from
any row reach the apply control, and Up or Left from it return to the list; Down from the search returns to the list. A
row is a `Walkable`, so a click is A there. The list scrolls under faded edges where it holds more rows than it shows,
and the pane where its sentences are longer than it.

**Compared with the recording** (`Cores-1280x800.spec`, read 2026-11-09), the differences, each named: the face is the
platform's sans serif until the screens' faces are bundled (§7.7), so words are a few units taller and the search box is
46 high at its own implicit height, centred on the strip's 44; the mark column is a gap with no box, as §15.3 Q4 asks of
an unmarked row, and a marked row says its mark in Accent in its status (the gallery's Snes9x); the apply control is
159 wide where Mistress's is 170, its words' own width; the strip's tabs are 44 tall with the bar under the chosen one,
as the recording draws them, but the gaps between them are the kit's TabButton's.

### 7.44 The page frame (stage 5, piece B; 2026-11-09)

The page frame is one screen for one thing, `EmuSen_Interface.md` §2.5 and §4 screen 3 (a game's page, its five sections
and their deeper pages, §18.2): a hero across the top under the OnArt gradient, the cover over the hero's foot, the title,
facts, actions and section strip beside the cover, and the content below as tiles or rows. It knows no game; piece G puts
`GamePageModel` behind it.

**Its places** (`emusen_kit_core::page`), from Mistress's `GamePage-1280x800.spec` and `GamePageStates-1280x800.spec`:
the hero 0,0 across the screen and 360 high; the cover at 64,180, 268 high and as wide as its shape gives (201 for 3:4),
never wider than 240, the column beside it less the pane gap; the column at 336 (the safe area's left plus 272) holding
the title at 226, the facts at 286, the actions 44 high at 328 and the strip 44 high at 396, each to the safe area's
right; the content from 64,464 to the content's foot of §7.43, 238 high, in which rows show four whole rows of 54. Tiles
are pictures of 200 × 150, 24 apart, their label at 162 below the picture's top and their line at 190. §4.3 gives the
cover as 240 × 300; the recording draws 201 × 268, and the recording is the oracle.

**On Qt.** `PageLayout` puts the places on Qt, with the tile's measures as constants. `PageFrame.qml` is a `Frame` laid
out as §7.43's tool screen is, for the same reason: the hero runs edge to edge, which no content area could hold. Its
properties: `hero` and `cover` (pictures), `coverAspect`, `coverName` (what a reader hears for the cover), `title`,
`facts`; `actionModel` (roles `label`, `value`, `button`, the pad button whose glyph leads the words or below zero for
none, and `filled`, the screen's main action in the accent); `tileModel` (roles `label`, `line`, `picture`) or
`rowModel` (roles `label`, `value`, `steps`, `opens`, drawn as the kit's `MenuRow`), chosen by `showsTiles`; the signals
`acted(index)`, `tileChosen(index)` and `rowChosen(index)`. The hero is a backdrop to the fit audit, as a sky is, since
the cover, the title and the actions stand over it by design; with no picture it is Void, so the OnArt words keep their
contrast in the light variant too. A focused tile is ringed outside its picture, so the tiles' list stands the ring's
room outside the content's box and keeps it as margins, and nothing is cut at its edges. `ActionButton.qml` is the action
as a button: the glyph, the word, a value in Muted, filled in the accent where it is the main action, ringed when
focused, a `Walkable`.

**The walk.** The actions hold the focus at first (Play, as §4.3 says); Left and Right step them; Down reaches the
content and Up from its first row or tile returns to the actions. The shoulders step the strip's sections through the
frame. Left and Right on a stepped action (`Resume ◂ Slot 3 ▸`) move between the actions here; the stepping and its
marks are piece G's, with the slots it steps.

**Compared with the recordings** (read 2026-11-09), named: the title and facts are in the platform's sans serif, 60 and
30 high where Barlow's are 53 and 27; the actions are the kit's action buttons, 16 inside their edges, where Mistress's
cells are 8 to 21; the hero has no picture in the gallery, so its gradient is drawn over Void. The cover had no shadow
until §5.4's raised shadow became a token (§7.55).

### 7.45 What the tool screen and the page frame are held to, and what is open (2026-11-09)

**kit-core** (`tests/table.rs` 7, `tests/tool.rs` 4, `tests/page.rs` 3): the Cores recording's cells, a fill reaching the
trail, fills sharing by weight after their least, wide words giving way to a fill's least, a room under the leasts, content
alone fitting and shrinking, and no width from 0 to 1,600 in steps of 7.5 putting a column past the table for five sets of
columns; the Cores recording's places at 1280 × 800, the 16:9 shares, the pane spanning the list and the bar with whole
rows inside the safe area above the help bar at every size from 960 to 1,800 wide and 400 to 1,000 high, and one row
fewer and none cut 40 units shorter; the game page recording's places, a wide cover stopping short of the column, and the
places inside the screen at 16:9.

**The gallery** (`kit-gallery/tests/screens.rs` over `qml/Screens.qml`, one test, 24 states in 16.8 s, measured
2026-11-09): the tool screen, the page with tiles and the page with rows, in both variants at the four fit sizes (1280 ×
800, 1920 × 1200, 1920 × 1080, 1280 × 720), each through the fit audit, pad reach, pointer reach, no chrome, names,
palette reach and contrast of §3, with the pane spanning the list and the bar and every cell inside the list less its
trail as drawn, the walk between the parts at 1280 × 800, A on a row and a click on Apply reaching the host, and nothing
logged by Qt; `qml-rules` and `qmllint` pass the scene, and `qt-test`'s lint test the kit's new files. Grabs and
recordings are written beside the gallery's in `~/.cache/emusen/tmp/kit-gallery/`.

**Seeded faults**, measured 2026-11-09, each reverted, each caught: the table's room taken without its trail failed six
of `table.rs`'s seven tests, the Cores recording's cells among them; the pane laid only beside the list failed all four of
`tool.rs`'s; the cover's width uncapped failed `page.rs`'s wide cover; in the gallery, `ToolScreen`'s pane bound to the
list's height failed the spanning check in every tool state, `PageFrame`'s actions without their way down to the content
failed pad reach for every row in the page's row states, and `ColumnFit` fitting a table two trails wider than its list
put the status words past their rows in every tool state (the fit audit's words past their owner).

**Rule Q.** The registry gains, additions only: `ColumnFit`, `PageLayout`, `ToolLayout` and the `Tones` namespace in
`EmuSen.Kit-1.0.qmltypes`; `SectionStrip`, `ToolScreen`, `PageFrame`, `ToneText` and `ActionButton` in the qmldir. The
module stays 1.0, as every kit addition before the first release has (§7.6). `EmuSen.Theme` re-exports none of them
(`EmuSen_Qt.md` §3.1: a theme never styles a tool or a page), so its registry and `theme-qt/tests/surface.rs` are
unchanged.

**Open.**
- **[decision]** The tool screen and the page end their content at 702, as the recordings do, 40 units above the band
  that has stood at 742 since 2026-11-05. Recommended: keep 702 while the stage 5 screens are compared with the C#
  recordings, and move the foot to the band (one more row on the tool screen) once stage 8 retires them, as one change
  in `tool::content_foot`.
- **[routine]** A stepped action's ◂ ▸ and its stepping (Resume's slots) are piece G's.
- The cover's raised shadow waited for a shadow token (§5.4); built 2026-11-09 (§7.55).

**Found by Cores, the first screen on it** (stage 5, piece F; 2026-11-09; `EmuSen_Qt.md` §7.7.F). Three defects the
gallery's still models could not show, each fixed in `ToolScreen.qml` with no change to the registry:
- The list and the pane faded their edges in the `EdgeFade` around them but did not say so where the audit reads it, on
  the scroller itself (as `MenuPanel` does), so a pane longer than its box failed the fit audit as cut with no fade
  (LRPS2's five sentences at 1280 × 800). Both scrollers now declare `fadeTop` and `fadeBottom`.
- A row's cells anchored to their parent, which is null while a model reset destroys the row: 111 warnings in one walk.
  The anchor reads `parent?.verticalCenter`, and the columns' reader and its connections take `null` until a host names
  its rows.
- The screen's words were the platform's sans serif with no way for a host to name its faces, so the header's facts in
  Mistress's longest case (no catalogue: 914 units in Barlow) took 1,066 and left the title 54 units for `CORES`.
  `titleFont`, `captionFont` and `bodyFont` join `cellFont`, the tokens' sizes in the platform's face by default.
- Not the kit's but its harness's: a row scrolled out of a clipping list by all but a rounding's sliver was read as
  shown, so the overlap rule counted it; `qt-test`'s tree takes less than half a pixel inside a clip as outside
  (`tests/culled.rs`).
- A tool's search types on the docked keyboard (`Keyboard`, §7.6), whose band covered the list's foot, the apply bar and
  the pane, words the gate counted as overlapped. `keysTop` (design units, below zero for none) is where a docked band
  begins: the list keeps its whole rows above it, and the bar and the pane are not drawn while one is docked. An
  addition to a QML file's properties; the registry is unchanged.

### 7.46 A long value on a `MenuRow` (stage 5, piece D; 2026-11-09)

Found by Settings' Library section in the program (`EmuSen_Qt.md` §7.7): a row's value was laid out at its own width,
so a folder's path of 70 characters (the test's scratch tree) took the whole row and squeezed the label to a negative
width, which the gate read as the label cut, past its box and past its clip. The value is now at most half the row and
cut at its start (`Text.ElideLeft`), so the label keeps the other half and the end of a path, the folder's own name,
shows, as a file manager's narrow column shows one. A value that fits is drawn as before; nothing of the surface
changed. The cut is still a finding of the gate's ("cut with an ellipsis"). Settings names it for a folder's value alone, on the
condition that focusing the row gives the whole path in the pane (`EmuSen_Qt.md` §7.7); any other value cut on a row
stays a finding. Held to by `qt-app/tests/settings.rs`; seeded, the cap taken out failed it again (the label cut and past its
box, the value past its row's clip).

### 7.47 The text sizes' scales (stage 5, piece D; 2026-11-09)

Decided 2026-11-09: the kit owns each text size's scale, since its style applies the text size to every screen.
kit-core's `TEXT_SIZES` holds the three sizes by the word the setting stores (`AppSettings.TextSize`) and the scale each
asks: `default` 1, `large` 1.15, `larger` 1.3 (`MAX_TEXT_SCALE`, the most the kit lays out; the figures are LunaP's
§214, the mockup's Larger of about 1.4 having been measured too large for the grid on 2026-11-03). `text_scale_of`
reads one, a word it does not know being 1. `Tokens` gained `textSize`, a word: setting it sets `textScale` to that
size's scale, so a host binds the setting's word and never a number. Settings' pane reads the same function for its
three sample lines (`EmuSen_Qt.md` §7.7). Rule Q: `textSize` is an addition to `Tokens` in the registry; nothing was
removed or changed. Held to by kit-core's `kit_7_47_each_text_size_word_has_its_scale_and_larger_is_the_most` and the
program's Settings test (the kit's `textScale` at `text_scale_of("large")` once Large is chosen); seeded, Large's scale
as 1.1 failed the first.

### 7.6 Building, linking, the registry, and what it is held to

**The linker is lld** (Fedora's `lld`, 22.1.8, installed 2026-11-07 at the tester's desk, as `EmuSen_Qt.md` §10.4
proposed). cxx-qt's build utilities find it and pass `-fuse-ld=lld` themselves, so no crate names a linker, and the two
GNU `ld.bfd` workarounds the kit carried for a day are gone from `build.rs`: the `--no-as-needed` naming of the Qt
libraries from §10.4, and `--undefined=cxx_qt_init_*` for the four initializers. The second one is worth keeping on
record: when a crate that links the kit as an rlib is the final link (the kit's own integration tests), rustc places
the extracted whole-archive `call-initializers` members after the rlib, and `ld.bfd`'s single pass has already left the
rlib's archive, so the init functions went undefined; `lld` resolves across the whole input and has no such order.

Measured on 2026-11-07 on the desktop, six compilers, CPU weight 20, the compiler cache (`sccache`) switched off so the
figures are the compilers' own: a cold debug build of the kit and its dependencies is **65 s** under `lld` (the first
build, under `ld.bfd`, was 73 s; cxx-qt's generated C++ dominates both, as `EmuSen_Qt.md` §10.2 found); a change to a
Rust file that holds no bridge rebuilds in **11.4 s** (5.1 s with the cache warm), a QML file in 5.0 s with the cache;
the ten test binaries build and link in **2.3 s** together. With the cache warm a cold build is 19.7 s, which is the
figure a day's work sees.

**Rule Q.** `Testing/qmltypes/EmuSen.Kit-1.1.qmltypes` and `EmuSen.Kit-1.1.qmldir` are the registry as
`qmltyperegistrar` and cxx-qt write them (`Frontend/target/cxxqt/qml_modules/EmuSen/Kit/`). `tests/qmltypes.rs`
compares the build's files to the committed ones with line numbers and header file names dropped, since those move
with every edit; a difference fails until it is reviewed, the minor bumped for an addition or the major for a removal,
and the files copied. Until 1.0 the comparison is strict equality; the addition and removal distinction is a reviewer's
step, not the test's.

**Tests** (`Frontend/kit-qt/tests`, 34 with §7.5's and §7.8's, no Qt application needed; the bridges themselves are
exercised by the harness of step 9): `kinds` (the mirrors pair by pair), `column` (every colour of both variants, the metrics by form and text
scale, motion in milliseconds and Qt's easing numbers, reduced motion zeroing movements and keeping fades at 120 ms),
`qt_keys` (every declinable press has a Qt key both ways, Shift+Tab, Enter, the mouse's buttons), `route` (the sheet
before the walk before the key), `router` (the queue applies at a pump only, the own tiers at 400 then 80 ms, the swap
once, the chord once with forget, a change of tiers) and `qmltypes`. Measured on 2026-11-07 with four seeded faults,
each reverted: `pad_button` off by one failed the kinds pairs; a declined press losing its key failed the route test; the
CubicOut curve numbered as InCubic failed the easing test; the swap applied inverted failed two router tests.

**Hand-written C++**, each part with its reason (decided 2026-11-08: cxx-qt is the binding; C++ glue is written where
cxx-qt-lib does not reach, each part with its reason, `EmuSen_Qt.md` §1.4). The kit's `cpp/kit_glue.cpp` is `send_key`,
a key event sent to the focus window (§7.3). The harness's `cpp/harness.cpp` holds the tree dump and QTest's verbs, which
cxx-qt-lib does not bind (§7.9); the contrast sampling and the picture difference, pixel loops over one grab (§7.10);
the Qt calls of §7.11 that cxx-qt-lib does not bind (`QAccessible::queryAccessibleInterface`, `QMetaProperty`'s
declaring class, `QQuickItem::scopedFocusItem` and `setFocus`, a property by an item's address,
`QQuickTest::qWaitForPolish`); and §7.14's message handler, since `qInstallMessageHandler` takes a C function pointer,
which neither cxx nor cxx-qt-lib can supply. §7.16's `GamePicture` and `kit_adopt_vulkan` (`cpp/game_picture.*`):
an item's scene-graph node with a native Vulkan texture, and a window's adopted instance and device, none of which
cxx-qt-lib binds.

### 7.7 Open

- **[routine]** Install Fedora's `lld` for the toolchain, as §10.4 proposed; both linker workarounds then go.
- **[routine]** The interface's typeface is not a token in kit-core (§1.3), so `Tokens` has no font family. Recommended:
  add `fontDisplayFamily` and `fontBodyFamily` to kit-core's tokens with their values read from `EmuSen_Interface.md`
  §5.2 when the first screen needs them, the gallery included, and bundle the faces as §10.2 found necessary.
- Owed by §7.5 to a later step: Moonrise's strikes paired with the chime (the cues play through Endymion's interface
  player since §7.15); the stars' per-tick cost measured on the handheld before stage 1 lands.
- Owed by §7.15: which of the interface's events rings which cue (a move, a section, a choice, back) is a screen's,
  decided with stage 1's first screen; the gallery program names no platform library, so a human look at it reads no
  pad until a host passes one **[routine]**; a uinput pad (a kernel device, seen by every program on the desk, as
  WiseMan's `UinputHotplugTests` makes under `EMUSEN_UINPUT_PADS=1`) is not used, since SDL's virtual pads prove the
  same path without one.
- Owed by §7.28 **[routine]**: a theme's button picture is drawn in its own colours, where ES-DE shifts it by
  `iconColor` (a later piece of stage 4). Decided 2026-11-08: a key held in the ES-DE view repeats at the keyboard's
  own rate, not ES-DE's tiers.
- Owed by §7.8: `RatingPicker` and `DatePicker` (`Keyboard` is built, §7.22); the drawn
  glyphs of the style (a check, a radio dot, a chevron) in place of the rings, discs and rectangles it draws now (the
  pad's buttons are drawn since §7.12); the desktop form's menu bar and status bar.
- Done 2026-11-08 (§7.24): a row whose control takes Up and Down itself (a `TextField`, a `ComboBox`) is passed by a
  vertical walk; the row walks first, the control takes the keys once chosen.
- Decided 2026-11-08 by the tester: the approved draft is Moonrise's oracle, so its caption stays at the draft's 15
  units as the audit's named exception to the floor (§7.10).
- Owed by §7.10 to stage 1: the frame raises `focusMoved` when the walk's focused role changes (B on the desktop's
  content gives the navigation the focus) but no item takes Qt's focus, so the walk's role and Qt's focus part there;
  the gallery's sections are pointer targets and shoulders until the desktop form's navigation is built. The other screens' faces (§5.2) are
  bundled when the first screen needs them; Moonrise's two are.

### 7.48 An orbit deleted with its screen (2026-11-09)

When Orrery's screens were torn down, Qt logged three `TypeError`s a cover, at `OrbitSlot.qml`'s `Translate` (`slot.view`
null) and at the theme's `OrbitCover.qml` (`slot.orbit` null): 14 to 22 lines at the end of a theme-audit run over Orrery,
by the screen placed last, measured on 2026-11-09. They were first seen after the kit's `ViewCurrent` (§7.39) and taken for its doing; they are not.
With `Walkable`'s line removed the same lines came back in the harness (32 of `OrbitSlot`), and a scene whose covers no key or
pointer had touched logged them too.

**The cause.** A `ListView` deleted at once, as a window deletes what it holds or the harness deletes a named item, releases
its delegates to `QQmlDelegateModel`, which deletes them later (`deleteLater`). Under `pragma ComponentBehavior: Bound`
the delegates read the screen's ids, and the deleted view's id now reads null, so their bindings run once more and every
read through the view throws. Measured in the harness on 2026-11-09 with probe scenes: a delegate of a type of its own
(the kit's `Walkable` or `OrbitSlot`, or a copy in a local file) inside a bound file logged on the delete; the same scene
without the pragma, or with a `FocusScope` written in the delegate's place, logged nothing; a `Loader` letting the same
screen go logged nothing, since Qt clears a loaded item's context before deleting it. theme-audit's own teardown between
screens, through the host's sandbox, was quiet; the lines came at the end of the run, when the harness's window goes with
the placed screen in it, as the program's does when it closes.

**The rule.** A delegate outlives its view while a screen is torn down, so a binding that reads the view must read it
as possibly null. `OrbitSlot` reads `view?.width`, `view?.contentX` and `view?.y` (`?? 0`), so no theme's orbit logs on the
kit's side; a theme's own delegate that reads its list by id does the same, as Orrery's `OrbitCover` now does for its
`turning` and `lit` (`EmuSen_Qt.md` §3.4.6). What the guard gives while the screen goes is never drawn.

Held to by `qt-test/tests/orbit.rs` over `OrbitScene.qml`: eight covers on an orbit in a bound file, the pad moving the
current one and the pointer making the fourth current, then the screen deleted at once with the harness's new `destroy`
(`Scene::destroy`, `harness_destroy`: deletes a named object as a window's teardown does). Qt must log nothing. Before the
change it logged 48 lines, all `OrbitSlot.qml:44-45`. Seeded fault: `OrbitSlot`'s `view?.` taken back to `view.` (the test
fails on 48 lines).

### 7.49 The tree read as the next frame draws it (2026-11-09)

theme-audit named Orrery's help bar in its `disabled` state with Open over Search and Home, at four sizes of five
offscreen (1920 × 1200 alone on the GPU). The entries' boxes in the recorded tree all stood at the Row's start
(x 96, 96, 96 at 1920 × 1200), while the picture recorded with it showed them in a row. Measured on 2026-11-09 with the
audit's held tree read twice: at once after `rest`, x 96, 96, 96; 300 ms later, x 96, 282.7, 492.8.

**The cause.** Walking the focus to a declined place gives the frame a new help model, the bar's `Repeater` makes its
entries again, and the `Row` places them at its next polish, which Qt runs only before a frame. `Scene::rest` returns at
once while a running animation never ends, which the harness takes for a design's resting motion, and every
`FrameAnimation` counts as one; the kit's `Sky` runs its `FrameAnimation` while its glow eases (§7.5), and the focus moving
from a cover to a place moved the glow. The tree was read between the change and its polish: a state no frame ever draws.
The light variant, whose sky is not drawn, waited for frames and read the row as drawn.

**The rule.** `harness_tree` waits for a scheduled polish first (`QQuickTest::qWaitForPolish`, Qt's own test API), so the
tree is the one the next frame draws, whatever is running. The audit's rules are unchanged. That `rest` cannot tell the
sky's easing from a resting motion stays as it was: a frame clock declares neither, and the polish wait is what the tree
needed.

**Why the polish is the moment.** Qt Quick places an item in two steps: a property change marks a positioner or layout
dirty and asks for a polish, and the polish, run on the GUI thread just before the scene graph is synchronised for a
frame, writes the children's `x` and `y` (Qt's documentation of `QQuickItem::updatePolish`, and `QQuickTest::qWaitForPolish`,
which Qt Quick Test provides for reading items after such a change). Between the two, a box is a value no frame draws;
after the polish it is the value the next frame draws. Waiting for a frame would also do, but not when no frame is due,
and a fixed wait only makes the race rarer. With no polish scheduled the read is as before. Measured on 2026-11-09 with
the change: qt-test's 29 tests and the program's (`emusen-qt`) 26 pass, as they did before it; none reads a tree
mid-layout, so it fixes no other finding or flake those tests knew. The finding it removes is Orrery's above; theme-audit
over the other four themes in every state is in `EmuSen_Qt.md` §3.7.2.

Held to by `qt-test/tests/polish.rs` over `PolishScene.qml`: a help bar whose model changes, one entry declined, while a
`FrameAnimation` runs; after `rest` the entries stand where they stood. Before the change the entries read x 0, 0, 0
where they stood at 0, 127.5 and 267.9. Seeded fault: the polish wait removed (the same three zeros).

### 7.50 A value takes what the label leaves (stage 5, piece E; 2026-11-09)

Found by Settings ▸ Consoles in the program (`EmuSen_Qt.md` §7.7.E): the Super Nintendo's SA-1 catch-up, a choice the
core labels `After every S-CPU instruction (exact)`, was cut at half its row (§7.46) beside a label of thirteen
characters, at every size of the gate, where Mistress's row shows it whole. A `MenuRow`'s value is now as wide as the
row leaves it after the insets, the label's own width, the gap and the marks, and never less than half the row, so a
long label still keeps its half and a folder's path longer than the room is still cut at its start (§7.46, and the named
exception of `EmuSen_Qt.md` §7.7). Nothing of the surface changed. Held to by the program's Consoles test (the value
whole at the gate's five sizes) and its Settings test (the ROM folder's path still cut, its pane whole); seeded, the
half-row cap put back failed the first with the value cut at all five sizes.

### 7.51 The rows' view keeps no current index of its own (stage 5, piece E; 2026-11-09)

Found by Settings ▸ Consoles in the program (`EmuSen_Qt.md` §7.7.E): B on a console's Firmware page was to return the
focus to the Firmware row, and the host's `focusRow(1)` took, then the first row took the focus back. Traced on the
software backend (the walk's `current` and each row's `activeFocus`, 2026-11-09): the rows removed and the page's rows
inserted, the walk at 0 and then at 1 by the host, then row 1 losing the active focus and row 0 gaining it, which the
row's handler turned into `focusRow(0)`. Qt's item views set a current index of their own when rows arrive in an empty
view and give that row the focus; `currentIndex: -1`, a constant binding, does not hold against it. The view now clears
its current index again whenever it sets one, so the walk's focus is the only one: the trace then ended at 1. Nothing of
the surface changed. Held to by the program's Consoles test (B on Firmware returns to the Firmware row); seeded, the
clearing taken out failed it (the focus on row 0).

### 7.52 The settings frame's sentence in a tone (stage 5, piece E; 2026-11-09)

Asked for by Settings ▸ Consoles (`EmuSen_Qt.md` §7.7.E), decided by the tester on 2026-11-09: Mistress tinted a
console's firmware line Success where nothing is wanting and Attention where every game needs a file that is not found,
and that line is often the focused row's sentence, which the frame draws. `SettingsFrame` gained `sentenceTone`, a
`Tones` value the host sets for the focused row (by what its sentence says, never a colour), `Text` unless set; the
sentence is a `ToneText`. Weighed as `EmuSen_Qt.md` §1.3 asks, an extension of the frame: one property, no second job.
Rule Q: an addition to `SettingsFrame` (qmldir), recorded in `CHANGELOG-kit.md`; nothing was removed or changed. Held to
by the program's Consoles test (the Super Nintendo's `✓ No firmware needed` drawn in the dark variant's Success,
#35A064); seeded, the tone left unbound failed it.

### 7.53 The clip and the page frame's own parts for a game's page (stage 5, piece G; 2026-11-09)

**`VideoTile`** (qmldir) is a clip over its still: the still (`still`) first; once `playing` turns true and the motion
token `poster` (1.5 s) has passed, the clip (`source`) plays, looped, and fades in over `posterFade`; when `playing` turns
false, or the source changes, the player goes and the still stands alone. It is silent unless `muted` is false. Reduced
motion keeps the still (`EmuSen_Interface.md` §5.9). `showing` says whether a frame of the clip stands over the still,
`running` whether its player is made, `frames` how many frames the kit's sink has drawn. The player, decoder and drawing
are **`VideoClip`** (qmldir), which the tile makes only while its clip is to play, so a window that shows no clip never
starts Qt Multimedia: made with every window, it started Qt Multimedia in every run of the program (2026-11-09). The decoder is Qt Multimedia's over the system's FFmpeg (`EmuSen_Qt.md` §1.8.9). On an RHI
backend `VideoOutput` draws; on the software backend, where `VideoOutput` draws nothing (measured 2026-11-09), the kit's
**`VideoFrames`** (registry; `cpp/video_frames.*`) does: a `QVideoSink` the player writes to, each frame converted with
`QVideoFrame::toImage` and drawn as an image node, cropped to fill as the still is; `frames` counts the frames drawn
since the last `clear()`. One tile serves the game's page's hero and, when it is mapped, the ES-DE engine's `video`
element, so a theme never imports QtMultimedia. The kit now links Qt's Multimedia module and its QML module depends on
`QtMultimedia`.

**`PageFrame`** gained, for a game's page (`EmuSen_Qt.md` §7.8):
- the hero's clip: `clipSource`, `clipPlaying` and `clipMuted` (not `clip`, which is every item's own), a `VideoTile` in place of the hero's picture under the OnArt
  gradient, its still the `hero`; `videoTile` names it;
- words beside a picture in place of tiles or rows, as the Overview shows them: `showsText`, `contentWords`,
  `contentPicture`, the picture 388 × 222 at the content's right and the words in the rest less a gap of 24, from
  Mistress's `GamePage` recording (`PageLayout`'s new constants `pictureWidth`, `pictureHeight`, `pictureGap`, from
  kit-core's `PageGeometry`);
- a stepped action: the action model's roles `stepped` (◂ ▸ drawn either side of the value, on `ActionButton` as
  `stepped`), `stepLeft` and `stepRight` (whether Left or Right steps it from where it stands), and the signal
  `actionStepped(index, by)`. Left or Right on an action that can step that way steps it; past either end the focus moves
  to the neighbour, as LunaP's `MoveInActions` does, so every action is reached. The host says which way each action can
  step, from the value it holds;
- `rowStepped(index, by)`: Left or Right on a row that steps.

A list whose model fills after the view was made has no current item, so no delegate took the focus and the walk found
the list itself (the program's first run of the game's page, 2026-11-09, where the actions and the rows come from models
read off the GUI thread): each of the frame's three lists now makes its first item current when it fills.

**Rule Q.** Additions to the registry (`VideoFrames`; `PageLayout`'s three constants) and the qmldir (`VideoTile`,
`VideoClip`), and
to `PageFrame`'s and `ActionButton`'s properties and signals. One change to a contract: the action model's delegate now
requires the roles `stepped`, `stepLeft` and `stepRight`, so a host's action model must name them; the gallery's does.
The module stays 1.0 (§7.6). `EmuSen.Theme` re-exports none of it.

**Held to**: kit-gallery's screens test (the page with tiles and with rows, both variants, the four sizes) and the
registry's comparison, and the program's game page test, which holds the clip on the software backend
(`EmuSen_Qt.md` §7.8).

### 7.55 The raised shadow as a token, and `RaisedShadow` on both backends (stage 5, piece K; 2026-11-09)

`EmuSen_Interface.md` §5.4 gives the interface one shadow: on a focused tile and on a card's open edge, offset 0,8, blur
24, black at 50%. It was not a token, so each drawer wrote it as its own literals (`NoticeLayer`'s 0.5, 24 and 8,
`MenuCard`'s offset of 8 and black), and the page frame's cover drew none (§7.44).

**The token.** kit-core's `tokens::raised_shadow(Form)` gives a `Shadow` (`colour`, `opacity`, `offset_x`, `offset_y`,
`blur`), the same in both forms' units, as the focus marks are. `Tokens` shows it as `shadowColor`, `shadowOpacity`,
`shadowOffsetX`, `shadowOffsetY` and `shadowBlur`, declared after every earlier property so that no earlier index in the
registry moves; the theme API's `Tokens` mirrors them (`CHANGELOG-theme-api.md`), as `theme-qt/tests/surface.rs`
requires of every kit token. A minor under rule Q (`CHANGELOG-kit.md`): additions only, the module still 1.0.

**Its drawers read it.** `NoticeLayer`'s `shadowOpacity` and `shadowRadius` default to `shadowOpacity` and `shadowBlur`,
and its offset and colour are the tokens', so the notice draws what it drew. `MenuCard`'s offset and colour are the
tokens'; its darkness of 90% and radius of 48 stay its own, being Mistress's panel shadow as measured on 2026-11-02
(`EmuSen_Qt.md` §7.2), a different fact from §5.4's.

**`RaisedShadow`** (qmldir) is the shadow a raised shape casts: an item with the shape's box and corner `radius`, placed
under the shape. On the GPU it is `MultiEffect` over a hidden caster of the shape, with the tokens as its shadow's colour,
opacity, offsets and `blurMax`, as the notice and the cards cast theirs. The software backend renders no shader effect
(§7.35), so there Qt's own `Canvas` casts it: its 2D context paints the shape with the tokens as `shadowColor` (the
colour at `shadowOpacity`), `shadowBlur`, `shadowOffsetX` and `shadowOffsetY`, through `QPainter`, into a canvas reaching
twice the blur and the larger offset past the shape, so no part of the shadow is cut. Qt's mechanism rather than a render
node of the kit's own (`EdgeMask`, §7.35), since this one draws as asked on the software backend. The shape itself is
drawn in the shadow's colour under the raised item, which covers it. `qml-rules` takes `contextType` as a name, not words,
as it takes `fragmentShader`.

**The cover casts it.** `PageFrame` places a `RaisedShadow` at the cover's box with the cover's `rowRadius`, under the
cover, so a game's page (`EmuSen_Qt.md` §7.8) shows its cover raised over the hero's foot.

**Measured** (2026-11-09, the gallery on the software backend). Under the cover, 6 units below its foot and so 2 units
inside the offset shape's foot, the light variant's frame (243) reads 176, 27.6% darker; the dark variant's (27, 27, 30)
reads (20, 20, 22). A Gaussian of σ = blur ÷ 2 = 12, the canvas's definition of `shadowBlur`, predicts 0.5 · Φ(2 ⁄ 12) =
28.3% at that point; the measurement is within 0.7 points of it. 6 units past the offset shape's foot it predicts
0.5 · Φ(−6 ⁄ 12) = 15.4%, 243 to 205.5; the frame reads 205. 40 units beside the cover the frame is untouched. The
GPU's `MultiEffect` blurs by its own mipmap levels up to `blurMax`, not a stated σ, so its falloff is not the canvas's;
that difference between the backends is in Qt's two mechanisms, each given the same token. On Vulkan (the RX 6800,
`qt-app`'s `acceptance.rs` on a game's page, 2026-11-09) the same point of the dark frame reads (18, 18, 20) against
(27, 27, 30) beside the cover, 33% darker where the canvas gives 26%.

**Held to:** kit-core's `tests/tokens.rs` (the raised shadow in both forms); kit-qt's `tests/column.rs` (the column's
shadow is kit-core's in both forms) and `tests/qmltypes.rs` (the registry and the qmldir, committed); `theme-qt`'s
`tests/surface.rs` (the mirror holds every kit token); `qml-rules`' `tests/rules.rs` (`contextType` a name); and
kit-gallery's `tests/screens.rs`, which now also holds the cover's shadow in both page states, both variants and the four
fit sizes: the frame under the cover at least an eighth darker than beside it, and 6 units past the shape lighter than
that and still a sixteenth darker, so the shadow is soft, besides the gate it already ran; and `qt-app`'s
`acceptance.rs`, which opens the first game's page from the grid on Vulkan and holds the same eighth (passed in 699 s).

**Seeded faults**, each reverted, each caught: the cover's shadow hidden (the gallery, every page state); the canvas's
blur set to none, a hard-edged shadow (the gallery's softness check, which the first form of the check, the darkening
alone, let through, so the check was strengthened); the theme mirror missing `shadowBlur` (`theme-qt`'s surface test).

**Rule Q and the shadow's colour** (2026-11-11). `shadowColor` is a colour property of the kit's `Tokens` after the
palette's, so theme-check's rule-Q test, which held the registry's colours to `ColourToken::ALL` alone, failed from the
day the token landed (seen 2026-11-11; theme-check is outside the workspace and no landing ran it). The name is kit-core's
now, `Shadow::COLOUR_PROPERTY`, and the test holds the registry's colour tokens to the palette's and the shadow's, in the
registry's order. A theme may override it as any colour token.

### 7.58 The walk presses on through a stepped action; the page frame's focus; Qt Multimedia's two messages (stage 5, piece G2; 2026-11-09)

**The walk and a stepped action.** The gate's walk (`emusen-qt-test`'s `walk_by`, §7.11) pressed each direction once
from each element. On a game's page Right on Resume steps its state and keeps the focus, so the walk recorded Resume →
Right as no move, and Favourite, which Right reaches from Resume only past its last state (LunaP's `MoveInActions`,
§7.53), was named unreached by the pad: a finding piece G named for the tester rather than fixed. The walk now presses
on as a player does: while a press leaves the focus where it was and the element declares a stepped value (the kit's
`ActionButton.stepped`, read by its name), it presses the same direction again, at most 64 times, and records where the
focus finally lands. An action that steps no further passes the press to the frame's navigation, so the walk ends at
the neighbour; one that wraps would end at the bound, as before, standing still. What the presses stepped is the host's
value, which the walk does not put back; a later press from the same action steps back the other way first. The
program's game page test holds the five states with no finding named (`EmuSen_Qt.md` §7.8).

**The page frame's focus** (`PageFrame`, two defects found by the program's game page test on 2026-11-09). (1) A page
opened on another game kept the focus where the last page left it: closed from Details by Hide from Library, the next
page opened on its Overview with the focus in the rows' list, hidden, so Down and Left and Right moved through a list
no one saw (the tree read the focus on an invisible `MenuRow`). LunaP's page opens with the focus on its main action;
the frame gains the signal `focusFirstAction()`, which the program's page emits when its game changes; and `contentKindChanged()`, emitted as the kind changes. (2) A section of another kind
chosen with the focus in the content (tiles to rows, rows to tiles, either to words) left the focus in the list now
hidden. LunaP's `FillContent` hands the focus on to the new content's first item, or to the actions where there is no
item; the frame now does the same when `showsTiles` or `showsText` changes with the focus in a list. The program's test
holds both: A on a state after Hide reaches the state, and the shoulder from a Media tile lands on Details' first row,
shown.

**Qt Multimedia's two messages** (`QT_INTERNAL`, §7.14), traced on 2026-11-09 under gdb on the theme-qt `qml` test, a
scene that plays no clip:
1. `WARNING: QApplication was not created in the main() thread.` is QtCore's, from `QCoreApplicationPrivate`'s
   constructor when the thread making the application is not the one Qt holds as the main thread. A hardware watchpoint
   on `QCoreApplicationPrivate::theMainThreadId` found it set before `main`, by `libQt6Multimedia.so.6`'s load-time
   initializer (`SymbolsResolverImpl::instance`, which makes a `QLibrary`, a `QObject`, so Qt adopts the loading thread,
   the process's main thread). The harness then makes the application on the thread libtest runs the test on. Every
   test binary that links the kit (which links Qt Multimedia since §7.53) logs it once, whether or not it plays a clip;
   the program makes its application on `main` and never does. Stopping it needs the application made on the process's
   main thread: each Qt test target built with `harness = false` and a runner of its own, a change to every crate's
   tests. Proposed to the tester; the line stays in `QT_INTERNAL` until then.
2. `ThreadLocalRhiHolder::ensureRhi(...): No RHI backend. Using CPU conversion.` is Qt Multimedia's, logged once per
   thread by `QVideoFrame::toImage` (the kit's `VideoFrames`) when no RHI can be made on the converting thread for a
   conversion on the GPU; it then converts on the CPU, the path the software tier needs. The `offscreen` platform the
   tests run on offers no OpenGL to make one. The frames are drawn (the program's test reads the clip's magenta). It is
   Qt's own fallback, harmless, and left in `QT_INTERNAL`.

### 7.62 The controller drawing (stage 5, piece H; 2026-11-09)

A controller drawn from data, its regions named by the controls a host's controller declares, a label beside the drawing
for each with a leader to its region, the focused control ringed and the held ones lit; for the bindings page of
`EmuSen_Input.md` §10.6 and `EmuSen_Qt.md` §7.7.H. **Decided by the tester on 2026-11-09 (D2):** drawings are data keyed
by the controller id a core declares, the regions named by the declared controls, with a generic drawing for a
controller that has none, so consoles look like themselves and a new core needs no code. Weighed as §1.3 of
`EmuSen_Qt.md` asks: an extension of the kit, a new type over nothing it had; the kit names no controller and no console,
and which file stands for which controller id is the host's.

- **A drawing file** (`emusen-kit-core::controller::drawing`) is JSON: `name` (what a reader calls it), `design` (its
  own width and height) and `items`, drawn in the layers LunaP's `ControllerDiagram` drew: the regions marked `behind`,
  the shell (`part`), each stick's gate, the regions, the marks (`mark`), the printed words (`word`), the knobs. A shape is
  a `circle`, `box` (with a corner radius, and `ends` rounding one side only), `pill`, `path` (SVG path data, absolute
  commands), `arcband`, `octagon`, `arrow`, `ring` (an outline, filled even-odd) or `union`, any of them `turn`ed about a
  point. A region (`region`, the control it stands for; `name`, its words; `side`, where its label stands) is a shape,
  or one of three helpers LunaP had: `round` (a round button, its edge its colour darkened by 35 %), `dpad` (a cross of
  four arms with their arrows and a hub) and `stick` (a gate, its directions as bands of a ring each with an arrow, and
  its click). A union is stroked whole before it is filled, so its seams do not show, as LunaP's `CombinedGeometry` drew.
- **Which drawing stands for a controller** (`drawing_for`): its own file where the file reads and has a region for every
  declared control; else the **generic drawing** (`generic`), the kit's two-stick pad with every RetroPad control
  (`generic.json`, LunaP's `Gamepad` drawing) and each stick's directions by its axes (`LeftY-` is up), with a round
  button under the shell for each declared control it has no place for. A region whose control is not declared is drawn
  and has no label.
- **The layout** (`lay_out`): each label on its region's side (the file's `side`, else left or right by the region's
  centre); the left and right columns as wide as their widest label and standing between the rows of labels above and
  below, which span the box; the drawing the largest that fits between, centred; the labels of a side packed in the
  order of their regions, each as near its region's height as the others let it, 2 apart; a straight leader from each
  region's centre to its label. `fits` is false while a label is unmeasured or a side cannot hold its labels.
- **What is painted** (`paints`): a focused region ringed in the accent 14 wide under it, a held one filled in the accent
  with a halo at alpha 90 and its edge the accent darkened, a direction pad's arm washed in the accent at alpha 130 rather
  than ringed (its neighbours touch it), as LunaP drew them. `svg` writes the paints and the words as one SVG document.
- **A stick's knob** (`knob`) stands where its directions push it, each axis -1 to 1: right less left, down less up, each
  direction's push held to 0 to 1, times the stick's travel from its centre, as LunaP's `SetStick` placed it. With
  nothing pushed it is at rest.
- **A direction** (`neighbour`, `toward`): the region nearest a region a way is, among those whose centres lie more than 6
  ahead, the one of least distance plus offset across, the offset counted five times where it is more than half again
  the distance ahead (LunaP's `Neighbour`); `toward` hops such neighbours, at most 32, to the first that stands for a
  declared control, and gives none where nothing lies that way. The walk (`walk`) remains for a host that steps the
  controls in their order.
- **`ControllerLayout`** (kit-qt `controller.rs`) puts it on Qt: `drawing` (the file's text, empty for the generic one),
  `controls`, `focused`, `held`, `pushes` (how far each control is pushed, 0 to 1, in the controls' order: a stick's
  directions place its knob), `texts`, `width`, `height`, `labelHeight`, `accent` (a colour) in; `labelWords` (each
  label's region name and text) and `labelBoxes` (each label's x, y, width and height in turn), which a label binds, `name`, `generic`, `fits`, `place`,
  `scale`, `designWidth`, `designHeight`, `leaders`, `focusLeader` and `picture` (the SVG in a `data:` URL) out; the
  invokables `setLabelWidth`, `labelRect`, `labelName` and `step` (the walk, round from either end).
- **`ControllerDrawing.qml`**: the leaders (Muted, the focused one's Accent), the picture as an `Image` of the SVG at the
  drawing's place, and a label for each control: its region's name and the host's words for it (`texts`) in Caption,
  measured and handed to the layout as it is made (`Repeater.itemAdded`) and whenever its width changes, its edge
  Accent and 2 wide while focused; a click on a label raises `chosen`. The labels bind the layout's properties rather
  than place themselves, a design choice of 2026-11-10: the QML rules (§7.9) allow no function and nothing done on
  completion, and qmllint (§7.6) found the token's colour bound to a string.

**Why one picture and not a `Shape` per part** (measured on 2026-11-09): the drawing's colours are the console's
(Nintendo's red, Sony's symbols), not the palette's, and LunaP drew them inside one control the recorder never read
(`SettingsBindings-1280x800.spec`, `ControllerDiagram` with no paints). Drawn as a `Shape` per part, the gate's palette
rule found every part of the SNES pad's drawing outside the palette. The tester ruled on 2026-11-11 (R-20261111-114)
that a pad's drawing is a picture, like a cover: the palette and text-floor rules do not look inside it, and its labels,
leaders and focus stay on the palette, where the gate holds them (`settings_drawings.rs`, whose findings excuse no label
of the drawing's, `EmuSen_Qt.md` §7.7.H5). Words printed on the shell are part of the picture. The SVG is drawn by Qt's
own SVG image reader (`qt6-qtsvg`'s `imageformats/libqsvg.so`), which a build that bundles Qt must bundle too.
kit-core reads a drawing file with `serde_json`, a dependency the tester allowed on 2026-11-11 (R-20261111-115).

**Held to**: kit-core's `tests/controller.rs` (11: a direction to the nearest declared control that way; a stick's knob placed by its directions' pushes; a file's items and layers, the shapes' path data and a union stroked
first, the focus ring, the press and the arm's wash, a drawing standing for a controller only with every declared control,
the generic drawing from the controls alone, labels inside the box, beside the drawing and apart, the walk); the
program's `settings_drawings.rs` (every drawing at the gate's sizes, `EmuSen_Qt.md` §7.7.H).

### 7.63 The settings frame's page of content (stage 5, piece H; 2026-11-09)

A page under a section that is not rows, as the bindings page's strip, drawing and caption (Mistress's
`SettingsContent`): `SettingsFrame.pageShown` draws `pageContent` in a box of SurfaceRaised where the rows stand (the rows'
width, the pane's height, the pane's inset), hides the rows and gives them the focus back when it closes;
`pageSentence` is the pane's sentence meanwhile, since no row gives one. The host's content takes the keys the frame
declines (Left, Right and A on the bindings page).

**Rule Q** (§7.6): additions only. The registry gains `ControllerLayout`; the qmldir gains `ControllerDrawing`;
`SettingsFrame` gains `pageShown`, `pageContent` and `pageSentence`. The module stays 1.0. `EmuSen.Theme` re-exports none
of it.
**Each content view holds its model only while it is shown (2026-11-10).** `tileModel` and `rowModel` were aliases of
the two lists' models, and a game's page gives both the same model, so the hidden list made delegates for every section
too: seven media tiles as seven rows, the ones past its four visible rows made asynchronously in its cache buffer. A
section change resets the model; a reset that lands while those are still being made has Qt cancel them by indices
past the new count, and it logs `DelegateModel::cancel: index out range 6 4` (and 5), which failed qt-app's game page
test in 2 of 14 runs. The tiles' list now takes `tileModel` while tiles are shown and the rows' list `rowModel` while
rows are, null otherwise; the hidden list makes nothing. qt-app's game page test steps Media's scrolled tiles and
Details' rows six times at 16 ms and holds Qt silent; with the aliases it logs the two lines on the first run.

**Each content view lets its model go while the model resets (2026-11-11).** Holding the model only while shown
still left the shown list exposed. The page resets its model to the next section's items and only then says the kind
changed, so the outgoing list hears the reset first. Logged on 2026-11-11, States to Options: the tiles' count went from
9 to 4 before `showsTiles` turned false and the list let the model go. Tiles past the view were still being made in
Qt's default cache buffer, and the warning returned as `index out range 6 4` and `7 4` in 2 of 3 runs of the game page
test. Each list now also holds its model only while that model is settled, as a `ResetGate` over it says (below): false
from the model's `modelAboutToBeReset` to its `modelReset`. A list therefore
lets the model go before the reset changes its count, while every index it is still making is in range, and takes the
model back with the new content. No list is attached to the model across a reset, whichever section's content the
reset brings and whatever order the host announces it in. The cache buffer stays Qt's default, so what is drawn while
scrolling is unchanged. qt-app's game page test scrolls States' tiles and turns to Options and back three times at
16 ms, then holds Qt silent. Before the change it logged `index out range 7 4` on its first run.

**A model replaced in the middle of its reset (2026-11-11).** The first form kept each list's flag in QML: a
`Connections` on the model cleared it on `modelAboutToBeReset` and set it on `modelReset`. A model replaced between the
two never sent the second to the frame, whose `Connections` had moved to the new model, so that list kept no model for
good; where the replacement was made from inside the old model's signal, the moved `Connections` still heard that signal
after the change (measured: the frame handed the new model, its list holding none). The repair settled a replaced model
with `Qt.callLater`, which §8.4's QML rules forbid, as they forbid the handlers' assignments (`qml-rules`: `callLater`,
`function`); `qt_8_4_qml_rules_pass_the_kit_and_the_style` failed on those six lines.

**`ResetGate`: the state out of QML (2026-11-11).** The state is now a kit type in Rust (`kit-qt/src/reset_gate.rs`)
with one job: given a `model`, say whether it is `settled`. It connects to the model's `modelAboutToBeReset` (settled
false) and `modelReset` (settled true) through the kit's one declaration of `QAbstractItemModel` (`item_model.rs`, which
gains the two signals, declared private so nothing in the kit can send them). A model handed over drops both connections
to the old one, so the old model's signals are no longer heard, even from inside the emission that caused the handover
(Qt skips a connection dropped during an emission), and the gate is settled: Qt offers no way to ask a model whether it
is in a reset, so a model handed over counts as not resetting. Each connection holds the gate's address and is owned by
the gate, so it is dropped before the gate is; Qt drops it with the model. Nothing is in kit-core: the gate is three
assignments, each held by the test below. `PageFrame` binds each list's model declaratively,
`model: shown && tileGate.settled ? root.tileModel : null`, beside `Kit.ResetGate { model: root.tileModel ?? null }`,
and keeps no state of its own; the QML rules and qmllint pass the kit with nothing excused.

Held to: `kit-qt/tests/reset_gate.rs` over `kit-qt/tests/qml/ResetGateScene.qml`, a keyboard model relaid out (which
resets it). The gate's `settled` changes twice across a reset; a second model handed over the moment the first one's
reset begins leaves the gate settled; the first model's next reset is not heard and the second's is. Seeded on
2026-11-11, each failing that test: the reset never unsettling the gate, the handover not settling it, and the old
model's connections kept. kit-qt takes the harness (`emusen-qt-test`) as a dev-dependency for this test, as esde-qt,
qt-models and kit-gallery do. `kit-gallery/tests/page_frame.rs` still hands the frame another model from inside
the keyboard model's reset and reads the tile list showing it, and qt-app's game page test still scrolls States' tiles
and turns to Options and back at 16 ms with Qt silent. Rule Q's registry was reviewed and copied: one addition,
`ResetGate` with `model` and `settled`.

**A model deleted while the gate holds it (2026-11-11).** The gate kept the model's raw address and handed it back
through `model`; a model reached through a `property var` (as `PageFrame`'s `tileModel` and `rowModel` are) says nothing
when it is deleted, so the gate went on holding and exposing a dangling address, and a model later made at the same
address would have been taken for the old one by `setModel`'s same-model check and never connected. The gate now also
hears the model's `destroyed` (declared beside the reset signals in `item_model.rs`, private in the same way): from
then it holds no model, says so through `model`'s change signal, and is settled. The connections Qt drops with the
model are let go when the next model is handed over. Held to: the same test, a second gate in the scene handed its
model by assignment to the gate's `model`, with no binding; the model deleted, the gate holds none (`model === null`),
and a model handed after is held and its reset heard. That model is made only after the deletion (a `Component`'s
`createObject`), so the allocator may give it the deleted model's address, the case the fix is for; it cannot be made
to. Measured on 2026-11-11, printing the addresses `setModel` was handed: the deleted model and the one made after it
were 0x3490 bytes apart (each `KeyboardModel` takes about 13 KiB), and none of sixteen made one after another took the
deleted one's address. So the same-address case is argued from the code (`forget` nulls the address the same-model
check compares), and what the test holds of the fix is the gate holding none after the deletion, told through `model`'s
change signal, and settled. Seeded on 2026-11-11: the `destroyed` connection left out, the
gate still holding the deleted model. A binding does not show the fault: measured on 2026-11-11, a gate bound to the
model's id, and one bound through a `property var` (as `PageFrame`'s `tileModel` is), each passed the test with the
fault in place, because QML tells both of the deletion (the id's guard, and the guard a `var` property keeps on the
object it holds) and the binding hands the gate null itself. So `PageFrame` was not exposed; the gate now guards itself
for a model handed to it any other way.

### 7.105 The section strip is a scroller only where its names run past it; the harness reads an item's number (repair DC-B; 2026-11-11)

`SectionStrip` (§7.43) was a horizontal `ListView` with `interactive: false`. Where a screen lists more sections than
the strip holds, as Cores does over a world of fourteen engines, the gate's pointer reach found its wheel leaving it
still (EmuSen_Qt.md §7.9: five findings, one a size, measured on 2026-11-11 on the software backend and on Vulkan alike).
Making it interactive alone does not move it: a horizontal `ListView` takes no turn of an upright wheel (measured with
kit-gallery's `strip`: `contentX` 0 before and after three notches). A `WheelHandler` moves its names along by the
wheel's turn, horizontal or upright, within `originX` and the content's end, and is enabled only while the names run
past the strip (`contentWidth > width`).

The gate reads a flickable's wheel from its `interactive` (qt-test's tree, `harness.cpp`), and that rule is the gate's,
not the strip's to change. The strip is therefore a scroller exactly where it is one: `interactive` is
`contentWidth > width`, so where its names run past it the gate reads a scroller the wheel moves, and the wheel, through
the handler, does move it; the walk's shoulders step the chosen section, and the view's highlight range brings that name
into the strip whole, as every other view's current item. `acceptedButtons` is `Qt.NoButton`, so a mouse drag does not
move it (Qt 6.9's `Flickable.acceptedButtons`) and a click on a name still chooses it. That property is the mouse's
alone: a finger's drag flicked an interactive strip (measured on 2026-11-11 with kit-gallery's `strip` and qt-test's
`Scene::touch_drag`, one finger on a touch screen: `contentX` 0 before, 1618 after). A `DragHandler` for touch screens,
with no target and `CanTakeOverFromAnything` alone, so that it yields its grab to nothing, takes a finger's sideways drag
while the names run past the strip, and the strip stays where it is. Its upright axis is off (`yAxis.enabled: false`),
so a finger's upright drag that starts on the strip is the page's, as it was before the strip took touch: without that,
measured on 2026-11-11 with kit-gallery's `strip`, the handler took the upright drag too and the page under an
overflowing strip did not move. A tap shorter than the drag threshold still reaches the name (`strip` taps NES on the
page's overflowing strip and it is chosen). Where the names fit, the handler is off and a finger's drag from the strip
is the page's. A ListView's
`flickableDirection` set across was tried first: the ListView sets it again from its orientation, and set once made it
turned the gate's reading of the strip to an upright scroller, so it was taken out. The condition is written once, in
the strip's read-only `overflows`. Where the names fit,
`interactive` is false and the handler disabled: no scroller at all, and the turn goes to whatever is under the strip.

A pass of 2026-11-11 had kept the strip not interactive and changed the gate instead, counting any enabled
`WheelHandler` under a flickable as moving it. That rule had no test of its own and would have passed a handler that
moves nothing; it was taken out, and the harness reads `interactive` alone as before.

Held by kit-gallery's `strip` (`StripScene.qml`: thirteen names in a 600-unit strip: the gate reads it as a scroller the
wheel moves and pointer reach finds no still scroller; a drag leaves it still; three notches move it; a click on a whole
name chooses it; the walk's last section, PLAYSTATION 2, is brought in whole; on a page that scrolls, a finger's drag up
from a second overflowing strip moves the page and not the strip, and a finger's short tap on NES chooses it) and
`strip_fits` (three names that fit,
on a page that scrolls: the gate reads no scroller, the wheel over the strip moves the page and not the strip, a drag
leaves it still, a finger's drag up from it moves the page, and a click on SATURN chooses it). Seeded faults: the strip
never interactive (`strip` fails), always interactive (`strip_fits` fails), dragged by the mouse (`strip` fails), and
the touch handler gone (fault 49, `strip` fails at the finger's drag), and the handler taking the upright drag too
(fault 50, `strip` fails at the page's drag).

qt-test's `Scene::number` reads a number property of a tree's item by its id, as the harness's own settle already did,
so a test can read an item's `implicitWidth`, `contentX` or `x` where it has no objectName.

### 7.106 A column that gives way; words measured as Text lays them out (repair DC-B; 2026-11-11)

A table's column may give way first (`ColumnWidth::GivesWay { whole, short }`, kit-core's `table`). Its rows carry two
words: their whole words and short words that stand for them. The column takes its widest whole words where the room
holds them beside every other column's words: content columns' widest words, and each fill column's `least` or its
widest words, whichever is larger. Otherwise it takes what that room leaves, never less than its widest short words, and
each row whose whole words do not fit the width given shows its short words. Where the room cannot hold even the short
words, the column counts as a content column of their width and §7.43's scaling applies as before. A fill column
(`ColumnWidth::Fill`) now carries its widest words (`words`), so a column that gives way leaves it the room they take.
Its share of the room is unchanged.

On Qt, `ToolScreen`'s columns take a `shortKey` role, which names the role of a row's short words; an empty one gives
no way. `ColumnFit` measures the whole and short words of such a column, and every fill column's words. Each cell
decides from the width its words take as laid out: a hidden `Text` holding its whole words. Its `implicitWidth` is
what the cell would draw, so the choice and the cut agree to the pixel. Decided 2026-11-11 (EmuSen_Qt.md §7.9): Cores'
console column gives way, showing a core's first console and a count ("Genesis +3").

`ColumnFit`'s words were measured with `QFontMetricsF::horizontalAdvance`. Qt Quick's `Text` lays a line out with
design metrics (the default `Text.QtRendering`), and the two disagree by up to half a unit. On the gallery on
2026-11-11, "Genesis Plus GX" was measured at 195 or under and laid out at 195.5, so it was cut in a column given its
own measure. `text_width` now lays the line out with `QTextLayout` and `QTextOption::setUseDesignMetrics(true)`, as
`Text` does, and reads `naturalTextWidth`.

Held by kit-core's `table` (`kit_7_106_*`: Cores' measures at 1280 × 800 keep the status and the core's name whole, with
the console column between its short and its whole words; whole where the room holds everything; inside the table at
every width), by kit-gallery's `screens` (the tool screen's rows of one, four and twenty consoles at the four fit sizes
in both variants: "Atari 2600 +19" shown, the four whole or "Genesis +3", nothing cut), and by qt-app's `cores_fit` and
`cores_fit_gpu` (EmuSen_Qt.md §7.9).

### 7.107 The pointer focuses only an item that takes input (repair DC-B; 2026-11-11)

The frame's pointer focus (`FrameController`'s `hover`, `hoverAt` and `click`, §7.39) called `forceActiveFocus` on the
walkable under the pointer whatever its state. A `HoverHandler` still reports `hovered` on an item that is disabled
through a parent, and `forceActiveFocus` gives the focus to every focus scope above the item. When the topmost of those
scopes is disabled, Qt Quick moves the window's active focus to that scope's own scope. For a screen placed in the
window, that scope is the window's root item. Nothing then holds the focus, and no press reaches anything.

The program shows this under a card. The screen under a card is drawn and disabled (EmuSen_Qt.md §7.9, the slots'
`backdrop`). Three places in qt-app's `Main.qml` and `ThemeHost.qml` had walkables there before this fix: the own look's
Home screen under the Home card (`homeSlot`), its grid under the List card (`gamesSlot`), and the ES-DE view under every
card and screen over it (`esdeLoader`). The ES-DE view draws its own items, not the kit's `Walkable`, so it had no pointer
focus to lose; the two slots did, in the reference theme and in any EmuSen theme placed in them. In qt-app's `console` on 2026-11-11 the pointer stayed where the first walk had clicked a List card row.
After a console session, the List card opened over the grid again, and the theme's first grid cell came under the
resting pointer as the grid was made disabled. Its `hoverAt` took the focus from the card's row to the root, so Down
moved nothing and the walk read Home's status line ("1 controller · 12:00"), the first words after the root item.
Before the fix the walk failed six times in seven runs, and the focus log showed the cell's hover with `enabled` false
just before the root took the focus. After it, it passed four times in four.

`focus` now gives the focus only to an item that is enabled. `QQuickItem::isEnabled` is the effective state, so a
disabled parent counts. A disabled walkable under the pointer leaves the focus where it is. Held by kit-gallery's
`disabled` (`kit_7_107_*`: a card's row keeps the focus while the pointer comes to rest on a walkable in a disabled screen
beside it) and by qt-app's `home_continue` on the software tier (the pointer moved onto Home's Continue, disabled while
no game is suspended, leaves the focus on Home's first place; without the rule the focus leaves it for the item named `frame`).
qt-app's `console` on the GPU held it before §7.108 and no longer does: since then the grid cell comes under a resting
pointer, which focuses nothing whatever the cell's state, and the fault that drops the rule passed `console` on
2026-11-11.

A walkable that came under a resting pointer took the focus when the controller that owns it had not seen the pointer
there before, since each frame's controller kept its own last pointer position. §7.108 makes the fact the window's.

A click on a disabled walkable is Qt Quick's alone, not a rule of the kit's: Qt Quick delivers no tap to a `TapHandler`
whose item is disabled through a parent, so `click` is never called and its Accept after a refused focus is never
reached. Measured on 2026-11-11 with kit-gallery's `pointer` and a probe in `click` that prints a line and sends Return
to the focused item before anything else (the evidence's fault 35): the click on the disabled walkable printed nothing
and the card's row heard no Return, so the test passed; the same probe with a click on the enabled walkable `near`
printed the line once. No change to the kit can make that assert fail, so it holds Qt's behaviour, and the test says so;
the seeded fault 35, a click that is A however it comes, is recorded as missed for that reason. `click` is unchanged.

### 7.108 The pointer focuses only as it moves, a fact of the window (repair DC-B; 2026-11-11)

§7.39 gave the focus to a walkable the pointer came onto unless the pointer had not moved, and judged "moved" by the
position the walkable's own `FrameController` last saw. A controller made under a resting pointer had seen none, so its
first walkable under the pointer took the focus without the pointer moving. The program met it with a mouse left
resting while the pad was used: Start opens the Home card, whose frame has a controller of its own (`MenuCard.qml`), and
the row the card slid under the pointer took the focus in place of the first row; Home rebuilt after a session did the
same with whatever tile came there.

Whether the pointer has moved is now one fact for the window, not one for each controller. Qt raises a hover in two
ways: as a real pointer event is delivered to the window (a move, a press, a touch), and as items move under a pointer
at rest, when the window syncs a frame (`QQuickDeliveryAgent`'s frame-synchronous hover). Only the first is the pointer
moving. The kit's glue keeps a `PointerWatch` for each Qt Quick window, an event filter that is the window's child
(`kit_glue.cpp`): it marks the window moving as a real pointer event reaches it and clears the mark when the event loop
next turns, so the hovers delivered within that event see it and a frame's hovers later do not. `hoverAt` focuses only
while its item's window is moving. Each `FrameController` starts the watch of every Qt Quick window that has none as it
is made, before any hover in them, and the first one made also connects the glue to `QGuiApplication`'s
`focusWindowChanged`, so that a window made later is watched as it first takes the focus. Qt gives a window the focus
as it is shown; in kit-gallery's `pointer` (offscreen, 2026-11-11) a probe printed
the focus going to `scene` and then to `otherWindow` as the second window opened, and the pointer's first move in that
window, onto a walkable, focused it. Qt announces no window's making to code that did not make it: the window's own
events (show, expose) reach only its own filters and the application's, and an application-wide event filter that
started a window's watch at its first pointer event was tried on 2026-11-11 and not kept, since Qt calls application
filters only on the process's first thread and the test harness runs Qt on another (in kit-gallery's `pointer` it was
never called). So a window shown without taking the focus (one that refuses it, or a compositor that keeps the focus
elsewhere) is still unwatched at its first hover; unwatched, it cannot tell a move from a frame's hover, so it answers no
and starts the watch: its first hover takes nothing, and later moves focus. The connection's owner is a child object of
the application found by its name, as the watch is, not a static. An unwatched window answered yes until 2026-11-11,
so the first walkable that came under a resting pointer in it took the focus. The watch is no static: it is found by its object name among the window's
children.

The ES-DE view asks the same fact. Its carousel, grid and text list select the row under the pointer from their own
`HoverHandler`s (EmuSen_Qt.md §4.8.5), and a row that came under a resting pointer, as a gamelist is made under it, was
selected like a moved-onto one. `EsdeModel.hoverAt`, which the view's QML now calls in place of `hover(row)` (deprecated in EmuSen.Esde 1.1, EmuSen_Qt.md §4.8.3), takes the row's item and selects only while
`emusen_kit_qt::frame::pointer_moving` says that item's window is moving; a hover off every row is passed on as before.
`EsdeModel` starts the watch of every window as it is made, as a `FrameController` does, through
`frame::watch_windows`. The two are the kit's Rust functions over the glue, not copies of it; neither is in the QML
registry.

The watch is the window's rather than the application's: an application-wide filter, tried first, was installed and
called for none of the moves under the test harness, which runs Qt on a test's thread rather than the process's first
(measured on 2026-11-11 with kit-gallery's `pointer`); the window's own filter sees them on any thread.

`pointerMoved(at)` and the controller's saved position are no longer needed: the invokable stays in the 1.x registry,
does nothing, and the kit's QML no longer calls it (`Walkable.qml`, `Carousel.qml`); taking it out is a removal, a major
change of the registry. `hoverAt`'s `at` is likewise unread.

Held by kit-gallery's `pointer` (`PointerScene.qml`): with a card's row holding the focus and the pointer resting, a
walkable of a controller made under it (a `Loader` made active, as Start makes the Home card) takes nothing, a walkable
enabled under it takes nothing, Qt Quick gives a click on a disabled walkable no tap (§7.107), and the pointer's own move onto a walkable
focuses it; and in qt-app by `cards` on the software tier, where neither controller has seen the pointer before: the
pointer moved over Home to where the Home card's second row lies and left there, the card Start opens focuses its first
row; and from Consoles, the pointer moved to where another of Home's places lies and left there, Home shown again by B
focuses Games, not that place. On the GPU, `console` holds a controller that last saw the pointer elsewhere (the List
card's second row hovered, the pointer moved a little along it during a game and left there; the List card opened over
the grid with the game suspended focuses its first row, Down moving), and `esde` the Home card over the ES-DE theme after
a game session under a resting pointer (Continue focused, Down and Up moving).

kit-gallery's `pointer` also opens a second window after every controller, two walkables of the scene's controller in
it, and moves the pointer there with qt-test's `Scene::hover_in`: the pointer's first move in the window, onto one
walkable, focuses it; the other, shown under the resting pointer, takes nothing, and the pointer's move off and onto it
focuses it. esde-qt's `pointer`, offscreen: the pointer's move onto a gamelist's
fourth row selects it; Down to the fifth; back to the systems and into the next one's gamelist, which opens on the
fifth row and is made under the pointer at rest over its fourth: the fifth stays selected, and the pointer's move onto
the sixth selects it.

Seeded faults: `hoverAt` focusing on every hover (`pointer` fails at the late card); the watch never moving (`pointer`
fails at the move onto a walkable); an unwatched window answering yes (fault 48, `pointer` fails at the second window); the window shown
later not watched as it takes the focus (fault 54, `pointer` fails at the first move in the second window);
the ES-DE view's hover not asking the fact (fault 47, esde-qt's `pointer` fails, the fourth row selected); and the rule 0efa4a08f had, each controller judging "moved" by the position it last
saw (fault 44, measured on 2026-11-11: `cards` fails at the Home card, "focuses its first row, Tools, not Settings"; with
that assert set aside, the Home check fails alone, "focuses Games, not place.1"); and the same rule against qt-app's
`console` on the GPU (fault 45, run on 2026-11-11 on 7d3b6571b: `console` panics at its List card made under the resting
pointer, "focuses its first row", and the binary aborts). `cards` reaches Home through Consoles.

