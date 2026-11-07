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
