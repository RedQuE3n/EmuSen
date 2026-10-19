using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The runtime's own lines reach a client as log events of source "runtime", never the process's console, and a program's filer writes them where its log goes - see EmuSen_Backend.md §29.11, Q2.
    [Collection(TestCollections.ProcessGlobals)]
    public class RuntimeLogTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static string[] RuntimeLines(string drained) =>
            JsonDocument.Parse(drained).RootElement.EnumerateArray()
                .Where(e => e.GetProperty("kind").GetString() == "log" && e.GetProperty("source").GetString() == "runtime")
                .SelectMany(e => e.GetProperty("lines").EnumerateArray().Select(l => l.GetString()!)).ToArray();

        // A recording's end says what became of its pictures; the words arrive as a runtime line and the console hears nothing.
        [Fact]
        public void A_recording_s_words_are_a_runtime_log_event_and_not_console_output()
        {
            using CrystalSubscription events = _f.Runtime.Subscribe(0, CrystalAbi.EventLog);
            TextWriter was = Console.Out;
            var console = new StringWriter();
            Console.SetOut(console);
            try
            {
                using CrystalSession s = _f.Open();
                s.StartRecording(Path.Combine(_f.Root, "Recordings"), 1);
                var info = new CrystalFrameInfo();
                s.Frame(ref info);
                s.StopRecording();
            }
            finally
            {
                Console.SetOut(was);
            }
            Assert.DoesNotContain("[RECORD]", console.ToString());
            string[] lines = RuntimeLines(events.Drain());
            Assert.NotEmpty(lines);
            Assert.All(lines, l => Assert.StartsWith("[RECORD]", l));
        }

        // The filer writes the runtime's lines to the console as it stands when it files, and passes over the core's.
        [Fact]
        public void The_filer_writes_the_runtime_s_lines_to_the_console()
        {
            using var log = new CrystalRuntimeLog(_f.Runtime);
            using (CrystalSession s = _f.Open())
            {
                s.StartRecording(Path.Combine(_f.Root, "Recordings"), 1);
                var info = new CrystalFrameInfo();
                s.Frame(ref info);
                s.StopRecording();
            }
            TextWriter was = Console.Out;
            var console = new StringWriter();
            Console.SetOut(console);
            try { log.File(); }
            finally { Console.SetOut(was); }
            string[] filed = console.ToString().Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
            Assert.NotEmpty(filed);
            Assert.All(filed, l => Assert.StartsWith("[RECORD]", l));
        }

        // A core's own lines carry source "core".
        [Fact]
        public void Every_log_event_names_its_source()
        {
            using CrystalSubscription events = _f.Runtime.Subscribe(0, CrystalAbi.EventLog);
            using (CrystalSession s = _f.Open())
            {
                s.StartRecording(Path.Combine(_f.Root, "Recordings"), 1);
                var info = new CrystalFrameInfo();
                for (int i = 0; i < 120; i++) s.Frame(ref info);
                s.StopRecording();
            }
            var sources = JsonDocument.Parse(events.Drain()).RootElement.EnumerateArray().Where(e => e.GetProperty("kind").GetString() == "log").Select(e => e.GetProperty("source").GetString()).ToArray();
            Assert.NotEmpty(sources);
            Assert.All(sources, s => Assert.Contains(s, new[] { "core", "runtime" }));
        }
    }
}
