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
| Contrast: words drawn over what is behind them reach 4.5:1, or 3:1 for large type (24 px, or 18.66 px bold), over all but a spare tenth of what is behind them (since 2026-11-08, §7.10) | `contrast` | WCAG 2.1, 1.4.3 |

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
  C++ function of this step, 16 lines: a `QKeyEvent` press and release sent to `QGuiApplication::focusWindow()`, which
  Qt Quick delivers to the active focus item). B declined at the top also raises `backDeclined`, the host's signal to
  close. The player's A/B swap is applied once, in `PadRouter` (§7.4); the walk's own `set_swap_accept_back` stays off,
  so the fact has one owner.
- **The pointer.** `Walkable.qml`, the module's first component, is the §1.7 declaration made once: an `Item` with a
  `HoverHandler` calling `controller.hover(item, hovered)` and one `TapHandler` for the left, right and back buttons
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

A QML singleton whose queue tests push `PadEvent`s into, as the real reader will from the platform client later: `push
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
vertex layout against §1.4's budget. Qt Quick Shapes is the scene graph's own vector item: on a GPU its geometry
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
  `rate` is 48 000. Playback through Endymion's interface player and the pairing of Moonrise's `strike_times` with the
  chime come with the platform client (§7.7).

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
  `qWait`, `qWaitForWindowExposed`, `grabWindow`, a property read, the singleton), 241 lines. QTest's mouse, keyboard and
  window helpers are declared only under `QT_GUI_LIB`, which CMake defines and cc-rs does not, so the file defines it.

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
(`objectName`, `font.family`, `source`, `property`, `properties`, `sequence`, `Accessible.role`, `placeholderText`) and
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
gallery, then visits 72 states: both variants, three forms, three sections, and the four sizes of the harness's
`SIZES` (1280 × 800, 1920 × 1080, 1280 × 720, 3840 × 2160). In each state it focuses the first operable element, reads
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
  was red at (175, 145)); the GPU clips. So the carousel's worlds past its box, hidden on the GPU, were drawn over the
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
and it is writable, so a binding is written only where a press already broke it) are written back where they differ;
a path's offset is not written, since it follows its current index by the view's own animation, and writing it
mid-move was measured to leave the carousel between cards; the views are settled instead (below). A focus scope's
remembered item is given back with `setFocus`, which moves no focus outside the scope. Found on 2026-11-08: the dark
and the light Controls state at 1280 × 800 differed by six focus edges, because the first state's walk had pressed
Right on the tab bar (changing the tab) and Right from the combo box (leaving its row remembering the text field), and
every later state began there; it showed at one size only because that state is visited first.

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

Seen in every run since step 1, the debug and release gates alike and before any change of this section, and not part
of it: when the question opens, Qt logs `RangeError: Maximum call stack size exceeded` twenty times from `Sheet.qml`
and `Frame.qml`. The gate passes over it; it is owed to the sheet's focus handling (§7.7).

**Not written, and why.** `focus` (focusable): the dump has one fact, `operable`, and writing it twice would be an
invention. `sideways`, `live`, `element`, `volatile`, `part`, `title`, `footer`, `walk` and `entered`: nothing in the
kit has them yet, and the walk starts from every operable item, so it is never truncated. The grammar's presses after
the walk are owed to stage 1, where each screen's test says what its presses observe.

**Two toolkits' facts, read side by side.** A Qt `Text` is not hit-testable where Avalonia's `TextBlock` is, so C#'s
overlap rule, which reads drawn items without `nohit`, passes over words that kit-core's overlap rule still judges;
the Qt slack is kit-core's 1.5 above a unit of 1, where the C# recordings at 1920 × 1200 write 2; and `layout` and `box`
are scene pixels, where the C# writes the control's own. None changes a comparison (`InterfaceSpec.Compare` reads names,
roles, words, rectangles and the focus graph), and each is the toolkit's fact rather than one bent to match.

**The C++** grows by 59 lines (`harness.cpp` 420, `harness.h` 28): four facts in the dump (enabled, the item's scale in
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

**Rule Q.** `Testing/qmltypes/EmuSen.Kit-1.0.qmltypes` and `EmuSen.Kit-1.0.qmldir` are the registry as
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

**Hand-written C++** (`EmuSen_Qt.md` §1.4's tracked count, the lines of the files as `wc -l` counts them): 23 in the
kit (`cpp/kit_glue.h` and `cpp/kit_glue.cpp`, all of it `send_key`) and 389 in the harness (`cpp/harness.cpp` 365,
`cpp/harness.h` 24; §7.9, §7.10), **412 in all** on 2026-11-08 with the contrast sampling and the picture difference. Counted the same way, step 9's figure was 277 (the
harness 254), not the 264 first written here. Budget: under 600 for the whole front end. With §7.11's 59 lines,
`cpp/harness.cpp` is 420, `cpp/harness.h` 28, and the front end **471**; with the polish wait's include, 421 and **472**.
The C++ of §7.11 is there because each piece is a Qt call cxx-qt-lib does not bind: `QAccessible::queryAccessibleInterface`,
`QMetaProperty`'s declaring class, `QQuickItem::scopedFocusItem` and `setFocus`, a property by an item's address, and
`QQuickTest::qWaitForPolish`.

### 7.7 Open

- **[routine]** Install Fedora's `lld` for the toolchain, as §10.4 proposed; both linker workarounds then go.
- **[routine]** The interface's typeface is not a token in kit-core (§1.3), so `Tokens` has no font family. Recommended:
  add `fontDisplayFamily` and `fontBodyFamily` to kit-core's tokens with their values read from `EmuSen_Interface.md`
  §5.2 when the first screen needs them, the gallery included, and bundle the faces as §10.2 found necessary.
- Owed by §7.5 to a later step: the cues played through Endymion's interface player and Moonrise's strikes paired with
  the chime, both with the platform client; the stars' per-tick cost measured on the handheld before stage 1 lands.
- Owed by §7.8: `Keyboard` (the docked pad keyboard), `RatingPicker` and `DatePicker`; the drawn
  glyphs (the pad's buttons for the help bar, a check, a radio dot, a chevron) in place of the rings, discs and
  rectangles the style draws now; the desktop form's menu bar and status bar.
- Owed by §7.9 to stage 1: a row whose control takes Up and Down itself (a `TextField`, a `ComboBox`) cannot be passed
  by a vertical walk; the rule (the row walks first, the control takes the keys once chosen, as a console's settings
  do) is decided with the docked `Keyboard`. Until then a fixture declares its path with `KeyNavigation` on the control.
- Decided 2026-11-08 by the tester: the approved draft is Moonrise's oracle, so its caption stays at the draft's 15
  units as the audit's named exception to the floor (§7.10).
- Owed by §7.11: the question's opening logs `RangeError: Maximum call stack size exceeded` from `Sheet.qml` and
  `Frame.qml` twenty times, in every gate run since 54f2dc43c; a recursion between the sheet's focus and the frame's,
  read and fixed in the kit, not the gate.
- Owed by §7.10 to stage 1: the frame raises `focusMoved` when the walk's focused role changes (B on the desktop's
  content gives the navigation the focus) but no item takes Qt's focus, so the walk's role and Qt's focus part there;
  the gallery's sections are pointer targets and shoulders until the desktop form's navigation is built. The help
  bar's glyphs are empty circles until the pad's button glyphs are drawn (above). The other screens' faces (§5.2) are
  bundled when the first screen needs them; Moonrise's two are.
