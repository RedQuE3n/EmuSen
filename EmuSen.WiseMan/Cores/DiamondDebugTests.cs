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
    // The PlayStation through DianaSh's generic debugger and the target a frontend's debugger window is given: G7's breakpoints, stepping, watches, coverage and the call stack on the R3000A - see Diamond_Debugger.md.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondDebugTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondDebug_" + Guid.NewGuid().ToString("N"));
        private int _loads;

        // The program's addresses: the loop's JAL, its delay slot, the J back, the routine and its store, and the counter in RAM.
        private const uint Loop = 0x8001_001C, Slot = 0x8001_0020, Back = 0x8001_0024, Routine = 0x8001_0100, Store = 0x8001_010C, Count = 0x2_0000;

        public DiamondDebugTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            // Windows refuses to delete a library the process has loaded, and answers it as access denied.
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static int At(uint address) => unchecked((int)address);

        // A loop that calls a routine counting in RAM at 80020000h and storing the count's low byte in the scratchpad, after setting the GTE's VXY0, hand-assembled from IDT's Appendix A - see Diamond_Debugger.md.
        private static byte[] CallsProgram()
        {
            var w = new uint[0x47];
            new uint[]
            {
                0x3C10_8002, // lui s0, 0x8002
                0x3C11_1F80, // lui s1, 0x1F80
                0x3C08_4000, // lui t0, 0x4000
                0x4088_6000, // mtc0 t0, SR
                0x3C08_0012, // lui t0, 0x0012
                0x3508_3456, // ori t0, t0, 0x3456
                0x4888_0000, // mtc2 t0, VXY0
                0x0C00_4040, // loop: jal 0x80010100
                0,
                0x0800_4007, // j loop
                0,
            }.CopyTo(w, 0);
            new uint[]
            {
                0x8E09_0000, // lw t1, 0(s0)
                0,
                0x2529_0001, // addiu t1, t1, 1
                0xAE09_0000, // sw t1, 0(s0)
                0xA229_0010, // sb t1, 0x10(s1)
                0x03E0_0008, // jr ra
                0,
            }.CopyTo(w, 0x40);
            return SyntheticPsExe.Build(SyntheticPsExe.Words(w));
        }

        // The read program's loads: LW of the count, LBU of the scratchpad's byte 10h, and LWL and LWR of RAM's 20005h and 20006h.
        private const uint ReadLoop = 0x8001_0008, ReadByte = 0x8001_000C, ReadLeft = 0x8001_0010, ReadRight = 0x8001_0014;

        // A loop that loads RAM and the scratchpad in three widths and counts in RAM at 80020000h, hand-assembled from IDT's Appendix A - see Diamond_Debugger.md §5.1.
        private static byte[] LoadsProgram() => SyntheticPsExe.Build(SyntheticPsExe.Words(new uint[]
        {
            0x3C10_8002, // lui s0, 0x8002
            0x3C11_1F80, // lui s1, 0x1F80
            0x8E09_0000, // loop: lw t1, 0(s0)
            0x922A_0010, // lbu t2, 0x10(s1)
            0x8A0B_0005, // lwl t3, 5(s0)
            0x9A0B_0006, // lwr t3, 6(s0)
            0x2529_0001, // addiu t1, t1, 1
            0xAE09_0000, // sw t1, 0(s0)
            0x0800_4002, // j loop
            0,
        }));

        // The program loaded through discovery on the open kernel, its first frames run so that the kernel has handed it over.
        private (CoreEngine Engine, CoreDebugTarget Target) Load(int frames = 10, bool loads = false)
        {
            var found = CoreDiscovery.Found.Single(c => c.Info.Id == "diamond");
            // A folder per load: an engine holds its image mapped, and Windows refuses to write a file mapped elsewhere (EmuSen_Galaxia.md §3.8).
            string exe = Path.Combine(Directory.CreateDirectory(Path.Combine(_root, $"load{++_loads}")).FullName, loads ? "loads.psexe" : "calls.psexe");
            File.WriteAllBytes(exe, loads ? LoadsProgram() : CallsProgram());
            var engine = new CoreEngine(found.Open()!) { Options = TestCores.NoBattery };
            engine.LoadRom(exe);
            for (int i = 0; i < frames; i++) engine.RunFrame();
            return (engine, engine.CreateDebugTarget());
        }

        private static uint Word(CoreEngine e, string space, uint address) =>
            (uint)Enumerable.Range(0, 4).Sum(i => (long)e.ReadSpace(space, At(address) + i) << (8 * i));

        private static void ClearBreakpoints(BreakpointRegistry r)
        {
            foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id);
        }

        private static void HaltAt(CoreEngine engine, CoreDebugTarget t, uint address)
        {
            ClearBreakpoints(t.Breakpoints);
            t.Breakpoints.AddBreakpoint(At(address));
            engine.RunFrame();
            Assert.True(engine.IsHaltedAtBreakpoint);
            Assert.Equal(At(address), engine.HaltedAddress);
            ClearBreakpoints(t.Breakpoints);
        }

        [Fact]
        public void The_processors_spaces_and_registers_are_the_consoles()
        {
            var (engine, t) = Load();
            using (engine)
            {
                Assert.All(new[] { CoreInterface.CapDebug, CoreInterface.CapDebugStack, CoreInterface.CapDebugRegisters, CoreInterface.CapDebugDisassemble }, c => Assert.True(engine.Library.Has(c)));
                Assert.Equal(new[] { "r3000", "gte" }, t.DebugCpus.Select(c => c.Name));
                Assert.Equal("R3000BUS", t.DebugCpus[0].CodeSpace);
                Assert.Equal(new[] { "R3000BUS", "RAM", "SCRATCH", "VRAM", "SPURAM", "BIOS", "TTY" }, t.GetMemorySpaces().Select(s => s.Name));
                var spaces = engine.Machine.Info.Spaces;
                Assert.Equal(new[] { "R3000BUS", "RAM", "SCRATCH" }, spaces.Where(s => s.ReportsStores).Select(s => s.Name));
                Assert.Equal(new[] { "R3000BUS", "RAM", "SCRATCH" }, spaces.Where(s => s.ReportsLoads).Select(s => s.Name));
                Assert.True(engine.Library.Has(CoreInterface.CapDebugReads));
                Assert.True(spaces[0].View && spaces[0].ReadOnly && spaces[0].Size == 1L << 32);
                var regs = t.CpuRegisters.Current;
                Assert.Equal(45, regs.Count);
                Assert.Equal(new[] { "zero", "at", "v0" }, regs.Take(3).Select(r => r.Name));
                Assert.Equal(0x8002_0000ul, regs.Single(r => r.Name == "s0").Value);
                var shell = DianaShInterpreter.CreateDefault(t);
                Assert.Contains("gte", shell.Submit("cpus").Output);
                string gte = shell.Submit("regs gte").Output;
                Assert.Contains("VXY0", gte);
                Assert.Contains("00123456", gte);
                Assert.Contains("FLAG", gte);
                Assert.Equal(Word(engine, "RAM", Count), Word(engine, "R3000BUS", 0x8000_0000 + Count));
                Assert.Equal(Word(engine, "RAM", Count), Word(engine, "R3000BUS", 0xA000_0000 + Count));
            }
        }

        [Fact]
        public void A_breakpoint_halts_in_front_of_its_instruction_and_resumes()
        {
            var (engine, t) = Load();
            using (engine)
            {
                long frames = engine.TotalFrames;
                int id = t.Breakpoints.AddBreakpoint(At(Store));
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal((At(Store), 0u, "R3000"), (engine.HaltedAddress, engine.HaltedProcessor, engine.HaltedProcessorName));
                Assert.Equal(At(Store), t.DebugCpus[0].ProgramCounter!());
                Assert.Equal(frames, engine.TotalFrames);
                uint count = Word(engine, "RAM", Count);
                engine.RunFrame();
                Assert.Equal(At(Store), engine.HaltedAddress);
                Assert.Equal(count + 1, Word(engine, "RAM", Count));
                t.Breakpoints.RemoveBreakpoint(id);
                engine.RunFrame();
                Assert.False(engine.IsHaltedAtBreakpoint);
                Assert.Equal(frames + 1, engine.TotalFrames);
            }
        }

        // Single steps go through the JAL, its delay slot and the routine, and back after the JR's slot.
        [Fact]
        public void A_step_moves_one_instruction_through_the_delay_slots()
        {
            var (engine, t) = Load();
            using (engine)
            {
                HaltAt(engine, t, Loop);
                var seen = new List<uint>();
                for (int i = 0; i < 10; i++)
                {
                    t.Breakpoints.ArmSingleStep();
                    engine.RunFrame();
                    Assert.True(engine.IsHaltedAtBreakpoint);
                    seen.Add((uint)engine.HaltedAddress);
                }
                Assert.Equal(new[] { Slot, Routine, Routine + 4, Routine + 8, Routine + 12, Routine + 16, Routine + 20, Routine + 24, Back, Back + 4 }, seen);
                Assert.Contains("Stepping 3", DianaShInterpreter.CreateDefault(t).Submit("step 3").Output);
            }
        }

        // The call stack holds the JAL in front of a breakpoint in the routine, and stepping over the JAL runs the routine whole.
        [Fact]
        public void The_call_stack_holds_the_jal_and_a_step_over_runs_the_routine()
        {
            var (engine, t) = Load();
            using (engine)
            {
                // The plain frames before keep no stack, so the JAL is first watched in front of the loop.
                HaltAt(engine, t, Loop);
                HaltAt(engine, t, Store);
                var top = t.CallStack!.Frames[^1];
                Assert.Equal((At(Loop), At(Routine), CallFrameKind.Call), (top.Source, top.Target, top.Kind));
                Assert.Contains("80010100", DianaShInterpreter.CreateDefault(t).Submit("bt").Output);
                HaltAt(engine, t, Loop);
                uint before = Word(engine, "RAM", Count);
                Assert.True(t.Breakpoints.ArmStepToDepth(t.CallStack.Depth));
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal(At(Back), engine.HaltedAddress);
                Assert.Equal(before + 1, Word(engine, "RAM", Count));
            }
        }

        // A write watch on RAM records the routine's store with its value and the storing instruction; one on the scratchpad its byte.
        [Fact]
        public void Write_watches_see_the_stores_in_ram_and_the_scratchpad()
        {
            var (engine, t) = Load();
            using (engine)
            {
                int ram = t.Watches.AddWatch("RAM", (int)Count, 1);
                int scratch = t.Watches.AddWatch("SCRATCH", 0x10, 1);
                int bus = t.Watches.AddWatch("R3000BUS", At(0x8002_0000), 1);
                engine.RunFrame();
                var events = t.Watches.GetEvents(ram);
                Assert.NotEmpty(events);
                Assert.All(events, e => Assert.Equal((int)Count, e.Address));
                Assert.Equal(engine.ReadSpace("RAM", (int)Count), events[^1].Value);
                Assert.Contains("8001010C", events[0].Context);
                Assert.Equal(engine.ReadSpace("SCRATCH", 0x10), t.Watches.GetEvents(scratch)[^1].Value);
                Assert.Contains("80010110", t.Watches.GetEvents(scratch)[0].Context);
                Assert.Equal(t.Watches.GetEventCounts(ram), t.Watches.GetEventCounts(bus));
            }
        }

        // Read watches through DianaSh's command record each byte a load read with the loading instruction, in RAM, the scratchpad and the bus; LWL and LWR at the bytes they reach. A write watch beside them sees only the store.
        [Fact]
        public void Read_watches_see_the_loads_in_ram_the_scratchpad_and_the_bus()
        {
            var (engine, t) = Load(loads: true);
            using (engine)
            {
                var shell = DianaShInterpreter.CreateDefault(t);
                Assert.Contains("(read)", shell.Submit("watch add RAM 20000 4 read").Output);
                shell.Submit("watch add SCRATCH 10 1 read");
                shell.Submit("watch add R3000BUS 80020004 4 read");
                shell.Submit("watch add RAM 20000 1 write");
                engine.RunFrame();
                var (ram, scratch, bus, store) = (t.Watches.GetEvents(1, 400), t.Watches.GetEvents(2), t.Watches.GetEvents(3, 400), t.Watches.GetEvents(4));
                Assert.All(ram, e => Assert.Equal((WatchKind.Read, "PC=$80010008"), (e.AccessKind, e.Context)));
                Assert.Equal(new[] { 0x2_0000, 0x2_0001, 0x2_0002, 0x2_0003 }, ram.Take(4).Select(e => e.Address));
                Assert.Equal((0x10, engine.ReadSpace("SCRATCH", 0x10), "PC=$8001000C"), (scratch[^1].Address, scratch[^1].Value, scratch[^1].Context));
                Assert.Equal(new[] { (At(0x8002_0004), "PC=$80010010"), (At(0x8002_0005), "PC=$80010010"), (At(0x8002_0006), "PC=$80010014"), (At(0x8002_0007), "PC=$80010014") }, bus.Take(4).Select(e => (e.Address, e.Context)));
                Assert.All(store, e => Assert.Equal((WatchKind.Write, "PC=$8001001C"), (e.AccessKind, e.Context)));
                Assert.InRange(t.Watches.GetEventCounts(1).Total / 4 - t.Watches.GetEventCounts(4).Total, -1, 1); // a frame's edges may split one iteration: Diamond_OpenKernel_Phase2.md §26.5
                Assert.Contains(" R 0x20000 = 0x", shell.Submit("watch log 1 4").Output);
            }
        }

        // A read data breakpoint through `bp read` halts in front of the load with nothing of it run, the halt naming the byte; resumed, the load runs and the loop comes round to it again. One on the bus's 80020006h halts at the LWR, which reads it, and not at the LWL beside it.
        [Fact]
        public void A_read_data_breakpoint_halts_in_front_of_the_load()
        {
            var (engine, t) = Load(loads: true);
            using (engine)
            {
                var shell = DianaShInterpreter.CreateDefault(t);
                Assert.Contains("added on reads of SCRATCH", shell.Submit("bp read SCRATCH 10").Output);
                long frames = engine.TotalFrames;
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal(At(ReadByte), engine.HaltedAddress);
                Assert.Contains("read SCRATCH 0x10", t.Breakpoints.LastBreakReason);
                Assert.Equal(frames, engine.TotalFrames);
                uint count = Word(engine, "RAM", Count);
                var (before, after) = (new MemoryStream(), new MemoryStream());
                engine.SaveState(before);
                engine.RunFrame();
                Assert.Equal(At(ReadByte), engine.HaltedAddress);
                Assert.Equal(count + 1, Word(engine, "RAM", Count));
                engine.SaveState(after);
                Assert.NotEqual(before.ToArray(), after.ToArray());

                foreach (var bp in t.Breakpoints.GetDataBreakpoints()) t.Breakpoints.RemoveBreakpoint(bp.Id);
                shell.Submit("bp read R3000BUS 80020006");
                engine.RunFrame();
                Assert.True(engine.IsHaltedAtBreakpoint);
                Assert.Equal(At(ReadRight), engine.HaltedAddress);
                engine.RunFrame();
                Assert.Equal(At(ReadRight), engine.HaltedAddress);
            }
        }

        [Fact]
        public void Coverage_marks_the_routine_and_the_loop()
        {
            var (engine, t) = Load();
            using (engine)
            {
                Assert.Equal(0, t.Coverage!.InstructionsRecorded);
                t.Coverage.Arm();
                engine.RunFrame();
                Assert.True(t.Coverage.InstructionsRecorded > 0);
                Assert.Equal(7, t.Coverage.Statistics(At(Routine), 0x1C).Executed);
                Assert.Equal(4, t.Coverage.Statistics(At(Loop), 0x10).Executed);
                Assert.Equal(0, t.Coverage.Statistics(At(0x8001_0030), 0x40).Executed);
                Assert.True(t.Coverage.WasExecuted(At(0x0001_0100)), "the mirrors share a bit");
            }
        }

        // Disassembly by processor goes to its bus, with the GTE's register names; a JAL is a Call reference.
        [Fact]
        public void Code_lists_from_the_bus_with_the_gtes_names()
        {
            var (engine, t) = Load();
            using (engine)
            {
                var shell = DianaShInterpreter.CreateDefault(t);
                string listed = shell.Submit("disasm r3000 8001001C 3").Output;
                Assert.Equal(shell.Submit("disasm R3000BUS 8001001C 3").Output, listed);
                Assert.Contains("jal 0x80010100", listed);
                var jal = t.Disassemble("R3000BUS", At(Loop), 1).Single();
                Assert.Equal("jal", jal.Mnemonic);
                Assert.Equal(new byte[] { 0x40, 0x40, 0x00, 0x0C }, jal.Bytes);
                Assert.Equal((StaticReferenceKind.Call, At(Routine)), t.ClassifyStaticReference(jal));
                var mtc2 = t.Disassemble("R3000BUS", At(0x8001_0018), 1).Single();
                Assert.Equal(("mtc2", "t0, VXY0"), (mtc2.Mnemonic, mtc2.OperandText));
                Assert.Equal(t.Disassemble("RAM", 0x1_0100, 7).Select(i => i.Mnemonic + " " + i.OperandText), t.Disassemble("R3000BUS", At(Routine), 7).Select(i => i.Mnemonic + " " + i.OperandText));
            }
        }

        // Every registry armed with nothing to hit gives the plain run, frame for frame - the kit's C15, through the bridge.
        [Fact]
        public void An_armed_debugger_with_nothing_to_hit_is_the_plain_run()
        {
            var (plain, _) = Load(frames: 0);
            var (armed, t) = Load(frames: 0);
            using (plain)
            using (armed)
            {
                t.Breakpoints.AddBreakpoint(At(0xFFFF_FFF0));
                t.Watches.AddWatch("RAM", 0x1F_FFF0, 4);
                t.Watches.AddWatch("SCRATCH", 0x3F0, 4, WatchKind.Both);
                t.Breakpoints.AddDataBreakpoint("SCRATCH", 0x3FF, 0x3FF, -1, onRead: true, changedOnly: false, condition: null);
                t.Coverage!.Arm();
                t.CallStack!.ArmProfiler();
                for (int f = 0; f < 20; f++)
                {
                    plain.RunFrame();
                    armed.RunFrame();
                    Assert.False(armed.IsHaltedAtBreakpoint);
                    var (a, b) = (new MemoryStream(), new MemoryStream());
                    plain.SaveState(a);
                    armed.SaveState(b);
                    Assert.Equal(a.ToArray(), b.ToArray());
                }
                Assert.True(t.Coverage.InstructionsRecorded > 0 && t.CallStack.Depth >= 0);
            }
        }

        // What a frontend's debugger window is given for a PlayStation program: the generic target, its summary, its registers and its code.
        [Fact]
        public void A_frontends_debugger_window_gets_the_generic_target()
        {
            string exe = Path.Combine(_root, "calls.psexe");
            File.WriteAllBytes(exe, CallsProgram());
            ICore core = CoreFactory.Create(exe, headless: true, null, options: TestCores.NoBattery);
            core.LoadRom(exe);
            for (int i = 0; i < 10; i++) core.RunFrame();
            using (core as IDisposable)
            {
                var target = CoreFactory.Bundle(core).DebugTarget;
                Assert.IsType<CoreDebugTarget>(target);
                Assert.StartsWith("Diamond 0.1.0, psx, frame 10", target.GetSummaryText());
                var shell = DianaShInterpreter.CreateDefault(target);
                string regs = shell.Submit("regs").Output;
                Assert.Contains("zero", regs);
                Assert.Contains("CAUSE", regs);
                Assert.Contains("CAUSE", shell.Submit("regs r3000").Output);
                Assert.Equal("0", (shell.Submit("disasm cpu 80010100 2"), shell.LastExitStatus).Item2);
                Assert.StartsWith("Breakpoint #", shell.Submit("bp add 80010108").Output);
            }
        }
    }
}
