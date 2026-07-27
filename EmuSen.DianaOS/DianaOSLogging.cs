namespace EmuSen.DianaOS
{
    // Single kill switch for every trace/diagnostic-log flag anywhere in
    // this project, without touching any individual flag's own value -
    // flip this off and every call site that already checks its own
    // logging flag (see e.g. EmuSen.Debug.DebugSettings, which gates every
    // one of its own *Logging properties on this) silently stops logging,
    // flip it back on and whatever was individually enabled before comes
    // right back.
    //
    // Owned here, not by any core's own settings, because `log` (see
    // Commands/LogCommand.cs) and WatchRegistry's own live console echo
    // are core-agnostic infrastructure that needs to read/write this
    // regardless of which core (or none at all, in a standalone launch)
    // is loaded - a core's own settings class references THIS instead of
    // the other way around, the same dependency direction IDebugTarget/
    // ICheatCodeCodec already establish elsewhere in DianaOS.
    public static class DianaOSLogging
    {
        public static bool MasterEnabled { get; set; }
    }
}
