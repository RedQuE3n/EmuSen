using System.Collections.Generic;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Named byte-addressable regions, deliberately not SNES nouns - see EmuSen_Debugging_Tools_Reference_v5.md §3.1.
    public interface IDebugMemorySpace
    {
        string Name { get; }
        int Size { get; }
        bool IsWritable { get; }
        byte Read(int address);
        void Write(int address, byte value);

        // True where a read disturbs live state, so `search` can refuse the space - see §3.1a.
        bool HasSideEffects { get; }
    }

    // Where a vector's POINTER lives, not where it points - see `man vectors`.
    public readonly struct InterruptVector
    {
        public string Name { get; }
        public int Address { get; }
        public int Width { get; }

        // Which CPU mode this vector belongs to, blank when a core has only one.
        public string Mode { get; }

        public InterruptVector(string name, int address, int width = 2, string mode = "")
        {
            Name = name;
            Address = address;
            Width = width;
            Mode = mode;
        }
    }

    // What a CPU-bus address actually decodes to - see `man addr`.
    public readonly struct PhysicalAddress
    {
        // A memory-space name where one exists, else a description of the device.
        public string Space { get; }
        public int Offset { get; }

        // True when Offset indexes a real space `mem`/`dump` can be pointed at.
        public bool IsAddressable { get; }

        public PhysicalAddress(string space, int offset, bool isAddressable = true)
        {
            Space = space;
            Offset = offset;
            IsAddressable = isAddressable;
        }
    }

    // A hardware condition a core can detect and halt on - see `man bp`.
    public readonly struct BreakCondition
    {
        public string Name { get; }
        public string Description { get; }

        public BreakCondition(string name, string description)
        {
            Name = name;
            Description = description;
        }
    }

    // One block-transfer channel, in the terms a channel table prints - see `man dma`.
    public readonly struct DebugDmaChannel
    {
        public int Index { get; }
        public bool GeneralEnabled { get; }
        public bool HdmaEnabled { get; }

        // The mode/direction byte, left raw so a core's own decoder names it.
        public byte Control { get; }
        public byte DestinationRegister { get; }
        public int SourceAddress { get; }
        public int TransferSize { get; }

        // HDMA only: the table it walks, and where it has got to.
        public bool HdmaActive { get; }
        public int TableAddress { get; }
        public byte LineCounter { get; }
        public int IndirectAddress { get; }

        public DebugDmaChannel(int index, bool generalEnabled, bool hdmaEnabled, byte control,
            byte destinationRegister, int sourceAddress, int transferSize,
            bool hdmaActive, int tableAddress, byte lineCounter, int indirectAddress)
        {
            Index = index;
            GeneralEnabled = generalEnabled;
            HdmaEnabled = hdmaEnabled;
            Control = control;
            DestinationRegister = destinationRegister;
            SourceAddress = sourceAddress;
            TransferSize = transferSize;
            HdmaActive = hdmaActive;
            TableAddress = tableAddress;
            LineCounter = lineCounter;
            IndirectAddress = indirectAddress;
        }
    }

    // One already-formatted instruction; the viewer needs no ISA knowledge - see §3.1.
    public readonly struct DisassembledInstruction
    {
        public int Address { get; }
        public IReadOnlyList<byte> Bytes { get; }
        public string Mnemonic { get; }
        public string OperandText { get; }

        public DisassembledInstruction(int address, IReadOnlyList<byte> bytes, string mnemonic, string operandText)
        {
            Address = address;
            Bytes = bytes;
            Mnemonic = mnemonic;
            OperandText = operandText;
        }

        public int Length => Bytes.Count;
    }

    // Named after the commands each kind backs, not after any one CPU's mnemonics - see §3.1.
    public enum StaticReferenceKind { Call, Write, Read }

    // The contract a core implements; the read surface lives on ICoreTelemetry - see EmuSen_Cauldron.md §3.1.
    public interface IDebugTarget : ICoreTelemetry
    {
        IReadOnlyList<IDebugMemorySpace> GetMemorySpaces();

        // The watchpoint registry, exposed directly because it is already core-agnostic - see §3.5.
        WatchRegistry Watches { get; }

        // Frame-scoped value logging, same exposure pattern as Watches - see §3.13.
        FrameLogRegistry FrameLog { get; }

        // Control flow reaching an address, which Watches cannot see - see §3.26.
        BreakpointRegistry Breakpoints { get; }

        // Separate from Breakpoints: the two run different code at the same addresses - see §3.1a.
        BreakpointRegistry? CoprocessorBreakpoints => null;

        // Null when a core has no coverage hook - see §3.24.
        CoverageRegistry? Coverage => null;

        // Separate from Coverage for the same reason as the breakpoints above - see §3.1a.
        CoverageRegistry? CoprocessorCoverage => null;

        // Null when a core has no call/return seam wired - see §3.28.
        CallStackRegistry? CallStack => null;

        // Owned by the target, so a label set lives as long as the ROM does - see §3.29.
        LabelRegistry? Labels => null;

        // Fed from the same observer seams WatchRegistry uses - see §3.30.
        AccessCounterRegistry? AccessCounters => null;

        // Null when a core has no write-observer seam to hang this on - see §3.31.
        FreezeRegistry? Freezes => null;

        // The context, not an evaluator: the language is core-agnostic - see §3.27.
        IExpressionContext? Expressions => null;

        // Every separately-steppable processor, main CPU first - see `man cpus`.
        IReadOnlyList<DebugCpu> DebugCpus => System.Array.Empty<DebugCpu>();

        // Traffic across a coprocessor's register window - see `man copflow`.
        RegisterFlowRegistry? RegisterFlow => null;

        // Where each interrupt vector lives, for `vectors` to dereference - see `man vectors`.
        IReadOnlyList<InterruptVector> InterruptVectors => System.Array.Empty<InterruptVector>();

        // Which physical location a CPU-bus address decodes to - see `man addr`.
        PhysicalAddress? ResolvePhysical(int cpuAddress) => null;

        // Which named hardware conditions `bp when` can arm on this core - see `man bp`.
        IReadOnlyList<BreakCondition> BreakConditions => System.Array.Empty<BreakCondition>();

        // Live block-transfer channel state - see `man dma`.
        IReadOnlyList<DebugDmaChannel> DmaChannels => System.Array.Empty<DebugDmaChannel>();

        // The recorded transfer history behind `dma log` - see `man dma`.
        DmaLogRegistry? DmaLog => null;

        // What a DMA destination register is called on this console - see `man dma`.
        string? NameDmaDestination(byte register) => null;

        // The one registry the core reads rather than feeds - see §3.1a.
        CheatRegistry Cheats { get; }

        // One method, not a disassembler object: addressing modes stay in the core - see §3.1a.
        IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count);

        // Opaque core-defined decoder state a caller already knows - see `man disasm`.
        IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count, IReadOnlyList<string> hints)
            => Disassemble(spaceName, address, count);

        // Null whenever the target is not knowable from the bytes alone - see §3.1a.
        (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr);

        // Lets `tilemap` compute a cell address without knowing the entry format.
        int TilemapEntryStride { get; }

        // Every core decides what one entry is; an NES nametable is not this shape - see §3.1a.
        string DecodeTilemapEntry(IDebugMemorySpace space, int address);

        // Planar here, but nothing guarantees a future core's tiles are - see §3.1a.
        byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp);

        // Playback state keeps advancing; only the mix excludes it - see §3.1a.
        void SetChannelMuted(int index, bool muted);

        // Defaulted: a core applying cheats on its own frame hook has nothing to do - see Moon_Debug.md §3.1a.
        void ApplyCheats() { }
    }
}
