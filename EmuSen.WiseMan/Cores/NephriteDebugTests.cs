using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The Genesis through DianaOS's generic debugger: G7's breakpoints, stepping, watches, coverage and the call stack on each processor - see Nephrite_Native.md §43.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephriteDebugTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephriteDebug_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public NephriteDebugTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Put(byte[] r, int at, params ushort[] words)
        {
            foreach (ushort w in words)
            {
                r[at++] = (byte)(w >> 8);
                r[at++] = (byte)w;
            }
        }

        // A 68000 that starts a Z80 loop and calls a counting routine with the vertical interrupt on - see Nephrite_Native.md §43.
        private static byte[] CallsRom(bool interrupt)
        {
            var r = SyntheticMdRom.Cartridge();
            Put(r, 0, 0x00FF, 0xFE00, 0x0000, 0x0200);
            Put(r, 0x78, 0x0000, 0x0400);
            var loader = new List<ushort> { 0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100, 0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6 };
            byte[] z80 = { 0x3C, 0x32, 0x00, 0x01, 0xC3, 0x00, 0x00 };
            for (int i = 0; i < z80.Length; i++) loader.AddRange(new ushort[] { 0x13FC, z80[i], 0x00A0, (ushort)i });
            loader.AddRange(new ushort[] { 0x33FC, 0x0000, 0x00A1, 0x1200, 0x33FC, 0x0000, 0x00A1, 0x1100, 0x33FC, 0x0100, 0x00A1, 0x1200, 0x4EF9, 0x0000, 0x0280 });
            Put(r, 0x200, loader.ToArray());
            // Without the interrupt the six words that turn it on are NOPs, so the loop stays at $28C.
            if (interrupt) Put(r, 0x280, 0x33FC, 0x8164, 0x00C0, 0x0004, 0x027C, 0xF8FF);
            else Put(r, 0x280, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x4E71);
            Put(r, 0x28C, 0x4EB9, 0x0000, 0x0300, 0x60F8);
            Put(r, 0x300, 0x5282, 0x13C2, 0x00FF, 0x0020, 0x4E75);
            Put(r, 0x400, 0x4E73);
            return r;
        }

        private (CoreEngine Engine, CoreDebugTarget Target) Load(bool interrupt = true)
        {
            var found = CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite");
            string rom = Path.Combine(_root, "calls.md");
            File.WriteAllBytes(rom, CallsRom(interrupt));
            var engine = new CoreEngine(found.Open()!);
            engine.LoadRom(rom);
            return (engine, engine.CreateDebugTarget());
        }

        private static DebugCpu Cpu(CoreDebugTarget t, string name) => t.DebugCpus.Single(c => c.Name == name);

        private static void ClearBreakpoints(BreakpointRegistry r)
        {
            foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id);
        }

        [Fact]
        public void The_processors_spaces_and_registers_are_the_genesiss()
        {
            var (engine, t) = Load();
            using (engine)
            {
                engine.RunFrame();
                t.RefreshProviders();
                Assert.Equal(new[] { "m68k", "z80" }, t.DebugCpus.Select(c => c.Name));
                Assert.Equal(new[] { "M68KBUS", "Z80BUS" }, t.DebugCpus.Select(c => c.CodeSpace));
                Assert.Equal(new[] { "M68KBUS", "Z80BUS", "WRAM", "Z80RAM", "VRAM", "CRAM", "VSRAM", "ROM" }, t.GetMemorySpaces().Select(s => s.Name));
                Assert.Equal(new[] { "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "PC", "SR", "USP", "SSP" }, t.CpuRegisters.Current.Select(r => r.Name));
                var shell = DianaOSInterpreter.CreateDefault(t);
                Assert.Contains("z80", shell.Submit("cpus").Output);
                string regs = shell.Submit("regs z80").Output;
                Assert.Contains("IFF1", regs);
                Assert.DoesNotContain("SSP", regs);
                Assert.Equal(engine.ReadSpace("WRAM", 0x20), engine.ReadSpace("M68KBUS", 0xFF0020));
            }
        }

        [Fact]
        public void A_68000_breakpoint_halts_in_front_of_its_instruction_and_resumes()
        {
            var (engine, t) = Load();
            using (engine)
            {
                engine.RunFrame();
                long frames = engine.TotalFrames;
                int id = t.Breakpoints.AddBreakpoint(0x302);
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal((0x302, 0u, "M68K"), (engine.HaltedAddress, engine.HaltedProcessor, engine.HaltedProcessorName));
                Assert.Equal(0x302, Cpu(t, "m68k").ProgramCounter!());
                Assert.Equal(frames, engine.TotalFrames);
                t.Breakpoints.RemoveBreakpoint(id);
                engine.RunFrame();
                Assert.False(engine.IsHaltedAtBreakpoint);
                Assert.Equal(frames + 1, engine.TotalFrames);
            }
        }

        // Single steps on the 68000 go through the JSR into the routine and back; single steps on the Z80 halt on processor 1, one instruction each.
        [Fact]
        public void Each_processor_steps_one_instruction_at_a_time()
        {
            // The interrupt is left off: a step that fell on it would land in the handler, and the test is of the steps' order.
            var (engine, t) = Load(interrupt: false);
            using (engine)
            {
                engine.RunFrame();
                t.Breakpoints.AddBreakpoint(0x28C);
                engine.RunFrame();
                ClearBreakpoints(t.Breakpoints);
                var seen = new List<int>();
                for (int i = 0; i < 5; i++)
                {
                    t.Breakpoints.ArmSingleStep();
                    engine.RunFrame();
                    Assert.True(engine.IsHaltedAtBreakpoint);
                    seen.Add(engine.HaltedAddress);
                }
                Assert.Equal(new[] { 0x300, 0x302, 0x308, 0x292, 0x28C }, seen);

                var z80 = Cpu(t, "z80");
                var at = new List<int>();
                for (int i = 0; i < 6; i++)
                {
                    z80.Breakpoints.ArmSingleStep();
                    engine.RunFrame();
                    Assert.True(engine.IsHaltedAtBreakpoint && engine.HaltedProcessor == 1);
                    Assert.Equal(engine.HaltedAddress, z80.ProgramCounter!());
                    at.Add(engine.HaltedAddress);
                }
                int first = at.IndexOf(0);
                Assert.Equal(new[] { 0, 1, 4 }, at.Skip(first).Take(3));
                Assert.Contains("Stepping 3", DianaOSInterpreter.CreateDefault(t).Submit("step z80 3").Output);
            }
        }

        // The call stack holds the JSR in front of a breakpoint in the routine, and stepping over the JSR runs the routine whole.
        [Fact]
        public void The_call_stack_holds_the_jsr_and_a_step_over_runs_the_routine()
        {
            var (engine, t) = Load();
            using (engine)
            {
                t.Breakpoints.AddBreakpoint(0x302);
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                var top = t.CallStack!.Frames[^1];
                Assert.Equal((0x28C, 0x300, CallFrameKind.Call), (top.Source, top.Target, top.Kind));
                Assert.Contains("000300", DianaOSInterpreter.CreateDefault(t).Submit("bt").Output);
                ClearBreakpoints(t.Breakpoints);
                t.Breakpoints.AddBreakpoint(0x28C);
                engine.RunFrame();
                Assert.Equal(0x28C, engine.HaltedAddress);
                ClearBreakpoints(t.Breakpoints);
                byte before = engine.ReadSpace("WRAM", 0x20);
                Assert.True(t.Breakpoints.ArmStepToDepth(t.CallStack.Depth));
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal(0x292, engine.HaltedAddress);
                Assert.Equal((byte)(before + 1), engine.ReadSpace("WRAM", 0x20));
            }
        }

        // A write watch on WRAM records the routine's store with its value and the storing instruction; a watch on the Z80's RAM records the Z80's.
        [Fact]
        public void Write_watches_see_each_processors_stores()
        {
            var (engine, t) = Load();
            using (engine)
            {
                int main = t.Watches.AddWatch("WRAM", 0x20, 1);
                int z80 = t.Watches.AddWatch("Z80RAM", 0x100, 1);
                engine.RunFrame();
                engine.RunFrame();
                var events = t.Watches.GetEvents(main);
                Assert.NotEmpty(events);
                Assert.All(events, e => Assert.Equal(0x20, e.Address));
                Assert.Equal(engine.ReadSpace("WRAM", 0x20), events[^1].Value);
                Assert.Contains("302", events[0].Context);
                Assert.NotEmpty(t.Watches.GetEvents(z80));
                Assert.Equal(engine.ReadSpace("Z80RAM", 0x100), t.Watches.GetEvents(z80)[^1].Value);
            }
        }

        [Fact]
        public void Coverage_is_kept_per_processor()
        {
            var (engine, t) = Load();
            using (engine)
            {
                engine.RunFrame();
                var z80 = Cpu(t, "z80");
                z80.Coverage!.Arm();
                engine.RunFrame();
                Assert.True(z80.Coverage.InstructionsRecorded > 0);
                Assert.Equal((3, 7), (z80.Coverage.Statistics(0, 7).Executed, z80.Coverage.Statistics(0, 7).Total));
                Assert.Equal(0, t.Coverage!.InstructionsRecorded);
                t.Coverage.Arm();
                engine.RunFrame();
                Assert.True(t.Coverage.InstructionsRecorded > 0);
                Assert.Equal(3, t.Coverage.Statistics(0x300, 10).Executed);
                Assert.Equal(0, t.Coverage.Statistics(0x500, 16).Executed);
            }
        }

        // Disassembly by processor goes to its own bus, in its own instruction set; a JSR is a Call reference.
        [Fact]
        public void Each_processors_code_lists_from_its_bus()
        {
            var (engine, t) = Load();
            using (engine)
            {
                var shell = DianaOSInterpreter.CreateDefault(t);
                Assert.Equal(shell.Submit("disasm Z80BUS 0 3").Output, shell.Submit("disasm z80 0 3").Output);
                var jsr = t.Disassemble("M68KBUS", 0x28C, 1).Single();
                Assert.Equal("JSR", jsr.Mnemonic);
                Assert.Equal((StaticReferenceKind.Call, 0x300), t.ClassifyStaticReference(jsr));
                engine.RunFrame();
                Assert.Equal(new[] { "INC", "LD", "JP" }, t.Disassemble("Z80BUS", 0, 3).Select(i => i.Mnemonic));
            }
        }

        // Every registry armed with nothing to hit gives the plain run, frame for frame - the kit's C15, through the bridge.
        [Fact]
        public void An_armed_debugger_with_nothing_to_hit_is_the_plain_run()
        {
            var (plain, _) = Load();
            var (armed, t) = Load();
            using (plain)
            using (armed)
            {
                t.Breakpoints.AddBreakpoint(0x7FFFF0);
                Cpu(t, "z80").Breakpoints.AddBreakpoint(0xFFF0);
                t.Watches.AddWatch("WRAM", 0x8000, 4);
                t.Coverage!.Arm();
                Cpu(t, "z80").Coverage!.Arm();
                for (int f = 0; f < 3; f++)
                {
                    plain.RunFrame();
                    armed.RunFrame();
                    Assert.False(armed.IsHaltedAtBreakpoint);
                    Assert.Equal(plain.ReadSpace("WRAM", 0x20), armed.ReadSpace("WRAM", 0x20));
                    Assert.Equal(plain.ReadSpace("Z80RAM", 0x100), armed.ReadSpace("Z80RAM", 0x100));
                    var (a, b) = (new MemoryStream(), new MemoryStream());
                    plain.SaveState(a);
                    armed.SaveState(b);
                    Assert.Equal(a.ToArray(), b.ToArray());
                }
            }
        }
    }
}
