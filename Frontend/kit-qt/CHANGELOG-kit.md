# EmuSen.Kit changes

The kit is the QML module `EmuSen.Kit` (EmuSen_Qt.md §1.4), held by rule Q (§5.3): its registry, as qmltyperegistrar
writes it, is committed under `Testing/qmltypes/` and compared on every build. A minor adds and never removes; each
entry names what was added and the section of `EmuSen_Kit.md` that records it.

## 1.0, stage 3: an orbit deleted with its screen (2026-11-09)

No type, property, signal or invokable was added, removed or changed.

- `OrbitSlot` reads its `view` as possibly null, so covers outliving their deleted view log nothing. §7.48.

## 1.0, stage 5 piece A: the settings frame, the page of choices and the slider row (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`SettingsLayout`** (registry): the settings frame's measures as constant properties (`side`, `stripTop`,
  `stripHeight`, `stripGap`, `tabPadding`, `contentTop`, `paneWidth`, `paneHeight`, `paneInset`, `slotGap`, `lineGap`,
  `rowsShown`), `paneLeft` and `rowsWidth` from `width`, and the walk over a model's rows: `model`, `sections`,
  `section`, `trailShown`, `count`, `current`, `first`, `marked`, `sentence`; the signals `laidOut`, `walked`,
  `sectionChosen`, `stepped`, `slid`, `chosen`; the invokables `read`, `press`, `key`, `focusRow`, `scrollTo`.
  `EmuSen_Kit.md` §7.40.
- **`RowKinds`** (registry): `Action`, `Steps`, `Opens`, `Asks`, `Slider`, `Choice`, a settings row's kind. §7.40.
- **`SettingsFrame`** (qmldir): the frame of six sections or a trail, rows from a model and the pane with one slot.
  §7.40.
- **`ChoiceRow`** (qmldir): a row of a page of choices, the choice in force marked. §7.41.
- **`SliderRow`** (qmldir): a row with a slider, stepped by the pad and dragged by the pointer. §7.42.
- `MenuRow`'s ◂ ▸ and ▸ marks are no longer painted past a clip on the software backend; its surface is unchanged. §7.40.

## 1.0, stage 5 piece D: a long value on a row, and the text sizes (2026-11-09)

One property added; nothing removed or changed.

- `MenuRow`'s value is at most half the row and cut at its start, so a long path no longer squeezes the label out.
  `EmuSen_Kit.md` §7.46.
- **`Tokens.textSize`** (registry): a text size by the setting's word, which sets `textScale` to the kit's scale for it
  (kit-core's `TEXT_SIZES`). An addition; `textScale` is unchanged. §7.47.

## 1.0, stage 5 piece B: the tool screen and the page frame (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`ToolLayout`**, **`PageLayout`**, **`ColumnFit`** (registry): the tool screen's and the page frame's places, and a
  table's columns kept inside its width. `EmuSen_Kit.md` §7.43, §7.44.
- **`Tones`** (registry): the tones a frame's words take. §7.43.
- **`ToolScreen`**, **`PageFrame`**, **`SectionStrip`**, **`ActionButton`**, **`ToneText`** (qmldir): the tool screen,
  the page frame, the section strip with its search, an action on the frame, and words in a tone. §7.43–§7.45.
- `EmuSen.Theme` re-exports none of them, since a theme never styles a tool or a game's page (`EmuSen_Qt.md` §3.1).
