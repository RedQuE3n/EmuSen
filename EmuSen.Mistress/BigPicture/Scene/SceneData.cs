using System;
using System.Collections.Generic;
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
    }

    // A system with the theme resolved for it and its games; the carousel reads each system's own resolved view.
    public sealed record SceneSystem(ThemeSystem System, ResolvedTheme Theme, IReadOnlyList<SceneGame> Games);

    // Where a game's scraped images are, by ES-DE's media type name; null when there is none.
    public interface ISceneMedia
    {
        string? Find(ThemeSystem system, SceneGame game, string mediaType);
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

        public SceneSystem System => Systems[Math.Clamp(SystemIndex, 0, Systems.Count - 1)];

        public SceneGame? Game => System.Games.Count == 0 ? null : System.Games[Math.Clamp(GameIndex, 0, System.Games.Count - 1)];
    }
}
