using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Mistress.Pages
{
    // Controllers, Display, Sound, Library and EmuSen: each section's rows from the settings they change - see EmuSen_BigPicture.md §52.3.
    public sealed partial class SettingsScreen
    {
        public static readonly (string Value, string Text)[] ControllerTypes =
            [(AppSettings.ControllerTypeAutomatic, "Automatic"), ("Xbox", "Xbox"), ("PlayStation", "PlayStation"), ("Nintendo", "Nintendo"), ("Generic", "Generic")];

        public static readonly (string Value, string Text)[] Keyboards =
            [(AppSettings.OnScreenKeyboardAutomatic, "Automatic"), (AppSettings.OnScreenKeyboardSteam, "Steam"), (AppSettings.OnScreenKeyboardEmuSen, "EmuSen's")];

        public static readonly (string Value, string Text)[] ResumeChoices =
            [(AppSettings.ResumeAsk, "Ask each time"), (AppSettings.ResumeAlways, "Always resume"), (AppSettings.ResumeNever, "Always start again")];

        public static readonly (string Value, string Text)[] Appearances =
            [(AppSettings.AppearanceDark, "Dark"), (AppSettings.AppearanceLight, "Light"), (AppSettings.AppearanceSystem, "System")];

        public static readonly (string Value, string Text)[] Regions =
        [
            ("", "Automatic, from the file's name"), ("wor", "World"), ("us", "USA"), ("eu", "Europe"), ("jp", "Japan"), ("uk", "United Kingdom"), ("fr", "France"),
            ("de", "Germany"), ("sp", "Spain"), ("it", "Italy"), ("au", "Australia"), ("kr", "Korea"), ("br", "Brazil"),
        ];

        public static readonly (string Value, string Text)[] Languages =
            [("en", "English"), ("fr", "French"), ("de", "German"), ("es", "Spanish"), ("it", "Italian"), ("pt", "Portuguese"), ("ja", "Japanese"), ("nl", "Dutch")];

        // The CRT's three tiers, the quality parameter's values in order (EmuSen_CRT.md).
        public static readonly string[] CrtTiers = ["Performance", "Balanced", "Accurate"];

        private static Setting Pick(string key, string label, string sentence, (string Value, string Text)[] choices, string current, Action<string> set, Func<ScreenPane?>? pane = null) =>
            Choice(key, label, sentence, choices.Select(c => c.Text).ToList(), Array.FindIndex(choices, c => c.Value == current), i => set(choices[i].Value), pane);

        private AppSettings App => _host.App;

        private IReadOnlyList<Setting> ControllerRows()
        {
            IReadOnlyList<PlayerSeat> seats = _host.Players;
            int set = seats.Count(s => s.Pad is not null);
            return
            [
                Opens("players", "Players", "Which controller plays as which player.", $"{set} of {Math.Max(set, seats.Count)} set", () => Push("Players", PlayerRows, "players"), PlayersPane),
                Opens("bindings", "Controller bindings", "Each console's buttons on the controller in your hand.", null, () => OpenBindings(PadConsole, null)),
                Opens("profiles", "Profiles", "Named sets of a console's bindings, and the one its games start with.", PadConsole is { } pc ? _host.PadBindings.ConsoleDefault(pc) : GamepadBindings.DefaultProfile,
                    () => Push("Profiles", ProfileRows, "profiles"), ProfilesPane, enabled: PadConsole is not null),
                .. RumbleRows(),
                Switch("swap", "Swap A and B in menus", "A and B trade what they do in the library, the menus and every sheet, and so do X and Y, for a controller with a Nintendo layout. Games keep their own bindings.", App.SwapPadButtons, on => App.SwapPadButtons = on),
                Pick("chords", "Guide chords", "Guide held with Up saves, with Down loads, with R1 takes a screenshot; Auto keeps them off in Steam's Game Mode, whose own chords they would clash with.",
                    [(AppSettings.GuideChordsAuto, "Auto"), (AppSettings.GuideChordsOn, On), (AppSettings.GuideChordsOff, Off)], App.GuideChords, v => { App.GuideChords = v; _host.SaveApp("chords"); }),
                .. KeyRows(),
                Pick("type", "Controller type", "Which buttons the help bar draws. Automatic follows the controller last pressed; only the pictures change.", ControllerTypes, App.ControllerType, v => { App.ControllerType = v; _host.SaveApp("type"); }),
                Switch("firstonly", "Only the first controller", "Only the first controller connected steers the menus, and games hear player 1's controller alone, as when a wireless pad registers twice.", App.FirstControllerOnly, on => App.FirstControllerOnly = on),
                Switch("notices", "Controller notices", "A notice at the top of the screen when a controller is connected or disconnected.", App.ControllerNotifications, on => App.ControllerNotifications = on),
                Pick("keyboard", "On-screen keyboard", "What types into a text row and the search. Automatic uses Steam's keyboard under Steam and EmuSen's otherwise.", Keyboards, App.OnScreenKeyboard, v => { App.OnScreenKeyboard = v; _host.SaveApp("keyboard"); }),
                Switch("stickdpad", "Left stick as the D-pad", "In games whose console has no stick, the left stick also presses the D-pad.", App.AnalogStickAsDpad, on => App.AnalogStickAsDpad = on),
                .. StickAndResetRows(),
            ];
        }

        private ScreenPane PlayersPane()
        {
            var lines = _host.Players.Where(s => s.Pad is not null).Select(s => new SettingsLine($"Player {s.Player} · {s.Pad}{(s.Kept ? " · kept" : "")}{ProfileWords}")).ToList();
            int[] free = _host.Players.Where(s => s.Pad is null).Select(s => s.Player).ToArray();
            if (free.Length > 0) lines.Add(new SettingsLine($"Players {string.Join(" and ", free)} · none", SettingsTone.Muted));
            lines.Add(new SettingsLine("Left and Right on a controller's row pick its player.", SettingsTone.Muted));
            return new ScreenPane(lines);
        }

        // The profile the players' pads start the console's games with, as the Players pane names it.
        private string ProfileWords => PadConsole is { } console ? $" · {_host.PadBindings.ConsoleDefault(console)} profile" : "";

        private IReadOnlyList<Setting> PlayerRows()
        {
            var rows = new List<Setting>();
            string[] players = Enumerable.Range(1, Math.Max(4, _host.Players.Count)).Select(p => $"Player {p}").ToArray();
            IReadOnlyList<string> pads = _host.Pads;
            for (int i = 0; i < pads.Count; i++)
            {
                int pad = i;
                int seat = _host.Players.FirstOrDefault(s => s.Pad == pads[i])?.Player ?? 0;
                Setting row = Choice($"pad{i}", pads[i], "The player this controller plays as; a button held on it marks its row.", players, seat - 1, p => _host.Seat(pad, p + 1));
                rows.Add(_host.PadPressed(i) ? row with { Row = row.Row with { Status = "● " + row.Row.Status } } : row);
            }
            rows.Add(Choice("keyboardplayer", "Keyboard", "The player the keyboard plays as, beside that player's controller.", players, App.KeyboardPlayer - 1, p => { App.KeyboardPlayer = p + 1; _host.SaveApp("keyboardplayer"); }));
            rows.Add(Switch("claim", "The first controller pressed is player 1", "In a game, the first controller a button is pressed on is player 1 and the others follow in the order they are used.", App.ClaimPlayersByPress, on => App.ClaimPlayersByPress = on));
            rows.Add(Switch("external", "External controller first", "On a handheld, a controller connected to it is player 1 ahead of the built-in controls.", App.ExternalControllerFirst, on => App.ExternalControllerFirst = on));
            return rows;
        }

        // The console the Display rows' preview and levers act on: the game running, else the first console with a television.
        private string? LeverConsole => _host.RunningConsole is { } running && HasTelevision(running) ? running : ConsoleList.Select(s => s.Console).FirstOrDefault(HasTelevision);

        private IReadOnlyList<string> TvConsoles => _host.Systems.Select(s => s.Console).Distinct().Where(HasTelevision).ToList();

        private IReadOnlyList<Setting> DisplayRows() =>
        [
            ForEvery("shape", "Picture shape", "The screen each console was made for, or each of the frame's pixels square. Console's leaves each console its own.",
                _host.Systems.Select(s => s.Console).Distinct().ToList(), GraphicsKeys.PictureShape, [GraphicsKeys.TvShape, GraphicsKeys.SquarePixels], ["TV shape", "Square pixels"], GraphicsKeys.TvShape),
            CrtRow(),
            Opens("levers", "Shader levers", "The CRT's own levers for one console: the mask, the bloom, the scanlines and the glass.", null, () => { if (LeverConsole is { } c) Push("Shader levers", () => LeverRows(c), "levers"); },
                () => LeverConsole is { } c ? new ScreenPane([new SettingsLine($"The {NameOf(c)}'s CRT, as the game running shows it."), new SettingsLine("A opens its levers; each one redraws the picture.", SettingsTone.Muted)]) { Preview = new PanePreview.Frame(FrameCaption("live preview")) } : null,
                enabled: LeverConsole is not null),
            Opens("shaders", "Shaders", "Each console's screen filter from every one EmuSen has: the built-in filters and the slang catalogue's presets, their parameters, and the catalogue's download.",
                _host.RunningConsole is { } running ? Views.ShaderSettingsWindow.Describe(_host.Graphics.Value(running, GraphicsKeys.ScreenFilter)) : null, () => _host.Open(DoorShaders)),
            ForEvery("crop", "Crop overscan", "What is hidden at the picture's edges on a console that had a television: the whole picture, or the part a television showed. Console's leaves each its own.",
                TvConsoles, GraphicsKeys.Overscan, [GraphicsKeys.NoOverscan, GraphicsKeys.TvOverscan], ["Whole picture", "TV overscan"], GraphicsKeys.NoOverscan),
            Pick("variant", "Light or dark", "Dark is the big screen's default; Light, or the desktop's own choice.", Appearances, App.Appearance, v => { App.Appearance = v; _host.SaveApp("variant"); },
                () => new ScreenPane([new SettingsLine("Dark is the big screen's default; Light follows the desktop's choice when you ask it to.")]) { Preview = new PanePreview.Variants("Dark", "Light") }),
            Switch("art", "Background art on Home", "The focused game's art behind Home, dimmed and blurred.", App.HomeArt, on => App.HomeArt = on),
            Switch("videos", "Play videos", "A game's clip plays in its picture once the focus rests on it, where it has one.", App.PlayVideos, on => App.PlayVideos = on),
            Switch("smooth", "Smooth motion", "One frame per refresh when the display's rate is within 1% of the console's, the sound resampled by the difference.", _host.Graphics.SyncToDisplay, on => { _host.Graphics.SyncToDisplay = on; _host.SaveGraphics(null); }),
            Switch("statusbar", "Status bar", "The desktop's bar along the bottom of the window.", App.ShowStatusBar, on => App.ShowStatusBar = on),
            Switch("messages", "Messages", "The game's name, Paused, State saved and errors at the bottom left.", App.ShowStatusText, on => App.ShowStatusText = on),
            Switch("fps", "Frame rate", "The frames-per-second readout while a game runs.", App.ShowFpsBar, on => App.ShowFpsBar = on),
        ];

        private string FrameCaption(string what) => $"{_host.RunningTitle ?? "The last game"} · {what}";

        // One value set across consoles at once: the shared one, or Console's while they differ, which leaves each its own.
        private Setting ForEvery(string key, string label, string sentence, IReadOnlyList<string> consoles, string setting, string[] values, string[] texts, string fallback)
        {
            string[] held = consoles.Select(c => _host.Graphics.Value(c, setting) ?? fallback).Distinct().ToArray();
            int at = held.Length == 1 ? Array.IndexOf(values, held[0]) + 1 : 0;
            return Choice(key, label, sentence, ["Console's", .. texts], at, i =>
            {
                if (i == 0) return;
                foreach (string c in consoles) _host.Graphics.SetValue(c, setting, values[i - 1]);
                _host.SaveGraphics(null);
            });
        }

        private Setting CrtRow()
        {
            IReadOnlyList<string> tv = TvConsoles;
            string Tier(string c) => _host.Graphics.Value(c, GraphicsKeys.ScreenFilter) != CrtFilter.Name ? "Off"
                : CrtTiers[Math.Clamp((int)ParseFloat(_host.Graphics.ParametersFor(c, CrtFilter.Name).GetValueOrDefault("quality"), 1), 0, 2)];
            string[] held = tv.Select(Tier).Distinct().ToArray();
            string[] texts = ["Console's", "Off", .. CrtTiers];
            int at = held.Length == 1 ? Array.IndexOf(texts, held[0]) : 0;
            return Choice("crt", "CRT shader", "The television's picture for every console that had one.", texts, at, i =>
            {
                if (i == 0) return;
                foreach (string c in tv)
                {
                    _host.Graphics.SetValue(c, GraphicsKeys.ScreenFilter, i == 1 ? ScreenFilters.None : CrtFilter.Name);
                    if (i > 1) _host.Graphics.SetParameter(c, CrtFilter.Name, "quality", (i - 2).ToString(CultureInfo.InvariantCulture));
                }
                _host.SaveGraphics(null);
            }, () =>
            {
                string tier = held.Length == 1 ? held[0] : "Console's";
                string said = tier switch
                {
                    "Performance" => "Performance: a plain scanline and the mask, for a weak machine.",
                    "Balanced" => "Balanced: the mask and the bloom at half strength. Accurate costs 3× more; Performance is a plain scanline.",
                    "Accurate" => "Accurate: the whole tube, its glow, glass and phosphors, at three times Balanced's cost.",
                    "Off" => "Off: the console's pixels as they are, with no television.",
                    _ => "Each console keeps its own; Left and Right set one for them all.",
                };
                return new ScreenPane([new SettingsLine(said)]) { Preview = new PanePreview.Frame(FrameCaption($"{tier} CRT")) };
            });
        }

        private static double ParseFloat(string? text, double fallback) => double.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out double v) ? v : fallback;

        // The CRT's levers for one console, each its parameter's choices or its range stepped, and the tier's own values back with Reset.
        private IReadOnlyList<Setting> LeverRows(string console)
        {
            var rows = new List<Setting>();
            IReadOnlyDictionary<string, string> set = _host.Graphics.ParametersFor(console, CrtFilter.Name);
            foreach (var p in CrtFilter.Parameters.Where(p => p.Id != "quality"))
            {
                double value = ParseFloat(set.GetValueOrDefault(p.Id), p.Initial);
                void Store(double v) { _host.Graphics.SetParameter(console, CrtFilter.Name, p.Id, v.ToString(CultureInfo.InvariantCulture)); _host.SaveGraphics(console); }
                if (p.Choices is { Count: > 0 } choices)
                {
                    rows.Add(Choice(p.Id, p.Description, $"{p.Description}, for the {NameOf(console)}.", choices.ToList(), (int)Math.Round(value - p.Minimum), i => Store(p.Minimum + i)));
                    continue;
                }
                bool fraction = p.Minimum == 0 && p.Maximum == 1;
                string shown = fraction ? $"{Math.Round(value * 100)}%" : value.ToString("0.##", CultureInfo.InvariantCulture);
                string leverId = p.Id;
                rows.Add(new Setting(new PageRow(p.Id, p.Description) { Kind = PageRowKind.Steps, Status = shown, Sentence = $"{p.Description}, for the {NameOf(console)}." },
                    () => new ScreenPane([new SettingsLine($"{p.Description} at {shown}. Each lever redraws the preview; the tier's own values come back with Reset.")]) { Preview = new PanePreview.Frame(FrameCaption("live preview")) },
                    by => Store(Math.Clamp(Math.Round((value + by * p.Step) / p.Step) * p.Step, p.Minimum, p.Maximum))));
            }
            rows.Add(Act("reset", "Reset to the tier's values", "Every lever back to what the tier sets.", () => { _host.Graphics.ForgetParameter(console, CrtFilter.Name); _host.SaveGraphics(console); }));
            return rows;
        }

        private IReadOnlyList<Setting> SoundRows()
        {
            int volume = (int)Math.Round(Math.Clamp(App.Volume, 0, 1) * 100);
            int sounds = Math.Clamp(App.BigPictureInterface.NavigationVolume, 0, 100);
            return
            [
                Percent("volume", "Volume", "The game's volume.", volume, v => { App.Volume = v / 100.0; _host.SaveApp("volume"); },
                    () => new ScreenPane([new SettingsLine("The game's volume. On the handheld this row defers to the device's quick menu.")]) { Preview = new PanePreview.Meter(volume / 100.0) }),
                Switch("navsounds", "Navigation sounds", "The sounds for moving, choosing and going back, on a stream of their own beside the game's; a theme without one plays EmuSen's.", App.NavigationSounds, on => App.NavigationSounds = on),
                Percent("soundvolume", "Sound volume", "How loud the navigation sounds are, beside the game's own volume.", sounds, v => { App.BigPictureInterface.NavigationVolume = v; _host.SaveApp("soundvolume"); },
                    () => new ScreenPane([new SettingsLine("How loud the navigation sounds are, beside the game's own volume.")]) { Preview = new PanePreview.Meter(sounds / 100.0) }),
                OutputRow(),
            ];
        }

        // Sound ▸ Output: System default first, then the connected devices by name; a saved device not connected now is kept and listed (EmuSen_Audio_Sync.md §7.4).
        private Setting OutputRow()
        {
            List<string> outputs = _host.SoundOutputs.ToList();
            string chosen = App.SoundOutputDevice is { Length: > 0 } d ? d : AppSettings.SystemDefaultOutput;
            bool absent = !outputs.Contains(chosen);
            if (absent) outputs.Add(chosen);
            return Choice("output", "Output", "Where the game's sound and the interface's sounds play.", outputs, outputs.IndexOf(chosen),
                i => { App.SoundOutputDevice = outputs[i]; _host.SaveApp("output"); },
                () => new ScreenPane(absent
                    ? [new SettingsLine($"{chosen} is not connected, so the system default plays until it is.", SettingsTone.Attention) { Mark = "⚠" }, new SettingsLine("The choice is kept and tried again each time EmuSen starts.", SettingsTone.Muted)]
                    : [new SettingsLine(chosen == AppSettings.SystemDefaultOutput ? "The game and the interface's sounds play where the system sends sound." : $"The game and the interface's sounds play on {chosen}."),
                       new SettingsLine("A device not connected when EmuSen starts falls back to the system default.", SettingsTone.Muted)]));
        }

        private IReadOnlyList<Setting> LibraryRows()
        {
            string rom = App.RomDirectory is { Length: > 0 } r ? r : "Not set";
            return
            [
                Opens("rom", "ROM folder", "The folder the library lists, read in place and never moved.", rom, () => _host.Open(DoorRomFolder),
                    () => Said($"{Count(_host.AllGames, "game", "games")} under {rom}, read in place and never moved.", "A opens the folder picker; new games are found on the next start or with Check Again in the Library Manager.")),
                Opens("art", "Artwork folder", "Box art, named like the game's file; blank keeps it beside the games.", App.ArtworkDirectory is { Length: > 0 } a ? a : "beside the games", () => _host.Open(DoorArtFolder)),
                Choice("themes", "Themes", "The big picture's look: EmuSen's own, or an ES-DE theme's; Left and Right choose one, A opens the themes and their options.", _host.Looks, _host.LookAt, i => _host.ChooseLook(i)) with { Choose = () => _host.Open(DoorThemes) },
                Opens("scraping", "Scraping", "Where art and facts come from, and what is fetched.", App.Scraping ? "ScreenScraper" : "Off", () => Push("Scraping", ScrapingRows, "scraping")),
                Opens("collections", "Collections", "The automatic collections shown, your own, and how lists are sorted.", _host.CustomCollections.Count.ToString(CultureInfo.InvariantCulture), () => Push("Collections", CollectionRows, "collections")),
                Choice("hidden", "Hidden games", "Games hidden from the library. Their files are never touched; listing them lets you unhide them on their page.", ["Not listed", "Listed"], App.ShowHiddenGames ? 1 : 0,
                    i => { App.ShowHiddenGames = i == 1; _host.SaveApp("hidden"); }, () => Said($"{Count(_host.Hidden, "game is", "games are")} hidden.", "Listed shows them again with the rest, to unhide them.")),
                Opens("cheats", "Cheats", "The cheats of the game last played, turned on or off, and the cheat database.", null, () => _host.Open(DoorCheats)),
                Opens("discs", "Compressed discs", "The library's discs converted to CHD beside them, checked and listed; the library is only read.", null, () => _host.Open(DoorDiscs)),
                Opens("media", "ES-DE media folder", "An ES-DE downloaded_media folder whose art the theme shows; only ever read.", App.EsdeMediaDirectory is { Length: > 0 } m ? m : "None", () => _host.Open(DoorMediaFolder)),
                Opens("themefolder", "ES-DE theme folder", "A theme folder for ES-DE to read in place, which then joins the themes.", App.BigPictureTheme is { Length: > 0 } t && System.IO.Directory.Exists(t) ? t : "None", () => _host.Open(DoorThemeFolder)),
            ];
        }

        private IReadOnlyList<Setting> ScrapingRows()
        {
            var rows = new List<Setting>
            {
                Switch("screenscraper", "Use ScreenScraper", "Art and facts from ScreenScraper, for the games the Scraper is asked to fetch.", App.Scraping, on => App.Scraping = on),
                Opens("account", "Member account", "Your ScreenScraper account, for more requests a day and faster ones.", _host.ScrapeUser ?? "Not signed in", () => Push("Member account", AccountRows, "account"), () => Said(_host.AccountLine)),
                Pick("region", "Region", "Whose box art and names come first.", Regions, App.ScrapeRegion ?? "", v => { App.ScrapeRegion = v; _host.SaveApp("region"); }),
                Switch("regionfallback", "Other regions after", "Else world, USA, Europe, Japan, then any.", App.ScrapeRegionFallback, on => App.ScrapeRegionFallback = on),
                Pick("language", "Language", "The language of descriptions and names.", Languages, App.ScrapeLanguage ?? "en", v => { App.ScrapeLanguage = v; _host.SaveApp("language"); }),
                Switch("gamenames", "ScreenScraper's names", "Show the names ScreenScraper gives rather than the files'.", App.ScrapeGameNames, on => App.ScrapeGameNames = on),
                Switch("refresh", "Refresh what is kept", "Fetch again what a game already has.", App.ScrapeRefresh, on => App.ScrapeRefresh = on),
                Switch("openemu", "OpenEmu's sources", "OpenVGDB and libretro's thumbnails when ScreenScraper has nothing.", App.OpenEmuFallback, on => App.OpenEmuFallback = on),
            };
            foreach ((string key, string label, Func<bool> get, Action<bool> set) in Fetches())
                rows.Add(Switch(key, label, $"Fetch {label.ToLowerInvariant()}.", get(), set));
            return rows;
        }

        // The member account's sign-in, kept only when ScreenScraper accepts it; OpenVGDB's download and the cleanup of media whose game is gone.
        private IReadOnlyList<Setting> AccountRows()
        {
            var rows = new List<Setting>();
            if (_host.ScrapeUser is null)
                rows.Add(Asks("signin", "Log in", "Your user name and password, checked with one request to ScreenScraper and kept only if accepted, in a file only you can read.", () => _host.Open(DoorSignIn)));
            else
            {
                rows.Add(Act("check", "Check", "Asks ScreenScraper about the account kept.", () => _host.Open(DoorCheckAccount), _host.ScrapeUser));
                rows.Add(Asks("signout", "Log out", "Deletes the account's file.", () => _host.Open(DoorSignOut)));
            }
            rows.Add(Asks("openvgdb", "OpenVGDB", "OpenEmu's game database, used when ScreenScraper has nothing; A downloads it, or removes the copy kept.", () => _host.Open(DoorOpenVgdb)) with
            {
                Row = new PageRow("openvgdb", "OpenVGDB") { Kind = PageRowKind.Asks, Status = _host.OpenVgdbBytes is long b ? $"{b / 1048576.0:0.#} MB" : "Not downloaded", Sentence = "OpenEmu's game database, used when ScreenScraper has nothing; A downloads it, or removes the copy kept." },
            });
            rows.Add(Asks("cleanup", "Clean up orphaned media", "Media whose game is gone, moved out of the library's art.", () => _host.Open(DoorCleanUp)));
            return rows.Select(r => r with { Pane = r.Pane ?? (() => Said(_host.AccountLine, r.Row.Sentence ?? "")) }).ToList();
        }

        private IEnumerable<(string, string, Func<bool>, Action<bool>)> Fetches() =>
        [
            ("covers", "Covers", () => App.ScrapeCovers, v => App.ScrapeCovers = v), ("screenshots", "Screenshots", () => App.ScrapeScreenshots, v => App.ScrapeScreenshots = v),
            ("marquees", "Marquees", () => App.ScrapeMarquees, v => App.ScrapeMarquees = v), ("miximages", "Mix images", () => App.ScrapeMiximages, v => App.ScrapeMiximages = v),
            ("titles", "Title screens", () => App.ScrapeTitleScreens, v => App.ScrapeTitleScreens = v), ("backs", "Back covers", () => App.ScrapeBackCovers, v => App.ScrapeBackCovers = v),
            ("physical", "Physical media", () => App.ScrapePhysicalMedia, v => App.ScrapePhysicalMedia = v), ("fanart", "Fan art", () => App.ScrapeFanArt, v => App.ScrapeFanArt = v),
            ("manuals", "Manuals", () => App.ScrapeManuals, v => App.ScrapeManuals = v), ("videos", "Videos", () => App.ScrapeVideos, v => App.ScrapeVideos = v),
        ];

        private IReadOnlyList<Setting> CollectionRows()
        {
            BigPictureCollections c = App.BigPictureCollections;
            Setting Auto(string value, string text) => Switch("auto" + value, text, $"The automatic {text} collection in the list of consoles.", c.AutoCollections.Contains(value), on =>
            {
                c.AutoCollections.Remove(value);
                if (on) c.AutoCollections.Add(value);
            });
            var mine = new List<Setting>();
            if (_host.EditingCollection is { } editing)
                mine.Add(Act("finish", $"Finish editing {editing}", "Ends adding games to the collection with Y in a game list.", () => _host.Open(DoorFinishCollection)));
            mine.Add(Asks("newcollection", "New collection", "Name a collection, then add games to it with Y in any game list until you finish editing.", () => _host.Open(DoorNewCollection)));
            if (_host.CustomCollections.Count > 0)
                mine.Add(Opens("deletecollection", "Delete a collection", "The games stay where they are; only the list of them goes.", null,
                    () => Push("Delete a collection", () => _host.CustomCollections.Select(g => Asks("delete" + g.Id, g.Name, $"Deletes the collection {g.Name}; its games are not touched.", () => _host.Open(DoorDeleteCollection + g.Id))).ToList(), "deletecollection")));
            return
            [
                .. mine,
                Auto(BigPictureCollections.AllGames, "All Games"),
                Auto(BigPictureCollections.Favorites, "Favorites"),
                Auto(BigPictureCollections.LastPlayed, "Last Played"),
                Opens("custom", "Your collections", "Your own collections, made and changed in the Collections tool.", _host.CustomCollections.Count.ToString(CultureInfo.InvariantCulture), () => _host.Open(DoorCollections)),
                .. _host.CustomCollections.Select(g => Switch("show" + g.Id, g.Name, "Your own collection, shown in the list of consoles or not.", !c.HiddenCustomCollections.Contains(g.Id), on =>
                {
                    c.HiddenCustomCollections.Remove(g.Id);
                    if (!on) c.HiddenCustomCollections.Add(g.Id);
                })),
                Pick("grouping", "Group your collections", "If unthemed puts a collection the theme has no folder for inside Collections.",
                    [(BigPictureCollections.GroupUnthemed, "If unthemed"), (BigPictureCollections.GroupAlways, "Always"), (BigPictureCollections.GroupNever, "Never")], c.GroupCustomCollections, v => { c.GroupCustomCollections = v; _host.SaveApp("collections"); }),
                Switch("favfirst", "Favourites on top", "Favourite games sorted above the rest.", c.FavoritesFirst, on => c.FavoritesFirst = on),
                Switch("favfirstcustom", "Favourites on top in your collections", "The same, in your own collections.", c.FavoritesFirstCustom, on => c.FavoritesFirstCustom = on),
                Switch("stars", "Stars in your collections", "A star beside a favourite in your own collections.", c.StarsCustom, on => c.StarsCustom = on),
                Pick("sort", "Default sort order", "A list's own choice in its options lasts until EmuSen closes.", GameSort.All(false).Select(s => (s.Stored, s.Label)).ToArray(), GameSort.Parse(c.DefaultSortOrder).Stored, v => { c.DefaultSortOrder = v; _host.SaveApp("collections"); }),
                Switch("folderstop", "Folders on top", "A console's folders above its games, as ES-DE sorts them.", c.FoldersOnTop, on => c.FoldersOnTop = on),
                Pick("random", "Random entry button", "Either thumbstick pressed in jumps to a random game, or console.",
                    [(BigPictureCollections.RandomGames, "Games only"), (BigPictureCollections.RandomGamesAndSystems, "Games and consoles"), (BigPictureCollections.RandomDisabled, "Off")], c.RandomEntryButton, v => { c.RandomEntryButton = v; _host.SaveApp("collections"); }),
                .. RuntimeCatalog.Shelves.Where(s => s.EsdeSystem.Length > 0).GroupBy(s => s.EsdeSystem).Select(g => g.First()).Select(s =>
                    Switch("flatten" + s.EsdeSystem, $"Flatten {s.EsdeFullName} folders", "Every game of the console listed together, as ES-DE's flatten.txt does.", c.FlattenedSystems.Contains(s.EsdeSystem), on =>
                    {
                        c.FlattenedSystems.Remove(s.EsdeSystem);
                        if (on) c.FlattenedSystems.Add(s.EsdeSystem);
                    })),
            ];
        }

        private IReadOnlyList<Setting> EmuSenRows() =>
        [
            Opens("saves", "Saves and states", "Where states go, what happens when a game starts again, and how play time is counted.", null, () => Push("Saves and states", SaveRows, "saves"),
                () => Said("States and saves live beside the games, with one resume point kept per game.", $"Resume · {ResumeChoices.FirstOrDefault(c => c.Value == App.ResumeOnLaunch).Text ?? "Ask each time"}")),
            Choice("bigpicture", "Big picture", "Start full screen with larger text, for a handheld or a television; it takes effect at the next start.", ["On startup", "Off"], App.BigScreen ? 0 : 1, i => { App.BigScreen = i == 0; _host.SaveApp("bigpicture"); }),
            Switch("background", "Pause in the background", "A game paused because another window came to the front starts again when this one comes back.", App.PauseInBackground, on => App.PauseInBackground = on),
            Opens("logging", "Logging", "Whether the runtime writes its log, and where each session's log goes.", _host.Logging ? On : Off, () => Push("Logging", LoggingRows, "logging")),
            Opens("shell", "Shell", "DianaSh, EmuSen's own shell, for scripts and the runtime's commands.", null, () => _host.Open(DoorShell)),
            Act("about", "About", "EmuSen's version.", () => { }, $"EmuSen {_host.Version}"),
        ];

        private IReadOnlyList<Setting> LoggingRows() =>
        [
            Choice("log", "Logging", "The runtime's log, on or off, without changing what each part of it would say.", [On, Off], _host.Logging ? 0 : 1, i => _host.Logging = i == 0),
            Opens("logfolder", "Log folder", "Where each session's log file is written; blank writes none.", App.LogDirectory is { Length: > 0 } l ? l : "None", () => _host.Open(DoorLogFolder)),
        ];

        private IReadOnlyList<Setting> SaveRows() =>
        [
            Pick("resume", "Continue where you left off", "Every game is saved when it is closed; this is what happens the next time it starts.", ResumeChoices, App.ResumeOnLaunch, v => { App.ResumeOnLaunch = v; _host.SaveApp("resume"); }),
            Choice("playtime", "Play time counted up to", "A launch longer than this adds nothing to the game's play time, so a game left running overnight does not count the night.",
                PlayTime.Choices.Select(c => c.Text).ToList(), Array.FindIndex(PlayTime.Choices, c => c.Hours == Math.Clamp(App.MaxPlayTimeTracking, 0, 24)), i => { App.MaxPlayTimeTracking = PlayTime.Choices[i].Hours; _host.SaveApp("playtime"); }),
            Opens("statefolder", "State folder", "Where save states and their pictures go; blank keeps them under home.", App.StateDirectory is { Length: > 0 } s ? s : "home/Saves", () => _host.Open(DoorStateFolder)),
        ];
    }
}
