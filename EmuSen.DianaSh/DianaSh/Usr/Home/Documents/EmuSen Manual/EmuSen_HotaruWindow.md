# EmuSen_HotaruWindow — Hotaru, the shell's own window in Mistress

*Proposal, 2026-10-27. Not decided; nothing is built.* Asked by the tester the same day: Hotaru brought into Mistress
as a shell window of her own, opened as the other tools are (`EmuSen_LibraryTools.md` §1); in big picture, room left
beneath it for the on-screen keyboard. Menus call it *Shell*.

## 1. What exists

- **`DianaShConsoleWindow`** (`EmuSen_Debugging_Tools_Reference_v5.md` §3.17): a terminal-style window in Mistress onto
  the same `DianaShInterpreter` the F4 prompt uses, over the running game's service table where it can be, with
  `tmux` sessions, Mistress's own builtins (`EmulationControlCommands`) and its own `clear`.
- **The name Hotaru** is the shell program's game window: `hotaru <rom>` plays a game in it (`man hotaru`), and
  `emusen-shell` offers the command; Mistress's console does not, since Mistress runs its own games.
- **LunaP's `OnScreenKeyboard`**, opened over a window for text entry in big picture.

## 2. What changes

- **`DianaShConsoleWindow` becomes `HotaruWindow`**, the shell's window in Mistress, with the other tools' look (their
  toolbar, colours and type) and reached as they are: the Tools menu (*Shell*), the pad menu, big picture's menu, and F4
  as now. The interpreter, sessions and builtins are unchanged; nothing about the shell moves into the window.
- **A toolbar** for what a pad cannot type: history back and forward, complete (Tab), interrupt (Ctrl+C), clear, copy
  the last output, and the session switcher `tmux` already provides.
- **In a big-screen session**, Hotaru is a sheet whose shell fills the upper part of the screen and **stops above a
  docked on-screen keyboard**: the keyboard is laid out beneath the shell, never over it, so the prompt and the last
  lines stay in view while typing. The split follows the keyboard's own height at the screen's size; the output scrolls
  so the prompt sits just above the keyboard. A physical keyboard typing hides the docked one and gives the shell the
  whole sheet; a pad press brings it back.
- **The pad**: the keyboard takes the pad while it is shown, as today; the shoulders step through history and a
  trigger completes, so a command can be recalled and run without typing it again; B with nothing typed leaves.
- **The `hotaru` command** keeps its meaning in `emusen-shell`; Mistress's Hotaru still does not offer it, since Mistress
  runs its own games.

## 3. Beneath

- **LunaP**: `OnScreenKeyboard` gains a **docked** form (laid out in a host's layout, reporting its height) beside its
  present one over a window; any window can use it, so Ami's search or Setsuna's could too.
- No new DianaSh command: Hotaru is the shell itself.

## 4. Stages

| Stage | What | Model |
|---|---|---|
| **H1** | `HotaruWindow`: the rename, the family's look, the toolbar, the menus and the pad menu | routine |
| **H2** | The docked `OnScreenKeyboard` in LunaP, and Hotaru's big-screen sheet over it; the fit audits at 1280×800 and 1920×1200 and on the handheld's screen | routine |

## 5. Questions for the tester

1. **The window is renamed Hotaru**, the menus saying *Shell*. *Recommended:* yes.
2. **Docked keyboard, shell above it, hidden while a physical keyboard types.** *Recommended:* yes.
3. **Order.** *Recommended:* H1 and H2 need no shared frame, so they can come before Setsuna's M1, as the next Sonnet
   work.
