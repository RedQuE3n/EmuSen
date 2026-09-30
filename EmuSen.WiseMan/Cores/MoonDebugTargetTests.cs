using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The C# Moon's own debug-target cases, which poke its machine directly; the rest run on both engines in MoonDebugTargetContract - see Moon_Debug.md.
    public class MoonDebugTargetTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private (MoonCore Core, MoonDebugTarget Target) Load(params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: patches));
            _temporaryFiles.Add(path);

            var core = new MoonCore();
            core.LoadRom(path);
            return (core, new MoonDebugTarget(core));
        }

        [Fact]
        public void Cpu_registers_report_the_live_machine()
        {
            var (core, target) = Load();

            core.Cpu!.A = 0x12;
            core.Cpu.X = 0x34;
            target.RefreshProviders();

            var registers = target.CpuRegisters.Current;
            Assert.Equal(0x12u, registers.First(r => r.Name == "A").Value);
            Assert.Equal(0x34u, registers.First(r => r.Name == "X").Value);
        }

        // The NES had no load bars at all until 2026-08-04 - see Moon_Debug.md §3.2.
        [Fact]
        public void Hardware_load_is_normalized_against_one_native_frame()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build());
            _temporaryFiles.Add(path);
            var core = new MoonCore();
            core.LoadRom(path);

            // ~16.639ms is one full frame at 60.0985Hz, so half of it reads as ~50%.
            var target = new MoonDebugTarget(core, () => (8.3195, 4.16));
            target.RefreshProviders();

            var load = target.HardwareLoad.Current;
            Assert.Equal(50.0, Assert.Single(load, l => l.Name == "CPU+APU").Percent, 1);
            Assert.Equal(25.0, Assert.Single(load, l => l.Name == "PPU").Percent, 1);

            // Wall-clock emulator timings, not guest load - see EmuSen_Cauldron.md §4.5.
            Assert.All(load, l => Assert.Equal(DebugLoadKind.EmulatorCost, l.Kind));
        }

        [Fact]
        public void Hardware_load_over_a_full_frame_is_clamped()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build());
            _temporaryFiles.Add(path);
            var core = new MoonCore();
            core.LoadRom(path);

            var target = new MoonDebugTarget(core, () => (50.0, 50.0));
            target.RefreshProviders();

            Assert.All(target.HardwareLoad.Current, l => Assert.Equal(100.0, l.Percent));
        }

        // Real frames must produce real numbers, not the injected-only path above.
        [Fact]
        public void Running_frames_attributes_time_to_both_phases()
        {
            var (core, target) = Load();

            for (int i = 0; i < 3; i++) core.RunFrame();
            target.RefreshProviders();

            Assert.True(core.LastFrameCpuApuMs > 0, "the CPU+APU phase recorded no time");
            Assert.True(core.LastFramePpuMs > 0, "the PPU phase recorded no time");
            Assert.Equal(2, target.HardwareLoad.Current.Count);
        }

        // Guards the defaulted-no-op regression, not the refresh itself - see Moon_Debug.md §3.
        [Fact]
        public void Refreshing_through_the_interface_publishes_without_a_frame()
        {
            var (core, concrete) = Load();
            IDebugTarget target = concrete;

            core.Cpu!.A = 0x5A;
            target.RefreshProviders();

            Assert.Equal(0x5Au, target.CpuRegisters.Current.First(r => r.Name == "A").Value);
        }

    }
}
