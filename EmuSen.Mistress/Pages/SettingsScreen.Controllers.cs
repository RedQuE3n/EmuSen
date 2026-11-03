using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using SDL3;

namespace EmuSen.Mistress.Pages
{
    // The bindings page's state: the console, the player and the profile whose bindings it shows, the control focused, whether A is waiting for a press, and whether it is the keyboard's page or the tester runs.
    public sealed class BindingsPage(string console, string profile, bool keys = false)
    {
        public string Console { get; set; } = console;
        public string Profile { get; set; } = profile;
        public int Player { get; set; } = 1;
        public PadControl? Focus { get; set; }
        public bool Capturing { get; set; }
        public bool Keys { get; } = keys;
        public bool Testing { get; set; }
    }

    // Controllers ▸ Profiles, Rumble and the bindings page - see EmuSen_BigPicture.md §52.7 and EmuSen_Input.md §10.5.
    public sealed partial class SettingsScreen
    {
        private string? _padConsole;

        // The bindings page when it is the page shown.
        public BindingsPage? Binding => _pages.Count > 0 ? _pages[^1].Bindings : null;

        // The consoles with an engine and a pad, in the order Consoles lists them.
        private IReadOnlyList<string> PadConsoles => ConsoleList.Where(s => s.Engines.Count > 0 && s.Pad.Controls.Count > 0).Select(s => s.Console).ToList();

        // The console Profiles and the bindings page open on: the one stepped to, else the game running, else the first.
        private string? PadConsole
        {
            get
            {
                IReadOnlyList<string> consoles = PadConsoles;
                if (_padConsole is { } chosen && consoles.Contains(chosen)) return chosen;
                return _host.RunningConsole is { } running && consoles.Contains(running) ? running : consoles.FirstOrDefault();
            }
        }

        // --- Profiles ---

        private ScreenPane ProfilesPane()
        {
            if (PadConsole is not { } console) return Said("No console has a controller to bind yet.");
            IReadOnlyList<string> profiles = _host.PadBindings.Profiles(console);
            string chosen = _host.PadBindings.ConsoleDefault(console);
            return Said($"{NameOf(console)} · {Count(profiles.Count, "profile", "profiles")} · {chosen} starts its games",
                "A game can start with a profile of its own, chosen on its page.");
        }

        private IReadOnlyList<Setting> ProfileRows()
        {
            if (PadConsole is not { } console) return [];
            IReadOnlyList<string> consoles = PadConsoles;
            var rows = new List<Setting>
            {
                Choice("pconsole", "Console", "The console whose profiles these are.", consoles.Select(NameOf).ToList(), consoles.ToList().IndexOf(console), i => _padConsole = consoles[i]),
            };
            string chosen = _host.PadBindings.ConsoleDefault(console);
            foreach (string profile in _host.PadBindings.Profiles(console))
            {
                string name = profile;
                rows.Add(Opens("profile:" + name, name, name == GamepadBindings.DefaultProfile ? "The bindings every console starts with; it cannot be renamed or deleted." : $"A set of {NameOf(console)} bindings of your own.",
                    name == chosen ? "● Starts games" : null, () => Push(name, () => ProfilePage(console, name), "profile:" + name),
                    () => Said(name == chosen ? $"{name} is what every {NameOf(console)} game starts with, unless the game's page chooses another." : $"{name} is kept for the games whose page chooses it.", "A opens it: its bindings, starting games with it, a new name, or deleting it.")));
            }
            rows.Add(Asks("newprofile", "New profile", $"A copy of {chosen}, named by you, for {NameOf(console)} games.", () =>
                _host.Ask("New profile", $"A name for the new {NameOf(console)} profile", "", typed =>
                {
                    if (_host.PadBindings.CreateProfile(console, typed, chosen)) _host.SavePadBindings();
                })));
            return rows;
        }

        private IReadOnlyList<Setting> ProfilePage(string console, string profile)
        {
            bool isDefault = profile == GamepadBindings.DefaultProfile;
            bool chosen = _host.PadBindings.ConsoleDefault(console) == profile;
            return
            [
                Opens("pbindings", "Controller bindings", $"The {NameOf(console)} buttons this profile puts on your controller.", null, () => OpenBindings(console, profile)),
                Act("pdefault", "Starts every game", $"{NameOf(console)} games start with {profile} unless a game's page chooses another.", () =>
                {
                    if (_host.PadBindings.ChooseDefault(console, profile)) _host.SavePadBindings();
                }, chosen ? "● Chosen" : null, enabled: !chosen),
                new(new PageRow("prename", "Rename") { Kind = PageRowKind.Asks, Sentence = isDefault ? "Default keeps its name." : "A new name; games that chose it keep it.", IsEnabled = !isDefault },
                    Choose: () => _host.Ask("Rename", $"A new name for {profile}", profile, typed =>
                    {
                        if (!_host.PadBindings.RenameProfile(console, profile, typed)) return;
                        _host.SavePadBindings();
                        if (_pages.Count > 0 && _pages[^1].Title == profile) _pages[^1] = _pages[^1] with { Title = typed.Trim(), Rows = () => ProfilePage(console, typed.Trim()), From = "profile:" + typed.Trim() };
                    })),
                Act("pdelete", "Delete", isDefault ? "Default cannot be deleted." : $"Games that chose {profile} start with the console's profile again.", () =>
                {
                    if (!_host.PadBindings.DeleteProfile(console, profile)) return;
                    _host.SavePadBindings();
                    Pop();
                }, enabled: !isDefault),
            ];
        }

