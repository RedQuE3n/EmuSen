namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator -
    // see Man pages/EmuSen_Settings_Reference.md §1 for what each flag is
    // for and the investigation history behind it.
    public static class DebugSettings
    {
        // --- Cpu.cs ---
        public static bool CpuVerboseLogging = false;
        public static int CpuTraceCountdown = 0;
        public static bool HvIrqEnabled = true;

        // --- Dma.cs ---
        public static bool DmaVerboseLogging = true;
        public static bool DmaSourceAddrLogging = true;
        public static bool WindowHdmaLogging = false;

        // --- Spc700.cs ---
        public static bool Spc700VerboseLogging = false;

        // --- Ppu.cs ---
        public static bool CgWriteLogging = false;
        public static bool Bg3ScrollWriteLogging = false;
        public static bool CameraRamLogging = true;
        public static bool RenderReadLogging = true;
        public static bool AllScrollWriteLogging = true;

        // --- MemoryBus.cs ---
        public static bool HvIrqChangeLogging = false;
        public static bool MathUnitLogging = true;
        public static bool BgModeChangeLogging = true;
        public static bool MosaicWriteLogging = true;

        // --- Renderer.cs ---
        public static bool WindowingEnabled = true;
    }
}
