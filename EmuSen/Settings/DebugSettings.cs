namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator - see EmuSen_Settings_Reference.md §1.
    public static class DebugSettings
    {
        // Single kill switch for every *Logging flag below, without touching any of their individually-set.
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

        // Counts down one GSU instruction per line - see Venus_SuperFX.md §9.
        public static int SuperFxTraceCountdown = 0;

        // Scales the GSU's cycle cost, to test whether a failure is a GSU/S-CPU synchronisation problem - see Venus_SuperFX.md §8.
        public static int SuperFxSpeedDivisor = 1;

        // Logs the plot stream itself, skipping the first N plots so a later drawing pass can be reached - see Venus_SuperFX.md §8.
        public static int SuperFxPlotTraceSkip = 0;
        public static int SuperFxPlotTraceCountdown = 0;
        public static int SuperFxPlotTraceInstr = 0;

        // Logs GSU-side Game Pak RAM writes to one address, with the GSU PC that made them - answers "did the - see Venus_SuperFX.md §8.
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

        // Dumps the exact per-pixel main/sub compositing state (winning layer, raw colors, color-math.
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

        // Both directions of the $2140-$2143 / $00F4-$00F7 mailbox, logged on change only - see Venus_APU.md §1.2.
        private static bool _apuPortTrafficLogging = false;
        public static bool ApuPortTrafficLogging
        {
            get => MasterLoggingEnabled && _apuPortTrafficLogging;
            set => _apuPortTrafficLogging = value;
        }

        // Logs every KeyOn (note trigger): SRCN, the resolved sample- directory entry, computed start/loop.
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
        // Per-frame $2000-$2007 traffic with the dot it landed on - see Moon_PPU.md §7.
        private static bool _nesPpuWriteLogging = true;
        public static bool NesPpuWriteLogging
        {
            get => MasterLoggingEnabled && _nesPpuWriteLogging;
            set => _nesPpuWriteLogging = value;
        }
        // Per-frame $21xx traffic during active display - see Venus_PPU.md §7.3.
        private static bool _ppuActiveDisplayWriteLogging = true;
        public static bool PpuActiveDisplayWriteLogging
        {
            get => MasterLoggingEnabled && _ppuActiveDisplayWriteLogging;
            set => _ppuActiveDisplayWriteLogging = value;
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
