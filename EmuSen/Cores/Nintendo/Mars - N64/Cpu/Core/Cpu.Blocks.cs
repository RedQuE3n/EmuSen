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
            bool direct = pc - KernelDirectBase < KernelDirectSize;
            uint physical = (uint)pc & 0x1FFF_FFFF;

            // Code behind the TLB runs in blocks too, where its page is mapped; anything else is the interpreter's, which raises what is to be raised - see Mars_Recompiler.md §17.
            if (_branchPending || _mode != PrivilegeMode.Kernel || (pc & 3) != 0 || (!direct && !MappedFetch(pc, out physical)) || physical >= (uint)rdram.Length)
            {
                Step();
                return;
            }

            // The display processor may be drawing over code; the block's words are waited for before they are compared or shaped - see Mars_Rdp.md §2.6.
            if (_dpWriteMarks[physical >> 12] != 0 || _dpWriteMarks[(physical + BlockCache.MaxLength * 4 - 1) >> 12] != 0) _bus.Dp.WaitForReadRange(physical, BlockCache.MaxLength * 4, 5);

            Block? block = _blocks.Find(physical);
            if (block is null) _blocks.Place(block = BlockShape.Shape(physical, rdram, withinPage: !direct));

            // Past its page a mapped block's next word may be another frame's, so such a block is left to the interpreter from a mapped address - see §17.
            if (!direct && (physical & 0xFFF) + (uint)block.Length * 4 > 0x1000)
            {
                Step();
                return;
            }

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

                if (block.IdleCycles != 0 && SkipIdle && (direct || block.IdleAnywhere)) RunIdle(block, pc, capAt);
                else code(this);
            }
            catch (CpuException raised)
            {
                _lastCount = _bus.Count;
                LastException = raised;
                EnterException(raised);
                _bus.Tick(1);
            }

            BlockInstructions += Instructions - before;
            if (BlockCensus) { var e = BlockCensusCounts.GetValueOrDefault(physical); bool rsp = !_bus.Sp.Processor.Halted; long n = Instructions - before; BlockCensusCounts[physical] = (e.Item1 + 1, e.Item2 + n, e.Item3 + (rsp ? n : 0), block.Length); }
        }

        // The last page a fetch was translated through; forgotten whenever the TLB or coprocessor 0 is written - see Mars_Recompiler.md §17.
        [EmuSen.Common.SkipInState] private ulong _fetchPage = ulong.MaxValue;
        [EmuSen.Common.SkipInState] private uint _fetchFrame;

        // Off only to measure what mapped blocks save, or to show they change nothing - see §17.
        public static bool MappedBlocks = Environment.GetEnvironmentVariable("EMUSEN_MARS_NOMAPPEDBLOCKS") != "1";

        private bool MappedFetch(ulong pc, out uint physical)
        {
            ulong page = pc & ~0xFFFUL;
            if (page == _fetchPage)
            {
                physical = _fetchFrame | (uint)(pc & 0xFFF);
                return true;
            }

            physical = 0;
            if (!MappedBlocks) return false;
            if (WideAddressing || Segments.Decode(pc, Mode, WideAddressing).Access != SegmentAccess.Mapped) return false;
            if (Tlb.TryTranslate(pc, Cop0[EntryHiRegister], false, out uint mapped, out _) != TlbResult.Mapped) return false;

            (_fetchPage, _fetchFrame) = (page, mapped & ~0xFFFu);
            physical = mapped;
            return true;
        }

        internal void ForgetFetchPage() => _fetchPage = ulong.MaxValue;

        // Off only to measure what it saves - see Mars_Recompiler.md §15.
        public static bool SkipIdle = Environment.GetEnvironmentVariable("EMUSEN_MARS_NOIDLESKIP") != "1";

        // The idle loop run here instead of by its block: whole turns passed at once while the signal processor is halted, the processor stepped without the block's bookkeeping while it runs, and every exit left as the block leaves it - see Mars_Recompiler.md §15.
        private void RunIdle(Block block, ulong entry, long capAt)
        {
            MemoryBus bus = _bus;
            Rsp.Rsp rsp = bus.Sp.Processor;
            long stop = Math.Min(Math.Min(bus.NextEvent, _timerDue), capAt), written = bus.Written;
            int branch = block.IdleBranchCycles, slot = block.IdleCycles - branch;
            bool afterSlot = false, atSlot = false;
            long idleFrom = Instructions;
            bool whole = branch == 1 && slot == 1 && Rsp.Rsp.UseBlocks && !bus.Sp.SingleStepping && rsp.Coverage is not { IsArmed: true };

            while (true)
            {
                if (rsp.Halted)
                {
                    long turns = atSlot ? 0 : (stop - bus.Cycles - 1) / block.IdleCycles - 1;
                    if (turns > 0)
                    {
                        bus.Cycles += turns * block.IdleCycles;
                        Instructions += turns * 2;
                        IdleTurnsPassed += turns;
                    }
                }
                else if (whole)
                {
                    // One step a cycle and one cycle an instruction here, so the steps run are the instructions passed, and an event ends the run on its own step - see Mars_Rsp.md §11.
                    long ran = rsp.RunBlocks(stop - bus.Cycles - 1);
                    if (ran > 0)
                    {
                        bus.Cycles += ran;
                        Instructions += ran;
                        RspSteps += ran;
                        afterSlot = (ran & 1) != 0 ? atSlot : !atSlot;
                        atSlot = !afterSlot;
                        if (bus.Written != written || _mi.Asserted != _assertedSeen) break;
                    }
                }

                int cycles = atSlot ? slot : branch;
                afterSlot = atSlot;
                bus.Cycles += cycles;
                Instructions++;
                if (!rsp.Halted && RspRan(cycles, written)) break;
                if (bus.Cycles >= stop) break;
                atSlot = !atSlot;
            }

            // What the block's own exits leave: after the branch, pending into the slot; after the slot, back at the loop's head.
            Pc = afterSlot ? entry : entry + 4;
            NextPc = afterSlot ? entry + 4 : entry;
            _branchPending = !afterSlot;
            InDelaySlot = afterSlot;
            CurrentPc = afterSlot ? entry + 4 : entry;
            IdleInstructions += Instructions - idleFrom;
            AfterInstruction();
        }

        [EmuSen.Common.SkipInState] public long IdleTurnsPassed, IdleInstructions, RspSteps;

        public static readonly bool BlockCensus = Environment.GetEnvironmentVariable("EMUSEN_MARS_BLOCKCENSUS") == "1";
        public static readonly System.Collections.Generic.Dictionary<uint, (long, long, long, int)> BlockCensusCounts = new();

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
            SpInterface sp = _bus.Sp;
            RspSteps += cycles;
            if (cycles == 1 && !sp.SingleStepping) sp.Processor.StepOne();
            else sp.Step(cycles);
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
