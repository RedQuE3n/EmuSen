using System.Security.Cryptography;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.Snes;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Gate G5's goldens: twelve commercial games from the library, three anchors each, VenusRT's picture against Mesen's - see VenusRT_Native.md §41.6.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtGoldenTests
    {
        public const string Variable = "EMUSEN_VENUSRT_GOLDENS";
        public const int Window = 30;

        // One anchor: a frame, the scene Mesen shows there, and what VenusRT's picture was recorded as: "equal" or a cause, then its hash.
        public sealed record Anchor(int Frame, string Scene, string Recorded);

        public sealed record Golden(string File, string Md5, string Why, SnesPress[] Presses, Anchor[] Anchors);

        private static SnesPress P(int frame, PadButton b) => new(frame, b, 5);

        // Chosen from Mesen's pictures alone before VenusRT was run on them; ROMs are found by MD5, never committed.
        public static readonly Golden[] Goldens =
        {
            new("Super Mario RPG - Legend of the Seven Stars (U) [!].smc", "d0b68d68d9efc0558242f5476d1c5b81", "SA-1", Array.Empty<SnesPress>(),
                new[] { new Anchor(300, "the Square logo", "equal 2c3d80054787292a"), new Anchor(1200, "the garden at night, attract", "drift, equal 40 frames on (D-6, D-37) a20aa10c2349d4ae"), new Anchor(2400, "Mario on the map, attract", "drift, equal 42 frames on (D-6, D-37) a5fcdffe4cafff7e") }),
            new("Super Mario World 2 - Yoshi's Island (U) (V1.1).smc", "2150266ad9ea89aed241e949966514ae", "GSU-2", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "\"A long, long time ago\" under the Nintendo board", "equal 6128ce74a4d6b29b"), new Anchor(1800, "the stork over the night sky", "the stork alone, not logged f54ae4aef41532ce"), new Anchor(3000, "\"SCRREEEECH!!!\"", "equal f68d5ef7464cf859") }),
            new("Super Mario Kart (U) [!].smc", "8b13d7d413545582099ab35298405417", "DSP-1B", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "the title with karts passing", "equal eea24499eda2603c"), new Anchor(2400, "the attract race, start", "equal cbf0b94096358116"), new Anchor(3300, "the attract race, later", "equal f0c88e85d8bd8222") }),
            new("F1 ROC II - Race of Champions (U).smc", "cbf9f26076044edc913aabc22adbd690", "ST010", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "the title menu", "equal eac9527bfe9f3b28"), new Anchor(1500, "the first attract race", "equal 9e20b7edcd48a739"), new Anchor(3300, "the second attract race", "equal 9c44d1667798c463") }),
            new("Metal Combat - Falcon's Revenge (U) [!].smc", "7f159d48965a2802d9fd76c6acaed5af", "OBC1", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "\"Nintendo presents\" over the intro", "equal a0ffd942706a3540"), new Anchor(1500, "the title", "equal 4edd452badb0bc2c"), new Anchor(3000, "the attract fight", "equal 42e0b4d54f179a22") }),
            new("Captain America and The Avengers (U).smc", "b96b8908f88e999be63239444e4997df", "tricky list, open bus", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "the copyright page", "equal d9352985ec7ef486"), new Anchor(1500, "the title", "equal efaf9b4ebd1782a2"), new Anchor(2100, "Captain America's card", "equal 30185da569fd6392") }),
            new("ActRaiser (U) [!].smc", "2c740f599af49e4db5864817ce57cbdf", "tricky list, BRK/COP", new[] { P(1000, PadButton.Down), P(1100, PadButton.Start), P(1300, PadButton.Start), P(1500, PadButton.A), P(1700, PadButton.Start) },
                new[] { new Anchor(600, "the sword in the intro", "D-7 5b9e464697e864eb"), new Anchor(900, "the title menu", "equal f180d4a48a9a8e53"), new Anchor(1400, "\"Please create a name\" in the sky palace", "equal 2c4d540ad56e00d6") }),
            new("SMW.smc", "dbe1f3c8f3a0b2db52b7d59417891117", "tricky list, ORA [d]", new[] { P(600, PadButton.Start), P(700, PadButton.A), P(800, PadButton.A), P(1300, PadButton.A), P(1500, PadButton.A), P(1700, PadButton.A) },
                new[] { new Anchor(150, "\"Nintendo Presents\"", "equal 7a654f58801248e8"), new Anchor(1200, "the welcome message at Yoshi's house", "equal 759afbb3411436bd"), new Anchor(2400, "the overworld map", "equal eb6cc5041c418bdb") }),
            new("Hook (U) (2648).smc", "291e0beb91390a96cd1effd619157275", "tricky list, VRAM writes during display", Array.Empty<SnesPress>(),
                new[] { new Anchor(300, "the Sony Imagesoft logo", "equal 9d9e4fac812bc8ec"), new Anchor(600, "the copyright page", "equal 2eecc820a49d9233"), new Anchor(1500, "the intro's dialogue", "equal ce6979de75867d63") }),
            new("Breath of Fire (U) [!].smc", "cdef890e99a0fb62c4d449d867c4244a", "tricky list, VRAM reads", new[] { P(1900, PadButton.Start), P(2100, PadButton.Start), P(2300, PadButton.A), P(2500, PadButton.A) },
                new[] { new Anchor(700, "the copyright page", "equal 4e1f799880fbdabd"), new Anchor(1800, "the title", "equal 2e13fe99b52a9989"), new Anchor(2700, "the name entry", "equal 571411f365f99535") }),
            new("Axelay (U).smc", "d6ddca61a7f31eed247c429b0e528f3c", "tricky list, offset-per-tile", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "the Konami logo", "equal 2d92193a131065e0"), new Anchor(1500, "the city in the intro", "D-7 ddc596bb8602f751"), new Anchor(3000, "the planet in the intro", "D-7 ac943fa62256caff") }),
            new("NHL '94 (U) [!].smc", "960f6297b96ee768b97bd795bb1136f7", "tricky list, mode 7 scroll latch (the intro)", Array.Empty<SnesPress>(),
                new[] { new Anchor(600, "the EA Sports logo", "equal c415a598c615d12e"), new Anchor(1500, "the title and licence", "equal cf5277f7f77fb355"), new Anchor(2200, "the credits with a spinning puck", "the puck alone, not logged 4095c4a9591bb755") }),
        };

        private readonly ITestOutputHelper _output;

        public VenusRtGoldenTests(ITestOutputHelper output) => _output = output;

        // The library: the variable's folder, or AppSettings.RomDirectory when it says "library".
        private static string? Library => Environment.GetEnvironmentVariable(Variable) switch
        {
            null or "" => null,
            "library" => AppSettings.Load().RomDirectory,
            var dir => dir,
        };

        // A golden's ROM by name under SNES/, else by MD5 among the library's files of a ROM's size.
        private static string? Find(string library, Golden g)
        {
            string named = Path.Combine(library, "SNES", g.File);
            if (File.Exists(named) && Md5(named) == g.Md5) return named;
            return Directory.EnumerateFiles(library, "*.s?c", SearchOption.AllDirectories).FirstOrDefault(f => Md5(f) == g.Md5);
        }

        private static string Md5(string path) => Convert.ToHexString(MD5.HashData(File.ReadAllBytes(path))).ToLowerInvariant();

        // FNV-1a over a picture's size and its 15-bit words.
        public static string Hash(SnesPicture p)
        {
            ulong h = 14695981039346656037;
            void Add(int v) { h = (h ^ (uint)v) * 1099511628211; }
            Add(p.Width);
            Add(p.Height);
            foreach (ushort px in p.Pixels) Add(px & 0x7FFF);
            return h.ToString("x16");
        }

        private static bool Lit(SnesPicture? p) => p is not null && p.Pixels.Any(x => (x & 0x7FFF) != 0);

        // The golden table's choices are fixed before any engine runs: three lit anchors a game, one game at least per coprocessor.
        [Fact]
        public void The_goldens_cover_every_coprocessor_with_three_anchors_each()
        {
            Assert.Equal(12, Goldens.Length);
            Assert.All(Goldens, g => Assert.Equal(3, g.Anchors.Length));
            foreach (string chip in new[] { "SA-1", "GSU", "DSP", "ST01", "OBC1" }) Assert.Contains(Goldens, g => g.Why.Contains(chip));
            Assert.Equal(Goldens.Length, Goldens.Select(g => g.Md5).Distinct().Count());
        }

        // Opt-in: each anchor's VenusRT picture equal to one of Mesen's within the window, or the cause recorded, and its hash as recorded.
        [Fact]
        public void The_goldens_against_Mesen()
        {
            if (Library is not { } library || !Directory.Exists(library))
            {
                _output.WriteLine($"{Variable} unset, not run");
                return;
            }
            string scratch = Path.Combine(Path.GetTempPath(), "EmuSenVenusRtGoldens_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(scratch);
            var failures = new List<string>();
            var table = new List<string> { "game\tframe\tscene\tresult\tlag\tdiffering\tvenusrt_hash\trecorded" };
            try
            {
                foreach (var g in Goldens)
                {
                    if (Find(library, g) is not { } found)
                    {
                        _output.WriteLine($"{g.File}: not in the library, not run");
                        continue;
                    }
                    string rom = Path.Combine(scratch, g.Md5 + Path.GetExtension(found));
                    File.Copy(found, rom, overwrite: true);
                    if (VenusRtSnesEngine.DspRequest(rom) is { } dsp && VenusRtSnesEngine.DspFirmware(rom) is null)
                    {
                        _output.WriteLine($"{g.File}: no {dsp.Name} among the player's images, not run");
                        continue;
                    }
                    var ours = new VenusRtSnesEngine().Run(rom, g.Anchors.Select(a => a.Frame).ToArray(), g.Presses);
                    foreach (var a in g.Anchors)
                    {
                        var mine = ours.At(a.Frame)!.Picture!;
                        string hash = Hash(mine);
                        string result, lag = "", differing = "";
                        if (MesenProbeSnesEngine.Available)
                        {
                            var theirs = new MesenProbeSnesEngine(Path.Combine(scratch, "mesen")).Run(rom, Enumerable.Range(a.Frame - Window, 2 * Window + 1).ToArray(), g.Presses);
                            Assert.True(Lit(theirs.At(a.Frame)?.Picture), $"{g.File} {a.Frame}: Mesen's anchor picture is blank");
                            var best = theirs.Snapshots.Select(s => (s.Frame, SnesDifferential.Pictures(mine, s.Picture, SnesDifferential.MesenRowOffset).Differing))
                                .OrderBy(x => x.Differing).ThenBy(x => Math.Abs(x.Frame - a.Frame)).First();
                            result = best.Differing == 0 ? "equal" : "differs";
                            (lag, differing) = ((best.Frame - a.Frame).ToString(), best.Differing.ToString());
                            Directory.Delete(Path.Combine(scratch, "mesen"), recursive: true);
                            if (a.Recorded.Length > 0 && (result == "equal") != a.Recorded.StartsWith("equal "))
                                failures.Add($"{g.File} {a.Frame}: {result}, recorded {a.Recorded}");
                        }
                        else result = "no probe";
                        if (a.Recorded.Length > 0 && !a.Recorded.EndsWith(" " + hash)) failures.Add($"{g.File} {a.Frame}: hash {hash}, recorded {a.Recorded}");
                        table.Add(string.Join('\t', g.File, a.Frame, a.Scene, result, lag, differing, hash, a.Recorded));
                    }
                }
            }
            finally
            {
                try { Directory.Delete(scratch, recursive: true); } catch (IOException) { }
            }
            foreach (string line in table) _output.WriteLine(line);
            if (Environment.GetEnvironmentVariable(VenusRtTestRomRunnerTests.ReportVariable) is { } report) File.WriteAllLines(report, table);
            Assert.True(failures.Count == 0, string.Join("\n", failures));
        }
    }
}
