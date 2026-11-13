# EmuSen.Esde changes

The ES-DE painter is the QML module `EmuSen.Esde` (EmuSen_Qt.md §4.8), held by rule Q (§5.3): its registry, as
qmltyperegistrar writes it, and its QML files' surface (R-20261111-99) are committed under `Testing/qmltypes/` and
compared on every build. Each entry names what changed and the section that records it.

## 1.3, piece P4: a theme's video element plays (2026-11-12)

- Registry, added: the `ElementModel.Video` paint kind. §4.5.6.
- QML files, added: the type `VideoElement`, a theme's video element over its still; `Picture.brightness`. §4.5.6.
- The module depends on `QtMultimedia`, which `VideoElement` imports itself; `ClipElement` reached it through the kit.

## 1.2, stage 4 piece X: the help bar, the text rules and the media viewer (2026-11-11)

- Registry, added: `EsdeModel.defaultFont`, `selectionSeed`, `reference` and `manualPages`; `MarkShapes.aspect` and
  `baseline`; the type `TextLayout`. §4.8, §4.9, §4.13.
- QML files, added: the types `GlyphLine`, `GlyphText`, `ClipElement` (with `volume` and `fade`) and `ManualElement`;
  `ownFace` on every element that draws words; `TextElement.everyLine` and `snapRows`; `HelpElement.rowHeight`,
  `barWidth`, `barStart`, `discEdge`, `faceName` and the object names `help-`, `helpPicture-` and `helpGlyph-` with the
  entry's label; `Mark.aspect` and `baselineShare`; `View.defaultFont` and `selectionSeed`; `TextBlock.contentWidth`,
  `contentHeight`, `wholeRows`, `windowHeight`, `measuredHeight` and `baselineY` as numbers. §4.8.7, §4.9, §4.12, §4.14.
- QML files, added: the object name `viewClock` under `View`, the inner object that holds the scene's clock
  (`clockMs`, `tickedMs`), which the view's root does not offer (decided 2026-11-12 by the tester, R-20261112-20). §4.8.4.
- QML files, removed or changed: `TextBlock.pitch` and `halfLeading`, and `contentWidth` and `contentHeight` as aliases;
  `HelpElement.heightShare`, `inkShare`, `lineHeight` and `glyphSide`, and `labelPx` from read-only to required.
- Registry, removed: `EsdeModel.hover(row)`, deprecated in 1.1 (R-20261111-98) and removed in this minor (decided
  2026-11-11 by the tester, R-20261111-108: before the first release a call deprecated in one minor may go in any later
  one); `hoverAt(item, row)` is the call. Its test of the deprecated behaviour went with it; the lint stays, as
  `qt_5_3_the_removed_hover_is_called_by_no_qml_and_offered_by_no_registry` in `tests/lint.rs`, holding that no QML
  calls `hover` and the registry offers none.
- The minor moved to 1.2 for all of these: until the first release an addition, a change and a removal each bump the
  minor (decided 2026-11-11 by the tester, R-20261111-107, EmuSen_Qt.md §5.3).

## 1.1, repair DC-B: the view's hover asks the window's pointer fact (2026-11-11)

- **`EsdeModel.hoverAt(item, row)`** (registry, added): the pointer onto the row `item` draws, or off every row (-1);
  it selects only while the item's window is delivering a real pointer event, so a gamelist made under a resting
  pointer keeps its selection. The view's carousel, list and grid call it. EmuSen_Qt.md §4.8.4, EmuSen_Kit.md §7.108.
- **`EsdeModel.hover(row)`** (registry, kept, **@deprecated**): it selects the row under a resting pointer as under a
  moving one, as in 1.0, and nothing in the repository calls it once the view's QML calls `hoverAt`. It stays in 1.1
  doing what it did; it was removed in 1.2 (R-20261111-108). Qt 6.11's `qmltyperegistrar` writes no deprecation into
  the registry, so the note is this entry and the invokable's doc comment in `src/model.rs`.
- The minor moved to 1.1 for the addition. The C++ types keep their `exports` at 1.0 in the 1.1 registry: an export
  is the version a type was first offered at, and no type was added; rule Q's version is the qmldir's.

## 1.0 (2026-11-08)

The module as first committed: the primitives, the elements, `View`, and the models of §4.8.
