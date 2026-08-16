# EmuSen.LunaP 0.8.0 — Frontend Adoption Game Plan

*Successor to `EmuSen_LunaP_Gameplan.md`, whose seven phases are done. That document planned building the toolkit and migrating the frontends onto what existed at the time. This one plans adopting what the toolkit has grown since it left the repository — a different question, and worth a separate document because the justification is different too.*

---

## 0. Orientation, if you're new to this

`EmuSen.LunaP` is a shared Avalonia toolkit. It used to live in this repository; it now lives in its own and arrives as a NuGet package (`EmuSen.LunaP`, currently 0.8.0). `EmuSen.Mistress` (the testing studio) and `EmuSen.Hotaru` (the console-style frontend) both consume it, as does `EmuSen.Serenity` and the out-of-tree `EmuSen.Pegasus`.

`EmuSen_LunaP.md` is the reference for what the toolkit *is*. This document is only about what the two frontends should adopt from it next, and in what order.

---

## 1. Why there is new work at all

The first gameplan's Phase 6 migrated the frontends onto the toolkit as it stood, and Phase 7 added a widget set anchored to real consumers. Both finished. **The work here exists because the toolkit kept growing after it left**, and the surface it grew is not speculative — 0.8.0 ships commands, `AppWindow`, `LunaTable<T>`, `IdleCursor`, `FileDrop` and stock-control theming, none of which existed when Phase 6 ran.

That is a genuinely different justification from the first gameplan's, and it should be held to the same standard. Phase 6's discipline was that every widget had to delete a hand-written copy of itself. **Three of the four paths below meet that bar; one does not, and is marked as the speculative one rather than dressed up.**

One path has already been taken, and is the model for the rest:

- **The menu (done, 2026-08-16).** `MainWindow`'s XAML menu and its three sync-on-open handlers became `LunaAction`s; `MainWindow` became a `ToolWindow`. It deleted a hand-written `ICommand`, retired a documented design, and **found two shipping defects in full-screen handling that the old design concealed**. `EmuSen_Settings_Reference.md` §4.12, §4.13 and §4.19 carry it.

That outcome is the argument for the rest of this document: the migrations are worth doing for the deletion, and the defects they surface are worth more than the deletion.

---

## 2. Shape of the paths

Four paths. **A, B and C are mutually independent** — any order, any subset, no shared files except that A and B both touch `ActiveCheatsWindow`. D depends on nothing but is the only one that is not a de-duplication. E is listed to be refused, not taken.

Each path states what deletion validates it, because a LunaP migration with nothing to delete is a rewrite wearing a migration's clothes.

---

### Path A — Typed lists (`LunaList<T>`) ✅ done

*Taken 2026-08-16. Three of the four sites below migrated; the fourth could not, for a reason this section had not checked. `EmuSen_Settings_Reference.md` §4.11a is the record. What it turned out to teach:*

- ***`Chose` is a selection change, not an activation.*** *`RomBrowserWindow` was first wired `Chose += entry => Close(entry.FullPath)`, which would have closed a modal on one click where it had always taken a double-click. Caught by measuring the contract before trusting it, not by review.*
- ***`LunaList<T>` is `where T : class`.*** *`CheatDatabaseWindow.GamesList` holds a `readonly struct` and is therefore not migratable without reshaping a DianaOS type used by four other call sites. The plan below claimed four sites; there were three.*
- ***The library's `SelectedIndex = 0` was load-bearing***, *not incidental — `FilterBar.Submitted` launches the top match. §5's open question is answered by the code rather than by preference, and the behaviour was kept.*
- ***`x:TypeArguments` works and is the right choice***, *because a list built in code is outside the XAML namescope and `GetControl` cannot find it. Every existing test lookup survived unchanged.*
- ***Measure against the unmodified tree, not against a number in a document.*** *The suite reads 1m5s before and after; an earlier session had recorded ~53s, and comparing to that would have invented a twelve-second regression.*

