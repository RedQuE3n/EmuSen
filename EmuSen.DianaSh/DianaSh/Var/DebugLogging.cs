namespace EmuSen.DianaSh.DianaSh.Var
{
    // The emulator's one logging kill switch, leaving each flag's own value alone - see `man log` and EmuSen_DianaSh.md §13.5.
    public static class DebugLogging
    {
        public static bool MasterEnabled { get; set; }
    }
}
