using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Input;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Pages
{
    // Controllers ▸ Keyboard keys and Hotkeys, the tester and the conflict notice, the stick's rows and the reset, which the binding window held until 2026-11-03 - see EmuSen_Input.md §10.6.
    public sealed partial class SettingsScreen
    {
        // The hotkey a row of the Hotkeys page listens for, if any.
        private HotkeyAction? _hotkeyListening;

        public HotkeyAction? HotkeyListening => _hotkeyListening;

        public void CancelHotkey()
        {
            _hotkeyListening = null;
            Read();
        }

        // The controls the tester sees held, given by the host each poll; the drawing lights them.
        public IReadOnlyList<PadControl> Held { get; private set; } = [];

        public bool SetHeld(IReadOnlyList<PadControl> held)
        {
            if (held.SequenceEqual(Held)) return false;
            Held = held;
            return true;
        }

        public bool Listening => Binding is { Capturing: true } || _hotkeyListening is not null;

        private IReadOnlyList<Setting> KeyRows() =>
        [
            Opens("keys", "Keyboard keys", "Each console's buttons on the keyboard, drawn on its controller.", null, () => OpenBindings(PadConsole, null, keys: true), enabled: PadConsole is not null),
            Opens("hotkeys", "Hotkeys", "The emulator's own keys: fast forward, rewind, states, pause and the rest, shared by every console.", null, () => Push("Hotkeys", HotkeyRows, "hotkeys")),
        ];

        private IReadOnlyList<Setting> StickAndResetRows()
        {
            int deadzone = (int)Math.Round(Math.Clamp(App.StickDeadzone, 0.05, 0.95) * 100);
            return
            [
                new(new PageRow("deadzone", "Stick deadzone") { Kind = PageRowKind.Steps, Status = $"{deadzone}%", Sentence = "How far a stick must travel before it counts as a direction. Raise it if your stick drifts." },
                    () => new ScreenPane([new SettingsLine("How far a stick must travel before it counts as a direction. Raise it if your stick drifts.")]) { Preview = new PanePreview.Meter(deadzone / 100.0) },
                    by => { App.StickDeadzone = Math.Clamp(deadzone + 5 * by, 5, 95) / 100.0; _host.SaveApp("deadzone"); }),
                Switch("mirror", "Mirror player 1 to player 2", "Player 1's controller also plays as player 2, for games that read the second controller, as Super Mario All-Stars' classic games do. Leave off for two players.", App.MirrorPlayer1ToPlayer2, on => App.MirrorPlayer1ToPlayer2 = on),
                Act("resetbindings", "Reset every binding", "Every console's buttons on the pad and the keyboard, and the hotkeys, back to their defaults; named profiles are yours and stay.", () =>
                {
                    _host.KeyBindings.ResetToDefaults();
                    _host.Hotkeys.ResetToDefaults();
                    _host.SaveKeys();
                    _host.PadBindings.ResetToDefaults();
                    _host.SavePadBindings();
                }),
            ];
        }

        // --- The keyboard's page ---

        public string? BoundKey(BindingsPage page, PadControl control) => _host.KeyBindings.For(page.Console).ButtonToKey.TryGetValue(control, out Key key) ? key.ToString() : null;

        private ScreenPane KeyPane(BindingsPage page)
        {
            if (page.Focus is not { } focus) return Said($"The {NameOf(page.Console)} has no controls to bind.");
            string name = ControlName(focus);
            var lines = new List<SettingsLine> { new(name) { Size = 32 }, new($"{NameOf(page.Console)} · Keyboard · {name}", SettingsTone.Muted) };
            if (page.Capturing) lines.Add(new SettingsLine($"Press the key for {name}. Escape cancels.", SettingsTone.Attention) { Mark = "●" });
            else
            {
                lines.Add(new SettingsLine(BoundKey(page, focus) is { } key ? $"Bound to the key {key}." : "Not bound to a key."));
                lines.Add(new SettingsLine("A listens for a key and rebinds it; Y resets it; X clears it. A key a hotkey has is taken from the hotkey.", SettingsTone.Muted));
            }
            if (ConflictWords(page.Console) is { } conflict) lines.Add(new SettingsLine(conflict, SettingsTone.Attention) { Mark = "⚠" });
            return new ScreenPane(lines);
        }

        // The key heard while the keyboard's page listens: a bare modifier is no binding, Escape cancels, and a hotkey's key is taken from it, since hotkeys are every console's.
        public bool RebindKey(Key key)
        {
            if (Binding is not { Keys: true, Capturing: true, Focus: { } focus } page) return false;
            if (IsModifier(key)) return true;
            page.Capturing = false;
            if (key == Key.Escape) return true;
            if (_host.Hotkeys.TryGetAction(key, out HotkeyAction clash)) _host.Hotkeys.Unbind(clash);
            _host.KeyBindings.For(page.Console).Rebind(focus, key);
            _host.SaveKeys();
            return true;
        }

        private static bool IsModifier(Key key) => key is Key.LeftCtrl or Key.RightCtrl or Key.LeftAlt or Key.RightAlt or Key.LeftShift or Key.RightShift or Key.LWin or Key.RWin;

        private void ResetKey(BindingsPage page, PadControl control)
        {
            ControllerKeyMap map = _host.KeyBindings.For(page.Console);
            if (DefaultPadKeyMap.Bindings().TryGetValue(control, out Key key)) map.Rebind(control, key);
            else map.Unbind(control);
            _host.SaveKeys();
        }

        // X on the keyboard's page: the focused control unbound.
        public void ClearKey()
        {
            if (Binding is not { Keys: true, Focus: { } focus } page) return;
            _host.KeyBindings.For(page.Console).Unbind(focus);
            _host.SaveKeys();
        }

        // A console's clashes, among the keys of the controls it has and the hotkeys; two consoles sharing a key is no clash, which is why each has its own keys (EmuSen_Input.md §5.1).
        public IReadOnlyList<string> Conflicts(string console)
        {
            HashSet<PadControl> controls = ControlsOf(console).ToHashSet();
            var owners = new Dictionary<Key, List<string>>();
            void Add(Key key, string owner)
            {
                if (!owners.TryGetValue(key, out List<string>? list)) owners[key] = list = [];
                list.Add(owner);
            }
            foreach ((PadControl control, Key key) in _host.KeyBindings.For(console).ButtonToKey.Where(kv => controls.Contains(kv.Key))) Add(key, ControlName(control));
            foreach ((HotkeyAction action, Key key) in _host.Hotkeys.ActionToKey) Add(key, HotkeyBindingMap.DisplayName(action));
            return owners.Where(kv => kv.Value.Count > 1).Select(kv => $"{kv.Key} is bound to {string.Join(" and ", kv.Value)}").ToList();
        }

        private string? ConflictWords(string console) => Conflicts(console) is { Count: > 0 } found ? $"Conflict on the {NameOf(console)}: {string.Join("; ", found)}." : null;

        // --- The tester ---

        public void ToggleTesting()
        {
            if (Binding is not { Keys: false } page) return;
            page.Testing = !page.Testing;
            page.Capturing = false;
        }

        public void StopTesting()
        {
            if (Binding is { } page) page.Testing = false;
        }

        private ScreenPane TesterPane(BindingsPage page)
        {
            var lines = new List<SettingsLine>
            {
                new("Testing") { Size = 32 },
                new($"{NameOf(page.Console)} · Player {page.Player}", SettingsTone.Muted),
                new(Held.Count > 0 ? $"Held: {string.Join(", ", Held.Select(ControlName))}" : "Press anything on the pad or the keyboard."),
                new("Every press lights its button on the drawing and does nothing else. Hold B for a second, or press Escape, to stop.", SettingsTone.Muted),
            };
            return new ScreenPane(lines);
        }

        // --- Hotkeys ---

        private IReadOnlyList<Setting> HotkeyRows()
        {
            HashSet<Key> clashing = _host.Hotkeys.ActionToKey.Values.Where(k => _host.Hotkeys.ActionToKey.Values.Count(o => o == k) > 1 || ConsolesHolding(k).Any()).ToHashSet();
            return Enum.GetValues<HotkeyAction>().Select(action =>
            {
                bool listening = _hotkeyListening == action;
                string status = listening ? "Press a key…" : _host.Hotkeys.ActionToKey.TryGetValue(action, out Key key) ? (clashing.Contains(key) ? $"⚠ {key}" : key.ToString()) : "Not bound";
                return new Setting(new PageRow("hk:" + action, HotkeyBindingMap.DisplayName(action)) { Kind = PageRowKind.Action, Status = status, Sentence = HotkeyBindingMap.IsHeld(action) ? "Applies while the key is held." : "Fires once each time the key is pressed." },
                    () => HotkeyPane(action), Choose: () => _hotkeyListening = listening ? null : action);
            }).ToList();
        }

        private IEnumerable<string> ConsolesHolding(Key key) => _host.KeyBindings.ByConsole.Where(kv => kv.Value.TryGetControl(key, out PadControl c) && ControlsOf(kv.Key).Contains(c)).Select(kv => NameOf(kv.Key));

        private ScreenPane HotkeyPane(HotkeyAction action)
        {
            var lines = new List<SettingsLine> { new(HotkeyBindingMap.DisplayName(action)) };
            if (_hotkeyListening == action) lines.Add(new SettingsLine("Press its key. Escape cancels.", SettingsTone.Attention) { Mark = "●" });
            else if (_host.Hotkeys.ActionToKey.TryGetValue(action, out Key key))
            {
                lines.Add(new SettingsLine($"Bound to the key {key}."));
                foreach (string console in ConsolesHolding(key)) lines.Add(new SettingsLine($"Conflict: the {console} also has {key} for a button.", SettingsTone.Attention) { Mark = "⚠" });
            }
            else lines.Add(new SettingsLine("Not bound to a key."));
            lines.Add(new SettingsLine("A listens for a key; X clears it. A hotkey is every console's, so its key is taken from any console's button.", SettingsTone.Muted));
            return new ScreenPane(lines);
        }

        // The key heard while a hotkey's row listens: taken from every console's buttons, then the hotkey's.
        public bool RebindHotkey(Key key)
        {
            if (_hotkeyListening is not { } action) return false;
            if (IsModifier(key)) return true;
            _hotkeyListening = null;
            if (key == Key.Escape)
            {
                Read();
                return true;
            }
            foreach (ControllerKeyMap map in _host.KeyBindings.ByConsole.Values)
                if (map.TryGetControl(key, out PadControl clash)) map.Unbind(clash);
            _host.Hotkeys.Rebind(action, key);
            _host.SaveKeys();
            Read();
            return true;
        }

        public void ClearHotkey(string key)
        {
            if (!key.StartsWith("hk:", StringComparison.Ordinal) || !Enum.TryParse(key[3..], out HotkeyAction action)) return;
            _host.Hotkeys.Unbind(action);
            _host.SaveKeys();
            Read();
        }
    }
}
