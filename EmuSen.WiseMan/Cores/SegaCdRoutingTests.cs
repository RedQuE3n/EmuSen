using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Media;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // With the Sega CD offered, the extensions it shares with the PlayStation still route each disc to its own engine, and a cartridge's .bin to the Genesis - see Nephrite_Plan.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class SegaCdRoutingTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSegaCdRouting_" + Guid.NewGuid().ToString("N"));

        public SegaCdRoutingTests(ITestOutputHelper output)
        {
            _output = output;
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The tester's library, read and never written, where EMUSEN_ROUTING_LIBRARY names it.
        private static string? Library => Environment.GetEnvironmentVariable("EMUSEN_ROUTING_LIBRARY") is { } l && Directory.Exists(l) ? l : null;

        private (string? Console, string? Engine) Routed(string path)
        {
            var (core, routed) = CoreCatalog.Route(path);
            string? system = routed?.System?.Id ?? EmuSen.Crystal.Systems.SystemPacks.All.FirstOrDefault(p => p.Entry.Console == core?.Console)?.Entry.Id;
            string? engine = system is null ? null : CoreDiscovery.ForSystem(system).FirstOrDefault()?.Info.Id;
            _output.WriteLine($"{Path.GetRelativePath(Library ?? _root, path)} | {core?.Console ?? "nowhere"} | {engine ?? "-"} | {routed?.Reason}");
            return (core?.Console, engine);
        }

        // Offered to players, the Sega CD has its shelf with no development cores asked for, and the 32X has none.
        [Fact]
        public void The_sega_cd_is_offered_and_the_32x_is_not()
        {
            Assert.Contains("Sega CD", CoreCatalog.DiscoveredConsoles);
            Assert.DoesNotContain("32X", CoreCatalog.DiscoveredConsoles);
            Assert.Contains(CoreCatalog.ShelvesInReleaseOrder, s => s.Core.Console == "Sega CD" && s.EsdeSystem == "segacd");
            Directory.CreateDirectory(_root);
            File.WriteAllBytes(Path.Combine(_root, "Sapphire Coast.iso"), SyntheticMdRom.Disc());
            Assert.Equal("Sega CD (Nephrite)", Assert.Single(RomLibrary.Scan(_root).Entries).Shelf);
        }

        // The tester's discs and cartridges as the library lists them, a sheet's tracks under its sheet: every PlayStation sheet and lone track to Diamond, every Sega CD sheet to Nephrite, every cartridge .bin to the Genesis.
        [Fact]
        public void The_testers_discs_and_cartridges_each_route_to_their_own_engine()
        {
            if (Library is not { } library)
            {
                _output.WriteLine("skipped: EMUSEN_ROUTING_LIBRARY names the tester's library, read only");
                return;
            }
            string[] Files(string folder, params string[] extensions)
            {
                if (!Directory.Exists(Path.Combine(library, folder))) return Array.Empty<string>();
                var all = Directory.EnumerateFiles(Path.Combine(library, folder), "*", SearchOption.AllDirectories).ToArray();
                var tracks = all.Where(ImageSets.IsSheet).SelectMany(ImageSets.Companions).ToHashSet(StringComparer.Ordinal);
                return all.Where(f => extensions.Contains(Path.GetExtension(f), StringComparer.OrdinalIgnoreCase) && !tracks.Contains(f)).Order(StringComparer.Ordinal).ToArray();
            }
            string[] psx = Files("psx", ".cue", ".bin", ".chd", ".iso"), segaCd = Files("SCD", ".cue", ".chd", ".iso"), genesis = Files("genesis", ".bin");
            Assert.Contains(psx, f => Path.GetFileName(f) == "Mega Man Legends 2 (USA) (Track 1).bin");
            Assert.NotEmpty(segaCd);
            Assert.NotEmpty(genesis);
            Assert.All(psx, f => Assert.Equal(("PlayStation", "diamond"), Routed(f)));
            Assert.All(segaCd, f => Assert.Equal(("Sega CD", "nephrite"), Routed(f)));
            Assert.All(genesis, f => Assert.Equal(("Genesis", "nephrite"), Routed(f)));
        }

        // A CHD's first bytes are its own compressed container's, which neither console's recogniser reads, so its folder's ES-DE name decides, and elsewhere it opens nowhere with the words saying why.
        [Fact]
        public void A_chd_is_routed_by_its_folders_name()
        {
            string Chd(string folder)
            {
                string dir = Path.Combine(_root, folder);
                Directory.CreateDirectory(dir);
                string path = Path.Combine(dir, "Disc.chd");
                File.WriteAllBytes(path, "MComprHD"u8.ToArray().Concat(new byte[0x8000]).ToArray());
                return path;
            }
            Assert.Equal(("PlayStation", "diamond"), Routed(Chd("psx")));
            Assert.Equal(("Sega CD", "nephrite"), Routed(Chd("segacd")));
            Assert.Equal((null, null), Routed(Chd("SCD")));
            Assert.Contains("could be", CoreCatalog.Route(Chd("Games")).Routed?.Reason);
        }
    }
}
