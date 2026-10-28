# EmuSen_HotaruWindow — Shell, the shell's own window in Mistress

*The class is still named `HotaruWindow`; it takes the plain name when next changed.*

*Decided 2026-10-27 (§5). H1 built (§6); H2 not yet.* Asked by the tester the same day: the shell brought into Mistress
as a window of its own, opened as the other tools are (`EmuSen_LibraryTools.md` §1); in big picture, room left
beneath it for the on-screen keyboard. Menus call it *Shell*.

## 1. What exists

- **`DianaShConsoleWindow`**, now `HotaruWindow` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.17): a terminal-style window in Mistress onto
  the same `DianaShInterpreter` the F4 prompt uses, over the running game's service table where it can be, with
  `tmux` sessions, Mistress's own builtins (`EmulationControlCommands`) and its own `clear`.
- **The `hotaru` command** belongs to the shell program (`EmuSen.Hotaru`, a separate program from this window) and `hotaru <rom>` plays a game in its window (`man hotaru`);
  `emusen-shell` offers the command; Mistress's console does not, since Mistress runs its own games.
- **LunaP's `OnScreenKeyboard`**, opened over a window for text entry in big picture.

## 2. What changes

- **The window becomes `HotaruWindow`**, the shell's window in Mistress, with the other tools' look (their
  toolbar, colours and type) and reached as they are: the Tools menu (*Shell*), the pad menu, big picture's menu, and F4
  as now. The interpreter, sessions and builtins are unchanged; nothing about the shell moves into the window.
- **A toolbar** for what a pad cannot type: history back and forward, complete (Tab), interrupt (Ctrl+C), clear, copy
  the last output, and the session switcher `tmux` already provides.
- **In a big-screen session**, the Shell window is a sheet whose shell fills the upper part of the screen and **stops above a
  docked on-screen keyboard**: the keyboard is laid out beneath the shell, never over it, so the prompt and the last
  lines stay in view while typing. The split follows the keyboard's own height at the screen's size; the output scrolls
  so the prompt sits just above the keyboard. A physical keyboard typing hides the docked one and gives the shell the
  whole sheet; a pad press brings it back.
- **The pad**: the keyboard takes the pad while it is shown, as today; the shoulders step through history and a
  trigger completes, so a command can be recalled and run without typing it again; B with nothing typed leaves.
- **The `hotaru` command** keeps its meaning in `emusen-shell`; Mistress's Shell window still does not offer it, since Mistress
  runs its own games.

## 3. Beneath

- **LunaP**: `OnScreenKeyboard` gains a **docked** form (laid out in a host's layout, reporting its height) beside its
  present one over a window; any window can use it, so Scraper's search or Packages' could too.
- No new DianaSh command: the Shell window is the shell itself.

## 4. Stages

| Stage | What | Model |
|---|---|---|
| **H1** | `HotaruWindow`: the rename, the family's look, the toolbar, the menus and the pad menu | routine |
| **H2** | The docked `OnScreenKeyboard` in LunaP, and the Shell window's big-screen sheet over it; the fit audits at 1280×800 and 1920×1200 and on the handheld's screen | routine |

## 5. Decisions

Decided 2026-10-27 by the tester: the window is renamed Shell, the menus saying the same; the on-screen keyboard is docked
beneath the shell and hidden while a physical keyboard types; H1 and H2 come next, ahead of Packages' M1.

## 6. H1 as built

- **Rename**: `DianaShConsoleWindow` is `HotaruWindow` (`EmuSen.Mistress/Views/HotaruWindow.cs`); the interpreter, `tmux`
  sessions and `EmulationControlCommands` are unchanged. Its title is *Shell*, and it is framed as a menu in a big-screen session.
- **Reached** from the Settings menu as *Shell...* and from the pad menu's EmuSen page as *Shell*, which big picture shares; the Settings page already fills the big-screen menu at nine rows.
  There is no Tools menu. One `WindowSlot` keeps a second choice on the open window. F4 remains the pad menu's Start key in the themed view.
- **Toolbar** (LunaP `ToolBar` of `LunaAction`s): Back, Forward (history), Complete, Clear, Copy, Interrupt.
  - LunaP's `ConsolePane` gained `HistoryBack()`, `HistoryForward()`, `Insert()`, `SetInput()`, `InputText` and `OutputFocusable` (off here, so the pad's Down from the toolbar reaches the input box), on LunaP's `hotaru-h1` branch.
  - Complete finishes the first word from `DianaShInterpreter.CommandNames`: the longest shared prefix, and a space when one command remains.
  - Copy puts the newest non-empty output on the clipboard.
- **Pad** (`IPadDriven`): the shoulders step through history, a trigger completes, and B clears a typed line, then closes the window.
- **Interrupt** is the toolbar's sixth action, and Ctrl+C in the window does the same: `DianaShInterpreter.Interrupt()`, which stops the running line at its next check, prints `^C` and leaves `$?` at 130 (EmuSen_DianaSh.md §10.2). Ctrl+C no longer copies a selection; Copy is the toolbar's.
