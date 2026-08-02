namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator -
    // see EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Settings_Reference.md §1 for what each flag is
    // for and the investigation history behind it.
    public static class DebugSettings
    {
        // Single kill switch for every *Logging flag below, without
        // touching any of their individually-set values - flip this off
        // and every call site that already checks e.g.
        // DebugSettings.CpuVerboseLogging silently stops logging (no code
        // changes needed anywhere else, since the intersection happens in
        // each property's getter below), flip it back on and whatever was
        // individually enabled before comes right back. Feature toggles
        // that aren't about logging output (HvIrqEnabled, WindowingEnabled)
        // are deliberately NOT gated by this - this only silences trace
        // output, it doesn't change emulation behavior.
        //
        // Defaults to off - several individual flags below default to true
        // (leftover from the investigations that added them), which meant
        // a stock build logged a steady stream of DMA/scroll/math-unit
        // traces on every run whether anyone asked for it or not. This is
        // the actual on/off switch for that; the individual flags keep
        // whatever value they're set to underneath, so re-enabling this
        // brings back exactly what was configured before, per this
        // comment's own original design.
        //
        // Forwards to EmuSen.DianaOS.DianaOS.Etc.DianaOSLogging.MasterEnabled rather
        // than holding its own field - DianaOS's own `log` command and
        // WatchRegistry's live echo need to read/write this without
        // depending on this (or any other core's) settings class, so the
        // actual flag lives there; every caller here (VenusCore,
        // Mistress's DebugSettingsWindow, Pharaoh) keeps working
        // against this same property name unchanged.
        public static bool MasterLoggingEnabled
        {
            get => EmuSen.DianaOS.DianaOS.Etc.DianaOSLogging.MasterEnabled;
            set => EmuSen.DianaOS.DianaOS.Etc.DianaOSLogging.MasterEnabled = value;
        }

        // --- Cpu.cs ---
        private static bool _cpuVerboseLogging = false;
        public static bool CpuVerboseLogging
        {
            get => MasterLoggingEnabled && _cpuVerboseLogging;
            set => _cpuVerboseLogging = value;
        }
        public static int CpuTraceCountdown = 0;
        public static bool HvIrqEnabled = true;

        // --- Coprocessors/SuperFx ---
        // Counts down one GSU instruction per line - see Venus_SuperFX.md §9.
        public static int SuperFxTraceCountdown = 0;

        // Scales the GSU's cycle cost, to test whether a failure is a
        // GSU/S-CPU synchronisation problem - see Venus_SuperFX.md §8.
        public static int SuperFxSpeedDivisor = 1;

        // Logs the plot stream itself, skipping the first N plots so a later
        // drawing pass can be reached - see Venus_SuperFX.md §8.
        public static int SuperFxPlotTraceSkip = 0;
        public static int SuperFxPlotTraceCountdown = 0;
        public static int SuperFxPlotTraceInstr = 0;

        // Logs GSU-side Game Pak RAM writes to one address, with the GSU PC that
        // made them - answers "did the chip write this, and from where"
        // for output the chip builds with stores rather than PLOT.
        // -1 is off - see Venus_SuperFX.md §8.
        public static int SuperFxRamWriteTraceAddr = -1;
        public static int SuperFxRamWriteTraceCountdown = 0;

        // --- Dma.cs ---
        private static bool _dmaVerboseLogging = true;
        public static bool DmaVerboseLogging
        {
            get => MasterLoggingEnabled && _dmaVerboseLogging;
            set => _dmaVerboseLogging = value;
        }
        private static bool _dmaSourceAddrLogging = true;
        public static bool DmaSourceAddrLogging
        {
            get => MasterLoggingEnabled && _dmaSourceAddrLogging;
            set => _dmaSourceAddrLogging = value;
        }
        private static bool _windowHdmaLogging = false;
        public static bool WindowHdmaLogging
        {
            get => MasterLoggingEnabled && _windowHdmaLogging;
            set => _windowHdmaLogging = value;
        }

        // --- Renderer.Scanline.cs ---
        // Dumps the exact per-pixel main/sub compositing state (winning
        // layer, raw colors, color-math participation, blend result) for
        // one target scanline - added for the Zelda: A Link to the Past
        // color-math investigation, where the aggregate symptom (a wrong
        // blended color) gave no way to see which of the two blend
        // operands was actually wrong without this. ColorMathBlendScanline
        // defaults to -1 (never matches a real py) so this stays inert
        // until both a scanline AND MasterLoggingEnabled are set.
        private static bool _colorMathBlendLogging = false;
        public static bool ColorMathBlendLogging
        {
            get => MasterLoggingEnabled && _colorMathBlendLogging;
            set => _colorMathBlendLogging = value;
        }
        public static int ColorMathBlendScanline = -1;

        // --- Spc700.cs ---
        private static bool _spc700VerboseLogging = false;
        public static bool Spc700VerboseLogging
        {
            get => MasterLoggingEnabled && _spc700VerboseLogging;
            set => _spc700VerboseLogging = value;
        }

        // Both directions of the $2140-$2143 / $00F4-$00F7 mailbox, logged
        // on change only - see Venus_APU.md §1.2.
        private static bool _apuPortTrafficLogging = false;
        public static bool ApuPortTrafficLogging
        {
            get => MasterLoggingEnabled && _apuPortTrafficLogging;
            set => _apuPortTrafficLogging = value;
        }

        // --- DspVoice.cs ---
        // Logs every KeyOn (note trigger): SRCN, the resolved sample-
        // directory entry, computed start/loop address, the BRR header
        // byte actually found there, and the voice's pitch/volume - added
        // for the "audio still sounds garbled after fixing SPC700 timing"
        // investigation, to check whether the SPC700 sound driver is
        // triggering voices with sane-looking sample pointers at all,
        // independent of whatever BrrDecoder/DspVoice do with them
        // afterward.
        private static bool _dspKeyOnLogging = false;
        public static bool DspKeyOnLogging
        {
            get => MasterLoggingEnabled && _dspKeyOnLogging;
            set => _dspKeyOnLogging = value;
        }

        // --- Ppu.cs ---
        private static bool _cgWriteLogging = false;
        public static bool CgWriteLogging
        {
            get => MasterLoggingEnabled && _cgWriteLogging;
            set => _cgWriteLogging = value;
        }
        private static bool _bg3ScrollWriteLogging = false;
        public static bool Bg3ScrollWriteLogging
        {
            get => MasterLoggingEnabled && _bg3ScrollWriteLogging;
            set => _bg3ScrollWriteLogging = value;
        }
        private static bool _cameraRamLogging = true;
        public static bool CameraRamLogging
        {
            get => MasterLoggingEnabled && _cameraRamLogging;
            set => _cameraRamLogging = value;
        }
        private static bool _renderReadLogging = true;
        public static bool RenderReadLogging
        {
            get => MasterLoggingEnabled && _renderReadLogging;
            set => _renderReadLogging = value;
        }
        private static bool _allScrollWriteLogging = true;
        public static bool AllScrollWriteLogging
        {
            get => MasterLoggingEnabled && _allScrollWriteLogging;
            set => _allScrollWriteLogging = value;
        }

        // --- MemoryBus.cs ---
        private static bool _hvIrqChangeLogging = false;
        public static bool HvIrqChangeLogging
        {
            get => MasterLoggingEnabled && _hvIrqChangeLogging;
            set => _hvIrqChangeLogging = value;
        }
        private static bool _mathUnitLogging = true;
        public static bool MathUnitLogging
        {
            get => MasterLoggingEnabled && _mathUnitLogging;
            set => _mathUnitLogging = value;
        }
        private static bool _bgModeChangeLogging = true;
        public static bool BgModeChangeLogging
        {
            get => MasterLoggingEnabled && _bgModeChangeLogging;
            set => _bgModeChangeLogging = value;
        }
        private static bool _mosaicWriteLogging = true;
        public static bool MosaicWriteLogging
        {
            get => MasterLoggingEnabled && _mosaicWriteLogging;
            set => _mosaicWriteLogging = value;
        }

        // --- Renderer.cs ---
        public static bool WindowingEnabled = true;

        // ANDed into TM/TS to isolate one layer at a time - see EmuSen_Debugging_Tools_Reference_v5.md §3.19.
        public static int LayerEnableMask = 0x1F;

        // Dumps mid-frame PPU register state, which `regs` cannot see - see §3.19.
        public static int ScanlineRegisterDumpLine = -1;
    }
}
