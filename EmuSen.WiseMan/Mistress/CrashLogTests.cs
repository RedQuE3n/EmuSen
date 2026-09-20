using System;
using System.IO;
using EmuSen.Mistress;

namespace EmuSen.WiseMan.Mistress
{
    // A fault is written whole, with what was running, to a file of its own - see EmuSen_Settings_Reference.md §4.27.
    public class CrashLogTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenCrash_" + Guid.NewGuid().ToString("N"));

        public CrashLogTests() => CrashLog.DirectoryOverride = _dir;

        public void Dispose()
        {
            CrashLog.DirectoryOverride = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public void A_fault_is_written_with_its_kind_its_context_and_its_whole_trace()
        {
            Exception fault;
            try { throw new InvalidOperationException("outer", new IndexOutOfRangeException("inner")); }
            catch (Exception caught) { fault = caught; }

            string? path = CrashLog.Write("emulation halt", fault, "N64, frame 12: RenderScale=2");

            Assert.NotNull(path);
            string text = File.ReadAllText(path!);
            Assert.Contains("emulation halt", text);
            Assert.Contains("RenderScale=2", text);
            Assert.Contains("inner", text);
            Assert.Contains(nameof(A_fault_is_written_with_its_kind_its_context_and_its_whole_trace), text);
        }
    }
}
