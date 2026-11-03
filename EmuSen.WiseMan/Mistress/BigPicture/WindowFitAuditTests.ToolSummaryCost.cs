using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.Tools.Packages;
using EmuSen.Tools.Scraper;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Each tool's summary read timed with its managed allocation, on a synthetic library of 1,000 games or the library EMUSEN_U7_LIBRARY names, read only - see EmuSen_Interface.md §19.3.
    public partial class WindowFitAuditTests
    {
        [Theory]
        [InlineData(8)]
        [InlineData(1000)]
        public void Each_tools_summary_read_is_timed(int games)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_U7_COST") != "1") return;
            string? real = Environment.GetEnvironmentVariable("EMUSEN_U7_LIBRARY");
            if (real is null || games != 8)
            {
                Directory.CreateDirectory(_romDir);
                byte[] snes = SyntheticRom.BuildBlank();
                for (int i = 0; i < games; i++)
                    File.WriteAllBytes(Path.Combine(_romDir, $"Game {i:D4} (USA)" + (i % 2 == 0 ? ".sfc" : ".nes")), i % 2 == 0 ? snes : new byte[64]);
            }
            Session.Dispatch(() =>
            {
                (MainWindow window, PadDriver _) = Library(1280, 800, bigScreen: false, more: a => { a.Scraping = true; if (real is not null && games == 8) a.RomDirectory = real; });
                try
                {
                    typeof(MainWindow).GetMethod("OpenMediaStore", Hidden)!.Invoke(window, []);
                    string cheats = Path.Combine(CheatFile.DirectoryPath, "SNES");
                    Directory.CreateDirectory(cheats);
                    for (int i = 0; i < Math.Min(games, 300); i++)
                        CheatFile.SaveTo(Path.Combine(cheats, $"Game {i:D4} (USA).json"), new CheatFile { Cheats = [new CheatFileEntry { Description = "Lives", Enabled = i % 4 == 0 }] });
                    object crystal = typeof(MainWindow).Assembly.GetType("EmuSen.Mistress.CoreSettings")!.GetProperty("Runtime")!.GetValue(null)!;
                    var runtime = (CoresRuntime)typeof(MainWindow).GetMethod("CoresRuntimeOf", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Public)!.Invoke(null, [crystal])!;
                    foreach ((string name, Func<ToolSummary?> read) in new (string, Func<ToolSummary?>)[]
                    {
                        ("scraper", () => ScraperPane.ReadSummary(CancellationToken.None)),
                        ("packages", () => PackagesPane.ReadSummary(CancellationToken.None)),
                        ("cores", () => CoresPane.ReadSummary(runtime, CancellationToken.None)),
                        ("cheats", () => GameCheats.Summary(CancellationToken.None)),
                    })
                    {
                        read();
                        foreach (int round in new[] { 1, 2 })
                        {
                            long before = GC.GetAllocatedBytesForCurrentThread();
                            var clock = Stopwatch.StartNew();
                            ToolSummary? said = read();
                            clock.Stop();
                            _out.WriteLine($"cost {name} games={(real is not null && games == 8 ? "library" : games.ToString())} round={round} ms={clock.Elapsed.TotalMilliseconds:F1} kb={(GC.GetAllocatedBytesForCurrentThread() - before) / 1024} said={said?.Marked ?? "null"}");
                        }
                    }
                }
                finally
                {
                    Stop(window);
                    window.Close();
                }
            }, default).GetAwaiter().GetResult();
        }
    }
}