<details>
<summary>Original plan text</summary>

**The strongest evidence in the codebase, and the one I would take first.**

Four sites hold a collection parallel to a `ListBox` of projected strings and recover the model by indexing with `SelectedIndex`:

| Site | Parallel collection | Recovery |
| --- | --- | --- |
| `Mistress/Views/MainWindow.axaml.cs:62` | `_libraryEntries` | `_libraryEntries[LibraryList.SelectedIndex]` |
| `Mistress/Views/RomBrowserWindow.axaml.cs:24` | `_entries` | `_entries[RomList.SelectedIndex].FullPath` |
| `Mistress/Views/CheatDatabaseWindow.axaml.cs:49` | `_games` | `_games[GamesList.SelectedIndex]` |
| `Mistress/Views/CheatDatabaseWindow.axaml.cs:117` | systems list | `SystemsList.SelectedIndex` into the scan result |

`MainWindow`'s own comment already names the hazard: *"Parallel to LibraryList's item strings, which are titles only."* That is a comment explaining a shape that `LunaList<T>` exists to remove — the list keeps the model type, and `Chose` hands back a `RomEntry` rather than an index.

**What validates it:** four parallel collections deleted, and four index-bounds guards (`if (index < 0 || index >= _entries.Count) return;`) with them.

**A second, quieter win.** `LunaList<T>.Refresh` *"[r]eplaces every row, keeping the selection if `Key` still matches something."* Two sites hand-roll around that today: `MainWindow` resets to `SelectedIndex = 0` on every refresh, and `ActiveCheatsWindow.axaml.cs:196-200` saves the selected id and restores it by hand after rebuilding. The second is a copy of what `Refresh` does natively.

**Constraint checked, not assumed:** `LunaList<T>` is documented as *single-selection*. All four sites are single-selection `ListBox`es today, so nothing is lost. **A fifth site would need re-checking** — this is not a general answer for any list.

**Defects to look for before fixing anything.** Following the menu path's example, write the test against the unmodified window first. The specific thing to probe: `MainWindow.ShowLibraryEntries` forces `SelectedIndex = 0` after a filter change, so a filter that narrows the list moves the selection to an unrelated game. Whether that is a defect or intended is a question for the project owner, and it should be *answered* before `Refresh` silently changes the behaviour by preserving selection instead.

**What this path does not cover:** `ActiveCheatsWindow`'s `CheatsList` is deliberately excluded — it already uses `SelectedItem as CheatRow` rather than an index, so it has no parallel array to delete, and it is Path B's subject for a different reason.

</details>

---

### Path B — The cheat table (`LunaTable<CheatRow>`) ✅ done

*Taken 2026-08-16. `EmuSen_Settings_Reference.md` §4.14a is the record. What it turned out to teach:*

- ***The deletion landed: `CheatRow` is a plain class.*** *The last `INotifyPropertyChanged` in either frontend is gone, and §5's rule about not smuggling one in is true again rather than nearly true.*
- ***A table is not a `ListBox`,*** *so `SelectionChanged`/`SelectedItem` had to become `Chose`/`Selected`. `Select` is silent here as it is on `LunaList`, which means anything selecting in code must update what depends on the selection itself — the Remove button is wired to `Chose` and re-synced at the end of `Refresh`.*
- ***Column headings are the one unavoidable visual change.*** *A table has a header row and a list does not. Recorded rather than slipped in; sorting and remembered widths stay off, per this section's own split.*
- ***It found a LunaP accessibility defect within minutes.*** *A named `LunaTable` contained an anonymous `ListBox`, caught by the consumer's own `AccessibilityTests`. Fixed upstream in `LunaP.md` §78.4 and bridged here until a release carries it.*

<details>
<summary>Original plan text</summary>

**The path with a project-level argument behind it, not just a deletion.**

