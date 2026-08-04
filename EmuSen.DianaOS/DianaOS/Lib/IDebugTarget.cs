using System.Collections.Generic;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // A single named, byte-addressable region a memory viewer can read (and
    // sometimes write). Deliberately NOT modeled around SNES-specific names
    // like "VRAM"/"CGRAM" at the interface level - those are just strings a
    // given core happens to report. A generic memory-viewer UI lists
    // whatever GetMemorySpaces() returns and doesn't need to know what kind
    // of console it's looking at. An NES core would report a completely
    // different set ("CPU Bus", "PPU Nametables", "OAM", "Palette") through
    // the exact same shape.
    public interface IDebugMemorySpace
    {
        string Name { get; }
        int Size { get; }
        bool IsWritable { get; }
        byte Read(int address);
        void Write(int address, byte value);

        // True if Read() can have a real side effect beyond returning a
        // byte - a space routed through a live hardware bus (registers
        // like the SNES's RDNMI, which clears the pending-NMI flag on
        // read, or OPHCT/OPVCT, which toggle a byte-order latch on read)
        // rather than a plain in-memory array. Exists specifically so a
        // bulk, read-every-address tool (see the `search` command) can
        // refuse to scan a space where doing so would silently disturb
        // running emulation state, instead of assuming every Read() call
        // is free of consequences the way it would be for WRAM/VRAM/etc.
        // A core with no such live-register space can just return false
        // everywhere.
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

    // One disassembled instruction, already fully formatted - a generic
    // disassembly viewer just prints Address/Bytes/Mnemonic/OperandText for
    // each entry without needing to know anything about the source CPU's
    // addressing modes or syntax conventions. Bytes lets a viewer show the
    // raw hex alongside the mnemonic, the way every real disassembler does.
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

    // Which kind of static code-to-address reference an instruction makes -
    // see IDebugTarget.ClassifyStaticReference for the full picture. Named
    // after the shell commands each kind backs (`callers`/`writers`/
    // `readers`), not after any one CPU's own mnemonics, since a future
    // core's equivalent "this is a call"/"this stores to X"/"this loads
    // from X" instructions won't share the 65816's mnemonic names either.
    public enum StaticReferenceKind { Call, Write, Read }

    // The contract a console core implements to plug into the shared debug
    // toolchain. Started out scoped to read/query capabilities only, with
    // breakpoints and single-stepping called out as a future extension
    // pending the execution loop supporting a mid-frame pause - that's now
    // done (see Breakpoints below and VenusCore.RunFrame()'s halt/resume
    // handling), added the same additive way this comment originally
    // anticipated: a new property, no breaking change to what was already
    // here.
    // The read surface lives on ICoreTelemetry - see EmuSen_Cauldron.md §3.1 for what stayed here and why.
    public interface IDebugTarget : ICoreTelemetry
    {
        IReadOnlyList<IDebugMemorySpace> GetMemorySpaces();

        // The watchpoint mechanism (see WatchRegistry.cs) - exposed
        // directly rather than re-wrapped into more IDebugTarget methods,
        // since WatchRegistry is already core-agnostic on its own. Every
        // core's implementation owns and returns the same instance its
        // memory-bus write path actually reports writes to.
        WatchRegistry Watches { get; }

        // The frame-scoped value logging mechanism (see
        // FrameLogRegistry.cs) - same exposure pattern as Watches above.
        // Every core's implementation owns the same instance its
        // per-frame boundary hook feeds.
        FrameLogRegistry FrameLog { get; }

        // The execution-breakpoint mechanism (see BreakpointRegistry.cs) -
        // same exposure pattern as Watches/FrameLog. Every core's
        // implementation owns the same instance its own instruction-step
        // loop consults before executing each instruction. Unlike Watches
        // (which only sees memory access), this catches control flow
        // reaching an address at all, including register-only conditions
        // (a CMP/branch pair) that never touch memory.
        BreakpointRegistry Breakpoints { get; }

        // Breakpoints on a cartridge coprocessor's own CPU, kept separate from
        // Breakpoints above because the two run different code at the same
        // addresses - an SA-1 game's $00:82D7 is not the S-CPU's $00:82D7.
        // Null when this core has no separately-steppable coprocessor, which
        // is the common case - see Venus_SA1.md §11.5.
        BreakpointRegistry? CoprocessorBreakpoints => null;

        // Whole-run execution coverage (see CoverageRegistry.cs), fed from the
        // same per-instruction seam as Breakpoints. Null when a core has no
        // coverage hook - see EmuSen_Debugging_Tools_Reference_v5.md §3.24.
        CoverageRegistry? Coverage => null;

        // Coverage of a coprocessor's own instruction stream, separate from
        // Coverage for the same reason CoprocessorBreakpoints is separate.
        CoverageRegistry? CoprocessorCoverage => null;

        // The live call/return chain (see CallStackRegistry.cs), fed from the
        // core's own call and return opcodes. Null when a core has no such
        // seam wired - see EmuSen_Debugging_Tools_Reference_v5.md §3.28.
        CallStackRegistry? CallStack => null;

        // Named addresses (see LabelRegistry.cs). Owned by the target so a
        // label set survives for as long as the loaded ROM does, the same
        // lifetime every other registry here has. Null means `label` is
        // unavailable for this core - see §3.29.
        LabelRegistry? Labels => null;

        // Per-address read/write/execute tallies (see
        // AccessCounterRegistry.cs), fed from the same observer seams
        // WatchRegistry uses. Null when a core has no such seam - see §3.30.
        AccessCounterRegistry? AccessCounters => null;

        // Addresses pinned to a value by undoing writes to them (see
        // FreezeRegistry.cs). Null when a core has no write-observer seam to
        // hang this on - see §3.31.
        FreezeRegistry? Freezes => null;

        // Live-state expression evaluation, backing `eval` and conditional
        // breakpoints (see ExpressionEvaluator.cs). Deliberately the context,
        // not an evaluator: the language is core-agnostic, only the symbols
        // and memory behind it are per-core. Null when a core hasn't
        // published one - see §3.27.
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

        // The RAM-poke cheat engine (see CheatRegistry.cs) - same exposure
        // pattern as Watches/FrameLog, but the data flow runs the other
        // direction: instead of the core feeding data into the registry,
        // the implementation re-applies every enabled cheat FROM this
        // registry INTO its own memory spaces once per frame (see
        // CheatRegistry.ApplyAll's own comment). Exposed here so it works
        // without the debug toolchain being otherwise active - a player
        // enabling a cheat shouldn't require opening the F4 prompt for
        // anything except adding the cheat itself.
        CheatRegistry Cheats { get; }

        // Disassembles <count> instructions starting at <address> in the
        // given memory space. Deliberately just this one method rather
        // than a richer disassembler object - the interface doesn't need
        // to know anything about a given CPU's addressing modes,
        // instruction lengths, or flag-dependent operand widths (the
        // 65816's M/X-flag-dependent immediate length being the obvious
        // SNES-specific example); all of that lives entirely inside each
        // core's implementation, which returns plain, already-formatted
        // DisassembledInstruction records. A target with no disassembler
        // at all can legitimately return an empty list.
        IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count);

        // Opaque core-defined decoder state a caller already knows - see `man disasm`.
        IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count, IReadOnlyList<string> hints)
            => Disassemble(spaceName, address, count);

        // Classifies whether a disassembled instruction statically
        // references an address - the code-only counterpart to
        // Disassemble() above, backing the `callers`/`writers`/`readers`
        // commands' "who could call/write/read this address" static scans.
        // Deliberately returns null for anything whose target ISN'T
        // knowable from the instruction bytes alone (a 65816 direct-page,
        // indexed, or indirect form, whose real address depends on runtime
        // register/D-register state) rather than guessing - same principle
        // Disassemble()'s own formatting already follows.
        //
        // Moved here from the shell layer specifically because it's pure
        // ISA knowledge (which opcodes are calls/stores/loads, and which of
        // *those* have a statically-resolvable operand) - exactly the same
        // "core does the decoding, the command does the scanning/matching"
        // split TilemapEntryStride/DecodeTilemapEntry and DecodeTilePixels
        // below already use, so `callers`/`writers`/`readers` don't need to
        // know anything about a specific CPU's opcode table. A target with
        // no meaningful concept of this (or that just hasn't implemented it
        // yet) can always return null for every instruction.
        (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr);

        // How many bytes one DecodeTilemapEntry() call consumes - 2 for
        // the SNES's packed BG screen word (tile index/palette/priority/
        // flip all in one 16-bit entry). Lets the generic `tilemap`
        // command compute each grid cell's address without knowing the
        // entry format itself.
        int TilemapEntryStride { get; }

        // Decodes one raw tilemap/nametable entry into a short, already-
        // formatted label. Deliberately NOT a generic bit-layout the
        // interface parses itself - an NES core's nametable+attribute-
        // table split isn't even the same shape as the SNES's single
        // packed word (one byte per tile, plus a separate, coarser
        // attribute byte covering a 2x2 tile block), so every core
        // decides for itself what "one entry" looks like and how to
        // render it. Same "core does the decoding, the command does the
        // grid-walking/formatting" split as Disassemble()/
        // CpuRegisters above - built specifically so a menu cursor's
        // position or a HUD tile change can be confirmed by comparing
        // tilemap entries as text instead of eyeballing two screenshots.
        string DecodeTilemapEntry(IDebugMemorySpace space, int address);

        // Decodes one 8x8 tile's pixel-index grid (64 entries, row-major,
        // each a palette index 0..2^bpp-1, 0 = transparent by this
        // toolchain's own convention) from raw bytes at <address> in
        // <space> - backs the `tile` command's ASCII rendering. Same "core
        // does the decoding, the command does the formatting" split as
        // DecodeTilemapEntry above: an SNES tile is bpp/2 bitplane pairs of
        // 16 bytes each (this core's own planar format), but nothing
        // guarantees a future core's tile format looks anything like that
        // (a bitmap-tile format wouldn't be planar at all), so the shell
        // layer has no business assuming one shape here either. <bpp>
        // values this target doesn't support should throw a clear
        // ArgumentException rather than silently returning garbage - the
        // command surfaces that message as-is, matching how a bad <space>
        // name already fails via FindSpace.
        byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp);

        // Mutes/unmutes one channel/voice for isolation testing - the
        // channel's own playback/envelope state keeps advancing normally
        // (matching real hardware muting semantics, same reasoning as
        // AudioSettings.Muted's "still advance playback state" behavior -
        // Venus_APU.md §3.4), it's just excluded from the final mixed
        // output. Lets a suspected-broken instrument be isolated (mute
        // everything else, then GetAudioSamples/`audiodump`) or ruled out
        // (mute just it, confirm the rest of the mix is unaffected)
        // without needing a separate solo-rendering pipeline. A core with
        // no such channel can no-op.
        void SetChannelMuted(int index, bool muted);

        // Defaulted: a core applying cheats on its own frame hook has nothing to do - see Moon_Debug.md §3.1a.
        void ApplyCheats() { }
    }
}
