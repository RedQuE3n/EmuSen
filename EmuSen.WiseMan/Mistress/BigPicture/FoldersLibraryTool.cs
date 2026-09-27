using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class FoldersLibraryFactAttribute : FactAttribute
    {
        public FoldersLibraryFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_REALLIB") != "1")
                Skip = "Lists the player's ROM library read-only for P109; set EMUSEN_BIGPICTURE_REALLIB=1 - see EmuSen_BigPicture.md §30";
        }
    }

    // P109 on the player's own library, read and never written: the top entries each console opens on, and the first showing foldered against flattened.
    [Collection(TestCollections.ProcessGlobals)]
    public class FoldersLibraryTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FoldersLibraryTool).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public FoldersLibraryTool(ITestOutputHelper output) => _out = output;

        private static double Median(List<double> xs) => xs.Order().ElementAt(xs.Count / 2);

        [FoldersLibraryFact]
        public Task The_library_s_consoles_open_on_their_folders() => Session.Dispatch(() =>
        {
            string roms = AppSettings.Load().RomDirectory ?? throw new InvalidOperationException("no RomDirectory in appsettings");
            RomLibraryResult scan = RomLibrary.Scan(roms);
            using var theme = new SyntheticTheme();
            ThemedSession.Write(theme);
            IReadOnlyList<ThemedShelf> Shelves(bool flat) => EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder
                .Select(s => new ThemedShelf(new ThemeSystem(s.EsdeSystem, s.EsdeFullName, s.EsdeSystem),
                    scan.Entries.Where(e => e.Shelf == s.Name).Select(e => new SceneGame(e.Title, e.FullPath) { FolderPath = GameFolders.Of(roms, e.FullPath) })
                        .OrderBy(g => g.Name, StringComparer.OrdinalIgnoreCase).ToList()) { Flatten = flat })
                .ToList();

            var lines = new List<string> { $"{DateTime.Now:yyyy-MM-dd HH:mm}, {scan.Entries.Count} files under the ROM folder (listed, not opened), Debug build, headless, no window" };
            TimeSpan now = TimeSpan.Zero;
            var screen = new Size(1280, 800);
            IReadOnlyList<ThemedShelf> foldered = Shelves(false), flattened = Shelves(true);
            var view = new ThemedLibrary(() => now);
            view.Show(theme.Root, screen, foldered, null);
            foreach (SceneSystem s in view.Stage!.Current.Data.Systems)
            {
                ThemedShelf shelf = foldered.Single(x => x.System.Name == s.System.Name);
                int stems = shelf.Games.GroupBy(g => Path.GetFileNameWithoutExtension(g.File), StringComparer.Ordinal).Count(g => g.Count() > 1);
                int deepest = shelf.Games.Max(g => GameFolders.Depth(g.FolderPath));
                lines.Add($"{s.System.Name}: {shelf.Games.Count} games, opens on {s.Games.Count} entries ({s.Games.Count(g => g.Folder)} folders, {s.Games.Count(g => !g.Folder)} games), "
                    + $"deepest folder {deepest}, stems in two folders {stems}");
                if (s.Games.Count(g => g.Folder) is > 0 and <= 40) lines.Add("  " + string.Join(", ", s.Games.Where(g => g.Folder).Select(g => g.Name)));
            }

            var cold = new List<(double Foldered, double Flat)>();
            var warm = new List<(double Foldered, double Flat)>();
            for (int i = 0; i < 5; i++)
            {
                var clock = Stopwatch.StartNew();
                new ThemedLibrary(() => now).Show(theme.Root, screen, foldered, null);
                double a = clock.Elapsed.TotalMilliseconds;
                clock.Restart();
                new ThemedLibrary(() => now).Show(theme.Root, screen, flattened, null);
                cold.Add((a, clock.Elapsed.TotalMilliseconds));
                clock.Restart();
                view.Show(theme.Root, screen, foldered, null);
                double b = clock.Elapsed.TotalMilliseconds;
                clock.Restart();
                view.Show(theme.Root, screen, flattened, null);
                warm.Add((b, clock.Elapsed.TotalMilliseconds));
            }
            string Line(string what, List<(double Foldered, double Flat)> xs) =>
                $"{what}: foldered median {Median(xs.Select(x => x.Foldered).ToList()):F1} ms [{string.Join(", ", xs.Select(x => x.Foldered.ToString("F1")))}], "
                + $"flattened median {Median(xs.Select(x => x.Flat).ToList()):F1} ms [{string.Join(", ", xs.Select(x => x.Flat.ToString("F1")))}], "
                + $"ratio {Median(xs.Select(x => x.Foldered).ToList()) / Median(xs.Select(x => x.Flat).ToList()):F2}";
            lines.Add(Line("first Show of a new view (theme read, stage built)", cold));
            lines.Add(Line("Show of a shown view", warm));

            string output = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "pass6", "real-library.txt");
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            File.WriteAllLines(output, lines);
            foreach (string line in lines) _out.WriteLine(line);
        }, default);
    }
}
