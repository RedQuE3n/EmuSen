using System.Collections.Generic;

namespace EmuSen.Debug
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

    // One named register/flag value for display. BitWidth drives formatting
    // (e.g. pad to 2 hex digits for an 8-bit register, 4 for 16-bit) - kept
    // generic rather than typed per-register because different CPUs have
    // wildly different register sets (the 65816 has PB/DB/D/an E flag the
    // 6502 doesn't; a 6502 target would just report a different list
    // through this same shape).
    public readonly struct DebugRegisterValue
    {
        public string Name { get; }
        public ulong Value { get; }
        public int BitWidth { get; }

        public DebugRegisterValue(string name, ulong value, int bitWidth)
        {
            Name = name;
            Value = value;
            BitWidth = bitWidth;
        }
    }

    // One sprite/OBJ entry, shaped generically enough to cover consoles
    // with very different sprite hardware (the SNES's OAM low+high table
    // split, the NES's flatter 4-byte-per-sprite OAM) - a generic sprite
    // viewer just needs a rectangle, a tile/palette reference, and flip/
    // priority flags, regardless of how the underlying hardware actually
    // stores that.
    public readonly struct DebugSpriteInfo
    {
        public int Index { get; }
        public int X { get; }
        public int Y { get; }
        public int Width { get; }
        public int Height { get; }
        public int TileIndex { get; }
        public int PaletteIndex { get; }
        public int Priority { get; }
        public bool FlipX { get; }
        public bool FlipY { get; }

        public DebugSpriteInfo(int index, int x, int y, int width, int height, int tileIndex, int paletteIndex, int priority, bool flipX, bool flipY)
        {
            Index = index;
            X = x;
            Y = y;
            Width = width;
            Height = height;
            TileIndex = tileIndex;
            PaletteIndex = paletteIndex;
            Priority = priority;
            FlipX = flipX;
            FlipY = flipY;
        }
    }

    // One palette's worth of colors, already resolved to display-ready RGB
    // - so a palette-viewer widget never needs to know a given console's
    // native color format (SNES BGR555, NES's 64-color master palette,
    // etc.); each DebugTarget converts its own native format once, here.
    public readonly struct DebugPaletteInfo
    {
        public int Index { get; }
        public IReadOnlyList<(byte r, byte g, byte b)> Colors { get; }

        public DebugPaletteInfo(int index, IReadOnlyList<(byte r, byte g, byte b)> colors)
        {
            Index = index;
            Colors = colors;
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

    // The contract a console core implements to plug into the shared debug
    // toolchain. Deliberately scoped to read/query capabilities only for
    // now - breakpoints, single-stepping, and disassembly are planned as a
    // separate future extension to this same interface once the execution
    // loop actually supports pausing mid-frame (it currently only runs a
    // whole frame at a time), and once a real disassembler exists (today
    // there's only an opcode *executor*, nothing that turns bytes back into
    // mnemonics for display). Adding those later as new interface members
    // is expected to be additive, not a breaking change to what's here.
    public interface IDebugTarget
    {
        // Short console name for display ("SNES", "NES", etc).
        string CoreName { get; }

        IReadOnlyList<IDebugMemorySpace> GetMemorySpaces();

        // Grouped separately (CPU vs "video") since almost every console
        // draws this exact line in its own documentation and debuggers -
        // keeps a generic register-panel UI able to show two logical
        // sections without needing to know which registers belong to which
        // half itself.
        IReadOnlyList<DebugRegisterValue> GetCpuRegisters();
        IReadOnlyList<DebugRegisterValue> GetVideoRegisters();

        IReadOnlyList<DebugSpriteInfo> GetSprites();
        IReadOnlyList<DebugPaletteInfo> GetPalettes();

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

        // Monotonic frame counter, incremented once per rendered frame -
        // the shared "what moment is this" reference used to correlate a
        // screenshot, a log line, or a future GUI debugger's state all
        // against the exact same instant, regardless of which core is
        // running (an NES target reports its own frame counter through
        // this same property, no different in kind from SNES's).
        long FrameCount { get; }

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

        // Free-text escape hatch covering whatever isn't (yet) exposed as
        // structured data above - lets a target be genuinely useful on day
        // one without needing every possible piece of state modeled
        // up front. A generic UI can render this in a fallback text panel.
        string GetSummaryText();
    }
}
