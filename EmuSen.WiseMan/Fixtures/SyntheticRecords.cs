using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Fixtures
{
    // A games.db of its own holding each game's last-played time, so a scene's lastplayed gameselector asks the library's recent call - see EmuSen_Backend.md §28.11.
    public sealed class SyntheticRecords : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSyntheticRecords", Guid.NewGuid().ToString("N"));
        private readonly GameRecords _records;

        // Every synthetic system's games, the one public constructor a class fixture may have.
        public SyntheticRecords() : this(SyntheticLibrary.Systems.SelectMany(s => SyntheticLibrary.Games(s.System, s.Extension))) { }

        public static SyntheticRecords Of(IEnumerable<SceneGame> games) => new(games);

        private SyntheticRecords(IEnumerable<SceneGame> games)
        {
            Directory.CreateDirectory(_root);
            _records = GameRecords.Open(Path.Combine(_root, "games.db"));
            foreach (SceneGame g in games)
                if (g.LastPlayed is { } when) _records.Started(g.File, when);
        }

        public RecentGames Recent => _records.Recent;

        public void Dispose()
        {
            _records.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }
    }
}
