using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Common;
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

        // The display processor's page marks, the writer's and the reader's, read before every word the bus writes or reads in RDRAM - see Mars_Rdp.md §2.6.1.
        [EmuSen.Common.SkipInState] private readonly long[] _dpMarks;
        [EmuSen.Common.SkipInState] private readonly long[] _dpWriteMarks;
        public readonly SiInterface Si;
        public readonly AiInterface Ai;
        public readonly Vi.Vi Vi;
        public readonly MiInterface Mi = new();

        [EmuSen.Common.SkipInState] public RomImage? Cart;

        // Told of every processor store to a memory, for `watch` and data breakpoints - see Mars_Debug.md §3.
        [EmuSen.Common.SkipInState] public IWriteObserver? WriteObserver;

        // Consulted on every cartridge ROM read, by ROM offset, and never written back - see Mars_Cheats.md §6.
        [EmuSen.Common.SkipInState] public IRomReadPatcher? RomPatcher;

        // The cartridge's save chip on the second domain and the joybus's fifth channel - see Mars_Save.md §1.
        [EmuSen.Common.SkipInState] public SaveChip Save = new(N64SaveType.Unknown);

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

        // The earliest cycle the VI or AI has something to do; zero until the first tick asks them - see Mars_Performance.md §9.
        [EmuSen.Common.SkipInState] private long _nextEvent;

        internal long NextEvent => _nextEvent;

        // Counts every write to memory that is not the processor's own direct store, for the blocks - see Mars_Recompiler.md §4.
        [EmuSen.Common.SkipInState] public long Written;

        private long _countBias;

        // Stubs until each device exists; a register nobody models still has to read back - see §2.2.
        [EmuSen.Common.SkipInState] private readonly Dictionary<uint, uint> _registers = new();

        public MemoryBus(bool expansionPak = false)
        {
            Rdram = new byte[expansionPak ? RdramSizeExpanded : RdramSize];
            RdramHidden = new byte[Rdram.Length / 2];
            Sp = new SpInterface(this);
            Pi = new PiInterface(this);
            Dp = new DpInterface(this);
            _dpMarks = Dp.Marks;
            _dpWriteMarks = Dp.WriteMarks;
            Si = new SiInterface(this);
            Ai = new AiInterface(this);
            Vi = new Vi.Vi(this);

            // Nonzero tells libdragon's IPL3 that RDRAM needs no initialising - see Mars_TestOracle.md §3.
            _registers[MemoryMap.RiSelect] = 0x14;
        }

        // The reflected fields, then what reflection cannot walk: the stub registers, the save chip, each port's pak - see Mars_SaveStates.md §3.
        public void WriteState(BinaryWriter w) => WriteState(w, snapshot: false);

        // A snapshot holds the thread where it stands and carries the words it has not run; a state waits for it - see Mars_Rdp.md §2.7.
        public void WriteState(BinaryWriter w, bool snapshot)
        {
            if (snapshot) Dp.Hold();
            else Dp.Join();

            try
            {
                // Settled first, so a state holds what a device stepped every tick would - see Mars_Performance.md §9.
                Settle();
                Sp.Processor.WidenAccumulator();
                WriteStateBody(w);
                if (snapshot) Dp.WritePending(w);
            }
            finally
            {
                if (snapshot) Dp.Resume();
            }
        }

        private void WriteStateBody(BinaryWriter w)
        {
            StateSerializer.Write(w, this);

            w.Write(_registers.Count);
            foreach (var (address, value) in _registers.OrderBy(pair => pair.Key))
            {
                w.Write(address);
                w.Write(value);
            }

            Save.WriteState(w);

            foreach (Controller port in Si.Controllers)
            {
                w.Write(port.Pak != null);
                if (port.Pak != null) StateSerializer.Write(w, port.Pak);
            }
        }

        public void ReadState(BinaryReader r) => ReadState(r, snapshot: false);

        public void ReadState(BinaryReader r, bool snapshot)
        {
            Dp.Join();
            StateSerializer.Read(r, this);
            Sp.Processor.AccumulatorWritten();
            Written++;

            _registers.Clear();
            for (int count = r.ReadInt32(); count > 0; count--) _registers[r.ReadUInt32()] = r.ReadUInt32();

            Save.ReadState(r);

            foreach (Controller port in Si.Controllers)
            {
                if (!r.ReadBoolean())
                {
                    port.Pak = null;
                    continue;
                }

                port.Pak ??= new ControllerPak(null);
                StateSerializer.Read(r, port.Pak);
                port.Pak.Dirty = true;
            }

            Ai.DropUndrained();

            Vi.Rebase();
            Ai.Rebase();
            Reschedule();
            Dp.Processor.Refresh();
            if (snapshot) Dp.ReadPending(r);
            Dp.RefreshShadow();
        }

        // The RSP runs in step with the processor; the VI and AI act only when one of them is due - see Mars_Performance.md §9.
        public void Tick(long cycles)
        {
            Cycles += cycles;
            if (!Sp.Processor.Halted) Sp.Step(cycles);
            if (Cycles >= _nextEvent) RunEvents();
        }

        // In the order a tick once stepped them, the VI's half lines before the AI's samples - see Mars_Performance.md §9.
        internal void RunEvents()
        {
            Vi.Catch();
            Ai.Catch();
            _nextEvent = Math.Min(Vi.Due, Ai.Due);
        }

        // Before a write changes what either clock runs at, both are brought up to now at the old rate - see Mars_Performance.md §9.
        public void Settle()
        {
            Vi.Settle();
            Ai.Settle();
        }

        public void Reschedule()
        {
            Vi.Schedule();
            Ai.Schedule();
            _nextEvent = Math.Min(Vi.Due, Ai.Due);
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
            if (physical < Rdram.Length)
            {
                if (_dpWriteMarks[physical >> 12] != 0) Dp.WaitForRead(physical, 0);
                return ReadArray32(Rdram, physical);
            }

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

            if (SaveWindow(physical)) return Save.Read32(physical - MemoryMap.CartDomain2Address2);

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
            Written++;

            if (physical < Rdram.Length)
            {
                if (_dpMarks[physical >> 12] != 0) Dp.WaitFor(physical, 1);
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

            if (SaveWindow(physical))
            {
                Save.Write32(physical - MemoryMap.CartDomain2Address2, value);
                return;
            }

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

        // Three windows take a whole word from the processor whatever size it names: the signal processor's memories, PIF RAM and the save chip - see §2.4.
        private bool LatchesWholeWords(uint physical) =>
            SignalProcessorMemory(physical, out _, out _) || InRange(physical, MemoryMap.PifRamBase, MemoryMap.PifRamSize) || SaveWindow(physical);

        // The processor's own stores: those windows latch whole words, and the cartridge bus keeps one - see §2.4, §7.7 and Mars_Save.md §3.
        public void Store(uint physical, ulong value, int size)
        {
            StoreThrough(physical, value, size);
            if (StoresWatched) Report(physical, size);
        }

        // Reporting a store costs about as much as the instruction, so it is done only while something listens - see Mars_Performance.md §16.
        public bool StoresWatched => WriteObserver is { Listening: true };

        // What a store left behind, byte by byte in the space it landed in - see Mars_Debug.md §3.
        private void Report(uint physical, int size)
        {
            bool whole = LatchesWholeWords(physical);
            uint first = whole && size < 8 ? physical & ~3u : physical;
            int count = whole && size < 8 ? 4 : size;

            for (uint at = first; at < first + count; at++)
            {
                if (at < Rdram.Length) WriteObserver!.OnWrite("RDRAM", (int)at, Rdram[at]);
                else if (SignalProcessorMemory(at, out byte[] bank, out uint offset)) WriteObserver!.OnWrite(bank == SpDmem ? "DMEM" : "IMEM", (int)offset, bank[offset]);
                else if (InRange(at, MemoryMap.PifRamBase, MemoryMap.PifRamSize)) WriteObserver!.OnWrite("PIFRAM", (int)(at - MemoryMap.PifRamBase), PifRam[at - MemoryMap.PifRamBase]);
            }
        }

        private void StoreThrough(uint physical, ulong value, int size)
        {
            if (CartridgeBus(physical)) Pi.CartridgeStore(WholeWord(physical, value, size));

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
            physical >= MemoryMap.CartDomain1Address2 && CartridgeBus(physical);

        // Every store on the cartridge's bus is kept, though only the ROM window hands it back - see Mars_Save.md §3.
        private static bool CartridgeBus(uint physical) =>
            physical >= MemoryMap.CartDomain2Address1 && physical < MemoryMap.PifRomBase &&
            !InRange(physical, MemoryMap.IsViewerBase, MemoryMap.IsViewerSize);

        private static bool SaveWindow(uint physical) =>
            physical >= MemoryMap.CartDomain2Address2 && physical < MemoryMap.CartDomain1Address2;

        // A transfer reaches the save chip by a path of its own, which FlashRAM answers differently - see Mars_Save.md §4.
        public byte CartridgeDmaRead8(uint physical) =>
            SaveWindow(physical) ? Save.DmaRead8(physical - MemoryMap.CartDomain2Address2) : Read8(physical);

        public void CartridgeDmaWrite8(uint physical, byte value)
        {
            if (SaveWindow(physical)) Save.DmaWrite8(physical - MemoryMap.CartDomain2Address2, value);
            else Write8(physical, value);
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

            uint word = ReadArray32(rom, offset);
            return RomPatcher is null ? word : Patched(RomPatcher, word, offset);
        }

        // Each byte asked for by its own offset, so a patch lands whichever lane or transfer reaches it - see Mars_Cheats.md §6.
        private static uint Patched(IRomReadPatcher patcher, uint word, uint offset)
        {
            for (int lane = 0; lane < 4; lane++)
            {
                int shift = 24 - 8 * lane;
                if (patcher.TryPatch(offset + (uint)lane, (byte)(word >> shift), out byte patched))
                {
                    word = (word & ~(0xFFu << shift)) | ((uint)patched << shift);
                }
            }

            return word;
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

        // One big-endian word rather than four byte accesses, over the same bytes - see Mars_Performance.md §17.
        private static uint ReadArray32(byte[] memory, uint offset) =>
            BinaryPrimitives.ReadUInt32BigEndian(memory.AsSpan((int)offset));

        private static void WriteArray32(byte[] memory, uint offset, uint value) =>
            BinaryPrimitives.WriteUInt32BigEndian(memory.AsSpan((int)offset), value);
    }
}
