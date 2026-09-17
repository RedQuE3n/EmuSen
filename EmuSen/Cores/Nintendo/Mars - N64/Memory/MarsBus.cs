using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars.Rom;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // Everything the VR4300 can reach, and the clock that advances when it does - see Mars_Memory.md §2.
    public sealed class MarsBus
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
        public readonly MiInterface Mi = new();

        public RomImage? Cart;

        // One counter for the whole machine, so no instruction can forget to advance it - see §3.
        public long Cycles;

        private long _countBias;

        // Stubs until each device exists; a register nobody models still has to read back - see §2.2.
        private readonly Dictionary<uint, uint> _registers = new();

        public MarsBus(bool expansionPak = false)
        {
            Rdram = new byte[expansionPak ? RdramSizeExpanded : RdramSize];
            RdramHidden = new byte[Rdram.Length / 2];
            Sp = new SpInterface(this);
            Pi = new PiInterface(this);
            Dp = new DpInterface(this);

            // Nonzero tells libdragon's IPL3 that RDRAM needs no initialising - see Mars_TestOracle.md §3.
            _registers[MemoryMap.RiSelect] = 0x14;
        }

        public void Tick(long cycles)
        {
            Cycles += cycles;
            Sp.Step(cycles);
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

        // The processor's own stores: signal processor memory latches whole words, whatever the size - see §2.4.
        public void Store(uint physical, ulong value, int size)
        {
            if (SignalProcessorMemory(physical, out _, out _))
            {
                if (size == 8) Write32(physical, (uint)(value >> 32));
                else Write32(physical & ~3u, (uint)(value << (8 * (4 - size - (int)(physical & 3)))));
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
