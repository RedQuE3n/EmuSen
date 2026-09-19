using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Blocks;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Compiled blocks between the checks the interpreter makes every step - see Mars_Recompiler.md §3.
    public sealed partial class Cpu
    {
        [EmuSen.Common.SkipInState] private readonly BlockCache _blocks;
        [EmuSen.Common.SkipInState] private long _capAt;

        // What the blocks cost and carry, for the harness and the debugger - see Mars_Recompiler.md §7.
        [EmuSen.Common.SkipInState] public long BlockRuns;
        [EmuSen.Common.SkipInState] public long BlockInstructions;

        // Off by the tests that count what was compiled; on, a hot block is interpreted until the compiler's thread publishes it - see Mars_Recompiler.md §2.4.
        [EmuSen.Common.SkipInState] public bool CompileInBackground = true;

        public long BlocksCompiled => _blocks.Compiled;

        public long BlocksDiscarded => _blocks.Discarded;

        public long BlocksReshaped => _blocks.Reshaped;

        public long CompileTicks => _blocks.CompileTicks;

        // One block where one can run, one instruction through the interpreter where none can - see Mars_Recompiler.md §3.1.
        public void StepBlock(long capAt, long fields)
        {
            ulong pc = Pc;
            byte[] rdram = _bus.Rdram;
            uint physical = (uint)pc & 0x1FFF_FFFF;

            if (_branchPending || _mode != PrivilegeMode.Kernel || pc - KernelDirectBase >= KernelDirectSize || physical >= (uint)rdram.Length || (pc & 3) != 0)
            {
                Step();
                return;
            }

            // The display processor may be drawing over code; the block's words are waited for before they are compared or shaped - see Mars_Rdp.md §2.6.
            if (_dpMarks[physical >> 12] != 0 || _dpMarks[(physical + BlockCache.MaxLength * 4 - 1) >> 12] != 0) _bus.Dp.WaitForRange(physical, BlockCache.MaxLength * 4, 5);

            Block? block = _blocks.Find(physical);
            if (block is null) _blocks.Place(block = BlockShape.Shape(physical, rdram));

            // Read once: code the compiler's thread publishes after this read runs at the next entry, after the comparison - see Mars_Recompiler.md §2.4.
            BlockCode? code = block.Code;

            if (code is null)
            {
                if (!block.Refused && ++block.Runs == BlockCache.Threshold)
                {
                    // The shape was taken from the words at the first run; code loaded over a cold block keeps it, so it is taken again - see Mars_Recompiler.md §2.3.
                    Block current = BlockShape.Shape(physical, rdram);
                    if (current.Length != block.Length || current.Refused)
                    {
                        _blocks.Reshaped++;
                        _blocks.Place(current);
                        Interpret(current.Length, capAt, fields);
                        return;
                    }

                    block.Image = rdram.AsSpan((int)physical, block.Length * 4).ToArray();
                    if (CompileInBackground) BlockCompiler.Enqueue(block, _blocks, rdram.Length);
                    else BlockCompiler.Compile(block, _blocks, rdram.Length);
                    code = CompileInBackground ? null : block.Code;
                }

                if (code is null)
                {
                    Interpret(block.Length, capAt, fields);
                    return;
                }
            }
            else if (!rdram.AsSpan((int)physical, block.Length * 4).SequenceEqual(block.Image))
            {
                _blocks.Discarded++;
                _blocks.Place(block = BlockShape.Shape(physical, rdram));
                Interpret(block.Length, capAt, fields);
                return;
            }

            _capAt = capAt;
            BlockRuns++;
            long before = Instructions;

            try
            {
                CurrentPc = pc;
                InDelaySlot = false;
                VerifyMode();

                bool asserted = _mi.Asserted;
                if (_recheck || asserted != _assertedSeen) CheckInterrupts(asserted);

                code(this);
            }
            catch (CpuException raised)
            {
                _lastCount = _bus.Count;
                LastException = raised;
                EnterException(raised);
                _bus.Tick(1);
            }

            BlockInstructions += Instructions - before;
        }

        // Exactly <steps> instructions: a block may run only as many cycles as instructions remain, and each costs at least one - see Mars_Recompiler.md §6.
        public void RunBlocks(long steps)
        {
            long end = Instructions + steps;

            while (Instructions < end) StepBlock(_bus.Cycles + (end - Instructions), _bus.Vi.Fields);
        }

        // The interpreter along the block's own straight line, and no further - see Mars_Recompiler.md §3.1.
        private void Interpret(int length, long capAt, long fields)
        {
            ulong expected = Pc;

            for (int i = 0; i < length && Pc == expected && _bus.Vi.Fields == fields && _bus.Cycles < capAt; i++)
            {
                Step();
                expected += 4;
            }
        }

        // What a step does after its instruction: the events due, the timer, the count - see Mars_Recompiler.md §3.2.
        internal void AfterInstruction()
        {
            if (_bus.Cycles >= _bus.NextEvent) _bus.RunEvents();
            if (_bus.Cycles >= _timerDue) TimerReached();
            _lastCount = _bus.Count;
        }

        // The signal processor's share of one instruction, and whether what it did ends the block - see Mars_Recompiler.md §4.
        internal bool RspRan(long cycles, long written)
        {
            _bus.Sp.Step(cycles);
            return _bus.Written != written || _mi.Asserted != _assertedSeen;
        }

        internal void ReturnedAfterSlot()
        {
            _returnAfterSlot = false;
            ReturnObserver?.Invoke();
        }

        // Debug builds prove, after every instruction a block runs on from, that the interpreter's check would have done nothing - see Mars_Recompiler.md §5.
        internal void VerifyBlockStep()
        {
            bool asserted = _mi.Asserted;

            if (_recheck || asserted != _assertedSeen)
            {
                throw new InvalidOperationException("A block ran on past a change the interpreter would have checked before its next instruction.");
            }

            VerifySkippedCheck(asserted);
            VerifyMode();
        }
    }
}
