using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.DianaSh.DianaSh.Bin;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // Interrupt() stops a running line on whichever shell the switch chose; EMUSEN_DIANASH_NATIVE=0 runs these on the C# fallback - see EmuSen_DianaSh.md §10.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class InterruptTests
    {
        private static (string Output, long LateMs) RunInterrupted(DianaShInterpreter shell, string line)
        {
            var raised = new Stopwatch();
            var thread = Task.Run(() => { Thread.Sleep(50); raised.Start(); shell.Interrupt(); });
            string output = shell.Submit(line).Output;
            long late = raised.ElapsedMilliseconds;
            thread.Wait();
            return (output, late);
        }

        [Theory]
        [InlineData("while true; do echo x | grep x; done")]
        [InlineData("until false; do true; done")]
        [InlineData("for i in 1 2 3; do while true; do true; done; done")]
        public void A_loop_stops_within_100_ms_with_a_caret_c_and_status_130(string line)
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null);
            var (output, late) = RunInterrupted(shell, line);
            Assert.True(late < 100, $"{late} ms");
            Assert.EndsWith("^C", output);
            Assert.Equal("130", shell.Submit("echo $?").Output);
        }

        [Fact]
        public void The_next_command_runs_and_an_interrupt_with_nothing_running_does_nothing()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null);
            RunInterrupted(shell, "while true; do true; done");
            Assert.Equal("ok", shell.Submit("echo ok").Output);
            shell.Interrupt();
            Assert.Equal("ok", shell.Submit("echo ok").Output);
        }
    }
}
