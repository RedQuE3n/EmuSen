# EmuSen.Esde changes

The ES-DE painter is the QML module `EmuSen.Esde` (EmuSen_Qt.md §4.8), held by rule Q (§5.3): its registry, as
qmltyperegistrar writes it, and its QML files' surface (R-20261111-99) are committed under `Testing/qmltypes/` and
compared on every build. Each entry names what changed and the section that records it.

## 1.1, stage 4 piece X: the help bar, the text rules and the media viewer (2026-11-11)

- Registry, added: `EsdeModel.defaultFont`, `selectionSeed`, `reference` and `manualPages`; `MarkShapes.aspect` and
  `baseline`; the type `TextLayout`. §4.8, §4.9, §4.13.
- QML files, added: the types `GlyphLine`, `GlyphText`, `ClipElement` (with `volume` and `fade`) and `ManualElement`;
  `ownFace` on every element that draws words; `TextElement.everyLine` and `snapRows`; `HelpElement.rowHeight`,
  `barWidth`, `barStart`, `discEdge`, `faceName` and the object names `help-`, `helpPicture-` and `helpGlyph-` with the
  entry's label; `Mark.aspect` and `baselineShare`; `View.defaultFont` and `selectionSeed`; `TextBlock.contentWidth`,
  `contentHeight`, `wholeRows`, `windowHeight`, `measuredHeight` and `baselineY` as numbers. §4.8.7, §4.9, §4.12, §4.14.
- QML files, removed or changed: `TextBlock.pitch` and `halfLeading`, and `contentWidth` and `contentHeight` as aliases;
  `HelpElement.heightShare`, `inkShare`, `lineHeight` and `glyphSide`, and `labelPx` from read-only to required. Rule Q
  asks the major for these; whether the module goes to 2.0 waits on the tester, and the number is settled when this
  lands beside the other 1.1.

## 1.0 (2026-11-08)

The module as first committed: the primitives, the elements, `View`, and the models of §4.8.
