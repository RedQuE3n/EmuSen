using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Apu;
using EmuSen.Cores.Nintendo.MoonRT;
using Xunit.Abstractions;
using NesApu = EmuSen.Cores.Nintendo.Moon.Apu.Apu;

namespace EmuSen.WiseMan.Cores
{
    // The APU defects stage 2e fixed, each failing on the unmodified core - see Moon_Native.md §3.12.
    public class MoonApuDefectTests
    {
        private readonly ITestOutputHelper _output;

        public MoonApuDefectTests(ITestOutputHelper output) => _output = output;

        // Past the power-on $4017 write, so the sequencer is running in four-step mode.
        private static NesApu Settled()
        {
            var apu = new NesApu();
            apu.Reset();
            apu.SetSampleRate(44100);
            apu.Step(16);
            return apu;
        }

        // Steps one cycle at a time until <changed> holds, and returns how many cycles that took.
        private static int CyclesUntil(NesApu apu, Func<bool> changed, int limit = 60000)
        {
            for (int n = 1; n <= limit; n++)
            {
                apu.Step(1);
                if (changed()) return n;
            }
            throw new InvalidOperationException("never changed");
        }

        // A1: the flag rises at 29828 and 29829 with the inhibit set, and the IRQ line never does.
        [Fact]
        public void The_frame_flag_rises_for_two_cycles_while_the_irq_is_inhibited()
        {
            var apu = Settled();
            if (!apu.NextCycleIsGet) apu.Step(1);
            apu.Step(1);
            Assert.True(apu.IsGetCycle);
            apu.WriteRegister(0x4017, 0x40);

            var set = new List<int>();
            for (int n = 1; n <= 40000; n++)
            {
                apu.Step(1);
                if (apu.FrameIrqReadable) set.Add(n);
                Assert.False(apu.IrqAsserted);
            }
            Assert.Equal(new[] { 29828 + 4, 29829 + 4 }, set);
        }

        // A2: an enable with the buffer still full asks for no byte until the enable's lag has run, even if the buffer empties inside it.
        [Fact]
        public void An_enable_with_a_full_buffer_waits_out_its_lag()
        {
            DmcChannel Primed()
            {
                var dmc = new DmcChannel();
                dmc.Reset();
                dmc.RateIndex = 15;
                dmc.SampleLength = 17;
                dmc.SetEnabled(true, onGetCycle: true);
                while (!dmc.DmaRequested) dmc.StepTimer();
                dmc.CompleteDma(0xAA);
                dmc.SetEnabled(false, onGetCycle: true);
                return dmc;
            }

            var dry = Primed();
            int empties = 0;
            while (dry.BufferFull) { dry.StepTimer(); empties++; }

            foreach (int early in new[] { 1, 2 })
            {
                var dmc = Primed();
                for (int i = 0; i < empties - early; i++) dmc.StepTimer();
                Assert.True(dmc.BufferFull);
                dmc.SetEnabled(true, onGetCycle: true);

                var requested = new List<bool>();
                for (int i = 0; i < 3; i++) { dmc.StepTimer(); requested.Add(dmc.DmaRequested); }
                Assert.False(dmc.BufferFull);
                Assert.Equal(new[] { false, false, true }, requested);
            }
        }

        // Pulse 1 with a length of 2, and the cycle at which five-step mode's first clock lands after a $4017 write.
        private static NesApu PulseWithLength(byte control, byte lengthIndex, out int clockAt)
        {
            NesApu Make()
            {
                var apu = Settled();
                apu.WriteRegister(0x4015, 0x01);
                apu.WriteRegister(0x4000, control);
                apu.WriteRegister(0x4003, lengthIndex);
                apu.Step(1);
                apu.WriteRegister(0x4017, 0xC0);
                return apu;
            }

            var dry = Make();
            dry.WriteRegister(0x4000, 0x10);
            int before = dry.Pulse1.LengthCounter;
            clockAt = CyclesUntil(dry, () => dry.Pulse1.LengthCounter != before);
            return Make();
        }

        // A3: a halt written the cycle before a length clock is too late for it, and two cycles before is not (blargg's len_halt_timing).
        [Fact]
        public void A_halt_written_the_cycle_before_a_clock_does_not_stop_it()
        {
            var late = PulseWithLength(0x10, 0x18, out int clock);
            late.Step(clock - 1);
            late.WriteRegister(0x4000, 0x30);
            late.Step(1);
            Assert.Equal(1, late.Pulse1.LengthCounter);

            var early = PulseWithLength(0x10, 0x18, out clock);
            early.Step(clock - 2);
            early.WriteRegister(0x4000, 0x30);
            early.Step(2);
            Assert.Equal(2, early.Pulse1.LengthCounter);

            var release = PulseWithLength(0x30, 0x18, out clock);
            release.Step(clock - 1);
            release.WriteRegister(0x4000, 0x10);
            release.Step(1);
            Assert.Equal(2, release.Pulse1.LengthCounter);
        }

