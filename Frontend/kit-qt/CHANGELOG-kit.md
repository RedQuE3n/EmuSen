# EmuSen.Kit changes

The kit is the QML module `EmuSen.Kit` (EmuSen_Qt.md §1.4), held by rule Q (§5.3): its registry, as qmltyperegistrar
writes it, is committed under `Testing/qmltypes/` and compared on every build. A minor adds and never removes; each
entry names what was added and the section of `EmuSen_Kit.md` that records it.

## 1.0, stage 5 piece A: the settings frame, the page of choices and the slider row (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`SettingsLayout`** (registry): the settings frame's measures as constant properties (`side`, `stripTop`,
  `stripHeight`, `stripGap`, `tabPadding`, `contentTop`, `paneWidth`, `paneHeight`, `paneInset`, `slotGap`, `lineGap`,
  `rowsShown`), `paneLeft` and `rowsWidth` from `width`, and the walk over a model's rows: `model`, `sections`,
  `section`, `trailShown`, `count`, `current`, `first`, `marked`, `sentence`; the signals `laidOut`, `walked`,
  `sectionChosen`, `stepped`, `slid`, `chosen`; the invokables `read`, `press`, `key`, `focusRow`, `scrollTo`.
  `EmuSen_Kit.md` §7.36.
- **`RowKinds`** (registry): `Action`, `Steps`, `Opens`, `Asks`, `Slider`, `Choice`, a settings row's kind. §7.36.
- **`SettingsFrame`** (qmldir): the frame of six sections or a trail, rows from a model and the pane with one slot.
  §7.36.
- **`ChoiceRow`** (qmldir): a row of a page of choices, the choice in force marked. §7.37.
- **`SliderRow`** (qmldir): a row with a slider, stepped by the pad and dragged by the pointer. §7.38.
- `MenuRow`'s ◂ ▸ and ▸ marks are no longer painted past a clip on the software backend; its surface is unchanged. §7.36.