`ActiveCheatsWindow.axaml:60-85` is a `ListBox` whose `DataTemplate` lays out four columns by hand — a checkbox, a fixed-width kind label, an ellipsised description, and a monospace detail. `Views/CheatRow.cs` is the row model, and it implements `INotifyPropertyChanged` for exactly one reason: so the template's `IsChecked="{Binding Enabled, Mode=TwoWay}"` can write back.

`LunaTable<T>` takes a checkbox column as a projection and a writer — `new LunaColumn<CheatRow>("on", r => r.Enabled, (r, v) => r.Enabled = v, ...)` — with no binding, no `DataTemplate` and no interface on the model.

**What validates it:** the `DataTemplate` and `CheatRow`'s entire `INotifyPropertyChanged` implementation, which is **the only `INotifyPropertyChanged` in either frontend**. The first gameplan's §5 says plainly that this codebase is *"deliberately code-behind with direct control manipulation… no `INotifyPropertyChanged` layer smuggled in under 'framework'."* Removing the one exception is a stated project goal, not a preference.

**What comes free and should be treated as a change, not a bonus:** sorting, resizable and remembered column widths (`TableKey`), and per-column alignment. These are visible behaviour changes to a window that currently has none of them. The first gameplan's Phase 1 rule was *"not a re-skin"* — that rule was about the theme, but the spirit applies: **land the migration behaviour-neutral first, then turn sorting on as its own decision.**

**What this path does not cover:** the cheat *database* window's two lists (Path A) and the master switch, tabs and file buttons, which are already LunaP and already right.

</details>

---

### Path C — Pointer and drop (`IdleCursor`, `FileDrop`)

**The smallest path, the only one that touches Hotaru, and the only one that adds a capability rather than removing a copy.**

Neither frontend has any cursor management — `grep -rn "Cursor" EmuSen.Mistress EmuSen.Hotaru` returns nothing. Both can go full screen. A mouse pointer parked over an emulated game is the ordinary result.

Neither frontend accepts a dropped file — no `AllowDrop`, no `DragOver`, no `DataFormats` anywhere. Dragging a ROM onto the window does nothing.

`IdleCursor` attaches to a control rather than a window, which matters here: Mistress wants it over `GameFrame` and *not* over the menu bar, and that is a distinction a window-level flag cannot express. `FileDrop` removes two silent failure modes (a forgotten `AllowDrop`, a forgotten effect in `DragOver`), neither of which produces an error.

**Both are `IDisposable` and both must be disposed.** A cursor left hidden by an object nobody unsubscribed is an application whose pointer is gone for good — that is the toolkit's own warning and it is the main risk in this path.

**Honest accounting:** this path deletes nothing. It is justified by the two frontends' full-screen behaviour being visibly unfinished, not by duplication. It is cheap enough (~2 lines each, plus disposal) that the bar is met differently — but it should be recorded as the path that did not meet Phase 6's rule.

**Hotaru's share stops here.** Its `GameWindow` is a bare `GameFrameControl` host driven by hotkeys (`GameWindow.axaml.cs:362-374`), and that is deliberate. Giving it a menu would change what it is, which is a product decision and not a migration. **This document does not propose it.**

---

### Path D — Hotkey discoverability

**The only path answering a question the first gameplan left open, and the one most likely to be deferred.**

`EmuSen_LunaP_Gameplan.md` §6 lists, unresolved: *"Hotkey/keybinding help overlays. Both frontends have hotkeys, neither has a UI for discovering them."*

Two things have changed since. `LunaAction.HelpText` exists and is described as *"the sentence after the label: the tooltip on a toolbar button, and the accessible help text everywhere."* And Mistress's menu is now made of `LunaAction`s, so there is somewhere for a key name to be displayed.

