# EmuSen.Kit changes

The kit is the QML module `EmuSen.Kit` (EmuSen_Qt.md §1.4), held by rule Q (§5.3): its registry, as qmltyperegistrar
writes it, is committed under `Testing/qmltypes/` and compared on every build. Until the first release every change, an
addition, a change or a removal, bumps the minor (R-20261111-107, R-20261111-108); each entry names what changed and the
section of `EmuSen_Kit.md` that records it.

## 1.4: pictures decoded at the pixels they cover, and the design height's one owner (2026-11-14)

Additions only; no type, property, signal or invokable was removed or changed.

- **`DrawnPixels`** (registry, C++): the device pixels its box covers in its window, `pixels`, through every ancestor's
  scale and the window's pixel ratio. `HelpBar`'s theme picture, `PdfPage`'s page and `ControllerDrawing`'s drawing
  decode at it. `EmuSen_Kit.md` §7.111 (R-20261114-5).
- **`CanvasUnits`** (registry, singleton): `designHeight`, a constant, kit-core's `canvas::DESIGN_HEIGHT`;
  `NormalizedCanvas.designHeight` binds to it. `EmuSen_Kit.md` §7.112 (R-20261114-7).
- The minor is 1.4, one above WiseMan's 1.3 (R-20261111-107); the registry, qmldir and QML-file surface are
  `Testing/qmltypes/EmuSen.Kit-1.4.*`, each 1.3 file with these added.

## 1.3, stage 5 piece P5: a page of a PDF (2026-11-12)

Additions only; no type, property, signal or invokable was removed or changed.

- **`PdfPage`** (qmldir): one page of a PDF drawn to its box through Qt PDF, with `source`, `page`, and the file's
  `pages` and the page's `pageSize` in points; the module depends on `QtQuick.Pdf`. `EmuSen_Kit.md` §7.110.
- The minor is 1.3, one above WiseMan's 1.2 (rule Q, EmuSen_Qt.md §5.3; R-20261111-107); the registry, qmldir and
  QML-file surface are `Testing/qmltypes/EmuSen.Kit-1.3.*`, each 1.2 file with these added.

## 1.2, stage 5 piece H: the controller drawing and the settings frame's page of content (2026-11-09; 1.2 on 2026-11-12)

Additions only; no type, property, signal or invokable was removed or changed.

- **`ControllerLayout`** (registry): a drawing file and the controls a host declares in; the drawing's place, scale and
  picture, the labels' boxes and names, the leaders and the walk out. `EmuSen_Kit.md` §7.62.
- **`ControllerLayout.texts`, `labelWords` (told by `worded`), `labelBoxes`; `accent` a colour** (registry): each label's words and box as
  properties a delegate binds, so `ControllerDrawing` keeps the QML rules (no function, nothing done on completion, no
  words of its own) and qmllint takes the token's colour. §7.62.
- **`ControllerLayout.pushes`** (registry): how far each control is pushed, in the controls' order; a stick's knob
  stands where its directions push it (kit-core's `controller::knob`). §7.62.
- **`ControllerDrawing`** (qmldir): the controller's picture, a label for each control with its leader, the focused one
  ringed; `pushes` beside `held`. §7.62.
- **`SettingsFrame.pageShown`, `pageContent`, `pageSentence`; the object names `settingsContent` and
  `settingsContentSlot`** (qmldir): a page of content in place of the rows, and the pane's sentence while it is shown. §7.63.
- The minor is 1.2, one above WiseMan's 1.1, for these additions (rule Q, EmuSen_Qt.md §5.3; R-20261111-107); the
  registry, qmldir and QML-file surface are `Testing/qmltypes/EmuSen.Kit-1.2.*`, each 1.1 file with these added.

## 1.1, stage 4 piece X: the help bar's pair glyph, the clip's player and fade, the page's clip volume (2026-11-11)

Additions only; no type, property, signal or invokable was removed or changed. Rule Q now holds the QML files' own
surface too (R-20261111-99): each root object's properties and signals and every object name, read by
`emusen_qt_test::qml_surface` and committed as `Testing/qmltypes/EmuSen.Kit-1.1.qmlfiles`. The minor moved for these:

- **`PadGlyphMarks`** (registry): `pair`, `paired`, `discShare` and `setPair`; **`VideoFrames`** (registry): `fitted`
  and `stretched`. `EmuSen_Kit.md` §7.71, §7.53.
- **`HelpBar`**: `pairGlyph`, `label`. **`PadGlyph`**: `pair`, `paired` (read-only). §7.71.
- **`VideoClip`**: the object name `videoPlayback`; `fillMode`, `delay`, `fade`; read-only `position`, `heard`, `level`.
  **`VideoTile`**: `fillMode`, `delay`, `fade`. §7.53.
- **`PageFrame`**: `clipVolume`, the share of full its clip is heard at (R-20261111-100). §7.53.
- **`SectionStrip`**: read-only `overflows`, added on WiseMan in repair DC-B (§7.105) while the kit was 1.0 and held
  in this minor's QML-file surface once the two met.
- The minor is 1.1, one above WiseMan's 1.0: until the first release every interface change bumps the minor (decided
  2026-11-11 by the tester, R-20261111-107).

