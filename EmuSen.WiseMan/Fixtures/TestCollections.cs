namespace EmuSen.WiseMan.Fixtures
{
    // Tests run in parallel by collection. One collection opts out, and it holds every class that
    // reaches process-wide mutable state - see EmuSen_Debugging_Tools_Reference_v5.md §3.56.
    public static class TestCollections
    {
        // Four kinds of shared state, one collection, because a class can only be in one:
        //
        //   config    ConfigStore/DataStore/ConfigFile overrides and CheatDatabaseInstaller.FetchOverride,
        //             plain statics set in a constructor and nulled in Dispose.
        //   SDL       AudioPlayer opens and closes real devices; SDL's init state is process-wide.
        //   trace     CpuBinaryTrace/GsuBinaryTrace/ApuWriteTrace are one buffer for the whole process.
        //   Venus     every main 65816 has _traceBinary set, so any test running a Venus core writes
        //             into that same buffer the moment a trace test enables it.
        //
        // AudioLatencyDriftTests is why these are not four collections: it drives SDL and a Venus core
        // both, and xUnit gives a class exactly one collection.
        //
        // Not here on purpose: CoreOptions.BatteryRamDisabled. Cartridge latches it at construction and
        // nothing in the suite ever assigns it false, so concurrent writes are all the same value.
        public const string ProcessGlobals = "Process globals";
    }

    [CollectionDefinition(TestCollections.ProcessGlobals, DisableParallelization = true)]
    public sealed class ProcessGlobalsCollection { }
}
