using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // `bp ... if` and `bp ... log` on the Rust cores, typed at whichever shell runs, the halts read from the engine - see `man bp`.
    [Collection(TestCollections.ProcessGlobals)]
    public class BreakpointConditionTests : IDisposable
    {
        private readonly List<string> _files = new();

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // A breakpoint's address, a condition true there, one false there, and a logpoint with the line it records.
        public sealed record Core(string Name, string Address, string True, string False, string Log, string Logged);

        public static readonly TheoryData<string> Cores = new() { "moonrt", "mercuryrt", "marsrt" };

        // MoonDebugEngineTests' loop, where A is 7 at the INC; MercuryDebugClaims' CountForever, where HL is $C000 at the INC (HL); MarsDebugClaims', where t0 counts at the SW.
        private static Core Of(string name) => name switch
        {
            "moonrt" => new Core(name, "8008", "A==7", "A==0x99", "A, [0x42]==[0x42]", "A=0x7  [0x42]==[0x42]=0x1"),
            "mercuryrt" => new Core(name, $"{SyntheticGbRom.EntryPoint + 3:X4}", "HL==0xC000", "HL==0x1234", "HL", "HL=0xC000"),
            _ => new Core(name, "A400004C", "t0==3", "t0==0", "t0", "t0=0x1"),
        };

        private (ICore Core, DianaShInterpreter Shell) Load(string name)
        {
            var (core, target) = LoadTarget(name);
            return (core, DianaShInterpreter.CreateDefault(target));
        }

        private (ICore Core, IDebugTarget Target) LoadTarget(string name)
        {
            if (name == "marsrt")
            {
                string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, MarsDebugClaims.CountForever)));
                _files.Add(rom);
                MarsDebugRig rig = MarsDebugRig.MarsRt(rom);
                return (rig.Core, rig.Target);
            }
            string path = name == "moonrt" ? NesRom() : SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) }));
            _files.Add(path);
            CoreBundle bundle = CoreFactory.Load(path, options: TestCores.NoBattery);
            return (bundle.Core, bundle.DebugTarget);
        }

        private static string NesRom()
        {
            byte[] image = SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, MoonDebugEngineTests.Program), (0x20, new byte[] { 0xEE, 0x43, 0x00, 0x40 }) });
            image[16 + 0x3FFA] = 0x20;
            image[16 + 0x3FFB] = 0x80;
            return SyntheticNesRom.WriteTemp(image);
        }

        private static void Run(ICore core, int frames)
        {
            for (int f = 0; f < frames && !core.IsHaltedAtBreakpoint; f++) core.RunFrame();
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_false_condition_never_halts(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                Assert.Contains("added at", shell.Submit($"bp add {c.Address} if {c.False}").Output);
                Run(core, 3);
                Assert.False(core.IsHaltedAtBreakpoint, $"halted at ${core.HaltedAddress:X} though {c.False} is false there");
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_true_condition_halts_at_its_address(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                shell.Submit($"bp add {c.Address} if {c.True}");
                Run(core, 3);
                Assert.True(core.IsHaltedAtBreakpoint);
                Assert.Equal(Convert.ToInt32(c.Address, 16), core.HaltedAddress);
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_logpoint_records_its_values_and_carries_on(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                Assert.Contains("logging at", shell.Submit($"bp add {c.Address} log \"{c.Log}\"").Output);
                Run(core, 1);
                Assert.False(core.IsHaltedAtBreakpoint);
                string log = shell.Submit("bp log 1").Output;
                Assert.Contains(c.Logged, log);
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_malformed_condition_is_refused_when_it_is_set(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                var result = shell.Submit($"bp add {c.Address} if \"{c.True} &&\"");
                Assert.StartsWith("Error: ", result.Output);
                Assert.Equal("1", shell.LastExitStatus);
                Assert.StartsWith("Error: ", shell.Submit($"bp add {c.Address} if nosuchsymbol==1").Output);
                Assert.StartsWith("Error: ", shell.Submit($"bp add {c.Address} log \"(A\"").Output);
                Assert.Equal("No active breakpoints.", shell.Submit("bp list").Output.TrimEnd());
            }
        }

        // What a hit refreshes before its condition reads a register allocates nothing - see `man bp`.
        [Theory]
        [MemberData(nameof(Cores))]
        public void A_hits_refresh_of_the_registers_allocates_nothing(string name)
        {
            var (core, target) = LoadTarget(name);
            using (core as IDisposable)
            {
                Run(core, 2);
                DebugExpressions.RefreshRegisters(target);
                long before = GC.GetAllocatedBytesForCurrentThread();
                for (int i = 0; i < 300; i++) DebugExpressions.RefreshRegisters(target);
                Assert.Equal(0, GC.GetAllocatedBytesForCurrentThread() - before);
            }
        }

        // A hit refreshes the registers each scope names, the video and sound rows the zero-per-core context reads among them, and none of the views.
        [Fact]
        public void A_hit_reads_every_register_fresh_and_refreshes_no_view()
        {
            var target = new CountingTarget();
            BreakpointExpressions.Add(target, target.Breakpoints, 0x10, 0x10, "LINE==2 && APU0==2", null);
            Assert.False(target.Breakpoints.ShouldBreak(0x10));
            Assert.True(target.Breakpoints.ShouldBreak(0x10));
            BreakpointExpressions.Add(target, target.Gsu.Breakpoints, 0x20, 0x20, "R15==3", null);
            Assert.True(target.Gsu.Breakpoints.ShouldBreak(0x20));
            Assert.Equal((3, 3, 3, 3), (target.Cpu.Refreshes, target.Video.Refreshes, target.Apu.Refreshes, target.Cop.Refreshes));
            Assert.Equal((0, 0, 0, 0), (target.SpriteRows.Refreshes, target.PaletteRows.Refreshes, target.ChannelRows.Refreshes, target.LoadRows.Refreshes));
        }

        // A provider whose value is the number of times it was refreshed.
        private sealed class Counted<T>(Func<int, T> value) : IRealtimeProvider<T>
        {
            public int Refreshes { get; private set; }
            public T Current { get; private set; } = value(0);
            public void Refresh() => Current = value(++Refreshes);
        }

        // A target with no context of its own, so a condition reads the zero-per-core context's rows, and a coprocessor with its own registry.
        private sealed class CountingTarget : IDebugTarget
        {
            private static IReadOnlyList<DebugRegisterValue> Row(string name, int n) => new[] { new DebugRegisterValue(name, (ulong)n, 16) };

            public readonly Counted<IReadOnlyList<DebugRegisterValue>> Cpu = new(n => Row("A", n)), Video = new(n => Row("LINE", n)), Apu = new(n => Row("APU0", n)), Cop = new(n => Row("R15", n));
            public readonly Counted<IReadOnlyList<DebugSpriteInfo>> SpriteRows = new(_ => Array.Empty<DebugSpriteInfo>());
            public readonly Counted<IReadOnlyList<DebugPaletteInfo>> PaletteRows = new(_ => Array.Empty<DebugPaletteInfo>());
            public readonly Counted<IReadOnlyList<DebugAudioChannelInfo>> ChannelRows = new(_ => Array.Empty<DebugAudioChannelInfo>());
            public readonly Counted<IReadOnlyList<DebugLoadInfo>> LoadRows = new(_ => Array.Empty<DebugLoadInfo>());
            public readonly DebugCpu Gsu;
            private readonly DebugCpu[] _cpus;

            public CountingTarget()
            {
                Gsu = new DebugCpu("gsu", "a coprocessor", new BreakpointRegistry()) { Registers = Cop };
                _cpus = new[] { new DebugCpu(EmuSen.Crystal.Debugger.DebugCpus.MainName, "the main processor", Breakpoints) { Registers = Cpu }, Gsu };
            }

            public string CoreName => "COUNTING";
            public long FrameCount => 0;
            public int MaxSprites => 0;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => Cpu;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => Video;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => Apu;
            public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => Cop;
            public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => SpriteRows;
            public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => PaletteRows;
            public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => ChannelRows;
            public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => LoadRows;
            public void RefreshProviders() => throw new InvalidOperationException("a hit refreshed every provider");
            public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (Array.Empty<byte>(), 0, 0);
            public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (Array.Empty<byte>(), 0, 0);
            public (short[] Samples, int SampleRate) GetAudioSamples() => (Array.Empty<short>(), 0);
            public string GetSummaryText() => "";
            public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => Array.Empty<IDebugMemorySpace>();
            public WatchRegistry Watches { get; } = new();
            public FrameLogRegistry FrameLog { get; } = new();
            public BreakpointRegistry Breakpoints { get; } = new();
            public CheatRegistry Cheats { get; } = new();
            public IReadOnlyList<DebugCpu> DebugCpus => _cpus;
            public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count) => Array.Empty<DisassembledInstruction>();
            public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr) => null;
            public int TilemapEntryStride => 0;
            public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => "";
            public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => Array.Empty<byte>();
            public void SetChannelMuted(int index, bool muted) { }
        }
    }
}
