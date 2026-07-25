namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator -
    // see Man pages/EmuSen_Settings_Reference.md §1 for what each flag is
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
        public static bool MasterLoggingEnabled = false;

        // --- Cpu.cs ---
        private static bool _cpuVerboseLogging = false;
        public static bool CpuVerboseLogging
        {
            get => MasterLoggingEnabled && _cpuVerboseLogging;
            set => _cpuVerboseLogging = value;
        }
        public static int CpuTraceCountdown = 0;
        public static bool HvIrqEnabled = true;

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

        // --- Spc700.cs ---
        private static bool _spc700VerboseLogging = false;
        public static bool Spc700VerboseLogging
        {
            get => MasterLoggingEnabled && _spc700VerboseLogging;
            set => _spc700VerboseLogging = value;
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
    }
}