`EmuSen_Settings_Reference.md` §4.19 records why the menu deliberately does **not** carry shortcuts today: Mistress's keys live in `HotkeyBindingMap` and are user-rebindable, so a second independent binding via `LunaAction.Shortcut` is how a menu starts advertising a key that a rebind has since moved. The path here is therefore **not** "set `Shortcut` on the actions". It is: drive the labels and `HelpText` *from* `HotkeyBindingMap`, so a rebind updates the menu, and the binding remains single-sourced.

Hotaru's F1–F9 map is a hardcoded `switch` and would have to become data before it could be displayed at all. That is the larger half of this path and the reason to sequence it last.

**Prediction, recorded so it can be wrong:** I expect the Mistress half to be small (a label formatter plus a refresh when the bindings window closes) and the Hotaru half to be most of the work. If that inverts, the sequencing below is wrong.

---

### Path E — `AppWindow`, and why not

`AppWindow` gives a window a menu bar, toolbar, status line, dockable side panels and central content as first-class properties. Mistress currently hand-rolls two of those: a `MenuBar` docked in XAML, and a status bar that is a `Border` over a two-column `Grid` (`MainWindow.axaml:16-36`).

**This is deliberately not proposed.** `AppWindow` owns its content layout, so adopting it means deleting `MainWindow.axaml` and rebuilding `LibraryView`, `LibraryFilter`, `LibraryList`, the two library text blocks and `GameFrame` in code — every one of which is reached by `x:Name` from a 1,300-line code-behind and from several test files. The gain is a status bar and a menu bar that are already working. That is a large, wide-reaching diff for no deletion and no defect.

The first gameplan already drew this line for the same window: *"`MainWindow` and `GameWindow` themselves are **not** migrated wholesale… Rewriting either is a different project and not this one."* That still holds.

**What would change the answer:** wanting side panels. `AppWindow.AddPanel` with a `PanelKey` is real work to hand-roll, and a docked debugger or cheat panel is the kind of thing that would justify the rewrite. Absent that, this stays refused.

---

## 3. Sequencing

1. ~~**Path A**, first~~ — **done**. The pattern repeated three times, not four.
2. **Path C**, second, because it is small, independent, and the only one that touches Hotaru. Good to land while Path A's shape is still fresh.
3. ~~**Path B**, third~~ — **done**, migration only. Sorting is still its own decision and has not been taken.
4. **Path D**, last, and only if the Hotaru half is wanted — the Mistress half alone is worth little, since a menu that names its keys while the console frontend does not is half an answer to §6.

A, B and C are independent; nothing here forces this order except that it front-loads the strongest evidence.

---

## 4. Rules carried forward

From the first gameplan, and from the menu path that has already been taken:

- **Every path must delete something, or say plainly that it does not.** Path C does not, and says so.
- **Write the test against the unmigrated window first.** Phase 6 learned this twice, and the menu path found two shipping defects by doing it.
- **Change lookups, not test bodies.** The menu path's 16 existing tests survived with one helper changed; that is what makes them a safety net rather than a restatement of the new code.
- **Assert on something rendered, not only on models.** Every menu test would have passed against a menu bar that drew nothing; one test now opens the real item and checks the built `MenuItem` follows its action.
- **The comment rule.** One line in any `.cs` file, pointing here. The migration sites are where this is most often broken.

---

## 5. Open questions, not decided here

- ~~**Does the library list's selection reset matter?**~~ **Answered by the code, 2026-08-16**: it is load-bearing for the search-then-Enter flow, and was kept. Path A did not change it.
- **Should the cheat table sort?** Still open. Path B made it possible (`TableKey` unset, no `Sort` on any column) and deliberately did not decide it.
- **Should Hotaru's hotkeys become data?** Path D needs it; nothing else does. It is the largest single item in this document and the least certainly wanted.
- **`ActionGroup.Checked` is read-only** despite LunaP 0.8.0's XML documentation describing a setter. Verified by reflection over the shipped assembly. This is an upstream doc bug and should be filed against the LunaP repository; nothing in this plan is blocked by it, because `member.IsChecked = true` does what the doc's setter claims.
