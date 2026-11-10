# EmuSen.Kit changes

The kit is the QML module `EmuSen.Kit` (EmuSen_Qt.md §1.4), held by rule Q (§5.3): its registry, as qmltyperegistrar
writes it, is committed under `Testing/qmltypes/` and compared on every build. A minor adds and never removes; each
entry names what was added and the section of `EmuSen_Kit.md` that records it.

## 1.0, stage 5 piece H: the controller drawing and the settings frame's page of content (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`ControllerLayout`** (registry): a drawing file and the controls a host declares in; the drawing's place, scale and
  picture, the labels' boxes and names, the leaders and the walk out. `EmuSen_Kit.md` §7.62.
- **`ControllerDrawing`** (qmldir): the controller's picture, a label for each control with its leader, the focused one
  ringed. §7.62.
- **`SettingsFrame.pageShown`, `pageContent`, `pageSentence`** (qmldir): a page of content in place of the rows, and the
  pane's sentence while it is shown. §7.63.

## 1.0, stage 5 piece K: the raised shadow as a token, and the item that casts it (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed. The new properties are declared after
every earlier one, so no earlier property's index in the registry moved.

- **`Tokens.shadowColor`, `shadowOpacity`, `shadowOffsetX`, `shadowOffsetY`, `shadowBlur`** (registry): the one shadow of
  `EmuSen_Interface.md` §5.4, black at 50%, offset 0,8, blur 24, from kit-core's `tokens::raised_shadow`. `EmuSen_Kit.md`
  §7.55.
- **`RaisedShadow`** (qmldir): the shadow cast by a raised shape of the item's box and `radius`, through `MultiEffect` on
  the GPU and Qt's `Canvas` on the software backend. §7.55.
- `PageFrame`'s cover casts it; `NoticeLayer`'s `shadowOpacity` and `shadowRadius` default to the tokens and its offset
  and colour are theirs, so what it draws is unchanged. §7.55.

## 1.0, stage 5 piece G2: the page frame's focus (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`PageFrame.focusFirstAction()`** (qmldir, a signal a host emits): the focus on the first action, as a page opens.
- **`PageFrame.contentKindChanged()`** (qmldir, a signal): the content's kind changed; with the focus in the content it
  goes on to the list now shown, on its first item, or to the actions where words are shown. §7.58.

## 1.0, stage 5 piece E: a sentence's tone, a value beside a short label, the rows' view (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`SettingsFrame.sentenceTone`** (qmldir): the tone (`Tones`) of the focused row's sentence, `Text` unless the host
  says otherwise. §7.52.
- `MenuRow`'s value takes what its label leaves of the row, at least half; its surface is unchanged. §7.50.
- `SettingsFrame`'s rows keep no current index of the view's own; its surface is unchanged. §7.51.

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

## 1.0, stage 5 piece G: the clip, and a game's page's parts of the page frame (2026-11-09)

Additions, and one contract change to `PageFrame`'s action model.

- **`VideoTile`** (qmldir): a clip over its still, after the poster's delay, silent unless unmuted; Qt Multimedia over
  the system's FFmpeg. `EmuSen_Kit.md` §7.50.
- **`VideoClip`** (qmldir): VideoTile's player, made only while a clip is to play. §7.50.
- **`VideoFrames`** (registry): a clip's frames drawn on the software backend, where `VideoOutput` draws nothing. §7.50.
- **`PageLayout.pictureWidth`**, **`pictureHeight`**, **`pictureGap`** (registry): the Overview's picture. §7.50.
- `PageFrame`: `clipSource`, `clipPlaying`, `clipMuted`, `videoTile`; `showsText`, `contentWords`, `contentPicture`; the
  signals `actionStepped` and `rowStepped`. `ActionButton`: `stepped`. §7.50.
- Changed: `PageFrame`'s action model must give the roles `stepped`, `stepLeft` and `stepRight`. §7.50.
- The module depends on `QtMultimedia`.

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
