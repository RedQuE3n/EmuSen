using System.Reflection;
using System.Security.Cryptography;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Core;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.MercuryLegacy
{
    // What couples C# Mercury's branch to main: the ABI, the state format, the input script, the CPU's bus and the rig's extensions - see Mercury_Native.md §14.7.
    public class PinTests
    {
        [Fact]
        public void The_library_is_a_core_abi_v1_game_boy_with_the_extensions_the_rig_and_the_mixer_use()
        {
            CoreLibrary library = Differential.Library();
            Assert.Equal(Differential.AbiMajor, library.AbiVersion >> 16);
            Assert.True(library.Info.Claims(".gb") && library.Info.Claims(".gbc"), "the library does not claim Game Boy images");
            foreach (string name in Differential.Extensions) Assert.True(library.Export(name) != 0, $"the library lacks {name}");
        }

        [Fact]
        public void Both_engines_write_state_version_11_with_mercurys_magic()
        {
            Assert.Equal(Differential.StateVersion, MercuryCore.StateVersion);
            using var pair = new Differential(SyntheticGbRom.Build());
            byte[] rust = pair.RustState();
            Assert.Equal(MercuryCore.StateMagic, BitConverter.ToUInt32(rust, 0));
            Assert.Equal(Differential.StateVersion, BitConverter.ToInt32(rust, 4));
        }

        // The bench's script as the goldens name it, "bench <fingerprint>": every committed game trace's line must be this project's script.
        [Fact]
        public void The_bench_script_is_the_one_the_goldens_name()
        {
            string games = Path.Combine(Differential.Root(), "EmuSen.WiseMan", "Cores", "MercuryGoldens", "games");
            int checkedTraces = 0;
            foreach (string trace in Directory.Exists(games) ? Directory.GetFiles(games, "*.trace") : Array.Empty<string>())
            {
                string[] input = File.ReadLines(trace).Single(l => l.StartsWith("input ", StringComparison.Ordinal)).Split(' ');
                int frames = File.ReadLines(trace).Count(l => l.Length > 0 && char.IsDigit(l[0]));
                Assert.Equal("bench", input[1]);
                Assert.Equal(input[2], Convert.ToHexString(SHA256.HashData(Enumerable.Range(0, frames).Select(f => (byte)Script.Bench(f)).ToArray()))[..16]);
                checkedTraces++;
            }
            Assert.True(checkedTraces > 0, "no game trace committed to check the script against");
        }

        // The audit added PendingInterrupts and the HDMA rule SetHalted; a C# CPU on another bus would not be the one the goldens recorded.
        [Fact]
        public void The_cpus_bus_has_the_members_the_goldens_were_recorded_with()
        {
            string[] members = typeof(ICpuBus).GetMembers(BindingFlags.Public | BindingFlags.Instance).Select(m => m.Name).Where(n => !n.StartsWith("get_", StringComparison.Ordinal)).Order(StringComparer.Ordinal).ToArray();
            Assert.Equal(new[] { "PendingInterrupts", "Read", "SetHalted", "Stop", "Tick", "Write" }, members);
        }
    }

    // The bench's input: Start for five frames of every ninety, A for five frames half way; the goldens' MercuryGolden.ScriptMask("bench").
    public static class Script
    {
        public static uint Bench(int frame) => (frame % 90) switch { < 5 => 1u << 7, >= 45 and < 50 => 1u << 4, _ => 0u };
    }

    // The live comparison, frame by frame and instruction by instruction, on the programs, the corpus and the games - see Mercury_Native.md §14.7.
    public class DifferentialTests
    {
        private readonly ITestOutputHelper _output;

        public DifferentialTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<string> Programs => new()
        {
            "busy", "interrupts", "colour", "colour-running", "four-channels", "interrupts-on-gbc", "colour-priority", "sweep-overflow", "mbc3-latch",
        };

        [Theory]
        [MemberData(nameof(Programs))]
        public void A_synthetic_program_runs_identically(string name)
        {
            foreach (var (k, r, c) in name is "busy" or "interrupts" ? MercuryPrograms.Boards : new[] { ((byte)0, (byte)0, (byte)0) })
            {
                var (rom, model, frames) = name switch
                {
                    "busy" => (SyntheticGbRom.Build(romBanks: 4, cartridgeType: k, ramSizeCode: r, cgbFlag: c, patches: (0, MercuryPrograms.Busy)), GbModel.Auto, 600),
                    "interrupts" => (MercuryPrograms.InterruptsRom(k, r, c), GbModel.Auto, 600),
                    "colour" => (MercuryPrograms.ColourRom(), GbModel.Auto, 600),
                    "colour-running" => (MercuryPrograms.ColourRom(halt: false), GbModel.Auto, 600),
                    "four-channels" => (SyntheticGbRom.Build(patches: (0, MercuryPrograms.FourChannels)), GbModel.Auto, 600),
                    "interrupts-on-gbc" => (MercuryPrograms.InterruptsRom(0x03, 0x02, 0x00), GbModel.GameBoyColor, 600),
                    "colour-priority" => (MercuryPrograms.ColourPriorityRom(), GbModel.Auto, 30),
                    "sweep-overflow" => (MercuryPrograms.SweepOverflowRom(), GbModel.Auto, 10),
                    _ => (MercuryPrograms.Mbc3LatchRom(), GbModel.Auto, 2),
                };
                using var pair = new Differential(rom, model);
                for (int f = 0; f < frames; f++) Assert.True(pair.Frame(), $"frame {f}: an illegal opcode");
                _output.WriteLine($"{name} ${k:X2} ${r:X2} ${c:X2}: {pair.Frames} frames, {pair.Samples} samples identical");
            }
        }

        [Fact]
        public void Random_programs_step_identically()
        {
            int steps = 0;
            foreach (var (k, r, c) in MercuryPrograms.Boards)
                for (int seed = 0; seed < 16; seed++)
                {
                    using var pair = new Differential(MercuryPrograms.RandomRom(k, r, c, seed));
                    for (int i = 0; i < 5000 && pair.Step(); i++) steps++;
                }
            _output.WriteLine($"{steps} instructions identical");
        }

        // The rig's tick and stop extensions against MemoryBus.Tick and MemoryBus.Stop.
        [Fact]
        public void The_rigs_tick_and_stop_move_the_machine_as_csharps_bus_does()
        {
            using var pair = new Differential(MercuryPrograms.ColourRom());
            for (int i = 1; i < 2000; i += 37) pair.Tick(i);
            pair.Csharp.Bus!.Write(0xFF4D, 0x01);
            pair.Rust.WriteSpace(MercuryCore.SpaceCpuBus, 0xFF4D, 0x01);
            pair.CompareState("KEY1 armed");
            pair.Stop();
            Assert.True(pair.Csharp.Bus.DoubleSpeed);
            for (int i = 1; i < 2000; i += 41) pair.Tick(i);
        }

        // Every corpus ROM to its verdict or 3,600 frames, rendering skipped as the baseline runs it.
        [Fact]
        public void The_corpus_runs_identically()
        {
            string? root = Environment.GetEnvironmentVariable("EMUSEN_MERCURYRT_CORPUS");
            if (root is null || !Directory.Exists(root)) { _output.WriteLine("EMUSEN_MERCURYRT_CORPUS unset, not run"); return; }
            int roms = 0, frames = 0;
            foreach (string path in Directory.GetFiles(root, "*.gb*", SearchOption.AllDirectories).Where(p => p.EndsWith(".gb") || p.EndsWith(".gbc")).Order(StringComparer.Ordinal))
            {
                using var pair = new Differential(File.ReadAllBytes(path), skipRendering: true);
                for (int f = 0; f < 3600 && pair.Frame(soundAndPicture: false) && !Verdict(pair.Csharp); f++) { }
                roms++;
                frames += pair.Frames;
            }
            _output.WriteLine($"{roms} ROMs, {frames} frames identical in state and serial");
        }

        // A verdict by the serial port (blargg's text, mooneye's Fibonacci run or its $42s) or blargg's cart-RAM status.
        private static bool Verdict(MercuryCore core)
        {
            IReadOnlyList<byte> serial = core.Bus!.SerialLog;
            string text = string.Concat(serial.Select(b => (char)b));
            if (text.Contains("Passed") || text.Contains("Failed")) return true;
            if (serial.Count >= 6 && (serial.TakeLast(6).SequenceEqual(new byte[] { 3, 5, 8, 13, 21, 34 }) || serial.TakeLast(6).All(b => b == 0x42))) return true;
            byte[] ram = core.Cart!.Ram;
            return ram.Length >= 4 && ram[1] == 0xDE && ram[2] == 0xB0 && ram[3] == 0x61 && ram[0] != 0x80;
        }

        [Fact]
        public void The_games_run_identically_with_the_bench_script()
        {
            string? folder = Environment.GetEnvironmentVariable("EMUSEN_MERCURYRT_ROMS");
            if (folder is null || !Directory.Exists(folder)) { _output.WriteLine("EMUSEN_MERCURYRT_ROMS unset, not run"); return; }
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                using var pair = new Differential(File.ReadAllBytes(path));
                for (int f = 0; f < 3600; f++)
                {
                    pair.Press(Script.Bench(f));
                    Assert.True(pair.Frame());
                }
                _output.WriteLine($"{Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)))[..16]}: {pair.Frames} frames identical");
            }
        }
    }
}
