using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // One game as a themed view shows it: its file, its metadata and its flags - see EmuSen_BigPicture.md §13.2.
    public sealed record SceneGame(string Name, string File)
    {
        public string? Description { get; init; }
        public string? Developer { get; init; }
        public string? Publisher { get; init; }
        public string? Genre { get; init; }
        public string? Players { get; init; }
        public DateTime? ReleaseDate { get; init; }
        public DateTime? LastPlayed { get; init; }
        public float? Rating { get; init; }
        public TimeSpan? PlayTime { get; init; }
        public int PlayCount { get; init; }
        public bool Favorite { get; init; }
        public bool Completed { get; init; }
        public bool Folder { get; init; }
        public bool KidGame { get; init; }
        public bool Broken { get; init; }
        public bool AltEmulator { get; init; }
        public bool InCollection { get; init; }
        public string? Emulator { get; init; }

        // ES-DE's controller metadata, one of THEMES.md's customControllerIcon names; null when none is set (§29).
        public string? Controller { get; init; }

        // A folder that opens one of its games directly, which the folder badge marks with a link (§29).
        public bool FolderLink { get; init; }

        // A PDF manual exists for the game, which the manual badge marks (§29).
        public bool Manual { get; init; }

        // The system the game belongs to, which a collection lists it beside others (§22).
        public ThemeSystem? Source { get; init; }

        // For a folder, the game whose media it shows, as ES-DE's grouped collections show a member (§22).
        public SceneGame? Face { get; init; }

        // A custom collection listed in the grouped system: opened like a folder, drawn without the folder mark, as ES-DE 3.4.1 draws it (§22).
        public bool IsCollection { get; init; }

        // ES-DE's "Hide metadata fields" for this entry: its text fields but the description, its dates, rating, badges and metadata elements are not drawn (§22).
        public bool HideMetadata { get; init; }

        // The game whose media is looked up, and the system it is looked up under.
        public SceneGame Shown => Face ?? this;

        public ThemeSystem SourceIn(SceneSystem listed) => Shown.Source ?? listed.System;
        // ES-DE's sortname: the list sorts by it where set, and shows the name (§4.59 of the settings reference).
        public string? SortName { get; init; }

        // ES-DE's "Exclude from game counter": left out of a system's game counts.
        public bool NotCounted { get; init; }

        // Listed only while the player shows hidden games.
        public bool Hidden { get; init; }
    }

    // A system with the theme resolved for it and its games; the carousel reads each system's own resolved view.
    public sealed record SceneSystem(ThemeSystem System, ResolvedTheme Theme, IReadOnlyList<SceneGame> Games)
    {
        // Whether a textlist marks favourites with a star here; ES-DE's favorites collection and custom collections do not by default (§22).
        public bool Stars { get; init; } = true;

        // What a game's systemName and systemFullname read here instead of the system's: blank at the grouped collections' top, a collection's name inside it (§22).
        public string? Heading { get; init; }

        // Whether favourites are listed first here, so the scroll overlay shows a star over them (§29).
        public bool FavoritesOnTop { get; init; } = true;
    }

    // Where a game's scraped images are, by ES-DE's media type name; null when there is none.
    public interface ISceneMedia
    {
        string? Find(ThemeSystem system, SceneGame game, string mediaType);

        // Which of ES-DE's media types any of the games has, for the variant triggers.
        IReadOnlySet<string> Present(ThemeSystem system, IReadOnlyList<SceneGame> games) =>
            ThemeCapabilities.MediaTypes.Where(t => games.Any(g => Find(system, g, t) is not null)).ToHashSet(StringComparer.Ordinal);

        // A value that changes when a system's media may have changed; null when asking again is the only test.
        string? Stamp(ThemeSystem system) => null;
    }

    // Everything a view is drawn from besides the theme: the systems and games, the selection, the media, the time and the device.
    public sealed record SceneData(IReadOnlyList<SceneSystem> Systems, Size Screen)
    {
        public int SystemIndex { get; init; }
        public int GameIndex { get; init; }
        public ISceneMedia? Media { get; init; }
        public DateTime Now { get; init; } = new(2026, 9, 24, 12, 0, 0);
        public DeviceStatus Status { get; init; } = new(Wifi: true, BatteryPercent: 80);
        public bool HideMetadata { get; init; }
        public SceneMotion Motion { get; init; } = SceneMotion.Esde;

        // The connected pad's printing, which the help bar's buttons are drawn in (§15).
        public PadFamily Family { get; init; }

        // The A and B functions swapped, so the help bar names the other buttons (settings reference §4.61).
        public bool SwapFaceButtons { get; init; }

        // ES-DE's DisplayClock setting, off in ES-DE by default whatever the theme sets (§13.8); the tests of stage (b) draw it.
        public bool ShowClock { get; init; } = true;

        // A shown clock follows the wall clock by itself rather than showing Now, as a session's does (§29).
        public bool LiveClock { get; init; }

        // ES-DE's "Display on-screen help"; off leaves every helpsystem element undrawn (§29).
        public bool ShowHelp { get; init; } = true;

        // The system status indicators the player keeps, ANDed with each systemstatus element's entries (§29).
        public DeviceIndicators StatusShown { get; init; } = DeviceIndicators.All;

        // ES-DE's "Enable textlist quick scrolling overlay" (§29).
        public bool ScrollOverlay { get; init; }

        // What the help bar's entries depend on beyond the view (§22).
        public HelpContext Help { get; init; } = new();

        public SceneSystem System => Systems[Math.Clamp(SystemIndex, 0, Systems.Count - 1)];

        public SceneGame? Game => System.Games.Count == 0 ? null : System.Games[Math.Clamp(GameIndex, 0, System.Games.Count - 1)];
    }
}
