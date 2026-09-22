using System;
using System.IO;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Rsp;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The idle loop run in one piece, and the signal processor run in blocks while it idles, leave the machine the instructions one by one leave - see Mars_Recompiler.md §15 and Mars_Rsp.md §11.
    [Collection("MarsStatics")]
    public class MarsIdleTests : IDisposable
    {
        private const uint Imem = 0x0400_1000, SpStatus = 0x0404_0010, SpPc = 0x0408_0000, MiMask = 0x0430_000C;

        // A counted loop of scalar, vector and store work whose status write raises the interrupt half way through, from a block that has run thousands of times; the padding moves every step to the other instruction of the idle loop.
        private static uint[] SignalProgram(bool padded)
        {
            var words = new System.Collections.Generic.List<uint>
            {
                0x2008_0000, 0x2009_4000,
                0x2108_0001, 0x4888_0800, 0x4A02_0850, 0xAC08_0010, 0x0008_5342, 0x000A_5100, 0x408A_2000, 0x1509_FFF8, 0x0000_0000,
            };
            if (padded) words.Insert(0, 0x0000_0000);
            words.Add(0x0000_000D);
            return words.ToArray();
        }

        public void Dispose()
        {
            Cpu.SkipIdle = true;
            Rsp.UseBlocks = true;
            Rsp.CompileBlocksInBackground = true;
            Rsp.FoldAfter = 10000;
        }

        // The CPU takes the signal processor's interrupt on the instruction it lands on, so a step early or late, or the wrong one of the two, changes the state.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void The_idle_loop_and_the_processors_blocks_leave_the_state_the_single_steps_leave(bool padded)
        {
            Rsp.CompileBlocksInBackground = false;

            (byte[] stepped, _, _) = Run(skipIdle: false, blocks: false, padded);
            (byte[] passed, long turns, _) = Run(skipIdle: true, blocks: false, padded);
            (byte[] compiled, long turnsToo, long blockSteps) = Run(skipIdle: true, blocks: true, padded);

            Assert.True(turns > 0 && turnsToo > 0, "no idle turn was passed, so the test compared nothing");
            Assert.True(blockSteps > 10_000, $"only {blockSteps} steps ran in blocks, so the test compared nothing");
            Assert.Equal(stepped, passed);
            Assert.Equal(stepped, compiled);
        }

        // A task a block flooded the pool that a frame's presentation waits on, stalling boots for seconds - see Mars_Rsp.md §11.1.
        [Fact]
        public void Blocks_compiled_in_the_background_are_compiled_off_the_pool_and_change_nothing()
        {
            Rsp.CompileBlocksInBackground = false;
            (byte[] stepped, _, _) = Run(skipIdle: false, blocks: false, padded: false);

            Rsp.CompileBlocksInBackground = true;
            long before = System.Threading.Interlocked.Read(ref Rsp.PoolCompiles);
            (byte[] compiled, _, long blockSteps, long blocksCompiled) = RunCounting(skipIdle: true, blocks: true, padded: false);

            Assert.True(blocksCompiled > 0, "no block was compiled, so the test compared nothing");
            Assert.Equal(0, System.Threading.Interlocked.Read(ref Rsp.PoolCompiles) - before);
            Assert.Equal(stepped, compiled);
        }

        // A block runs first uninlined and later folded; either tier, or the change between them, leaves the state the single steps leave - see Mars_Rsp.md §11.2.
        [Theory]
        [InlineData(1)]
        [InlineData(50)]
        [InlineData(int.MaxValue)]
        public void Either_tier_of_block_and_the_move_between_them_leave_the_state_the_single_steps_leave(int foldAfter)
        {
            Rsp.CompileBlocksInBackground = false;
            (byte[] stepped, _, _) = Run(skipIdle: false, blocks: false, padded: false);

            Rsp.FoldAfter = foldAfter;
            (byte[] compiled, _, long blockSteps) = Run(skipIdle: true, blocks: true, padded: false, out MarsCore core);

            Assert.True(blockSteps > 10_000, $"only {blockSteps} steps ran in blocks, so the test compared nothing");
            if (foldAfter == int.MaxValue) Assert.Equal(0, core.Bus!.Sp.Processor.BlocksFolded);
            else Assert.True(core.Bus!.Sp.Processor.BlocksFolded > 0, "no block was folded, so the second tier was not compared");
            Assert.Equal(stepped, compiled);
        }

        private static (byte[] State, long Turns, long BlockSteps, long Compiled) RunCounting(bool skipIdle, bool blocks, bool padded)
        {
            (byte[] state, long turns, long steps) = Run(skipIdle, blocks, padded, out MarsCore core);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (System.Threading.Interlocked.Read(ref core.Bus!.Sp.Processor.BlocksCompiled) == 0 && clock.ElapsedMilliseconds < 5000) System.Threading.Thread.Sleep(5);
            return (state, turns, steps, System.Threading.Interlocked.Read(ref core.Bus.Sp.Processor.BlocksCompiled));
        }

        private static (byte[] State, long Turns, long BlockSteps) Run(bool skipIdle, bool blocks, bool padded) => Run(skipIdle, blocks, padded, out _);

        private static (byte[] State, long Turns, long BlockSteps) Run(bool skipIdle, bool blocks, bool padded, out MarsCore machine)
        {
            Cpu.SkipIdle = skipIdle;
            Rsp.UseBlocks = blocks;

            string path = Path.Combine(Path.GetTempPath(), "mars_idle_" + Guid.NewGuid().ToString("N") + ".z64");
            File.WriteAllBytes(path, SyntheticN64Rom.BuildRunningFromRdram(new uint[] { 0x1000_FFFF, 0x0000_0000 }));

            var core = new MarsCore(batteryRamDisabled: true);
            machine = core;
            core.LoadRom(path);
            core.Cpu!.CompileInBackground = false;
            for (int i = 0; i < 3; i++) core.RunFrame();

            uint[] program = SignalProgram(padded);
            for (int i = 0; i < program.Length; i++) core.Bus!.Write32(Imem + (uint)i * 4, program[i]);
            // The handler keeps the count it was entered at, so an interrupt seen a step late is a different state, and then idles.
            uint[] handler = { 0x401A_4800, 0x3C1B_8000, 0xAF7A_0300, 0x1000_FFFF, 0x0000_0000 };
            for (int i = 0; i < handler.Length; i++) core.Bus!.Write32(0x180 + (uint)i * 4, handler[i]);
            core.Bus!.Write32(MiMask, 0x0002);
            core.Cpu.Cop0[Cpu.StatusRegister] |= 0x0401;
            core.Cpu.Cop0Written();
            core.Bus.Write32(SpPc, 0);
            core.Bus.Write32(SpStatus, 0x0105);
            for (int i = 0; i < 4; i++) core.RunFrame();

            Assert.True(core.Bus.Sp.Processor.Halted, "the signal processor's program did not reach its break");
            Assert.True((core.Cpu.Cop0[Cpu.StatusRegister] & 2) != 0, "the CPU never took the signal processor's interrupt");
            using var state = new MemoryStream();
            core.SaveState(state);
            return (state.ToArray(), core.Cpu.IdleTurnsPassed, core.Bus.Sp.Processor.BlockSteps);
        }
    }
}