## 1.0, stage 4 piece H: a model's reset told to a view without QML state (2026-11-11)

Additions only; no type, property, signal or invokable was removed or changed.

- **`ResetGate`** (registry): `model` (a list model) and `settled`, false from the model's `modelAboutToBeReset` to its
  `modelReset`; a model handed over is settled and the old one is no longer heard; a model deleted while held is let go,
  `model` then null. `EmuSen_Kit.md` §7.58.
- `PageFrame`'s two lists hold their models through two of them in place of its QML flags, `Connections` and
  `Qt.callLater`, which the QML rules forbid; what it shows is unchanged. §7.58.

## 1.0, repair DC-B: the pointer's fact the window's, the section strip a scroller where it overflows (2026-11-11)

No type, property, signal or invokable was added, removed or changed in the registry.

- `FrameController.hoverAt` focuses only while its item's window is delivering a real pointer event, a fact each Qt
  Quick window keeps once; `pointerMoved` stays in the registry and does nothing, and the kit's QML no longer calls it.
  `EmuSen_Kit.md` §7.108.
- `SectionStrip` is interactive only while its names run past it (its read-only `overflows`, the one place the condition
  is written), and neither a mouse drag nor a finger's drag moves it; where its names fit a finger's drag is the page's.
  §7.105.
- Where its names run past it, `SectionStrip` takes a finger's sideways drag only: an upright one that starts on it is
  the page's, and a short tap still chooses a name. §7.105.
- The window's fact answers no for a window whose watch no controller started, and the kit's Rust gives it to the ES-DE
  view (`frame::pointer_moving`, `frame::watch_windows`), whose hovers ask it as `hoverAt` does. §7.108.

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

## 1.0, stage 4 piece X7: the filled disc's share of its box (2026-11-11)

Additions only; no type, property, signal or invokable was removed or changed.

- **`glyph::FILLED_DISC_SHARE`** (Rust) and **`PadGlyphMarks.discShare`** (registry, read-only, constant): the share of
  its box the filled set's disc fills, from the radius it is drawn with. `EmuSen_Kit.md` §7.71.

## 1.0, stage 4 piece X2: the body face as bytes, and a fitted clip started at once (2026-11-09)

Additions; every default keeps a tile as it was.

- **`fonts::BODY`**, **`fonts::BODY_URL`** (Rust): the body face's bytes and its resource path, for a component that lays
  words out itself. `EmuSen_Kit.md` §7.60.
- **`VideoTile.fillMode`**, **`VideoTile.delay`**, **`VideoClip.fillMode`**, **`VideoClip.delay`** (qmldir): fitted
  inside the tile, and started without the poster's delay. §7.61.
- **`VideoFrames.fitted`** (registry): the software backend's frames fitted inside the item. §7.61.

## 1.0, stage 5 piece E: a sentence's tone, a value beside a short label, the rows' view (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`SettingsFrame.sentenceTone`** (qmldir): the tone (`Tones`) of the focused row's sentence, `Text` unless the host
  says otherwise. §7.52.
- `MenuRow`'s value takes what its label leaves of the row, at least half; its surface is unchanged. §7.50.
- `SettingsFrame`'s rows keep no current index of the view's own; its surface is unchanged. §7.51.

## 1.0, stage 4 piece X: a pair as one glyph, and a host's words on the help bar (2026-11-09)

Additions only; no type, property, signal or invokable was removed or changed.

- **`PadGlyphMarks.pair`** and **`paired`** (registry): a second press drawn with `button` as one glyph where kit-core
  has one for the two (a d-pad axis, the shoulders, the triggers), and whether it is. §7.54.
- `PadGlyph.pair` and `paired`; `HelpBar.pairGlyph` (false by default) and `HelpBar.label` (null by default), QML
  properties of the qmldir's types. §7.54.

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