        // --- Rumble ---

        // The switch, shown where the systems table says a console's games drive a motor: the game running's console, else any.
        private IReadOnlyList<Setting> RumbleRows()
        {
            List<string> shaking = _host.Systems.Where(s => s.Rumble).Select(s => s.Console).Distinct().ToList();
            bool shown = _host.RunningConsole is { } running ? shaking.Contains(running) : shaking.Count > 0;
            if (!shown) return [];
            string names = string.Join(", ", shaking.Select(NameOf).Distinct());
            return [Switch("rumble", "Rumble", "The motors in your controller, where the console's games drive them; a change reaches a game already running.", App.Rumble, on => App.Rumble = on,
                () => Said(App.Rumble ? "Your controller shakes when the game drives its motor." : "Your controller never shakes.", $"Games of these consoles can: {names}."))];
        }

        // --- The bindings page ---

        // Controllers ▸ Controller bindings, or a profile's: a page of content, the console's controller drawn with its focused control's binding in the pane.
        private void OpenBindings(string? console, string? profile, bool keys = false)
        {
            if (console is null) return;
            var page = new BindingsPage(console, profile ?? _host.PadBindings.ConsoleDefault(console), keys) { Focus = ControlsOf(console).FirstOrDefault() };
            _pages.Add(new Page(keys ? "Keyboard keys" : "Controller bindings", () => [], keys ? "keys" : _pages.Count == 0 ? "bindings" : "pbindings", Bindings: page));
        }

        public static string BindingKey(PadControl control) => "bind:" + control;

        public IReadOnlyList<PadControl> ControlsOf(string console) => _host.Systems.FirstOrDefault(s => s.Console == console && s.Pad.Controls.Count > 0)?.Pad.Controls ?? [];

        // The players the console's ports hold as its settings stand now, an adapter's included.
        public int PlayersOf(string console) => Math.Max(1, _host.PadNow(console).Players.Count);

        // The strip over the drawing, as the mockup draws it: the player shown, or the keyboard, then the consoles from the one shown onward, so it is always whole beside the first.
        public IReadOnlyList<string> BindingTabs(BindingsPage page) => [page.Keys ? "Keyboard" : $"Player {page.Player}", .. BindingConsolesFrom(page).Select(NameOf)];

        public IReadOnlyList<string> BindingConsolesFrom(BindingsPage page)
        {
            List<string> consoles = PadConsoles.ToList();
            int at = Math.Max(0, consoles.IndexOf(page.Console));
            return [.. consoles.Skip(at), .. consoles.Take(at)];
        }

        public static int BindingTab(BindingsPage page) => 1;

        // L1 and R1, or a tab: the console before or after, its own default profile, its first control focused.
        public void StepBindingConsole(int by)
        {
            if (Binding is not { } page) return;
            List<string> consoles = PadConsoles.ToList();
            int at = consoles.IndexOf(page.Console);
            if (consoles.Count == 0) return;
            ChooseBindingConsole(consoles[((at + by) % consoles.Count + consoles.Count) % consoles.Count]);
        }

        public void ChooseBindingConsole(string console)
        {
            if (Binding is not { } page || page.Console == console) return;
            page.Console = console;
            page.Profile = _host.PadBindings.ConsoleDefault(console);
            page.Player = Math.Min(page.Player, PlayersOf(console));
            page.Focus = ControlsOf(console).FirstOrDefault();
            page.Capturing = false;
            page.Testing = false;
        }

        // X: the next player the console's ports hold, whose bindings follow player 1's until one is changed.
        public void StepBindingPlayer()
        {
            if (Binding is not { Keys: false } page) return;
            page.Player = page.Player % PlayersOf(page.Console) + 1;
            page.Capturing = false;
        }

        public void FocusBinding(PadControl control)
        {
            if (Binding is { } page && ControlsOf(page.Console).Contains(control)) page.Focus = control;
        }

        // A button's pad binding in words; L2 and R2 are always their trigger, past half its travel, whatever button is bound beside it (EmuSen_Input.md §7.3).
        public string? BoundPad(BindingsPage page, PadControl control)
        {
            if (!PadControls.IsButton(control, out PadButton button)) return control <= PadControl.LeftStickRight ? "Left stick" : "Right stick";
            string? bound = _host.PadBindings.For(page.Console, page.Player, page.Profile).ButtonToPad.TryGetValue(button, out SDL.GamepadButton pad) ? PadWords(pad) : null;
            string? trigger = button switch { PadButton.L2 => "Left Trigger", PadButton.R2 => "Right Trigger", _ => null };
            return trigger is null ? bound : bound is null ? trigger : $"{bound} or {trigger}";
        }