        // A4: a reload the cycle before a length clock is lost when the count was running, and is not decremented when it was 0 (len_reload_timing).
        [Fact]
        public void A_reload_the_cycle_before_a_clock_is_lost_or_kept_whole()
        {
            var running = PulseWithLength(0x10, 0x38, out int clock);
            running.Step(clock - 1);
            int count = running.Pulse1.LengthCounter;
            running.WriteRegister(0x4003, 0x18);
            running.Step(1);
            Assert.Equal(count - 1, running.Pulse1.LengthCounter);

            var empty = PulseWithLength(0x10, 0x38, out clock);
            empty.WriteRegister(0x4015, 0x00);
            empty.WriteRegister(0x4015, 0x01);
            empty.Step(clock - 1);
            empty.WriteRegister(0x4003, 0x18);
            empty.Step(1);
            Assert.Equal(2, empty.Pulse1.LengthCounter);
        }

        // A5: the noise period table is in CPU cycles, the count the netlist's LFSR seeds give; the timer runs at half that rate.
        [Theory]
        [InlineData(0, 4)]
        [InlineData(3, 32)]
        [InlineData(8, 202)]
        [InlineData(15, 4068)]
        public void The_noise_channel_clocks_once_per_table_period(int index, int cpuCycles)
        {
            var apu = Settled();
            apu.WriteRegister(0x4015, 0x08);
            apu.WriteRegister(0x400C, 0x3F);
            apu.WriteRegister(0x400E, (byte)index);
            apu.WriteRegister(0x400F, 0x08);

            var changes = new List<int>();
            int last = apu.Noise.Output;
            for (int n = 0; n < cpuCycles * 200; n++)
            {
                apu.Step(1);
                if (apu.Noise.Output != last) { changes.Add(n); last = apu.Noise.Output; }
            }

            Assert.True(changes.Count > 20);
            int gcd = 0;
            for (int i = 1; i < changes.Count; i++) gcd = Gcd(gcd, changes[i] - changes[0]);
            Assert.Equal(cpuCycles, gcd);
        }

        private static int Gcd(int a, int b) => b == 0 ? a : Gcd(b, a % b);

        // A6: a stopped linear counter holds the sequencer but not the timer, so the steps keep one lattice across the gap.
        [Fact]
        public void The_triangle_timer_runs_while_its_sequencer_is_held()
        {
            const int period = 100;
            var apu = Settled();
            apu.WriteRegister(0x4015, 0x04);
            apu.WriteRegister(0x4008, 0xFF);
            apu.WriteRegister(0x400A, period);
            apu.WriteRegister(0x400B, 0x08);
            apu.WriteRegister(0x4017, 0x80);

            var changes = new List<int>();
            int last = apu.Triangle.Output, n = 0;
            void Run(int cycles)
            {
                for (int i = 0; i < cycles; i++, n++)
                {
                    apu.Step(1);
                    if (apu.Triangle.Output != last) { changes.Add(n); last = apu.Triangle.Output; }
                }
            }

            Run(3000);
            int beforeGap = changes.Count;
            apu.WriteRegister(0x4008, 0x00);
            apu.WriteRegister(0x400B, 0x08);
            apu.WriteRegister(0x4017, 0x80);
            Run(1234);
            apu.WriteRegister(0x4008, 0xFF);
            apu.WriteRegister(0x400B, 0x08);
            apu.WriteRegister(0x4017, 0x80);
            Run(3000);

            Assert.True(changes.Count > beforeGap + 10);
            Assert.All(changes, c => Assert.Equal(0, (c - changes[0]) % (period + 1)));
        }

        // A3 and A4's witnesses, and the rest of blargg's 2005 set: each writes its result to $F0, 1 for a pass.
        public static IEnumerable<object[]> Blargg2005() =>
            Enumerable.Range(1, 11).Select(i => new object[] { i });

        private static string? Blargg2005Rom(int number)
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtCorpusTests.CorpusVariable);
            if (folder is null) return null;
            string dir = Path.Combine(folder, "blargg_apu_2005.07.30");
            return Directory.Exists(dir) ? Directory.GetFiles(dir, $"{number:D2}.*.nes").SingleOrDefault() : null;
        }

        [Theory]
        [MemberData(nameof(Blargg2005))]
        public void Blarggs_2005_apu_tests_pass_on_both_engines(int number)
        {
            string? path = Blargg2005Rom(number);
            if (path is null)
            {
                _output.WriteLine($"{MoonRtCorpusTests.CorpusVariable} unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;

            var csharp = new MoonCore();
            csharp.LoadRom(path);
            for (int f = 0; f < 600; f++) csharp.RunFrame();
            Assert.Equal(1, csharp.ReadSpace(MoonCore.SpaceRam, 0xF0));

            Assert.True(MoonRtCore.Available, "MoonRT is not built");
            using var native = new MoonRtCore();
            native.LoadRom(path);
            for (int f = 0; f < 600; f++) native.RunFrame();
            Assert.Equal(1, native.ReadSpace(MoonCore.SpaceRam, 0xF0));
        }
    }
}
