using Xunit;

// AudioPlayerTests opens/closes real SDL audio devices via native
// P/Invoke calls - SDL's global init/quit state isn't something worth
// risking under concurrent test execution, and this test suite is small
// enough that parallelism wouldn't meaningfully speed it up anyway.
[assembly: CollectionBehavior(DisableTestParallelization = true)]