        private static string PadWords(SDL.GamepadButton pad) => System.Text.RegularExpressions.Regex.Replace(pad.ToString().Replace("DPad", "D-pad "), "(?<=[a-z])(?=[A-Z])", " ");

        public static string ControlName(PadControl control)
        {
            if (control < PadControl.LeftStickUp) return control.ToString();
            string words = System.Text.RegularExpressions.Regex.Replace(control.ToString(), "(?<=[a-z])(?=[A-Z])", " ");
            return words[..1] + words[1..].ToLowerInvariant();
        }

        // A listens: on the keyboard's page for a key, for any control; on the pad's for a button.
        private void StartCapture(BindingsPage page)
        {
            if (page.Testing || page.Focus is not { } focus) return;
            if (page.Keys || PadControls.IsButton(focus, out _)) page.Capturing = true;
        }

        public void CancelCapture()
        {
            if (Binding is { } page) page.Capturing = false;
        }

        // The press heard while capturing: the focused button bound to it in the profile's map for the player, and saved at once.
        public void Rebind(SDL.GamepadButton pad)
        {
            if (Binding is not { Capturing: true, Focus: { } focus } page || !PadControls.IsButton(focus, out PadButton button)) return;
            _host.PadBindings.Own(page.Console, page.Player, page.Profile).Rebind(button, pad);
            _host.SavePadBindings();
            page.Capturing = false;
        }

        // A trigger pulled for L2 or R2 binds it alone: the button beside it is cleared, since the trigger is always read (EmuSen_Input.md §7.3).
        public void RebindTrigger()
        {
            if (Binding is not { Capturing: true, Focus: { } focus } page || !PadControls.IsButton(focus, out PadButton button) || button is not (PadButton.L2 or PadButton.R2)) return;
            _host.PadBindings.Own(page.Console, page.Player, page.Profile).Unbind(button);
            _host.SavePadBindings();
            page.Capturing = false;
        }

        // Y: the focused button back to its default in the profile's map for the player.
        public void ResetBinding()
        {
            if (Binding is { Keys: true, Focus: { } control } keys)
            {
                ResetKey(keys, control);
                return;
            }
            if (Binding is not { Focus: { } focus } page || !PadControls.IsButton(focus, out PadButton button)) return;
            _host.PadBindings.Own(page.Console, page.Player, page.Profile).ResetButton(button);
            _host.SavePadBindings();
            page.Capturing = false;
        }

        public const string HintReset = "reset", HintPlayer = "player", HintClear = "clear", HintTest = "test";

        // A Rebind, the directions the regions, Y Reset, the shoulders the console, X the player where the console has more than one (the keyboard's page: Clear), Select the tester, B Back.
        private IReadOnlyList<ScreenHint> BindingHints(BindingsPage page)
        {
            if (page.Capturing) return [];
            if (page.Testing) return [new("East", "Hold to stop")];
            var hints = new List<ScreenHint> { new("South", "Rebind"), new("DPadLeftRight", "Region"), new("North", "Reset", HintReset), new("Shoulders", "Console") };
            if (page.Keys) hints.Add(new("West", "Clear", HintClear));
            else if (PlayersOf(page.Console) > 1) hints.Add(new("West", "Player", HintPlayer));
            if (!page.Keys) hints.Add(new("Select", "Test", HintTest));
            hints.Add(new("East", "Back"));
            return hints;
        }

        private ScreenPane BindingPane(BindingsPage page)
        {
            if (page.Testing) return TesterPane(page);
            if (page.Keys) return KeyPane(page);
            if (page.Focus is not { } focus) return Said($"The {NameOf(page.Console)} has no controls to bind.");
            string name = ControlName(focus);
            string where = string.Join(" · ", new[] { NameOf(page.Console), page.Profile == GamepadBindings.DefaultProfile ? null : page.Profile, page.Player > 1 ? $"Player {page.Player}" : null, name }.OfType<string>());
            var lines = new List<SettingsLine> { new(name) { Size = 32 }, new(where, SettingsTone.Muted) };
            if (!PadControls.IsButton(focus, out _))
                lines.Add(new SettingsLine("Read from the pad's own stick, so it has no button to bind."));
            else if (page.Capturing)
                lines.Add(new SettingsLine($"Press the button for {name} on the pad. Nothing pressed in five seconds, or Escape on the keyboard, leaves it as it was.", SettingsTone.Attention) { Mark = "●" });
            else
                lines.Add(new SettingsLine(BoundPad(page, focus) is { } pad ? (pad.EndsWith("Trigger", StringComparison.Ordinal) ? $"Bound to the pad's {pad}." : $"Bound to the pad's {pad} button.") : "Not bound to a button."));
            if (!page.Capturing) lines.Add(new SettingsLine($"A listens for a press and rebinds it; Y resets this button; the profile saves at once.", SettingsTone.Muted));
            if (ConflictWords(page.Console) is { } conflict) lines.Add(new SettingsLine(conflict, SettingsTone.Attention) { Mark = "⚠" });
            return new ScreenPane(lines);
        }
    }
}
