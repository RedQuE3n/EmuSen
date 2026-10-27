# EmuSen_ControllerWindow — Mako, the controller window on the shared frame

*Proposal, 2026-10-27; its questions answered by the tester the same day (§5). Nothing is built.* Asked by the tester the same day: the controller window remade on
the frame the other tools share (`EmuSen_LibraryTools.md` §1), named **Mako** (Makoto, Sailor Jupiter). Menus call it
*Controllers*.

## 1. What exists

`InputSettingsWindow` (`EmuSen_Input.md` §5–§8, `EmuSen_Settings_Reference.md` §4.81): one tab per console and a
General tab; each console's pad drawn with its bindings on it, a region chosen on the drawing to rebind it by key or
pad, lit by what is pressed; a player selector for the bindings per player (§8.4); a tester left by holding B; the
RetroPad template with sticks as the one contract every core reads (§7). Device identity and dead zones exist in
Endymion; hotkeys have their own commands (`EmulationControlCommands`).

**What it lacks** is shape: a tab per console that no longer scales, the pads connected and which player each is not
shown beside the bindings, and no way to copy one console's layout to another. The drawing, the capture and the
tester are kept.

## 2. The window

```
┌ Toolbar: [Rebind] [Clear] [Copy From… ▾] [Reset]   Player: [1 ▾]   Search… ┐
├────────────┬───────────────────────────────┬───────────────────────────────┤
│ Consoles   │ Button        Key      Pad    │   ┌──── the console's pad ───┐ │
│  SNES  ●   │ A             X        South  │   │  drawn, its bindings on  │ │
│  N64       │ B             Z        East   │   │  it, lit by what is      │ │
│  …         │ Start         Enter    Start  │   │  pressed now             │ │
│ Hotkeys    │ …                             │   └──────────────────────────┘ │
│ Controllers│                               │  Stick: dead zone ━━●──        │
│  1 Xbox ●  │                               │  Triggers: threshold ━●──      │
│  2 DualSense                               │  [Test]  (hold B to leave)     │
├────────────┴───────────────────────────────┴───────────────────────────────┤
│ SNES, player 1: 2 changed        Apply to: (•) this console ( ) every console │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **Consoles, Hotkeys and the connected controllers in the sidebar.** Each controller shows its family (Xbox,
  PlayStation, Nintendo, generic, from its device identity) and the player it is; choosing one sets the table's player.
- **The table**: each of the console's buttons with its key and pad binding, sortable and searchable; a row chosen and
  **Rebind** (or a region clicked on the drawing) captures the next press, as today; conflicts are named on the row.
- **The detail pane**: the drawing lit live, the sticks' dead zone and the triggers' threshold where the controller has
  them, and the tester.
- **Copy From…** takes another console's layout through the RetroPad template, so a layout made once serves every
  console whose pad has those buttons; nothing is copied to a button the target has not.
- **Apply** to this console or every console, for the player shown; nothing changes until Apply.
- **The big-screen layout** stays, the panes stacked for the pad, as the present window's are.
- The window fits 1280×800 and 1920×1200 (the fit audits).

## 3. Beneath

- **A DianaSh module `emusen.input`**, over Endymion, so the shell can do what the window does: `pad ls` (the
  controllers connected: their identity, family and player), `pad test` (presses printed as they come, a line each),
  `pad bind <console> <button> <binding> [--player n]` and `pad show <console> [--player n]` (the bindings as text).
- Endymion keeps the input rules; the window and the commands read and write through it.

## 4. Stages

| Stage | What | Model |
|---|---|---|
| **C1** | `emusen.input`: `pad ls`, `test`, `bind`, `show` | routine |
| **C2** | Mako on `ManagerLayout`: the sidebar, the table, the drawing and the tester moved over, Copy From, Apply | routine |
| **C3** | The big-screen layout and the fit audits | routine |

## 5. Decided 2026-10-27 by the tester

1. Apply's scopes: this console and every console, for the player shown.
2. Copy From… through the RetroPad template, nothing copied to a button the target lacks.
3. The present window retired once Mako has everything it has.
4. Rumble and turbo later, each a proposal of its own.
5. Order: after Setsuna's M1; all three stages routine.
