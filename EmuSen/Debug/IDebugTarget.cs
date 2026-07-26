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

    // One audio channel/voice's current debug-relevant state, generic
    // enough to cover very different sound hardware - a SNES core reports
    // its 8 S-DSP voices through this shape, an eventual NES core would
    // report its 5 APU channels (2 pulse, triangle, noise, DMC) the same
    // way, even though the underlying synthesis (BRR sample playback vs.
    // simple waveform generators) has nothing in common. `Level` is
    // deliberately a plain 0-100 scale rather than the SNES's native
    // 0-2047 envelope range, so a generic channel-viewer UI doesn't need
    // to know any one core's internal units; `Info` is a free-text escape
    // hatch for whatever's too core-specific to model generically (ADSR
    // stage, current sample source, pitch) - same "structured where cheap,
    // free-text where not" split GetSummaryText() already uses.
    public readonly struct DebugAudioChannelInfo
    {
        public int Index { get; }
        public string Name { get; }
        public bool Active { get; }
        public int Level { get; }
        public bool Muted { get; }
        public string Info { get; }

        public DebugAudioChannelInfo(int index, string name, bool active, int level, bool muted, string info)
        {
            Index = index;
            Name = name;
            Active = active;
            Level = level;
            Muted = muted;
            Info = info;
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
    // toolchain. Started out scoped to read/query capabilities only, with
    // breakpoints and single-stepping called out as a future extension
    // pending the execution loop supporting a mid-frame pause - that's now
    // done (see Breakpoints below and VenusCore.RunFrame()'s halt/resume
    // handling), added the same additive way this comment originally
    // anticipated: a new property, no breaking change to what was already
    // here.
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

        // The sound co-processor's own registers (65816-side cores: the
        // SPC700) plus the CPU<->APU communication ports (both
        // directions - see Spc700.ReadPort/WritePort's own comments on
        // why those are two independent one-byte latches per port, not
        // one). Added investigating a Super Metroid boot hang where the
        // only prior way to see SPC700 state at all was reading raw
        // verbose-trace text - there was no structured equivalent of
        // `regs` for it. A core without a distinct sound co-processor
        // (or one not yet modeled this way) can return an empty list.
        IReadOnlyList<DebugRegisterValue> GetApuRegisters();

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

        // The execution-breakpoint mechanism (see BreakpointRegistry.cs) -
        // same exposure pattern as Watches/FrameLog. Every core's
        // implementation owns the same instance its own instruction-step
        // loop consults before executing each instruction. Unlike Watches
        // (which only sees memory access), this catches control flow
        // reaching an address at all, including register-only conditions
        // (a CMP/branch pair) that never touch memory.
        BreakpointRegistry Breakpoints { get; }

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
        // GetCpuRegisters() above - built specifically so a menu cursor's
        // position or a HUD tile change can be confirmed by comparing
        // tilemap entries as text instead of eyeballing two screenshots.
        string DecodeTilemapEntry(IDebugMemorySpace space, int address);

        // Exports the current tile/character memory as a plain RGBA image,
        // for a headless harness to write straight to disk (see
        // EmuSen.HeadlessDebug's `vramsheet` verb) without ever needing a
        // Raylib window. Deliberately not "VRAM sheet" at the interface
        // level - an NES core's CHR-ROM/CHR-RAM pattern tables aren't VRAM
        // in the SNES sense, but the shape (some tile memory, decoded to a
        // grayscale/indexed sheet image) is the same across consoles. A
        // core with nothing analogous can return a 0x0 empty buffer.
        (byte[] Rgba, int Width, int Height) RenderTileSheet();

        // Exports the current color palette memory as a plain RGBA swatch
        // grid image, same rationale as RenderTileSheet() above - SNES
        // CGRAM vs an NES core's completely different palette RAM shape
        // are both just "N colors, already resolved to RGB" once decoded.
        // A core with no palette memory can return a 0x0 empty buffer.
        (byte[] Rgba, int Width, int Height) RenderPaletteSwatch();

        // Non-destructive snapshot of whatever audio samples are currently
        // buffered for output - Queue<short>.ToArray() under the hood on
        // the SNES side, specifically so this never steals samples out from
        // under a live audio-playback consumer of the same queue (see
        // Spc700.Dsp.AudioBuffer). Returned as already-interleaved 16-bit
        // PCM plus the sample rate it was produced at, so a headless
        // harness can write a standard .wav file without knowing anything
        // about the source sound co-processor. A core with no audio output
        // modeled yet can return an empty array.
        (short[] Samples, int SampleRate) GetAudioSamples();

        // Live per-channel/voice state - see DebugAudioChannelInfo's own
        // comment for why this is modeled generically. Built diagnosing a
        // "part of the music is missing" report, where the existing
        // toolchain could only inspect the final mixed audio output
        // (GetAudioSamples) or a KeyOn event as it happened (console-only
        // DspKeyOnLogging) - neither answers "is voice N currently active,
        // and what's its envelope actually doing right now." A core with
        // no distinct channel/voice concept can return an empty list.
        IReadOnlyList<DebugAudioChannelInfo> GetAudioChannels();

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
    }
}
