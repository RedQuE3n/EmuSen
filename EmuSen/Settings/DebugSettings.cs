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
    }
}
