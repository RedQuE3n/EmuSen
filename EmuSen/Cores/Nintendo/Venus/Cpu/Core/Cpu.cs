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

        // The PC/PB of whichever instruction is CURRENTLY executing, as
        // opposed to the live PC/PB fields above which point at the NEXT
        // instruction the moment Fetch8() advances past the opcode/operand
        // bytes (i.e. almost immediately, well before that instruction's
        // side effects - like a memory write - actually happen). Debug
        // tooling that wants "which instruction just wrote this value"
        // (see Dma.cs's LogSourceAddrWrite) needs these, not the live PC/PB
        // - using the live ones was a real bug caught mid-investigation:
        // every DMA source-address write was showing the SAME next-
        // instruction bytes regardless of which actual STA had just fired,
        // because by the time the write's side effect ran, PC had already
        // moved on.
        public ushort LastInstructionPC;
        public byte LastInstructionPB;
        
        public byte P;    
        public bool E;    

        // WAI/STP state - see OpWAI/OpSTP in Cpu.Opcodes.cs for how these get
        // set, and Step()/Nmi()/Irq() below for how they get checked/cleared.
        private bool _waitingForInterrupt;
        private bool _stopped;

        public Cpu(MemoryBus bus)
        {
            _bus = bus;
            BuildOpcodeTable(); 
            Reset();
        }

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
            if (_stopped) return 3;
            if (_waitingForInterrupt) return 2;

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
                Console.WriteLine($"[CPU] 0x{executedAtPB:X2}{executedAtPC:X4}: {inst.Name} (Opcode 0x{opcode:X2}) -> Target Addr: 0x{targetAddr:X6}");
                if (DebugSettings.CpuTraceCountdown > 0)
                {
                    DebugSettings.CpuTraceCountdown--;
                    if (DebugSettings.CpuTraceCountdown == 0) DebugSettings.CpuVerboseLogging = false;
                }
            }
            
            // Return the cycles consumed by this instruction
            // Note: In a fully accurate emulator, we would multiply this by 6, 8, or 12 
            // depending on the memory region, but for now, base cycles are fine.
            return inst.Cycles; 
        }

        // Runs the real 65816 NMI entry sequence: push PB (native mode only), PC, and P,
        // clear D, set I, then jump to the NMI vector ($FFEA native / $FFFA emulation).
        // The handler is expected to end with RTI. This is triggered externally (from the
        // main loop) when a simulated vblank occurs and NMITIMEN's enable bit is set.
        public void Nmi()
        {
            // NMI always wakes a WAI-halted CPU, same as it always services
            // regardless of the I flag (NMI is non-maskable).
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
                Console.WriteLine($"[CPU] NMI -> PC = 0x00{PC:X4}");
            }
        }

        // Runs the real 65816 IRQ entry sequence - identical structure to NMI, but
        // MASKABLE: it only actually fires if the I (interrupt disable) flag is
        // clear, matching real hardware where the CPU itself holds off IRQs while
        // that flag is set. Uses the IRQ vector, which is genuinely different from
        // NMI's in native mode ($FFEE vs $FFEA) but shares BRK's vector in
        // emulation mode ($FFFE) - a real, documented 6502/65816 quirk, not a typo.
        // Returns whether the interrupt actually fired, so the caller (which decides
        // WHEN to attempt this, based on H/V-IRQ timer conditions) knows whether to
        // also acknowledge/clear the pending condition.
        public bool Irq()
        {
            // WAI wakes on ANY interrupt condition, masked or not - only
            // whether it's actually SERVICED (jumps to the vector) respects
            // the I flag. So this clears _waitingForInterrupt unconditionally,
            // before the mask check below.
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