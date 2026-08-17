using System;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

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

    public partial class Cpu
    {
        private ICpuBus _bus;

        // Per-instruction tracing; on for a normal run it makes a working emulator look hung.

        public ushort A;  
        public ushort X;  
        public ushort Y;  
        public ushort S;  
        public ushort D;  
        
        public ushort PC; 
        public byte PB;   
        public byte DB;   

        // Snapshotted at instruction start, unlike the live PC/PB above - see Venus_CPU.md §2.
        public ushort LastInstructionPC;
        public byte LastInstructionPB;

        // Debug observers of call/return and interrupt flow - see `man bt`.
        [EmuSen.Common.SkipInState] public CallStackRegistry? CallStack;
        [EmuSen.Common.SkipInState] public BreakpointRegistry? Breakpoints;
        
        public byte P;    
        public bool E;    

        // WAI/STP state - see Venus_CPU.md §4.
        private bool _waitingForInterrupt;
        private bool _stopped;

        // Addressing-mode penalties the static per-opcode table cannot express - see Venus_CPU.md §8.
        private int _addrModeExtraCycles;

        // A struct key, so a locked loop costs a comparison and no allocation - see DebugTools.cs.
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

        // Holds delegates, so it cannot be serialized - see StateSerializer's own contract.
        [EmuSen.Common.SkipInState] private readonly DebugTools.RepeatCollapsingTrace<StepKey> _verboseTrace;
        private bool _wasVerboseLogging;

        // <name> distinguishes the SA-1's CPU in trace output - see Venus_SA1.md §4.1.
        [EmuSen.Common.SkipInState] private readonly string _name;
        [EmuSen.Common.SkipInState] private readonly bool _logReset;

        // Only the S-CPU; the SA-1's own core shares this class and has no Mesen counterpart to diff against.
        [EmuSen.Common.SkipInState] private readonly bool _traceBinary;

        public Cpu(ICpuBus bus, string name = "CPU", bool logReset = true)
        {
            _name = name;
            _logReset = logReset;
            _traceBinary = name == "CPU";
            _bus = bus;
            _verboseTrace = new DebugTools.RepeatCollapsingTrace<StepKey>(
                Console.WriteLine,
                key => $"[CPU] 0x{key.Pb:X2}{key.Pc:X4}: {OpcodeNames[key.Opcode]} (Opcode 0x{key.Opcode:X2}) -> Target Addr: 0x{key.TargetAddr:X6}",
                (cycleLength, repeats) => cycleLength == 1
                    ? $"[CPU]     ^ repeated {repeats}x total"
                    : $"[CPU]     ^ {cycleLength}-instruction loop above repeated {repeats}x total");
            Reset();
        }

        // For the one case Step cannot see coming: the process exiting with logging on.
        public void FlushVerboseTrace() => _verboseTrace.Flush();

        // The extra bus cycles a 16-bit operand costs - see Venus_CPU.md §8.8.
        private static int WidthPenalty(byte opcode, bool wide16A, bool wide16X) => OpcodeWidth[opcode] switch
        {
            WidthM => wide16A ? 1 : 0,
            WidthMRmw => wide16A ? 2 : 0,
            WidthX => wide16X ? 1 : 0,
            _ => 0,
        };

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

            if (_logReset)
            {
                Console.WriteLine($"[{_name}] Power-on Reset Complete.");
                Console.WriteLine($"[{_name}] PC set to: 0x{PB:X2}{PC:X4}");
            }
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
            // WAI/STP consume idle cycles, and pending DMA cost must drain on those paths too.
            if (_stopped) return _bus.GetAccessSpeedCycles(((uint)PB << 16) | PC) * 3 + DrainPendingDmaCycles();
            if (_waitingForInterrupt) return _bus.GetAccessSpeedCycles(((uint)PB << 16) | PC) * 2 + DrainPendingDmaCycles();

            ushort executedAtPC = PC;
            byte executedAtPB = PB;
            LastInstructionPC = executedAtPC;
            LastInstructionPB = executedAtPB;
            uint opcodeAddr = ((uint)executedAtPB << 16) | executedAtPC;
            _addrModeExtraCycles = 0;
            // Sampled before the instruction runs, since REP/SEP/PLP change them mid-flight.
            bool wide16A = !GetFlag(CpuFlags.M);
            bool wide16X = !GetFlag(CpuFlags.X);
            byte opcode = Fetch8();

            // Before Dispatch, so registers are this instruction's inputs - the point Mesen records at.
            if (CpuBinaryTrace.Enabled && _traceBinary)
            {
                CpuBinaryTrace.Record(opcodeAddr, opcode, CpuBinaryTrace.KindInstruction,
                    A, X, Y, S, D, DB, P, E);
            }

            uint targetAddr = Dispatch(opcode);

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
                // Flush whatever the trace still holds, or it strands until logging returns.
                _verboseTrace.Flush();
                _wasVerboseLogging = false;
            }

            // Each access charged at its own region speed, with one known residual - see Venus_CPU.md §8.
            int totalCycleUnits = OpcodeCycles[opcode] + _addrModeExtraCycles + WidthPenalty(opcode, wide16A, wide16X);
            int bytesFetched = OpcodeFixedBytes[opcode];
            if (bytesFetched == 0)
            {
                bytesFetched = (ushort)(PC - executedAtPC);
                if (bytesFetched > totalCycleUnits) bytesFetched = totalCycleUnits;
            }
            int remainderUnits = totalCycleUnits - bytesFetched;

            int masterClocks = bytesFetched * _bus.GetAccessSpeedCycles(opcodeAddr);
            if (remainderUnits > 0)
            {
                masterClocks += remainderUnits * (OpcodeInternalRemainder[opcode]
                    ? InternalCycleClocks
                    : _bus.GetAccessSpeedCycles(targetAddr));
            }

            int totalClocks = masterClocks + DrainPendingDmaCycles();
            if (CpuBinaryTrace.Enabled && _traceBinary) CpuBinaryTrace.SetLastCost(totalClocks);
            return totalClocks;
        }

        // Converted here, since Step returns master clocks now - see Venus_Memory.md §3.1.
        private int DrainPendingDmaCycles()
        {
            return _bus.TakePendingDmaCycles() * 8;
        }

        // NMI entry sequence - see Venus_CPU.md §3.
        public void Nmi()
        {
            _waitingForInterrupt = false;

            // Carries the interrupted address, so the trace shows where vblank landed.
            if (CpuBinaryTrace.Enabled && _traceBinary)
            {
                CpuBinaryTrace.Record(((uint)PB << 16) | PC, 0, CpuBinaryTrace.KindNmi, A, X, Y, S, D, DB, P, E);
            }

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
            NoteInterruptFrame(CallFrameKind.Nmi);

            if (DebugSettings.CpuVerboseLogging)
            {
                // Flush first, so an interrupted loop is written before this line, in execution order.
                _verboseTrace.Flush();
                Console.WriteLine($"[CPU] NMI -> PC = 0x00{PC:X4}");
            }
        }

        // Where `bt` and `runto nmi|irq|brk|cop` learn an interrupt was taken.
        private void NoteInterruptFrame(CallFrameKind kind)
        {
            CallStack?.NotePush((LastInstructionPB << 16) | LastInstructionPC, (PB << 16) | PC, kind);
            Breakpoints?.NoteInterrupt(kind);
        }

        // IRQ entry sequence - see Venus_CPU.md §3.
        public bool Irq()
        {
            _waitingForInterrupt = false;

            if (GetFlag(CpuFlags.I)) return false;

            ushort interruptedPC = PC;
            byte interruptedPB = PB;

            // After the I-flag check above, so only interrupts actually taken appear.
            if (CpuBinaryTrace.Enabled && _traceBinary)
            {
                CpuBinaryTrace.Record(((uint)PB << 16) | PC, 0, CpuBinaryTrace.KindIrq, A, X, Y, S, D, DB, P, E);
            }

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
            NoteInterruptFrame(CallFrameKind.Irq);

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