using System;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading;
using EmuSen.Common;

namespace EmuSen.WiseMan.Common
{
    // A writer disposed while its worker is still draining lets the worker finish and close the files - see EmuSen_Debugging_Tools_Reference_v5.md §2.1.
    public class CategorizedLogWriterTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenLogWriter", Guid.NewGuid().ToString("N"));

        public CategorizedLogWriterTests() => Directory.CreateDirectory(_dir);

        public void Dispose()
        {
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        // A console that holds its first write until let go, as a starved worker would.
        private sealed class HeldConsole(ManualResetEventSlim gate) : TextWriter
        {
            private int _writes;
            public override Encoding Encoding => Encoding.UTF8;
            public override void WriteLine(string? value)
            {
                if (Interlocked.Increment(ref _writes) == 1) gate.Wait();
            }
        }

        private static string ReadShared(string path)
        {
            using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            return new StreamReader(stream).ReadToEnd().ReplaceLineEndings("\n");
        }

        [Fact]
        public void A_worker_still_draining_after_dispose_gives_up_finishes_and_closes_the_files()
        {
            using var gate = new ManualResetEventSlim();
            var writer = new CategorizedLogWriter(new HeldConsole(gate), _dir);
            writer.WriteLine("first");
            writer.WriteLine("second");

            writer.Dispose();
            gate.Set();

            string general = Path.Combine(_dir, "general.log");
            var clock = Stopwatch.StartNew();
            while (ReadShared(general) != "first\nsecond\n" && clock.ElapsedMilliseconds < 5000) Thread.Sleep(20);
            Assert.Equal("first\nsecond\n", ReadShared(general));
        }
    }
}
