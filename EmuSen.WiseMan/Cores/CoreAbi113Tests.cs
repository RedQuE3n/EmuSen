using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Core ABI 1.13's six items through the host, on the test core that claims each - see EmuSen_CoreAPI.md §6.33-§6.38.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreAbi113Tests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreAbi113_" + Guid.NewGuid().ToString("N"));

        public CoreAbi113Tests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CoreEngine Engine(params byte[] payload)
        {
            string path = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(path, CoreAdapterTests.Image(payload.Length == 0 ? new byte[] { 1, 2, 3 } : payload));
            var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(path);
            return engine;
        }

        [Fact]
        public void The_interface_has_1_13s_three_capabilities()
        {
            Assert.True(CoreInterface.Major == 1 && CoreInterface.Minor >= 13);
            Assert.Equal(new[] { "TELEMETRY", "PRESENCE", "DEBUG_TRANSLATE" }, CoreInterface.CapabilityNames.Where(c => c.Bit >= 1UL << 23).Select(c => c.Name));
            Assert.True(CoreAdapterTests.Full.Has(CoreInterface.CapTelemetry | CoreInterface.CapPresence | CoreInterface.CapDebugTranslate));
            Assert.False(CoreAdapterTests.Plain.Has(CoreInterface.CapTelemetry));
        }

        // §6.33: machine info's units, read by id through debug_registers; a timer's registers are shown with the sound's.
        [Fact]
        public void A_unit_is_read_through_debug_registers_and_shown_by_its_kind()
        {
            using CoreEngine engine = Engine();
            for (int i = 0; i < 3; i++) engine.RunFrame();
            CoreUnit unit = Assert.Single(engine.Machine.Info.Units);
            Assert.Equal((256u, "Timer", "timer"), (unit.Id, unit.Name, unit.Kind));
            Assert.Equal(new[] { ("Count", 8), ("Frames", 64) }, unit.Registers);
            Assert.Equal(new long[] { 5, 3 }, engine.Machine.Registers(256));
            CoreDebugTarget target = engine.CreateDebugTarget();
            target.RefreshProviders();
            Assert.Equal(new[] { ("Count", 5UL), ("Frames", 3UL) }, target.ApuRegisters.Current.Select(r => (r.Name, r.Value)));
            Assert.Empty(target.VideoRegisters.Current);
        }

        // §6.34: the counters named in machine info, read into a buffer the host made, and the target's provider.
        [Fact]
        public void The_telemetry_counters_are_read_into_the_callers_buffer()
        {
            using CoreEngine engine = Engine();
            for (int i = 0; i < 5; i++) engine.RunFrame();
            Assert.Equal(new[] { "test.frames", "test.samples" }, engine.Machine.Info.Telemetry.Select(c => c.Name));
            Span<long> values = stackalloc long[4];
            Assert.Equal(2, engine.Telemetry(values));
            Assert.Equal(5, values[0]);
            Assert.Equal(1, engine.Telemetry(values[..1]) - 1);
            CoreDebugTarget target = engine.CreateDebugTarget();
            target.RefreshProviders();
            DebugCounterValue frames = target.Counters.Current[0];
            Assert.Equal(("test.frames", "Frames run", "frames", "total", 5L), (frames.Name, frames.Label, frames.Unit, frames.Kind, frames.Value));
        }

        // §6.36: a controller pulled out reaches the machine as the next frame starts, and only a change is sent.
        [Fact]
        public void A_controller_pulled_out_reaches_the_machine_at_the_next_frame()
        {
            using CoreEngine engine = Engine();
            engine.SetControllerConnected(1, false);
            Assert.Equal(0, engine.ReadSpace("RAM", 5));
            engine.RunFrame();
            Assert.Equal(2, engine.ReadSpace("RAM", 5));
            engine.SetControllerConnected(1, true);
            engine.SetControllerConnected(1, false);
            engine.RunFrame();
            Assert.Equal(2, engine.ReadSpace("RAM", 5));
            engine.SetControllerConnected(1, true);
            engine.RunFrame();
            Assert.Equal(0, engine.ReadSpace("RAM", 5));
        }

        // §6.35: the console's lines in a picture drawn at a multiple, which ICore.DisplayLines answers.
        [Fact]
        public void The_lines_of_a_picture_drawn_at_a_multiple_are_the_engines_display_lines()
        {
            using CoreEngine engine = Engine();
            engine.RunFrame();
            Assert.Equal(0, ((EmuSen.Cores.ICore)engine).DisplayLines);
            engine.Set("Wide", "true");
            engine.RunFrame();
            Assert.Equal(2, ((EmuSen.Cores.ICore)engine).DisplayLines);
        }

        // §6.37: a frame flagged CHEATS_HELD gets none of the host's cheats at its end; every other frame does.
        [Fact]
        public void A_frame_that_holds_the_cheats_gets_none_of_the_hosts()
        {
            using CoreEngine engine = Engine(0xC4, 2, 3);
            engine.Cheats.AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke("RAM", 0x10, 0x5A5A, width: 2) }, null, "two bytes, the host's to apply");
            var after = Enumerable.Range(0, 8).Select(_ =>
            {
                engine.WriteSpace("RAM", 0x10, 0);
                engine.RunFrame();
                return engine.ReadSpace("RAM", 0x10);
            }).ToArray();
            Assert.Equal(new byte[] { 0x5A, 0x5A, 0x5A, 0, 0x5A, 0x5A, 0x5A, 0 }, after);
        }

        // §6.38: an address the core's translation maps nowhere is unmapped; a mapped one goes to the console's map, which the test system has none of.
        [Fact]
        public void An_address_is_translated_before_the_consoles_map_names_it()
        {
            using CoreEngine engine = Engine();
            Assert.Equal(0x34u, engine.Machine.Translate(0, 0x1234));
            Assert.Null(engine.Machine.Translate(0, 0x1_0000));
            CoreDebugTarget target = engine.CreateDebugTarget();
            PhysicalAddress? unmapped = target.ResolvePhysical(0x1_0000);
            Assert.Equal(("unmapped", false), (unmapped!.Value.Space, unmapped.Value.IsAddressable));
            Assert.Null(target.ResolvePhysical(0x1234));
        }
    }
}
