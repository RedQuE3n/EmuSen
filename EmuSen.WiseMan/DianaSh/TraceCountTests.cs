using System.Collections.Generic;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // A trace of no instructions is refused, since a switch armed with 0 never ends - see EmuSen_DianaSh.md §13.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class TraceCountTests
    {
        private sealed class RecordingTrace : ICpuTraceSwitch
        {
            public List<string> Calls { get; } = new();
            public void Arm(int instructionCount) => Calls.Add($"arm {instructionCount}");
            public void Disarm() => Calls.Add("disarm");
        }

        [Theory]
        [InlineData("trace 0")]
        [InlineData("trace 80000000")]
        public void A_count_below_one_is_refused_and_arms_nothing(string line)
        {
            var trace = new RecordingTrace();
            var shell = DianaShInterpreter.CreateDefault(new GoldenDebugTarget(), cpuTraceSwitch: trace);
            var (_, output, _) = shell.Submit(line);
            Assert.Equal("Trace count must be at least 1.", output);
            Assert.Equal("1", shell.LastExitStatus);
            Assert.Empty(trace.Calls);
        }

        [Fact]
        public void A_count_of_one_or_more_is_armed_as_before()
        {
            var trace = new RecordingTrace();
            var shell = DianaShInterpreter.CreateDefault(new GoldenDebugTarget(), cpuTraceSwitch: trace);
            Assert.Equal("Trace armed for the next 16 instructions.", shell.Submit("trace 10").Output);
            Assert.Equal(new[] { "arm 16" }, trace.Calls);
        }
    }
}
