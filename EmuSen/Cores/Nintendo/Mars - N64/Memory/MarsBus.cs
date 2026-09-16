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
        public readonly byte[] SpDmem = new byte[MemoryMap.SpMemSize];
        public readonly byte[] SpImem = new byte[MemoryMap.SpMemSize];
        public readonly byte[] PifRam = new byte[MemoryMap.PifRamSize];

        public readonly IsViewer IsViewer = new();

        public readonly SpInterface Sp;
        public readonly PiInterface Pi;

        public RomImage? Cart;

        // One counter for the whole machine, so no instruction can forget to advance it - see §3.
        public long Cycles;

        private long _countBias;

        // Stubs until each device exists; a register nobody models still has to read back - see §2.2.
        private readonly Dictionary<uint, uint> _registers = new();

        public MarsBus(bool expansionPak = false)
        {
            Rdram = new byte[expansionPak ? RdramSizeExpanded : RdramSize];
            Sp = new SpInterface(this);
            Pi = new PiInterface(this);

            // Nonzero tells libdragon's IPL3 that RDRAM needs no initialising - see Mars_TestOracle.md §3.
            _registers[MemoryMap.RiSelect] = 1;
        }

        public void Tick(long cycles) => Cycles += cycles;

        // Derived rather than incremented: half the CPU clock, off the one counter - see §3.1.
        public uint Count => unchecked((uint)((Cycles >> 1) + _countBias));

        public void SetCount(uint value) => _countBias = unchecked(value - (Cycles >> 1));

        public uint Read32(uint physical)
        {
            if (physical < Rdram.Length) return ReadArray32(Rdram, physical);

            // Above the installed size reads zero rather than mirroring, which IPL3 depends on - see §2.1.
            if (physical < RdramSizeExpanded) return 0;

            if (InRange(physical, MemoryMap.SpDmemBase, MemoryMap.SpMemSize))
            {
                return ReadArray32(SpDmem, physical - MemoryMap.SpDmemBase);
            }

            if (InRange(physical, MemoryMap.SpImemBase, MemoryMap.SpMemSize))
            {
                return ReadArray32(SpImem, physical - MemoryMap.SpImemBase);
            }

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

            if (InRange(physical, MemoryMap.PiBase, 0x34)) return Pi.Read32(physical - MemoryMap.PiBase);

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

            if (InRange(physical, MemoryMap.SpDmemBase, MemoryMap.SpMemSize))
            {
                WriteArray32(SpDmem, physical - MemoryMap.SpDmemBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.SpImemBase, MemoryMap.SpMemSize))
            {
                WriteArray32(SpImem, physical - MemoryMap.SpImemBase, value);
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

            if (InRange(physical, MemoryMap.SpRegistersBase, 0x20))
            {
                Sp.Write32(physical - MemoryMap.SpRegistersBase, value);
                return;
            }

            if (InRange(physical, MemoryMap.PiBase, 0x34))
            {
                Pi.Write32(physical - MemoryMap.PiBase, value);
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
