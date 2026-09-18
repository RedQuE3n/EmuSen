using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars.Rom;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // Everything the VR4300 can reach, and the clock that advances when it does - see Mars_Memory.md §2.
    public sealed class MemoryBus
    {
        public const int RdramSize = 0x0040_0000;
        public const int RdramSizeExpanded = 0x0080_0000;

        public byte[] Rdram;

        // The display processor's extra bits beside each sixteen-bit word, which the CPU cannot see - see Mars_RdpCoverage.md §3.1.
        public byte[] RdramHidden;
        public readonly byte[] SpDmem = new byte[MemoryMap.SpMemSize];
        public readonly byte[] SpImem = new byte[MemoryMap.SpMemSize];
        public readonly byte[] PifRam = new byte[MemoryMap.PifRamSize];

        public readonly IsViewer IsViewer = new();

        public readonly SpInterface Sp;
        public readonly PiInterface Pi;
        public readonly DpInterface Dp;
        public readonly SiInterface Si;
        public readonly AiInterface Ai;
        public readonly Vi.Vi Vi;
        public readonly MiInterface Mi = new();

        public RomImage? Cart;

        // What the RDRAM registers read once IPL3 has set RDRAM up, as the corpus measured them, every 64 bytes - see Mars_Memory.md §8.5.
        private static readonly uint[] RdramRegisters =
        {
            0xB419_0010, 0, 0x2B3B_1A0B, 0, 0, 0, 0x101C_0A04, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        };

        // The MI's repeat stays inside one 2KB RDRAM row - see Mars_Memory.md §8.4.
        private const uint RepeatRowMask = 0x7FF;

        // One counter for the whole machine, so no instruction can forget to advance it - see §3.
        public long Cycles;

        private long _countBias;

        // Stubs until each device exists; a register nobody models still has to read back - see §2.2.
        private readonly Dictionary<uint, uint> _registers = new();

        public MemoryBus(bool expansionPak = false)
        {
            Rdram = new byte[expansionPak ? RdramSizeExpanded : RdramSize];
            RdramHidden = new byte[Rdram.Length / 2];
            Sp = new SpInterface(this);
            Pi = new PiInterface(this);
            Dp = new DpInterface(this);
            Si = new SiInterface(this);
            Ai = new AiInterface(this);
            Vi = new Vi.Vi(this);

            // Nonzero tells libdragon's IPL3 that RDRAM needs no initialising - see Mars_TestOracle.md §3.
            _registers[MemoryMap.RiSelect] = 0x14;
        }

        public void Tick(long cycles)
        {
            Cycles += cycles;
            Sp.Step(cycles);
            Vi.Step(cycles);
            Ai.Step(cycles);
        }

        // The eight interface registers and the eight the display processor owns - see Mars_Rsp.md §5.
        public uint ReadRspControl(int register) =>
            register < 8 ? Sp.Read32((uint)register << 2) : Read32(MemoryMap.DpCommandBase + ((uint)(register - 8) << 2));

        public void WriteRspControl(int register, uint value)
        {
            if (register < 8) Sp.Write32((uint)register << 2, value);
            else Write32(MemoryMap.DpCommandBase + ((uint)(register - 8) << 2), value);
        }

        // Derived rather than incremented: half the CPU clock, off the one counter - see §3.1.
        public uint Count => unchecked((uint)((Cycles >> 1) + _countBias));

        public void SetCount(uint value) => _countBias = unchecked(value - (Cycles >> 1));

        public uint Read32(uint physical)
        {
            if (physical < Rdram.Length) return ReadArray32(Rdram, physical);

            // Above the installed size reads zero rather than mirroring, which IPL3 depends on - see §2.1.
            if (physical < RdramSizeExpanded) return 0;

            if (InRange(physical, MemoryMap.RdramRegistersBase, MemoryMap.RdramRegistersSize)) return RdramRegisters[(physical >> 2) & 0xF];

            if (SignalProcessorMemory(physical, out byte[] bank, out uint offset)) return ReadArray32(bank, offset);

            if (InRange(physical, MemoryMap.IsViewerBase, MemoryMap.IsViewerSize))
            {
                return IsViewer.Read32(physical - MemoryMap.IsViewerBase);
            }

            if (InRange(physical, MemoryMap.PifRamBase, MemoryMap.PifRamSize))
            {
                return ReadArray32(PifRam, physical - MemoryMap.PifRamBase);
            }

            if (physical >= MemoryMap.CartDomain1Address2) return ReadCart32(physical - MemoryMap.CartDomain1Address2);

            if (InRange(physical, MemoryMap.SpRegistersBase, 0x20)) return Sp.Read32(physical - MemoryMap.SpRegistersBase);
            if (InRange(physical, MemoryMap.SpPcBase, 0x08)) return Sp.Pc;

            if (InRange(physical, MemoryMap.DpCommandBase, 0x20)) return Dp.Read32(physical - MemoryMap.DpCommandBase);

            if (InRange(physical, MemoryMap.PiBase, 0x34)) return Pi.Read32(physical - MemoryMap.PiBase);

            if (InRange(physical, MemoryMap.MiBase, 0x10)) return Mi.Read32(physical - MemoryMap.MiBase);

            if (InRange(physical, MemoryMap.ViBase, 0x38)) return Vi.Read32(physical - MemoryMap.ViBase);
            if (InRange(physical, MemoryMap.SiBase, 0x1C)) return Si.Read32(physical - MemoryMap.SiBase);
            if (InRange(physical, MemoryMap.AiBase, 0x18)) return Ai.Read32(physical - MemoryMap.AiBase);

            return _registers.TryGetValue(physical, out uint value) ? value : 0;
        }

        public void Write32(uint physical, uint value)
        {
            if (physical < Rdram.Length)
            {
                WriteArray32(Rdram, physical, value);
                return;
            }

            if (physical < RdramSizeExpanded) return;

            // Nothing measured says a write is kept, and nothing Mars runs initialises RDRAM - see Mars_Memory.md §8.5.
            if (InRange(physical, MemoryMap.RdramRegistersBase, MemoryMap.RdramRegistersSize)) return;

            if (SignalProcessorMemory(physical, out byte[] bank, out uint offset))
            {
                WriteArray32(bank, offset, value);
                return;
            }

            if (InRange(physical, MemoryMap.IsViewerBase, MemoryMap.IsViewerSize))
            {
                IsViewer.Write32(physical - MemoryMap.IsViewerBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.PifRamBase, MemoryMap.PifRamSize))
            {
                WriteArray32(PifRam, physical - MemoryMap.PifRamBase, value);
                return;
            }

            // The cartridge is read-only here; a write to it is dropped rather than refused - see §2.3.
            if (physical >= MemoryMap.CartDomain1Address2) return;

            if (InRange(physical, MemoryMap.SpPcBase, 0x08))
            {
                Sp.Pc = value;
                return;
            }

            if (InRange(physical, MemoryMap.SpRegistersBase, 0x20))
            {
                Sp.Write32(physical - MemoryMap.SpRegistersBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.DpCommandBase, 0x20))
            {
                Dp.Write32(physical - MemoryMap.DpCommandBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.PiBase, 0x34))
            {
                Pi.Write32(physical - MemoryMap.PiBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.MiBase, 0x10))
            {
                Mi.Write32(physical - MemoryMap.MiBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.ViBase, 0x38))
            {
                Vi.Write32(physical - MemoryMap.ViBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.SiBase, 0x1C))
            {
                Si.Write32(physical - MemoryMap.SiBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.AiBase, 0x18))
            {
                Ai.Write32(physical - MemoryMap.AiBase, value);
                return;
            }

            _registers[physical] = value;
        }

        public byte Read8(uint physical)
        {
            uint word = Read32(physical & ~3u);
            return (byte)(word >> (int)((3 - (physical & 3)) * 8));
        }

        public void Write8(uint physical, byte value)
        {
            uint aligned = physical & ~3u;
            int shift = (int)((3 - (physical & 3)) * 8);
            uint word = Read32(aligned);
            Write32(aligned, (word & ~(0xFFu << shift)) | ((uint)value << shift));
        }

        public ushort Read16(uint physical)
        {
            uint word = Read32(physical & ~3u);
            return (ushort)(word >> (int)((2 - (physical & 2)) * 8));
        }

        public void Write16(uint physical, ushort value)
        {
            uint aligned = physical & ~3u;
            int shift = (int)((2 - (physical & 2)) * 8);
            uint word = Read32(aligned);
            Write32(aligned, (word & ~(0xFFFFu << shift)) | ((uint)value << shift));
        }

        public ulong Read64(uint physical) => ((ulong)Read32(physical) << 32) | Read32(physical + 4);

        // Two windows latch a whole word from the processor whatever size it names: the signal processor's memories and PIF RAM - see §2.4.
        private bool LatchesWholeWords(uint physical) =>
            SignalProcessorMemory(physical, out _, out _) || InRange(physical, MemoryMap.PifRamBase, MemoryMap.PifRamSize);

        // The processor's own stores: those two windows latch whole words, and so does the cartridge bus - see §2.4 and §7.7.
        public void Store(uint physical, ulong value, int size)
        {
            if (CartridgeRom(physical)) Pi.CartridgeStore(WholeWord(physical, value, size));

            // Any store to RDRAM or its registers spends the MI's repeat; only the first kind repeats - see §8.4.
            if (physical < MemoryMap.RdramRegistersBase + MemoryMap.RdramRegistersSize && Mi.Repeating)
            {
                Mi.Repeating = false;

                if (physical < MemoryMap.RdramRegistersBase)
                {
                    Repeat(physical, size == 8 ? value : Doubled(WholeWord(physical, value, size)));
                    return;
                }
            }

            if (LatchesWholeWords(physical))
            {
                Write32(size == 8 ? physical : physical & ~3u, WholeWord(physical, value, size));
                return;
            }

            switch (size)
            {
                case 1: Write8(physical, (byte)value); return;
                case 2: Write16(physical, (ushort)value); return;
                case 4: Write32(physical, (uint)value); return;
                default: Write64(physical, value); return;
            }
        }

        // The processor's own loads, the only reads that see what a store left on the cartridge bus - see §7.7.
        public ulong Load(uint physical, int size)
        {
            if (CartridgeRom(physical) && size < 8) return Lane(CartridgeWord(physical, size), physical, size);

            return size switch
            {
                1 => Read8(physical),
                2 => Read16(physical),
                4 => Read32(physical),
                _ => Read64(physical),
            };
        }

        // A stored word comes back once; otherwise the word starts at the halfword named, which skips every other one - see §7.8.
        private uint CartridgeWord(uint physical, int size)
        {
            if (Pi.TakeStored(out uint stored)) return stored;

            uint aligned = physical & ~3u;
            uint word = Read32(aligned);
            return size < 4 && (physical & 2) != 0 ? (word << 16) | (Read32(aligned + 4) >> 16) : word;
        }

        // The length counts from the store's doubleword, and each byte takes its own lane of what was on the bus - see §8.4.
        private void Repeat(uint physical, ulong pattern)
        {
            uint doubleword = physical & ~7u;

            for (uint at = physical & 7; at < Mi.RepeatLength; at++)
            {
                uint address = (doubleword & ~RepeatRowMask) | ((doubleword + at) & RepeatRowMask);
                Write8(address, (byte)(pattern >> (int)(8 * (7 - (address & 7)))));
            }
        }

        private static ulong Doubled(uint word) => ((ulong)word << 32) | word;

        private static ulong Lane(uint word, uint physical, int size) => size switch
        {
            1 => (byte)(word >> (int)((3 - (physical & 3)) * 8)),
            2 => (ushort)(word >> (int)((2 - (physical & 2)) * 8)),
            _ => word,
        };

        // What the processor puts on the bus for a store of any size: the register shifted to its lane, or a doubleword's upper half - see §2.4.
        private static uint WholeWord(uint physical, ulong value, int size) =>
            size == 8 ? (uint)(value >> 32) : (uint)(value << (8 * (4 - size - (int)(physical & 3))));

        // The debug port is a device beside the bus here, or the corpus's own printing would hold the latch it tests - see §7.7.
        private static bool CartridgeRom(uint physical) =>
            physical >= MemoryMap.CartDomain1Address2 && physical < MemoryMap.PifRomBase &&
            !InRange(physical, MemoryMap.IsViewerBase, MemoryMap.IsViewerSize);

        public void Write64(uint physical, ulong value)
        {
            Write32(physical, (uint)(value >> 32));
            Write32(physical + 4, (uint)value);
        }

        // Past the end of the cartridge is zero here; what hardware returns is unmodelled - see §2.3.
        private uint ReadCart32(uint offset)
        {
            byte[]? rom = Cart?.Rom;
            if (rom is null || offset + 3 >= rom.Length) return 0;

            return ReadArray32(rom, offset);
        }

        private bool SignalProcessorMemory(uint physical, out byte[] bank, out uint offset)
        {
            uint local = physical - MemoryMap.SpDmemBase;

            bank = local % (2 * MemoryMap.SpMemSize) < MemoryMap.SpMemSize ? SpDmem : SpImem;
            offset = local % MemoryMap.SpMemSize;

            return physical >= MemoryMap.SpDmemBase && local < MemoryMap.SpMemWindow;
        }

        private static bool InRange(uint address, uint start, uint length) =>
            address >= start && address < start + length;

        private static uint ReadArray32(byte[] memory, uint offset) =>
            (uint)((memory[offset] << 24) | (memory[offset + 1] << 16) | (memory[offset + 2] << 8) | memory[offset + 3]);

        private static void WriteArray32(byte[] memory, uint offset, uint value)
        {
            memory[offset] = (byte)(value >> 24);
            memory[offset + 1] = (byte)(value >> 16);
            memory[offset + 2] = (byte)(value >> 8);
            memory[offset + 3] = (byte)value;
        }
    }
}
