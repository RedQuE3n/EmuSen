using System.Diagnostics;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Ps2;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Sapphire's stage S1 stub through the v1 adapter, discovery, routing and the conformance kit, on the synthetic program and discs - see Sapphire_Native.md §2-§6.
    [Collection(TestCollections.ProcessGlobals)]
    public class SapphireTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSapphire_" + Guid.NewGuid().ToString("N"));

        public SapphireTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        public static string LibraryPath => CoreAdapterTests.LibraryPath("sapphire");

        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static readonly (uint, byte[])[] NoFiles = Array.Empty<(uint, byte[])>();

        [Fact]
        public void The_library_is_on_the_core_abi_and_its_system_is_the_pack()
        {
            var library = CoreLibrary.Open(LibraryPath);
            Assert.True(library.Available, library.Report);
            Assert.Equal(("sapphire", "Sapphire", CoreInterface.CapAxes | CoreInterface.CapMedia | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble | CoreInterface.CapDebugReads), (library.Info.Id, library.Info.DisplayName, library.Capabilities));
            var system = library.Info.Systems.Single();
            Assert.Equal((Ps2System.Id, true), (system.Id, system.Development));
            Assert.Equal(Ps2System.CoreExtensions, system.Extensions);
            Assert.All(system.Extensions, e => Assert.Contains(e, Ps2System.Entry.Extensions));
            Assert.Equal(Ps2System.Bios.Select(b => (b.Name, b.Size)), system.Firmware.Select(f => (f.Name, f.Size)));
            Assert.All(system.Firmware, f => Assert.False(f.Required));
            Assert.Equal(new[] { ("ps2.dualshock2", 16, 4), ("ps2.digital", 14, 0) }, system.Controllers.Select(c => (c.Id, c.Buttons.Count, c.Axes.Count)));
            Assert.Equal(new[] { "region", "pad1", "pad2", "ee_tier", "vu1" }, library.Settings.Select(s => s.Key));
            Assert.Equal("ps2-0230a-20080220.bin", library.FirmwareFor(SyntheticPs2Elf.Build()).Single().Name);
            Assert.Equal("ps2-0230a-20080220.bin", library.FirmwareFor(SyntheticPs2Disc.Dvd().Iso).Single().Name);
            Assert.Empty(library.FirmwareFor(new byte[4096]));
            Assert.Same(Ps2System.Entry, SystemPacks.For("ps2")!.Entry);
        }

        [Fact]
        public void A_program_and_each_disc_form_make_a_machine_and_other_images_are_refused_with_words()
        {
            var library = CoreLibrary.Open(LibraryPath);
            foreach (var (image, settings, region, height) in new (byte[], string, string, int)[]
            {
                (SyntheticPs2Elf.Build(), "", "ntsc", 448),
                (SyntheticPs2Disc.Cd().DataTrack, "", "ntsc", 448),
                (SyntheticPs2Disc.Cd().Iso, "region=ps2.eu", "pal", 512),
                (SyntheticPs2Disc.Dvd().Iso, "", "ntsc", 448),
            })
            {
                using var machine = new CoreMachine(library, image, settings, NoFiles);
                var info = machine.Info;
                Assert.Equal(("ps2", region, 640, height), (info.System, info.Region, info.BaseWidth, info.BaseHeight));
                Assert.Equal(new[] { "R5900", "IOP", "VU0", "VU1" }, info.Processors.Select(p => p.Name));
                Assert.Equal((48000, "SPH2", Ps2System.DisplayAspect), (info.AudioRate, info.StateFormat, (double)info.AspectNum / info.AspectDen));
            }
            foreach (var (image, words) in new (byte[], string)[]
            {
                (SyntheticPsxDisc.Build().Iso, "Diamond"),
                (SyntheticPs2Elf.Build(type: SyntheticPs2Elf.IopModule), "IOP module"),
                (SyntheticPs2Elf.Build(elfClass: 2), "64-bit"),
                (SyntheticPs2Disc.Cd("VMODE = NTSC\r\n").Iso, "BOOT2"),
                (new byte[50_000], "neither"),
            })
            {
                var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(library, image, "", NoFiles));
                _output.WriteLine(refused.Message);
                Assert.Contains(words, refused.Message);
            }
        }

        // A PlayStation disc goes to the PlayStation and a PlayStation 2 CD or DVD to the PlayStation 2, by BOOT and BOOT2, in both halves of the recogniser, wherever the file sits - see Sapphire_Native.md §3.2.
        [Fact]
        public void A_playstation_disc_routes_to_the_playstation_and_a_playstation_2_disc_to_the_playstation_2()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            foreach (var (label, file, image, expected, serial) in new (string, string, byte[], string, string)[]
            {
                ("ps1 iso", "game.iso", SyntheticPsxDisc.Build().Iso, "psx", "EMUS-00001"),
                ("ps1 track", "game.bin", SyntheticPsxDisc.Build().DataTrack, "psx", "EMUS-00001"),
                ("ps2 cd iso", "game.iso", SyntheticPs2Disc.Cd().Iso, "ps2", SyntheticPs2Disc.Serial),
                ("ps2 cd track", "game.bin", SyntheticPs2Disc.Cd().DataTrack, "ps2", SyntheticPs2Disc.Serial),
                ("ps2 cd with the licence text", "game.iso", SyntheticPs2Disc.Cd(licence: true).Iso, "ps2", SyntheticPs2Disc.Serial),
                ("ps2 dvd", "game.iso", SyntheticPs2Disc.Dvd().Iso, "ps2", SyntheticPs2Disc.Serial),
            })
            {
                string extension = Path.GetExtension(file);
                var candidates = SystemPacks.All.Select(p => p.Entry).Where(e => e.Extensions.Contains(extension)).ToArray();
                Assert.Contains(candidates, c => c.Id == "ps2");
                var managed = Enumerable.Range(0, candidates.Length).Select(at => (at, SystemPacks.Literals.Single(l => l.Id == candidates[at].Id).Recognise?.Invoke(new Probe(image)))).Where(c => c.Item2 is not null).Select(c => (c.at, c.Item2!)).ToArray();
                var native = Enumerable.Range(0, candidates.Length).Select(at => (at, candidates[at].Recognise?.Invoke(new Probe(image)))).Where(c => c.Item2 is not null).Select(c => (c.at, c.Item2!)).ToArray();
                Assert.True(managed.Length == 1 && native.Length == 1, $"{label}: claimed by {managed.Length} and {native.Length}");
                foreach (string? folder in new[] { null, "psx", "ps2", "segacd", "roms" })
                {
                    var m = SystemRouting.DecideManaged(file, () => folder, candidates, managed);
                    var n = SystemCode.Route(file, folder, candidates, native)!;
                    Assert.Equal((expected, serial), (m.System?.Id, m.Recognition?.Serial));
                    Assert.Equal((expected, serial), (n.System?.Id, n.Recognition?.Serial));
                }
            }
        }

        private sealed class Probe(byte[] bytes) : IMediaProbe
        {
            public string Path => "image.iso";
            public string ImageName => "image.iso";
            public byte[] Read(int length) => bytes.Length <= length ? bytes : bytes[..length];
        }

        // A developer asking for development cores opens a program on Sapphire; a player is offered nothing - see EmuSen_CoreAPI.md §27.
        [Fact]
        public void A_program_opens_on_the_engine_only_when_development_cores_are_asked_for()
        {
            Assert.Contains(CoreDiscovery.Found, c => c.Info.Id == "sapphire");
            string path = Write("program.elf", SyntheticPs2Elf.Build());
            Assert.True(CoreFactory.IsSupported(path));
            var bundle = CoreFactory.Load(path);
            var core = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.Equal(("sapphire", "ps2"), (core.Info.Id, core.Machine.Info.System));
            core.RunFrame();
            Assert.Equal((640, 448), (core.ScreenWidth, core.ScreenHeight));
            (core as IDisposable)?.Dispose();

            Assert.True(CoreSidecar.Read(CoreSidecar.PathFor(LibraryPath))!.Development);
            CoreDiscovery.UseDevelopment(false);
            Assert.DoesNotContain(CoreDiscovery.Found, c => c.Info.Id == "sapphire");
            Assert.False(CoreFactory.IsSupported(path));
            Assert.False(CoreCatalog.IsRomExtension(".elf"));
            Assert.DoesNotContain("PlayStation 2", CoreCatalog.DiscoveredConsoles);
            Assert.DoesNotContain(CoreCatalog.ShelvesInReleaseOrder, s => s.System == "ps2");
        }

        // The gate of Sapphire_Plan.md §7's S1: the kit's C1-C15 on a synthetic program and synthetic CD and DVD images, nothing failing.
        [Fact]
        public async Task The_kit_passes_on_a_synthetic_program_and_synthetic_discs()
        {
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            Assert.True(File.Exists(runner), $"no kit runner at {runner}");
            foreach (var (name, image) in new[] { ("program.elf", SyntheticPs2Elf.Build()), ("cd.iso", SyntheticPs2Disc.Cd().Iso), ("dvd.iso", SyntheticPs2Disc.Dvd().Iso) })
            {
                var start = new ProcessStartInfo(runner) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in new[] { "--core", LibraryPath, "--image", Write(name, image), "--frames", "120" }) start.ArgumentList.Add(a);
                using var process = Process.Start(start)!;
                var stderr = process.StandardError.ReadToEndAsync();
                string output = process.StandardOutput.ReadToEnd();
                string errors = await stderr;
                await process.WaitForExitAsync();
                _output.WriteLine($"{name}:\n{output}{errors}");
                Assert.True(process.ExitCode == 0, output + errors);
                var passed = output.Split('\n').Where(l => l.StartsWith("pass C")).Select(l => l.Split(' ', StringSplitOptions.RemoveEmptyEntries)[1]).ToHashSet();
                Assert.All(Enumerable.Range(1, 15).Select(n => $"C{n}"), c => Assert.Contains(c, passed));
                Assert.DoesNotContain(output.Split('\n'), l => l.StartsWith("fail"));
            }
        }
    }
}
