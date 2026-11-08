using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Pages
{
    // The Pause Card's Game Options pages, described here so the card and Settings share one description of each row - see EmuSen_Interface.md §18.5.
    public sealed partial class SettingsScreen
    {
        public const string GameControllers = "game:controllers", GameDisplay = "game:display", GameSound = "game:sound", GameCards = "game:cards";
        public const string DeviceVolume = "the device's quick menu";
        public const string DoorBindings = "bindings", DoorSwapPlayers = "swapplayers";

        // Opens one of the game's pages as the only page, for the Pause Card to draw.
        public void OpenGame(string page)
        {
            _search = "";
            _section = Controllers;
            _pages.Clear();
            switch (page)
            {
                case GameControllers: Push("Controllers", GameControllerRows, page); break;
                case GameDisplay: Push("Display", GameDisplayRows, page); break;
                case GameSound: Push("Sound", GameSoundRows, page); break;
                case GameCards: Push("Memory Cards", GameCardRows, page); break;
            }
            Read();
        }

        // A game's own crop as a page of Settings, for its page's door: the switch and the four edges.
        public void OpenGameCrop(string game, string title)
        {
            _search = "";
            _section = Display;
            _pages.Clear();
            Push(title, () => GameCropRows(game), "gamecrop");
            Read();
        }

        // ingame-options-display: the console's picture shape, CRT, crop and the game's own crop, then full screen; each previews the paused frame.
        private IReadOnlyList<Setting> GameDisplayRows()
        {
            var rows = new List<Setting>();
            if (_host.RunningConsole is not { } console) return rows;
            string name = NameOf(console);
            Setting Framed(Setting row, string said) => row with { Pane = () => new ScreenPane([new SettingsLine(said)]) { Preview = new PanePreview.Frame(FrameCaption(row.Row.Status ?? "")) } };
            rows.Add(Framed(PictureShapeRow(console), $"The {name}'s screen shape, for every {name} game."));
            if (HasTelevision(console))
            {
                Setting crt = CrtProfileRow(console);
                rows.Add(Framed(crt with { Row = crt.Row with { Label = "CRT shader" } }, $"The {name}'s CRT, drawn on the paused frame behind the card: {crt.Row.Status}."));
                Setting crop = CropOverscanRows(console).First();
                rows.Add(Framed(crop, $"What every {name} game hides at the picture's edges."));
                if (ConsoleCrop.HasOverscan(console) && _host.RunningGame is { } game) rows.Add(GameCropSwitch(game, framed: true));
            }
            else rows.Add(Framed(ScreenRow(console), $"The {name}'s screen, drawn on the paused frame behind the card."));
            rows.Add(Choice("fullscreen", "Full screen", "The window over the whole screen.", [On, Off], _host.FullScreen ? 0 : 1, i => _host.FullScreen = i == 0));
            return rows;
        }

        // On, the game keeps its own crop in place of its console's, starting from the console's; A opens its four edges.
        private Setting GameCropSwitch(string game, bool framed)
        {
            GameCrop? own = _host.CropOf(game);
            Setting row = Choice("gamecrop", "This game's crop", "This game's own crop in place of its console's; A opens its four edges.", [On, Off], own is null ? 1 : 0,
                i => _host.SetCrop(game, i == 0 ? own ?? _host.ConsoleCropOf(game) : null),
                () => new ScreenPane([new SettingsLine(own is null ? "This game is drawn with its console's setting." : "This game has a crop of its own."),
                    new SettingsLine(own is null ? "Right turns it on, starting from the console's crop." : EdgesText(e => own.Value[e]), SettingsTone.Muted)])
                    { Preview = framed ? new PanePreview.Frame(FrameCaption("this game's crop")) : null });
            return row with { Choose = () => { if (_host.CropOf(game) is null) _host.SetCrop(game, _host.ConsoleCropOf(game)); Push("This game's crop", () => GameCropRows(game), "gamecrop"); } };
        }

        // The switch, then the four edges, shown from the console's crop until the game has its own; an edge changed makes it the game's own.
        private IReadOnlyList<Setting> GameCropRows(string game)
        {
            GameCrop shown = _host.CropOf(game) ?? _host.ConsoleCropOf(game);
            Setting toggle = GameCropSwitch(game, framed: false);
            return [toggle with { Choose = null }, .. EdgeRows("gamecrop:", i => shown[i], (i, percent) => _host.SetCrop(game, shown.With(i, percent)))];
        }

        // ingame-options-sound: the volume, mute and the interface's sounds; on the handheld the volume is the device's.
        private IReadOnlyList<Setting> GameSoundRows()
        {
            int sounds = Math.Clamp(App.BigPictureInterface.NavigationVolume, 0, 100);
            string[] levels = [Off, .. Enumerable.Range(1, 5).Select(i => $"On · {i * 20}%")];
            int at = !App.NavigationSounds ? 0 : Math.Clamp((int)Math.Round(sounds / 20.0), 1, 5);
            return
            [
                VolumeRow(game: true),
                Switch("mute", "Mute", "The game's sound off, its volume kept for when it comes back.", App.Mute, on => App.Mute = on),
                Choice("uisounds", "Interface sounds", "The sounds for moving, choosing and going back, and how loud they are beside the game.", levels, at, i =>
                {
                    App.NavigationSounds = i > 0;
                    if (i > 0) App.BigPictureInterface.NavigationVolume = i * 20;
                    _host.SaveApp("uisounds");
                }, () => new ScreenPane([new SettingsLine(App.NavigationSounds ? $"On at {sounds}%." : "Off.")]) { Preview = new PanePreview.Meter(App.NavigationSounds ? sounds / 100.0 : 0) }),
            ];
        }

        // ingame-options-memory-cards: the slots first, then the mode and the drawer, the same rows as Consoles ▸ the console ▸ Memory cards.
        private IReadOnlyList<Setting> GameCardRows() => _host.RunningConsole is not { } console ? [] :
            MediaOf(console).SelectMany(medium => (Setting?[]) [.. CardSlotRows(console, medium, game: true), CardModeRow(console, medium), CardsRow(medium)]).OfType<Setting>().ToList();

        // Each player's controller, the game's profile, the console's drawing and the swap, as ingame-options-controllers draws them.
        private IReadOnlyList<Setting> GameControllerRows()
        {
            var rows = new List<Setting>();
            string? console = _host.RunningConsole;
            IReadOnlyList<string> pads = _host.Pads;
            int players = Math.Max(2, console is null ? 1 : PlayersOf(console));
            for (int p = 1; p <= players; p++)
            {
                int player = p;
                int at = Enumerable.Range(0, pads.Count).FirstOrDefault(i => _host.PlayerOf(i) == player, -1);
                Setting row = Choice($"seat{p}", $"Player {p}", $"The controller player {p} plays on.", [NoPlayer, .. pads], at + 1,
                    i => { if (i == 0) { if (at >= 0) _host.Seat(at, 0); } else _host.Seat(i - 1, player); }, () => SeatPane(player));
                if (at < 0 && _host.Players.FirstOrDefault(s => s.Player == player) is { Pad: { } kept, Kept: true }) row = row with { Row = row.Row with { Status = $"{kept}, away" } };
                rows.Add(row);
            }
            if (console is not null)
            {
                IReadOnlyList<string> profiles = _host.PadBindings.Profiles(console);
                string chosen = _host.PadBindings.Resolve(console, _host.RunningGame);
                rows.Add(Choice("gprofile", "Profile", $"The {NameOf(console)} bindings this game plays with; the console's own unless one is chosen here.", profiles, profiles.ToList().IndexOf(chosen),
                    i => _host.GameProfile = profiles[i] == _host.PadBindings.ConsoleDefault(console) ? null : profiles[i],
                    () => Said($"This game plays with the {chosen} profile.", "Left and Right choose another; Settings ▸ Controllers ▸ Profiles makes them.")));
                rows.Add(Opens("gbindings", "Bindings", $"The {NameOf(console)}'s buttons on the controller in your hand, on its drawing.", null, () => _host.Open(DoorBindings),
                    () => Said($"The {NameOf(console)}'s drawing, with the {chosen} profile's buttons on it.", "A opens it over the card; B comes back here.")));
            }
            bool second = _host.Players.Any(s => s.Player == 2 && s.Pad is not null);
            rows.Add(Act("gswap", "Swap Players", "Players 1 and 2 trade controllers.", () => _host.Open(DoorSwapPlayers), enabled: second));
            return rows;
        }

        // Who plays on what, and what Left, Right and Bindings do.
        private ScreenPane SeatPane(int player)
        {
            string? pad = _host.Players.FirstOrDefault(s => s.Player == player)?.Pad;
            string profile = _host.RunningConsole is { } console ? $", with the {_host.PadBindings.Resolve(console, _host.RunningGame)} profile" : "";
            return Said(pad is null ? $"No one plays as Player {player}." : $"Player {player} plays on the {pad}{profile}.",
                "Left and Right pick another connected controller; Bindings opens the drawing for this console.");
        }
    }
}
