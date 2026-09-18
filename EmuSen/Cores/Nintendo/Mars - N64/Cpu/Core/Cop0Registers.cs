using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The COP0 file is not thirty-two words of storage: see Mars_Cop0.md.
    public sealed partial class Cpu
    {
        public const int ProcessorIdRegister = 15;
        public const int ConfigRegister = 16;
        public const int XContextRegister = 20;
        public const int ParityErrorRegister = 26;
        public const int CacheErrorRegister = 27;
        public const int TagLoRegister = 28;
        public const int TagHiRegister = 29;

        // What this part answers when asked who it is, and it cannot be written - see Mars_Cop0.md §3.
        public const uint ProcessorId = 0x0000_0B22;

        // Bits 30:28 read as ones whatever is written, which the corpus's own Config test shows - see Mars_Cop0.md §4.
        public const uint ConfigWritable = 0x0F00_800F;
        public const uint ConfigConstant = 0x7006_6460;

        // The TLB's own registers, each narrower than its word - see Mars_Tlb.md §7.1.
        public const ulong EntryLoWritable = 0x3FFF_FFFF;
        public const ulong PageMaskWritable = 0x01FF_E000;
        public const ulong EntryHiWritable = 0xC000_00FF_FFFF_E0FF;

        // The value a read-modify-write of Config depends on finding there - see Mars_Cop0.md §4.
        public const uint ConfigAtReset = 0x7006_E463;

        // Hardware keeps the low bits of these two for itself; software owns the rest - see §2.
        public const ulong ContextWritable = 0xFFFF_FFFF_FF80_0000;
        public const ulong XContextWritable = 0xFFFF_FFFE_0000_0000;

        // The fields a fault fills in, which is the half of these two that software does not own - see §7.
        public const ulong ContextBadVpn2 = 0x007F_FFF0;
        public const ulong XContextBadVpn2 = 0x7FFF_FFF0;
        public const ulong XContextRegion = 0x1_8000_0000;

        // Not storage at all: they read back the last value any COP0 write put on the bus - see §5.
        private ulong _cop0Latch;

        // Where Random last started counting down from 31, which a write to Wired resets - see Mars_Tlb.md §7.4.
        private long _randomStart;

        public static bool IsUnusedCop0Register(int register) =>
            register is 7 or 21 or 22 or 23 or 24 or 25 or 31;

        // Count comes off the machine clock rather than out of storage - see Mars_Memory.md §3.1.
        private ulong ReadCop0(int register)
        {
            if (IsUnusedCop0Register(register)) return _cop0Latch;

            return register switch
            {
                CountRegister => _bus.Count,
                RandomRegister => ReadRandom(),
                _ => Cop0[register],
            };
        }

        // Counts down from 31 to Wired and starts again; above 31 it runs through the whole six bits - see §7.4.
        private ulong ReadRandom()
        {
            ulong wired = Cop0[WiredRegister] & 0x3F;
            ulong period = wired <= 31 ? 32 - wired : 96 - wired;
            ulong elapsed = (ulong)(Instructions - _randomStart) % period;

            return unchecked(31 - elapsed) & 0x3F;
        }

        private void WriteCop0(int register, ulong value)
        {
            _cop0Latch = value;

            if (IsUnusedCop0Register(register)) return;

            // Status and Cause feed the interrupt check, and the others cost nothing to recheck - see Mars_Performance.md §10.
            _recheck = true;

            if (register == CountRegister)
            {
                _bus.SetCount((uint)value);
                _lastCount = _bus.Count;
                ScheduleTimer();
                return;
            }

            Cop0[register] = MaskCop0Write(register, value);
            if (register == StatusRegister) RefreshMode();

            if (register == WiredRegister) _randomStart = Instructions;

            // Writing the comparison value is how a handler acknowledges the timer - see Mars_Cpu.md §12.1.
            if (register == CompareRegister)
            {
                Cop0[CauseRegister] &= ~CauseInterruptTimer;
                _lastCount = _bus.Count;
                ScheduleTimer();
            }
        }

        // Narrower than a word, read-only, constant, or half owned by hardware - see Mars_Cop0.md §2.
        private ulong MaskCop0Write(int register, ulong value) => register switch
        {
            IndexRegister => value & 0x8000_003F,
            RandomRegister => Cop0[RandomRegister],
            EntryLo0Register or EntryLo1Register => value & EntryLoWritable,
            PageMaskRegister => value & PageMaskWritable,
            EntryHiRegister => value & EntryHiWritable,
            ContextRegister => (value & ContextWritable) | (Cop0[ContextRegister] & ~ContextWritable),
            WiredRegister => value & 0x3F,
            BadVirtualAddressRegister => Cop0[BadVirtualAddressRegister],
            StatusRegister => value & 0xFFF7_FFFF,
            ProcessorIdRegister => ProcessorId,
            ConfigRegister => (value & ConfigWritable) | ConfigConstant,
            LinkedAddressRegister => value & 0xFFFF_FFFF,
            XContextRegister => (value & XContextWritable) | (Cop0[XContextRegister] & ~XContextWritable),
            ParityErrorRegister => value & 0xFF,
            CacheErrorRegister => 0,
            TagLoRegister => value & 0xFFFF_FFFF,
            TagHiRegister => 0,
            _ => value,
        };
    }
}
