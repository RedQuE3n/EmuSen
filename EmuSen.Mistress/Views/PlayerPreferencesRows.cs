using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using SDL3;

namespace EmuSen.Mistress.Views
{
    // Preferences ▸ Controllers' players: which pad is which player, the keyboard's player, a press lighting its pad's row, and seats kept for pads gone - see EmuSen_Input.md §8.9.
    public sealed class PlayerPreferencesRows : IDisposable
    {
        public const string NoPlayer = "None";

        private readonly AppSettings _settings;
        private readonly GamepadManager? _pads;
        private readonly Func<int> _ports;
        private readonly StackPanel _rows = new() { Name = "PlayerRows", Spacing = 12 };
        private readonly Dropdown _keyboard = new() { Name = "KeyboardPlayerDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Dictionary<ConnectedPad, (FieldRow Row, Dropdown Choice)> _padRows = new();
        private readonly Dictionary<int, FieldRow> _keptRows = new();
        private readonly HashSet<ConnectedPad> _lit = new();
        private int _made;

        public static readonly string[] Players = Enumerable.Range(1, PlayerSlots.MaxPlayers).Select(Player).ToArray();

        public static string Player(int player) => $"Player {player}";

        // Pads plugged in and pulled out while the sheet is open, the seats traded, and a press on each poll.
        public PlayerPreferencesRows(AppSettings settings, GamepadManager? pads, Func<int>? ports = null)
        {
            _settings = settings;
            _pads = pads;
            _ports = ports ?? (() => 0);
            _keyboard.Fill(Players, Player(Math.Clamp(settings.KeyboardPlayer, 1, PlayerSlots.MaxPlayers)));
            _keyboard.Chose += chosen =>
            {
                if (Array.IndexOf(Players, chosen as string) is var at and >= 0) { _settings.KeyboardPlayer = at + 1; _settings.Save(); }
                Refresh();
            };
            _rows.Children.Add(new FieldRow
            {
                Name = "KeyboardPlayerRow",
                Label = "Keyboard",
                Hint = "The player the keyboard plays as, beside that player's controller. Player 1 unless changed.",
                Content = _keyboard,
            });
            if (_pads is not null)
            {
                _pads.PadChanged += OnPadChanged;
                _pads.Players.Changed += Refresh;
                _pads.Polled += Light;
            }
            Refresh();
        }

        public Control Rows => _rows;

        private void OnPadChanged(PadConnection _) => Refresh();

        // Rows added and removed for pads come and gone; the others keep their controls, so a focused choice stays focused.
        public void Refresh()
        {
            if (_pads is null) return;
            IReadOnlyList<ConnectedPad> pads = _pads.Pads;
            foreach (ConnectedPad gone in _padRows.Keys.Where(p => !pads.Contains(p)).ToList())
            {
                _rows.Children.Remove(_padRows[gone].Row);
                _padRows.Remove(gone);
                _lit.Remove(gone);
            }
            for (int i = 0; i < pads.Count; i++)
            {
                ConnectedPad pad = pads[i];
                if (!_padRows.TryGetValue(pad, out var made)) _padRows[pad] = made = Make(pad, _made++);
                made.Row.Label = Name(pad, pads);
                int player = _pads.Players.PlayerOf(pad);
                made.Choice.Fill(Players.Append(NoPlayer).ToArray(), player > 0 ? Player(player) : NoPlayer);
                made.Row.Hint = Hint(player);
            }
            RefreshKept();
            Order();
        }

        private (FieldRow, Dropdown) Make(ConnectedPad pad, int index)
        {
            var choice = new Dropdown { Name = $"PadPlayerDropdown{index}", HorizontalAlignment = HorizontalAlignment.Stretch };
            choice.Chose += chosen =>
            {
                int at = Array.IndexOf(Players, chosen as string);
                _pads!.Assign(pad, at + 1);
            };
            var row = new FieldRow { Name = $"PadPlayerRow{index}", Content = choice };
            _rows.Children.Add(row);
            return (row, choice);
        }

        // Two pads of one name are numbered in front, where a long name trimmed at its end keeps the number, and told apart by the light a press puts on the row.
        private static string Name(ConnectedPad pad, IReadOnlyList<ConnectedPad> pads)
        {
            var same = pads.Where(p => p.Name == pad.Name).ToList();
            return same.Count > 1 ? $"{same.IndexOf(pad) + 1} · {pad.Name}" : pad.Name;
        }

        private string Hint(int player)
        {
            int ports = _ports();
            string press = "Press a button on this controller to light its row.";
            if (player == 0) return $"Plays as no one, and still steers the menus. {press}";
            if (_pads!.FirstControllerOnly && player > 1) return $"First Controller is on, so games hear player 1's controller alone. {press}";
            if (ports > 0 && player > ports) return $"This game has {ports} controller {(ports == 1 ? "port" : "ports")}, so it does not hear player {player}. {press}";
            return $"Choosing a player another controller has trades the two. {press}";
        }

        // A seat kept for a pad that has gone, with the way to give it up.
        private void RefreshKept()
        {
            var kept = _pads!.Players.Seated().Where(s => !s.Pad.IsOpen).ToDictionary(s => s.Player, s => s.Pad);
            foreach (int player in _keptRows.Keys.Where(p => !kept.ContainsKey(p)).ToList())
            {
                _rows.Children.Remove(_keptRows[player]);
                _keptRows.Remove(player);
            }
            foreach (var (player, pad) in kept)
            {
                if (!_keptRows.TryGetValue(player, out FieldRow? row))
                {
                    int seat = player;
                    Button forget = Ui.Button("Forget", () => _pads.Players.Forget(seat));
                    forget.Name = $"ForgetPlayerButton{player}";
                    forget.HorizontalAlignment = HorizontalAlignment.Left;
                    _keptRows[player] = row = new FieldRow { Name = $"KeptPlayerRow{player}", Content = forget };
                    _rows.Children.Add(row);
                }
                row.Label = $"{Player(player)}: {pad.Name}, disconnected";
                row.Hint = "Kept for this controller, which gets the number back when it reconnects. A new controller may take it meanwhile. Forget gives it up now.";
            }
        }

        // The keyboard, then the pads in the order they connected, then the seats kept.
        private void Order()
        {
            var wanted = new List<Control> { _rows.Children[0] };
            wanted.AddRange(_pads!.Pads.Where(_padRows.ContainsKey).Select(p => (Control)_padRows[p].Row));
            wanted.AddRange(_keptRows.OrderBy(k => k.Key).Select(k => (Control)k.Value));
            if (wanted.SequenceEqual(_rows.Children)) return;
            for (int i = 0; i < wanted.Count; i++)
            {
                int at = _rows.Children.IndexOf(wanted[i]);
                if (at != i) _rows.Children.Move(at, i);
            }
        }

        // A pad with anything held lights its row, as RetroArch's and ES-DE's "press a button" lists do.
        private void Light()
        {
            foreach (var (pad, (row, choice)) in _padRows)
            {
                bool held = Held(pad);
                if (held == _lit.Contains(pad)) continue;
                if (held) _lit.Add(pad); else _lit.Remove(pad);
                Mark(choice, held);
            }
        }

        public bool IsLit(ConnectedPad pad) => _lit.Contains(pad);

        private static bool Held(ConnectedPad pad)
        {
            foreach (SDL.GamepadButton b in Enum.GetValues<SDL.GamepadButton>())
                if (b is not (SDL.GamepadButton.Invalid or SDL.GamepadButton.Count) && pad.IsRawPressed(b)) return true;
            return Math.Abs(pad.RawAxis(SDL.GamepadAxis.LeftTrigger)) > 0.5 || Math.Abs(pad.RawAxis(SDL.GamepadAxis.RightTrigger)) > 0.5;
        }

        private static void Mark(Dropdown choice, bool lit)
        {
            choice.Classes.Set("identify", lit);
            if (lit && Application.Current?.FindResource("LunaAccent") is IBrush accent)
            {
                choice.BorderBrush = accent;
                choice.BorderThickness = new Thickness(2);
                MenuRows.SetValueColor(choice, (accent as ISolidColorBrush)?.Color);
            }
            else
            {
                choice.ClearValue(TemplatedControl.BorderBrushProperty);
                choice.ClearValue(TemplatedControl.BorderThicknessProperty);
                MenuRows.SetValueColor(choice, null);
            }
        }

        public void Dispose()
        {
            if (_pads is null) return;
            _pads.PadChanged -= OnPadChanged;
            _pads.Players.Changed -= Refresh;
            _pads.Polled -= Light;
        }
    }
}
