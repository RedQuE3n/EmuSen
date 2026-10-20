using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems.Sms;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Endou's stage 7 gate as a player meets it: routing of the tester's library, a game of each system started through the session Mistress uses, battery files, and both cheat formats - see Endou_Native.md §14.
    [Collection(TestCollections.ProcessGlobals)]
    public class EndouPlayersTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenEndouPlayers_" + Guid.NewGuid().ToString("N"));

        public EndouPlayersTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The tester's mastersystem folder, read in place and never written, where EMUSEN_ENDOU_LIBRARY names it; its firmware files are left out by name.
        private static string? Library => Environment.GetEnvironmentVariable("EMUSEN_ENDOU_LIBRARY") is { } l && Directory.Exists(l) ? l : null;

        private static IEnumerable<string> Games(string library) =>
            Directory.EnumerateFiles(library).Where(f => !Path.GetFileName(f).Contains("BIOS", StringComparison.Ordinal)).Order(StringComparer.Ordinal);

        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        [Fact]
        public void The_testers_library_routes_each_image_to_its_system_and_its_shelf()
        {
            if (Library is not { } library)
            {
                _output.WriteLine("skipped: EMUSEN_ENDOU_LIBRARY names the tester's mastersystem folder, read only");
                return;
            }
            var games = Games(library).ToArray();
            var sms = games.Where(f => Path.GetExtension(f) == ".sms").ToArray();
            var sg = games.Where(f => Path.GetExtension(f) == ".sg").ToArray();
            Assert.Equal(games.Length, sms.Length + sg.Length);
            foreach (var (files, console, shelf) in new[] { (sms, "Master System", "Master System (Endou)"), (sg, "SG-1000", "SG-1000 (Endou)") })
            {
                Assert.NotEmpty(files);
                Assert.All(files, f => Assert.Equal((console, shelf, shelf), (CoreCatalog.ConsoleForRom(f), CoreCatalog.ShelfFor(f), EmuSen.Mistress.Library.RuntimeCatalog.ShelfFor(f))));
            }
            _output.WriteLine($"{sms.Length} .sms to the Master System's shelf, {sg.Length} .sg to the SG-1000's");
        }

        // The session Mistress loads a game through, headless: a library game of each system runs on Endou and draws.
        [Theory]
        [InlineData("Sonic the Hedgehog (UE) [!].sms", "sms", 300)]
        [InlineData("Flicky (SG-1000).sg", "sg1000", 120)]
        public void A_library_game_of_each_system_starts_through_the_session(string name, string system, int frames)
        {
            if (Library is not { } library)
            {
                _output.WriteLine("skipped: EMUSEN_ENDOU_LIBRARY names the tester's mastersystem folder, read only");
                return;
            }
            var session = new EmulatorSession();
            session.LoadRom(Path.Combine(library, name));
            var core = Assert.IsType<CoreEngine>(session.Core);
            Assert.Equal(("endou", system), (core.Info.Id, core.Machine.Info.System));
            for (int f = 0; f < frames; f++) session.RunFrame();
            byte[] frame = core.GetFrameBufferRgba();
            int lit = Enumerable.Range(0, frame.Length / 4).Count(p => frame[4 * p] + frame[4 * p + 1] + frame[4 * p + 2] > 0);
            core.ReturnFrameBuffer(frame);
            _output.WriteLine($"{name}: {session.TotalFrames} frames, {lit} lit pixels of {session.ScreenWidth}x{session.ScreenHeight}");
            Assert.Equal(frames, session.TotalFrames);
            Assert.True(lit > 1000, $"{lit} pixels lit");
            (core as IDisposable)?.Dispose();
        }

        // Rewind as Mistress keeps it, two snapshots a second: a rewind lands on the state held, and the run from it draws the same frames again (Endou_Plan.md §5.7).
        [Fact]
        public void A_rewind_lands_on_the_state_it_held_and_runs_on_alike()
        {
            string path = Write("Rewound.sms", SyntheticSmsRom.Picture());
            using var core = (CoreEngine)CoreFactory.Load(path).Core;
            var rewind = new RewindBuffer { Enabled = true, IntervalFrames = 30 };
            for (int f = 0; f < 300; f++)
            {
                core.RunFrame();
                core.DequeueAudioSamples(8192);
                if (f == 60) rewind.CaptureNow(core);
                else if (f > 60) rewind.OnFrameCompleted(core);
            }
            Assert.True(rewind.Depth > 0);
            long target = rewind.Moments()[rewind.Moments().Count / 2].Frame;
            byte[] held = rewind.StateAt(target)!;
            Assert.True(rewind.RewindTo(core, target));
            using var after = new MemoryStream();
            core.SaveState(after);
            Assert.True(held.AsSpan().SequenceEqual(after.ToArray()), "the state after the rewind is the one held");
            core.RunFrame();
            byte[] first = core.GetFrameBufferRgba().ToArray();
            Assert.True(rewind.RewindTo(core, target));
            core.RunFrame();
            Assert.Equal(first, core.GetFrameBufferRgba());
        }

        // A battery board's RAM is saved as an .srm under Saves/Master System and read back at the next load (Endou_Plan.md §4.4).
        [Fact]
        public void A_battery_board_s_srm_round_trips()
        {
            if (Library is not { } library)
            {
                _output.WriteLine("skipped: EMUSEN_ENDOU_LIBRARY names the tester's mastersystem folder, read only");
                return;
            }
            string game = Path.Combine(library, "Phantasy Star (UE) [!].sms");
            byte[] written = Enumerable.Range(0, 0x2000).Select(i => (byte)(i * 7 + 3)).ToArray();
            using (var core = (CoreEngine)CoreFactory.Load(game).Core)
            {
                Assert.Equal(new[] { (0u, ".srm") }, core.Machine.Info.Battery.Select(b => (b.Which, b.Suffix)));
                core.RunFrame();
                Assert.True(core.Machine.WriteSpace(core.Machine.Info.Spaces.Single(s => s.Name == "CARTRAM").Id, 0, written));
                core.SaveSram();
            }
            string srm = Path.Combine(DataStore.Saves, "Master System", "Phantasy Star (UE) [!].srm");
            Assert.Equal(written, File.ReadAllBytes(srm));
            using (var again = (CoreEngine)CoreFactory.Load(game).Core)
                Assert.Equal(written, again.Machine.Battery(0).Data);
        }

        // A Pro Action Replay code pokes the work RAM every frame; a Game Genie code patches the cartridge's read, heeding its reference (EmuSen_Cheats.md §11).
        [Fact]
        public void Both_cheat_formats_reach_the_machine()
        {
            // A Japanese header, so the Japanese Master System runs it, its open program checking no header: an export console's sums the patched byte, as a BIOS would, and refuses the game.
            byte[] image = SyntheticSmsRom.Banked(2, SyntheticSmsRom.Program(new[] { SyntheticSmsRom.Step.Keep(0x3F00), SyntheticSmsRom.Step.Keep(0x7F00) }));
            image[SyntheticSmsRom.HeaderAt + 15] = 0x3C;
            string path = Write("Cheats.sms", SyntheticSmsRom.WithSegaChecksum(image));
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(path, headless: true, cheats);
            using var core = (CoreEngine)bundle.Core;
            ICheatCodeCodec par = bundle.CheatAutoDetectCodec!, genie = bundle.CheatExplicitCodec!;
            var (ram, value) = par.Decode("00C1-0042");
            Assert.Equal((0x0100, (byte)0x42), (ram, value));
            cheats.AddRamPoke(par.SpaceName!, ram, value, "poke");
            var (address, patch) = genie.Decode("5AF-00C");
            Assert.Equal((0x3F00, (byte)0x5A), (address, patch));
            cheats.AddRomPatch(address, patch, genie.DecodeCompare("5AF-00C"), "patch");
            cheats.AddRomPatch(0x7F00, 0xA5, 0x77, "a reference the byte does not have");
            for (int f = 0; f < 60; f++) core.RunFrame();
            Assert.Equal(new byte[] { 0x5A, 0x01 }, new[] { core.ReadSpace("RAM", 0x0000), core.ReadSpace("RAM", 0x0001) });
            Assert.Equal(0x42, core.ReadSpace("RAM", 0x0100));
        }
    }
}
