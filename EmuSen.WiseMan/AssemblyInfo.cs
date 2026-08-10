// Tests run in parallel across collections, on as many threads as the machine has
// cores. Two collections opt out, and only two: Fixtures/TestCollections.cs names
// them and EmuSen_Debugging_Tools_Reference_v5.md §3.56 says what each protects.
// AudioPlayerTests' real SDL init/quit state is one of them - it is a genuine
// hazard, but it belongs to three classes rather than to the whole assembly, which
// is how it was scoped until §3.56 measured what that cost.
