namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator. Flip
    // these here instead of hunting through individual files. Renamed from their
    // original per-class names (several were all called "VerboseLogging" on
    // different classes, which doesn't work once collected into one place) but
    // otherwise behave identically to before - grep the old name in project
    // history if a mapping is ever unclear.
    public static class DebugSettings
    {
        // --- Cpu.cs ---

        // Logs every executed instruction (PC, opcode, name, target address).
        // Very high volume - intended for short, targeted traces, not routine play.
        public static bool CpuVerboseLogging = false;

        // Companion to CpuVerboseLogging: when set to a positive number alongside
        // enabling that flag, logs that many more instructions and then
        // auto-disables itself. Leave at 0 for an unbounded trace (rarely what you
        // want - a bounded trace was what found the H-blank wait-loop bug).
        public static int CpuTraceCountdown = 0;

        // Master on/off switch for H/V-IRQ support ($4200 bits 4-5, $4207-$420A).
        // This is a real, working feature (confirmed fixing the title-screen
        // lockup via the H-blank flag) - not a "broken, avoid" toggle, just a
        // convenient full disable if a future regression needs isolating.
        public static bool HvIrqEnabled = true;

        // --- Dma.cs ---

        // Logs every general (one-shot) DMA transfer: channel, direction, source,
        // destination register, size. High volume during level loads - enabled
        // here specifically to catch SMW's coin tile-animation upload (a
        // periodic VRAM CHR rewrite via $2118/$2119, per SMWCentral's
        // description of coins being animated as tiles rather than sprites -
        // if that transfer isn't landing right, the coin tile's pixel data
        // in VRAM could end up blank). Look for repeating small transfers to
        // $2118/$2119 at a consistent destination VRAM address, roughly once
        // per frame or every few frames, landing during vblank.
        public static bool DmaVerboseLogging = true;

        // Logs the CPU PC responsible for every DMA channel source-address
        // write - see LogSourceAddrWrite's comment in Dma.cs for why this
        // was added (Yoshi's sprite-graphics investigation) and what
        // finding it's meant to confirm or rule out. Separate from
        // DmaVerboseLogging so it can be toggled independently - this is a
        // narrower, more targeted trace for one specific question, not
        // general DMA visibility.
        public static bool DmaSourceAddrLogging = true;

        // Logs HDMA writes/block-fetches specifically for the window-position
        // registers ($2126-$2129) on channel 7 - used to trace the title screen's
        // window-wipe effect. High volume: fires every scanline while that
        // channel is active.
        public static bool WindowHdmaLogging = false;

        // --- Spc700.cs ---

        // Logs every executed SPC700 instruction. Same tradeoffs as
        // CpuVerboseLogging, for the audio CPU instead of the main one.
        public static bool Spc700VerboseLogging = false;

        // --- Ppu.cs ---

        // Logs every CGRAM color write (index, palette/entry, raw bytes). High
        // volume - a full palette refresh alone is 100+ writes.
        public static bool CgWriteLogging = false;

        // Logs every write to BG3's scroll registers specifically.
        public static bool Bg3ScrollWriteLogging = false;

        // Logs writes to $001A-$0021 - the zero-page RAM bytes the title-screen/
        // level NMI routine reads from immediately before flushing them to
        // BG1HOFS/VOFS ($210D/$210E) and BG2HOFS/VOFS ($210F/$2110), per the
        // MesenCE disassembly (LDA $1A/STA $210D, ... LDA $1E/STA $210F, etc).
        // Used to check whether the BG2 scroll jitter originates upstream of the
        // PPU register write itself - i.e. whether our CPU is computing a
        // different raw camera value than real hardware - now that the
        // write-twice register formula and write order are both confirmed
        // correct against MesenCE (see AllScrollWriteLogging/MathUnitLogging
        // history). Only logs on actual value change, so it stays low-volume.
        public static bool CameraRamLogging = true;

        // Logs BgScrollX[1]/BgScrollY[1] (BG2) at the moment RenderBg2 reads them
        // for scanline 0 of each frame, tagged [RENDER-READ]. Compare against the
        // last [SCROLL] BG2 write logged for that frame: if they ever differ, the
        // renderer is sampling before the NMI handler's writes have landed - a
        // timing bug distinct from the write-twice register formula itself, which
        // has separately been verified correct against documented hardware
        // behavior and against MesenCE's disassembly/write order.
        public static bool RenderReadLogging = true;

        // Logs every write to ALL FOUR backgrounds' scroll registers, in order,
        // with the shared latch value going into each calculation. Used to trace
        // the BG2 parallax jitter bug - specifically whether BG3's known-zero
        // writes are interleaving with BG2's and corrupting the shared latch
        // that all scroll writes depend on.
        public static bool AllScrollWriteLogging = true;

        // --- MemoryBus.cs ---

        // Logs $4200/$4207-$420A writes when they actually change the H/V-IRQ
        // enable bits or HTIME/VTIME. Low volume (only logs on real change) -
        // safe to leave on if debugging IRQ timing again.
        public static bool HvIrqChangeLogging = false;

        // Logs every hardware multiply/divide operation ($4203/$4206 triggers)
        // with operands and result. Used to trace the BG2 parallax jitter bug -
        // specifically whether SMW's scroll math depends on this unit and
        // whether the results look sane.
        public static bool MathUnitLogging = true;

        // Logs $2105 (BGMODE) writes when the byte actually changes, broken
        // out into mode/BG3-priority/per-BG tile-size bits. Low volume (games
        // set this rarely, usually once per level) - added alongside the
        // BGMODE tile-size (16x16) support in Renderer.Backgrounds.cs so a
        // play session can confirm whether a given level ever actually sets
        // bits 4-7, rather than trusting the fix blind.
        public static bool BgModeChangeLogging = true;

        // Logs every $2106 (MOSAIC) write with the scanline it landed on.
        // Added alongside the mosaic starting-scanline latch (see
        // Ppu.MosaicStartScanline) - confirms whether a level ever writes
        // this mid-frame (outside vblank), which is the only case where that
        // latch does anything different from the old always-anchor-at-0
        // behavior. Low volume - most games only ever touch this in NMI.
        public static bool MosaicWriteLogging = true;

        // --- Renderer.cs ---

        // Master on/off switch for window masking ($2123-$212F). Re-enabled -
        // the masking logic itself was independently verified correct
        // against three sources (SMWCentral, SNESdev wiki, fullsnes) back
        // when this was built. It was switched off because ONE specific,
        // separate bug (a stuck HDMA table on the title screen, a CPU-side
        // issue unrelated to windowing itself) froze a window at a single
        // pixel and hid most of the screen there - but leaving windowing off
        // globally to avoid that one scene means every OTHER game effect
        // that actually relies on windows (status bar splits, spotlight/
        // darkness effects) was silently rendering with no windowing at all,
        // which is a much bigger correctness cost than the one known,
        // already-identified title-screen bug. If that specific scene
        // regresses, it's a real but separate, already-tracked HDMA bug to
        // chase down on its own - not a reason to keep this off everywhere.
        public static bool WindowingEnabled = true;
    }
}
