using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The host pieces the cores share before any leaves its shim, each candidate defect shown failing first - see EmuSen_Backend.md §22.
    [Collection(TestCollections.ProcessGlobals)]
    public class SharedHostTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSharedHost_" + Guid.NewGuid().ToString("N"));

        public SharedHostTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string File(string name, byte[] bytes)
        {
            string path = Path.Combine(_root, name);
            System.IO.File.WriteAllBytes(path, bytes);
            return path;
        }

        private static void Frames(CrystalSession s, int count)
        {
            var info = new CrystalFrameInfo();
            for (int i = 0; i < count; i++) s.Frame(ref info);
        }

        // A game of each console a shim still runs: the Game Boy, the NES and the N64.
        private string Game(string console) => console switch
        {
            "gb" => File("Mono.gb", SyntheticGbRom.Build()),
            "nes" => File("Plain.nes", SyntheticNesRom.Build()),
            _ => File("Plain.z64", SyntheticN64Rom.Build()),
        };

        // §17.5 C2, §18.3 C1, §19.6 C2: one cause, a shim answering a refused state with an exception that carries no core status, so the API took it for a fault.
        [Theory]
        [InlineData("gb")]
        [InlineData("nes")]
        [InlineData("n64")]
        public void A_refused_state_reaches_the_API_as_state_refused(string console)
        {
            using var runtime = new CrystalRuntime();
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = Game(console), NoBattery = true });
            Frames(s, 2);
            foreach (byte[] state in new[] { Enumerable.Repeat((byte)'Z', 64).ToArray(), new byte[] { 1, 2, 3, 4 } })
            {
                var refused = Assert.Throws<CrystalException>(() => s.LoadState(File($"Foreign{state.Length}.state", state)));
                Assert.Equal(CrystalAbi.StateRefused, refused.Status);
            }
            Frames(s, 2);
        }

        // §19.5 item 8: a game no board of MoonRT builds reaches the API as a fault in the core's words, as every v1 core's refusal does, not as "no engine".
        [Fact]
        public void A_game_no_board_builds_reaches_the_API_as_a_fault()
        {
            using var runtime = new CrystalRuntime();
            var refused = Assert.Throws<CrystalException>(() => runtime.Open(new CrystalOpen { Path = File("Unbuilt.nes", SyntheticNesRom.Build(mapper: 200)), NoBattery = true }));
            Assert.Equal(CrystalSessionAbi.Fault, refused.Status);
            Assert.Contains("MoonRT (Rust) refused the game: mapper 200", refused.Message);
        }

        // §17.5 C3: a game's own create-time setting reaches its machine when the game opens, not only at a later load in the same session.
        [Fact]
        public void A_game_s_own_create_time_setting_reaches_the_machine_when_it_opens()
        {
            using var f = new CrystalFixture();
            string game = f.Game("Own.tst");
            using (CrystalSession first = f.Open(game)) first.SetSetting("Ram", "42", game: true);
            using CrystalSession again = f.Open(game);
            Span<byte> ram = stackalloc byte[1];
            again.ReadSpace("RAM", 0, ram);
            Assert.Equal(42, ram[0]);
        }

        // §17.2 E2: a create-time setting changed with the game loaded and no frame run makes the machine again at once; after a frame it waits for the next load.
        [Fact]
        public void A_create_time_setting_before_the_first_frame_makes_the_machine_again()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.tst", CoreAdapterTests.Image(1, 2, 3)));
            Assert.Equal(7, engine.ReadSpace("RAM", 0));
            engine.Set("Ram", "42");
            Assert.Equal(42, engine.ReadSpace("RAM", 0));
            engine.RunFrame();
            engine.Set("Ram", "9");
            Assert.Equal(42, engine.ReadSpace("RAM", 0));
            Assert.Equal("9", engine.Get("Ram"));
        }

        // §18.3 C2: the bridge's call stack is the machine's, so a game loaded again in the same engine starts with none of the last one's frames.
        [Fact]
        public void A_new_machine_starts_with_an_empty_call_stack()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            string game = File("game.tst", CoreAdapterTests.Image(1, 2, 3));
            engine.LoadRom(game);
            engine.Debug.CallStack.NotePush(0x10, 0x20, CallFrameKind.Call);
            engine.LoadRom(game);
            Assert.Equal(0, engine.Debug.CallStack.Depth);
        }

        // §18.3 C3, F13: a core whose loop stops for an interrupt without logging a call still reaches `runto irq`, the host noting the interrupt from the stop reason.
        [Fact]
        public void An_interrupt_stopped_for_without_a_call_reaches_runto_irq()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.tst", CoreAdapterTests.Image(0x16, 2, 3)));
            engine.Breakpoints.ArmRunToInterrupt(CallFrameKind.Irq);
            for (int i = 0; i < 8 && !engine.IsHaltedAtBreakpoint; i++) engine.RunFrame();
            Assert.True(engine.IsHaltedAtBreakpoint);
            Assert.Equal(3, engine.TotalFrames);
        }

        // §18.2 F7: a processor whose debug_set_breakpoints is refused cannot halt, so the debugger offers it no breakpoint and no step.
        [Fact]
        public void A_processor_whose_breakpoints_are_refused_cannot_halt()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.tst", CoreAdapterTests.Image(0xC2, 2, 3)));
            var cpus = engine.CreateDebugTarget().DebugCpus;
            Assert.Equal(new[] { ("counter", true), ("helper", false) }, cpus.Select(c => (c.Name, c.CanHalt)));
        }

        // §19.6 C5: coverage records the entry points the call stack sees on a v1 core, as the shims wired it.
        [Fact]
        public void Coverage_records_the_call_stack_s_entry_points()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.tst", CoreAdapterTests.Image(1, 2, 3)));
            CoverageRegistry coverage = engine.CreateDebugTarget().Coverage!;
            coverage.Arm();
            engine.Debug.CallStack.NotePush(0x10, 0x40, CallFrameKind.Call);
            Assert.Contains(coverage.EntryPoints(8), e => e.Address == 0x40 && e.Kind == CallFrameKind.Call);
        }

        // §17.2 E8: the labels are the engine's, so a label outlives the target it was made through.
        [Fact]
        public void Labels_outlive_the_debug_target_they_were_made_through()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.tst", CoreAdapterTests.Image(1, 2, 3)));
            engine.CreateDebugTarget().Labels!.Add(0x40, "reset");
            Assert.True(engine.CreateDebugTarget().Labels!.TryGetName(0x40, out string name));
            Assert.Equal("reset", name);
        }

        // §19.6 C6: a read between frames after a ROM patch is entered answers as the next frame will, the patches brought up first.
        [Fact]
        public void A_read_between_frames_answers_with_the_rom_patches_entered_since()
        {
            CoreDiscovery.UseDirectories(null);
            DiscoveredCore venus = CoreDiscovery.Found.Single(c => c.Info.Id == "venusrt");
            using var engine = new CoreEngine(venus.Open()!) { Options = TestCores.NoBattery };
            engine.LoadRom(File("game.sfc", SyntheticRom.Build((5, new byte[] { 0x80, 0xFE }))));
            engine.RunFrame();
            byte before = engine.ReadSpace("CpuBus", 0x8005);
            engine.Cheats.AddCheat(CheatKind.RomPatch, new[] { CheatWrite.Patch(0x8005, (byte)(before ^ 0xFF)) }, null, "flip");
            Assert.Equal((byte)(before ^ 0xFF), engine.ReadSpace("CpuBus", 0x8005));
        }

        // §17.5 C4: emusen-shell keys a game's pad bindings by its console, as Mistress does, not by the engine's name.
        [Fact]
        public void Hotaru_keys_pad_bindings_by_the_console()
        {
            using var f = new CrystalFixture();
            using CrystalSession s = f.Open(f.Game("Pad.tst"));
            using JsonDocument info = JsonDocument.Parse(s.Info());
            string console = info.RootElement.GetProperty("console").GetString()!;
            Assert.NotEqual(console, EmuSen.Hotaru.HotaruSession.CoreNameOf(s));
            Assert.Equal(console, EmuSen.Hotaru.HotaruSession.BindingsConsoleOf(s));
        }

        // The triples the generic engine sends, in ResolveRomPatches' order and within the console's range; no compare is uint.MaxValue (moved from the deleted NativeHostTests).
        [Fact]
        public void The_rom_patch_triples_keep_the_registrys_order_within_their_bounds()
        {
            var cheats = new CheatRegistry();
            cheats.AddRomPatch(0x9123, 0x42, 0x07, "compared");
            cheats.AddRomPatch(0x9123, 0x55, null, "later");
            cheats.AddRomPatch(0x3FFF, 0x01, null, "below");

            uint[] nes = CoreEngine.RomPatchTriples(cheats, 0x4020, 0xFFFF);
            Assert.Equal(new uint[] { 0x9123, 0x42, 0x07, 0x9123, 0x55, uint.MaxValue }, nes);
            uint[] gb = CoreEngine.RomPatchTriples(cheats, 0, 0x7FFF);
            Assert.Equal(new uint[] { 0x3FFF, 0x01, uint.MaxValue }, gb);
            Assert.Empty(CoreEngine.RomPatchTriples(new CheatRegistry(), 0, 0xFFFF));
        }
    }
}
