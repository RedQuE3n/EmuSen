using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    [Flags]
    public enum CpuFlags : byte
    {
        None = 0,
        C = (1 << 0), 
        Z = (1 << 1), 
        I = (1 << 2), 
        D = (1 << 3), 
        X = (1 << 4), 
        M = (1 << 5), 
        V = (1 << 6), 
        N = (1 << 7)  
    }

    public struct Instruction
    {
        public string Name;
        public Func<uint> AddrMode;    
        public Action<uint> Operate;   
        public byte Cycles;            
    }

    public partial class Cpu
    {
        private MemoryBus _bus;
        [EmuSen.Common.SkipInState] private Instruction[] _instructions = null!; 

        // Set to true for detailed per-instruction tracing. Leave false for normal runs -
        // printing a console line for every single instruction (including ones inside
        // polling loops that legitimately need many iterations, like the APU handshake)
        // makes a working emulator look like it's hung.

        public ushort A;  
        public ushort X;  
        public ushort Y;  
        public ushort S;  
        public ushort D;  
        
        public ushort PC; 
        public byte PB;   
        public byte DB;   

        // Snapshotted at instruction start, unlike the live PC/PB above -
        // see Venus_CPU.md §2.
        public ushort LastInstructionPC;
        public byte LastInstructionPB;
        
        public byte P;    
        public bool E;    

        // WAI/STP state - see Venus_CPU.md §4.
        private bool _waitingForInterrupt;
        private bool _stopped;

        // Equality key for one executed instruction, used only by
        // _verboseTrace to detect repeating polling/delay loops - see
        // DebugTools.RepeatCollapsingTrace<TKey> for why a struct key
        // instead of comparing the rendered strings. Name isn't part of
        // the key: it's a deterministic function of Opcode, looked up
        // again in Render() only when a line is actually emitted.
        private readonly struct StepKey : IEquatable<StepKey>
        {
            public readonly byte Pb;
            public readonly ushort Pc;
            public readonly byte Opcode;
            public readonly uint TargetAddr;

            public StepKey(byte pb, ushort pc, byte opcode, uint targetAddr)
            {
                Pb = pb;
                Pc = pc;
                Opcode = opcode;
                TargetAddr = targetAddr;
            }

            public bool Equals(StepKey other) =>
                Pb == other.Pb && Pc == other.Pc && Opcode == other.Opcode && TargetAddr == other.TargetAddr;

            public override bool Equals(object? obj) => obj is StepKey other && Equals(other);

            public override int GetHashCode() => HashCode.Combine(Pb, Pc, Opcode, TargetAddr);
        }

        // Holds delegates (a Console.WriteLine reference plus two closures)
        // internally - not serializable, same reasoning as _instructions
        // above (case 1 in StateSerializer's own doc comment), just not
        // caught at the time this field was added. Surfaced by the
        // headless debug harness's own --savestate/--loadstate smoke test:
        // reflecting into a delegate hits its private method-pointer field
        // (a raw IntPtr), which StateSerializer has no case for.
        [EmuSen.Common.SkipInState] private readonly DebugTools.RepeatCollapsingTrace<StepKey> _verboseTrace;
        private bool _wasVerboseLogging;

        public Cpu(MemoryBus bus)
        {
            _bus = bus;
            BuildOpcodeTable();
            _verboseTrace = new DebugTools.RepeatCollapsingTrace<StepKey>(
                Console.WriteLine,
                key => $"[CPU] 0x{key.Pb:X2}{key.Pc:X4}: {_instructions[key.Opcode].Name} (Opcode 0x{key.Opcode:X2}) -> Target Addr: 0x{key.TargetAddr:X6}",
                (cycleLength, repeats) => cycleLength == 1
                    ? $"[CPU]     ^ repeated {repeats}x total"
                    : $"[CPU]     ^ {cycleLength}-instruction loop above repeated {repeats}x total");
            Reset();
        }

        // Step() already flushes automatically the moment it notices
        // CpuVerboseLogging went from on to off (see _wasVerboseLogging),
        // which covers `trace off` and the countdown running out - this is
        // for the one case Step() can't see coming: the process exiting
        // while logging is still on, where nothing calls Step() again to
        // notice. Program.cs's shutdown path calls this before disposing
        // the log writer.
        public void FlushVerboseTrace() => _verboseTrace.Flush();

        public void Reset()
        {
            E = true;
            
            A = 0x0000;
            X = 0x0000;
            Y = 0x0000;
            D = 0x0000;
            S = 0x01FF; 
            
            PB = 0x00;
            DB = 0x00;

            P = (byte)(CpuFlags.M | CpuFlags.X | CpuFlags.I);

            _waitingForInterrupt = false;
            _stopped = false;
            
            byte resetLow = _bus.Read8(0x00FFFC);
            byte resetHigh = _bus.Read8(0x00FFFD);
            
            PC = (ushort)((resetHigh << 8) | resetLow);

            Console.WriteLine($"[CPU] Power-on Reset Complete.");
            Console.WriteLine($"[CPU] PC set to: 0x{PB:X2}{PC:X4}");
        }

        private byte Fetch8()
        {
            uint address = ((uint)PB << 16) | PC;
            byte data = _bus.Read8(address);
            PC++;
            return data;
        }

        private ushort Fetch16()
        {
            ushort low = Fetch8();
            ushort high = Fetch8();
            return (ushort)((high << 8) | low);
        }

        public int Step()
        {
            // STP: halted until Reset(). WAI: halted until Nmi()/Irq() clears
            // _waitingForInterrupt (see those methods below). Neither fetches
            // or executes anything while active - just consumes idle cycles
            // so the caller's per-scanline cycle budget still advances.
            //
            // DrainPendingDmaCycles() covers both early-return cases too,
            // not just the normal instruction path below - a general-
            // purpose DMA triggered by the instruction immediately before
            // a WAI/STP would otherwise have its cost silently dropped
            // (Dma.PendingCpuCycles accumulates the instant $420B is
            // written, mid-instruction, so it can already be nonzero the
            // moment this method is entered).
            if (_stopped) return 3 + DrainPendingDmaCycles();
            if (_waitingForInterrupt) return 2 + DrainPendingDmaCycles();

            ushort executedAtPC = PC;
            byte executedAtPB = PB;
            LastInstructionPC = executedAtPC;
            LastInstructionPB = executedAtPB;
            byte opcode = Fetch8();
            Instruction inst = _instructions[opcode];

            if (inst.Name == "NOP/UNK")
            {
                throw new NotImplementedException($"Unimplemented Opcode: 0x{opcode:X2} at PC: 0x{PB:X2}{(PC - 1):X4}");
            }

            uint targetAddr = inst.AddrMode();
            inst.Operate(targetAddr);

            if (DebugSettings.CpuVerboseLogging)
            {
                _verboseTrace.Log(new StepKey(executedAtPB, executedAtPC, opcode, targetAddr));
                if (DebugSettings.CpuTraceCountdown > 0)
                {
                    DebugSettings.CpuTraceCountdown--;
                    if (DebugSettings.CpuTraceCountdown == 0) DebugSettings.CpuVerboseLogging = false;
                }
                _wasVerboseLogging = DebugSettings.CpuVerboseLogging;
            }
            else if (_wasVerboseLogging)
            {
                // Verbose logging just turned off - by the countdown above,
                // by `trace off`, or by anything else flipping the flag -
                // flush whatever loop/tail _verboseTrace was still holding
                // so it isn't stranded until the next time logging happens
                // to turn back on (or lost entirely if it never does).
                _verboseTrace.Flush();
                _wasVerboseLogging = false;
            }

            // Return the cycles consumed by this instruction
            // Note: In a fully accurate emulator, we would multiply this by 6, 8, or 12
            // depending on the memory region, but for now, base cycles are fine.
            return inst.Cycles + DrainPendingDmaCycles();
        }

        // See Dma.PendingCpuCycles's own comment for why this exists and
        // its unit convention. Drained (read then zeroed) rather than
        // just read, so a DMA's cost is attributed exactly once, to
        // whichever Step() call notices it first.
        private int DrainPendingDmaCycles()
        {
            int cycles = _bus.Dma.PendingCpuCycles;
            if (cycles == 0) return 0;
            _bus.Dma.PendingCpuCycles = 0;
            return cycles;
        }

        // NMI entry sequence - see Venus_CPU.md §3. Triggered externally
        // from the main loop on simulated vblank when NMITIMEN's enable bit is set.
        public void Nmi()
        {
            _waitingForInterrupt = false;

            if (!E)
            {
                Push8(PB);
            }

            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.D, false);
            SetFlag(CpuFlags.I, true);

            ushort vectorAddr = E ? (ushort)0xFFFA : (ushort)0xFFEA;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);

            if (DebugSettings.CpuVerboseLogging)
            {
                // Flush first so a loop the NMI interrupted (still pending
                // in _verboseTrace, possibly mid-repeat) is written out
                // before this line, keeping the file in execution order.
                _verboseTrace.Flush();
                Console.WriteLine($"[CPU] NMI -> PC = 0x00{PC:X4}");
            }
        }

        // IRQ entry sequence - see Venus_CPU.md §3.
        public bool Irq()
        {
            _waitingForInterrupt = false;

            if (GetFlag(CpuFlags.I)) return false;

            ushort interruptedPC = PC;
            byte interruptedPB = PB;

            if (!E)
            {
                Push8(PB);
            }

            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.D, false);
            SetFlag(CpuFlags.I, true);

            ushort vectorAddr = E ? (ushort)0xFFFE : (ushort)0xFFEE;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);

            if (DebugSettings.CpuVerboseLogging)
            {
                _verboseTrace.Flush(); // see Nmi()'s own comment on why
                Console.WriteLine($"[IRQ FIRE] interrupted at 0x{interruptedPB:X2}{interruptedPC:X4} -> vector@0x{vectorAddr:X4}={low:X2}{high:X2} -> jumping to 0x00{PC:X4}");
            }

            return true;
        }

        // --- Stack Helpers ---

        private void Push8(byte value)
        {
            if (E)
            {
                _bus.Write8((uint)(0x0100 | (S & 0xFF)), value);
                S = (ushort)(0x0100 | ((S - 1) & 0xFF));
            }
            else
            {
                _bus.Write8(S, value);
                S--;
            }
        }

        private void Push16(ushort value)
        {
            Push8((byte)(value >> 8));
            Push8((byte)(value & 0xFF));
        }

        private byte Pop8()
        {
            if (E)
            {
                S = (ushort)(0x0100 | ((S + 1) & 0xFF));
                return _bus.Read8(S);
            }
            else
            {
                S++;
                return _bus.Read8(S);
            }
        }

        private ushort Pop16()
        {
            byte low = Pop8();
            byte high = Pop8();
            return (ushort)((high << 8) | low);
        }

        // --- Flag Helper Methods ---

        public void SetFlag(CpuFlags flag, bool condition)
        {
            if (condition) P |= (byte)flag;
            else P &= (byte)~flag;
        }

        public bool GetFlag(CpuFlags flag) { return (P & (byte)flag) != 0; }
        public bool IsMemory8Bit => E || GetFlag(CpuFlags.M);
        public bool IsIndex8Bit => E || GetFlag(CpuFlags.X);

        private void UpdateZN(ushort value, bool is8Bit)
        {
            if (is8Bit)
            {
                SetFlag(CpuFlags.Z, (value & 0xFF) == 0);
                SetFlag(CpuFlags.N, (value & 0x80) != 0);
            }
            else
            {
                SetFlag(CpuFlags.Z, value == 0);
                SetFlag(CpuFlags.N, (value & 0x8000) != 0);
            }
        }
    }
}