namespace EmuSen.Debug
{
    // Central hub for every debug/diagnostic toggle across the emulator - see EmuSen_Settings_Reference.md §1.
    public static class DebugSettings
    {
        // Single kill switch for every *Logging flag below, without touching any of their individually-set.
        public static bool MasterLoggingEnabled
        {
            get => EmuSen.Crystal.Debugger.DebugLogging.MasterEnabled;
            set => EmuSen.Crystal.Debugger.DebugLogging.MasterEnabled = value;
        }

        // Per-frame $2000-$2007 traffic with the dot it landed on - see Moon_PPU.md §7.
        private static bool _nesPpuWriteLogging = true;
        public static bool NesPpuWriteLogging
        {
            get => MasterLoggingEnabled && _nesPpuWriteLogging;
            set => _nesPpuWriteLogging = value;
        }
    }
}
