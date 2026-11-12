# EmuSen.Esde changes

The ES-DE painter is the QML module `EmuSen.Esde` (EmuSen_Qt.md §4.8), held by rule Q (§5.3): its registry, as
qmltyperegistrar writes it, is committed under `Testing/qmltypes/` and compared on every build. Each entry names what
changed and the section that records it.

## 1.1, repair DC-B: the view's hover asks the window's pointer fact (2026-11-11)

- **`EsdeModel.hoverAt(item, row)`** (registry, added): the pointer onto the row `item` draws, or off every row (-1);
  it selects only while the item's window is delivering a real pointer event, so a gamelist made under a resting
  pointer keeps its selection. The view's carousel, list and grid call it. EmuSen_Qt.md §4.8.4, EmuSen_Kit.md §7.108.
- **`EsdeModel.hover(row)`** (registry, kept, **@deprecated**): it selects the row under a resting pointer as under a
  moving one, as in 1.0, and nothing in the repository calls it once the view's QML calls `hoverAt`. It stays in 1.1
  doing what it did and is removed at the next major, decided 2026-11-11 (rule Q, EmuSen_Qt.md §5.3: a removal moves
  the major, after a minor in which the call is deprecated). Qt 6.11's `qmltyperegistrar` writes no deprecation into
  the registry, so the note is this entry and the invokable's doc comment in `src/model.rs`.
- The minor moved to 1.1 for the addition. The C++ types keep their `exports` at 1.0 in the 1.1 registry: an export
  is the version a type was first offered at, and no type was added; rule Q's version is the qmldir's.

## 1.0 (2026-11-08)

The module as first committed: the primitives, the elements, `View`, and the models of §4.8.
