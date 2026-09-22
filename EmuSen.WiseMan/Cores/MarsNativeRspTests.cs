using System;
using System.IO;
using System.Reflection;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.Mars.Rsp;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The Rust signal processor against the C# one, on random programs of every instruction class, compared as whole save states - see Mars_Native.md §3.3.
    [Collection("MarsStatics")]
    public class MarsNativeRspTests : IDisposable
    {
        private readonly bool _was = Rsp.UseNative;

        public void Dispose() => Rsp.UseNative = _was;

        // Every class the processor decodes, weighted toward the vector unit, with breaks and interface reads for the hand-back.
        private static uint Instruction(Random random)
        {
            uint R(int bits) => (uint)random.Next(1 << bits);
            int roll = random.Next(100);
            return roll switch
            {
                < 25 => 0x4A00_0000u | R(4) << 21 | R(5) << 16 | R(5) << 11 | R(5) << 6 | R(6),
                < 40 => (random.Next(2) == 0 ? 0x32u : 0x3Au) << 26 | R(5) << 21 | R(5) << 16 | R(4) << 11 | R(4) << 7 | R(7),
                < 48 => 0x4800_0000u | (uint)(random.Next(4) * 2) << 21 | R(5) << 16 | R(5) << 11 | R(4) << 7,
                < 62 => (uint)random.Next(0x08, 0x10) << 26 | R(5) << 21 | R(5) << 16 | (random.Next(2) == 0 ? R(16) : (uint)(ushort)(short)random.Next(-8, 8)),
                < 72 => R(5) << 21 | R(5) << 16 | R(5) << 11 | R(5) << 6 | new uint[] { 0, 2, 3, 4, 6, 7, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x08, 0x09 }[random.Next(18)],
                < 84 => new uint[] { 0x20, 0x21, 0x23, 0x24, 0x25, 0x27, 0x28, 0x29, 0x2B }[random.Next(9)] << 26 | R(5) << 21 | R(5) << 16 | R(16),
                < 92 => (uint)random.Next(1, 8) << 26 | R(5) << 21 | R(5) << 16 | R(6),
                < 94 => 0x0000_000Du,
                < 97 => 0x4000_0000u | R(5) << 16 | R(3) << 11,
                _ => (uint)random.Next(),
            };
        }

        private static void Randomise(MarsCore core, Random random)
        {
            var bus = core.Bus!;
            Rsp rsp = bus.Sp.Processor;
            for (int i = 0; i < 0x1000; i += 4)
            {
                uint word = Instruction(random);
                bus.SpImem[i] = (byte)(word >> 24); bus.SpImem[i + 1] = (byte)(word >> 16); bus.SpImem[i + 2] = (byte)(word >> 8); bus.SpImem[i + 3] = (byte)word;
            }
            random.NextBytes(bus.SpDmem);
            // Half the values from the edges, so equal operands, carries and clamps come up as often as they must.
            ushort[] edges = { 0, 1, 2, 0x7FFE, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF, 0x00FF, 0xFF00, 0x0100 };
            for (int i = 1; i < 32; i++)
                rsp.Gpr[i] = random.Next(2) == 0 ? (uint)random.Next() ^ ((uint)random.Next(2) << 31) : (uint)(int)(short)random.Next(-8, 8) + (uint)random.Next(2) * 0x7FFF_FFF8u;
            for (int i = 0; i < rsp.Vector.Length; i++) rsp.Vector[i] = random.Next(2) == 0 ? (ushort)random.Next(0x10000) : edges[random.Next(edges.Length)];
            rsp.WidenAccumulator();
            for (int i = 0; i < 8; i++) rsp.Accumulator[i] = ((ulong)(uint)random.Next() << 16 | (uint)random.Next(0x10000)) & 0xFFFF_FFFF_FFFF;
            rsp.AccumulatorWritten();
            rsp.Vco = (ushort)random.Next(0x10000); rsp.Vcc = (ushort)random.Next(0x10000); rsp.Vce = (byte)random.Next(0x100);
            foreach (var (name, value) in new (string, object)[] { ("_divideInput", (ushort)random.Next(0x10000)), ("_divideOutput", (ushort)random.Next(0x10000)), ("_divideInputLoaded", random.Next(2) == 0) })
                typeof(Rsp).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(rsp, value);
            rsp.Start((uint)random.Next(0x400) * 4);
            rsp.Broke = false;
        }

        private static byte[] State(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static MarsCore Machine()
        {
            string path = Path.Combine(Path.GetTempPath(), "mars_native_rsp_" + Guid.NewGuid().ToString("N") + ".z64");
            File.WriteAllBytes(path, SyntheticN64Rom.BuildRunningFromRdram(new uint[] { 0x1000_FFFF, 0x0000_0000 }));
            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(path);
            File.Delete(path);
            return core;
        }

        // Each entry point run to the same total of steps, restarting the processor whenever a break halts it.
        private static byte[] Run(MarsCore core, byte[] start, bool native, string entry, int steps, bool simd)
        {
            Rsp.UseNative = native;
            core.LoadState(new MemoryStream(start));
            var bus = core.Bus!;
            Rsp rsp = bus.Sp.Processor;
            rsp.UseSimd = simd;

            for (int done = 0; done < steps;)
            {
                if (rsp.Halted) { rsp.Start(rsp.Pc); rsp.Broke = false; }
                switch (entry)
                {
                    case "step": rsp.StepOne(); done++; break;
                    case "sp":
                        int chunk = Math.Min(37, steps - done);
                        long before = done;
                        bus.Sp.Step(chunk);
                        done += chunk;
                        break;
                    default:
                        long ran = rsp.RunBlocks(Math.Min(53, steps - done));
                        if (ran == 0) { rsp.StepOne(); ran = 1; }
                        done += (int)ran;
                        break;
                }
            }
            return State(core);
        }

        // Nanoseconds a step for each path, printed; run by hand with EMUSEN_RSP_BENCH=1 - see Mars_Native.md §3.4.
        [Fact]
        public void Bench()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_RSP_BENCH") != "1") return;
            MarsCore core = Machine();
            var random = new Random(7);
            Randomise(core, random);
            byte[] start = State(core);
            const int steps = 3_000_000;
            foreach (var (label, native, entry) in new[] { ("C# step", false, "step"), ("native step", true, "step"), ("C# sp", false, "sp"), ("native sp", true, "sp"), ("C# blocks", false, "blocks"), ("native blocks", true, "blocks") })
            {
                Run(core, start, native, entry, 200_000, simd: true);
                var clock = System.Diagnostics.Stopwatch.StartNew();
                Run(core, start, native, entry, steps, simd: true);
                Console.WriteLine($"BENCH {label,-14} {clock.Elapsed.TotalMilliseconds * 1e6 / steps,6:F1} ns a step");
            }
        }

        [Theory]
        [InlineData("step")]
        [InlineData("sp")]
        [InlineData("blocks")]
        public void The_native_processor_leaves_the_state_the_csharp_one_leaves(string entry)
        {
            Assert.True(MarsNative.Available, MarsNative.Report);
            MarsCore core = Machine();
            bool blocks = Rsp.UseBlocks, background = Rsp.CompileBlocksInBackground;
            Rsp.CompileBlocksInBackground = false;
            try
            {
                for (int seed = 1; seed <= 150; seed++)
                {
                    var random = new Random(seed);
                    Randomise(core, random);
                    byte[] start = State(core);

                    byte[] plain = Run(core, start, native: false, entry, 3000, simd: false);
                    byte[] simd = Run(core, start, native: false, entry, 3000, simd: true);
                    byte[] rust = Run(core, start, native: true, entry, 3000, simd: true);

                    Assert.True(plain.AsSpan().SequenceEqual(simd), $"seed {seed}: the C# plain and SIMD paths differ, so the oracle is in doubt");
                    Assert.True(plain.AsSpan().SequenceEqual(rust), $"seed {seed} via {entry}: the native processor's state differs from C#'s at byte {plain.AsSpan().CommonPrefixLength(rust)} of {plain.Length}");
                }
            }
            finally
            {
                Rsp.UseBlocks = blocks;
                Rsp.CompileBlocksInBackground = background;
            }
        }
    }
}
