namespace EmuSen.WiseMan.Fixtures
{
    // One collection runs alone and holds every class that reaches process-wide state: config, SDL, firmware - see EmuSen_Galaxia.md §3.14.
    public static class TestCollections
    {
        public const string ProcessGlobals = "Process globals";
    }

    [CollectionDefinition(TestCollections.ProcessGlobals, DisableParallelization = true)]
    public sealed class ProcessGlobalsCollection { }
}
