using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // A core's unit registers in `eval`, conditions and logpoints, after the processor's own names - see `man eval` and EmuSen_Backend.md §24.
    [Collection(TestCollections.ProcessGlobals)]
    public class UnitExpressionTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenUnitExpressions_" + Guid.NewGuid().ToString("N"));
        private readonly List<string> _files = new();

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        // A loop that sets X to 5, PPUCTRL to $10, the PPU's fine x to 3 and v to $2108, and pulse 1's duty to 3, then jumps back from $801E.
        private static readonly byte[] Program =
        {
            0xA2, 0x05,
            0xA9, 0x10, 0x8D, 0x00, 0x20,
            0xA9, 0x03, 0x8D, 0x05, 0x20, 0x8D, 0x05, 0x20,
            0xA9, 0x21, 0x8D, 0x06, 0x20,
            0xA9, 0x08, 0x8D, 0x06, 0x20,
            0xA9, 0xC0, 0x8D, 0x00, 0x40,
            0x4C, 0x00, 0x80,
        };

        private const string Jump = "801E";

        private (ICore Core, IDebugTarget Target, DianaShInterpreter Shell) Nes(int frames = 3)
        {
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, Program) }));
            _files.Add(path);
            CoreBundle bundle = CoreFactory.Load(path, options: TestCores.NoBattery);
            for (int f = 0; f < frames; f++) bundle.Core.RunFrame();
            bundle.DebugTarget.RefreshProviders();
            return (bundle.Core, bundle.DebugTarget, DianaShInterpreter.CreateDefault(bundle.DebugTarget));
        }

        private static string Eval(DianaShInterpreter shell, string expression) => shell.Submit($"eval {expression}").Output.TrimEnd();

        [Fact]
        public void A_PPU_register_is_named_by_eval_on_MoonRT()
        {
            var (core, _, shell) = Nes();
            using (core as IDisposable)
            {
                Assert.StartsWith("16  $10", Eval(shell, "ppu.PPUCTRL"));
                Assert.StartsWith("16  $10", Eval(shell, "PPUCTRL"));
                Assert.StartsWith("16  $10", Eval(shell, "ppu.ppuctrl"));
                Assert.StartsWith("3  $3", Eval(shell, "pulse1.Duty"));
                Assert.Contains("ppu.PPUCTRL", shell.Submit("eval symbols ppu.").Output);
            }
        }

        [Fact]
        public void A_condition_and_a_logpoint_read_a_unit_register_at_the_hit()
        {
            var (core, _, shell) = Nes(frames: 0);
            using (core as IDisposable)
            {
                Assert.Contains("added at", shell.Submit($"bp add {Jump} if \"ppu.PPUCTRL==0x11\"").Output);
                Assert.Contains("logging at", shell.Submit($"bp add {Jump} log \"ppu.v, PPUCTRL\"").Output);
                for (int f = 0; f < 3 && !core.IsHaltedAtBreakpoint; f++) core.RunFrame();
                Assert.False(core.IsHaltedAtBreakpoint);
                Assert.Contains("ppu.v=0x2108  PPUCTRL=0x10", shell.Submit("bp log 2").Output);

                Assert.Contains("added at", shell.Submit($"bp add {Jump} if \"ppu.PPUCTRL==0x10 && pulse1.Duty==3\"").Output);
                for (int f = 0; f < 3 && !core.IsHaltedAtBreakpoint; f++) core.RunFrame();
                Assert.True(core.IsHaltedAtBreakpoint);
                Assert.Equal(0x801E, core.HaltedAddress);
            }
        }

        [Fact]
        public void A_name_the_processor_or_a_label_holds_stays_theirs_and_two_units_name_neither()
        {
            var (core, target, shell) = Nes();
            using (core as IDisposable)
            {
                Assert.StartsWith("5  $5", Eval(shell, "x"));
                Assert.StartsWith("3  $3", Eval(shell, "ppu.x"));
                long v = (long)target.CpuRegisters.Current.Single(r => r.Name == "V").Value;
                Assert.StartsWith($"{v}  ${v:X}", Eval(shell, "v"));
                Assert.StartsWith("8456  $2108", Eval(shell, "ppu.v"));

                target.Labels!.Add(0x2000, "PPUCTRL");
                Assert.StartsWith("8192  $2000", Eval(shell, "PPUCTRL"));
                Assert.StartsWith("16  $10", Eval(shell, "ppu.PPUCTRL"));

                Assert.Contains("Unknown symbol", Eval(shell, "Duty"));
                Assert.StartsWith("3  $3", Eval(shell, "pulse1.Duty"));
                Assert.StartsWith("0  $0", Eval(shell, "pulse2.Duty"));
                Assert.StartsWith("Error", shell.Submit($"bp add {Jump} if Duty==3").Output);
                Assert.StartsWith("16  $10", Eval(shell, "cpu ppu.PPUCTRL"));
            }
        }

        [Fact]
        public void Resolving_a_unit_register_allocates_nothing()
        {
            var (core, target, _) = Nes();
            using (core as IDisposable)
            {
                IExpressionContext context = DebugExpressions.For(target);
                Assert.True(context.TryGetSymbol("ppu.PPUCTRL", out long value));
                Assert.Equal(0x10, value);
                Assert.True(context.TryGetSymbol("PPUCTRL", out _));
                long before = GC.GetAllocatedBytesForCurrentThread();
                for (int i = 0; i < 100; i++)
                {
                    context.TryGetSymbol("ppu.PPUCTRL", out _);
                    context.TryGetSymbol("PPUCTRL", out _);
                    context.TryGetSymbol("pulse1.Duty", out _);
                }
                Assert.Equal(0, GC.GetAllocatedBytesForCurrentThread() - before);
                Assert.Same(context, DebugExpressions.For(target));
            }
        }

        [Fact]
        public void A_core_that_lists_no_units_names_what_it_named()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
            var found = CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite");
            string rom = Path.Combine(Directory.CreateDirectory(Path.Combine(_root, "load")).FullName, "calls.md");
            File.WriteAllBytes(rom, NephriteDebugTests.CallsRom(interrupt: false));
            using var engine = new CoreEngine(found.Open()!) { Options = TestCores.NoBattery };
            engine.LoadRom(rom);
            engine.RunFrame();
            CoreDebugTarget target = engine.CreateDebugTarget();
            Assert.Empty(engine.Machine.Info.Units);

            string[] main = { "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "frame", "PC", "SR", "SSP", "stackdepth", "USP" };
            Assert.Equal(main, DebugExpressions.For(target).SymbolNames);
            foreach (DebugCpu cpu in target.DebugCpus)
                Assert.Equal(DebugCpuExpressionContext.For(target, cpu).SymbolNames, DebugExpressions.For(target, cpu).SymbolNames);
            var shell = DianaShInterpreter.CreateDefault(target);
            Assert.Contains("Unknown symbol", Eval(shell, "ppu.x"));
            Assert.Equal(Eval(shell, "cpu D2"), Eval(shell, "D2"));
        }
    }
}
