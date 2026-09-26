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

        // A ScreenScraper address in a fault reaches the file with its credentials blanked - see EmuSen_BigPicture.md §17.
        [Fact]
        public void A_screen_scraper_address_in_a_fault_is_written_without_its_credentials()
        {
            var fault = new InvalidOperationException("GET https://api.screenscraper.fr/api2/jeuInfos.php?devid=FAKECRASHID&devpassword=FAKECRASHPW&softname=EmuSen&ssid=FAKECRASHM&sspassword=FAKECRASHMPW&md5=00");

            string text = File.ReadAllText(CrashLog.Write("scrape", fault, "context devpassword=FAKECRASHPW")!);

            Assert.Contains("devpassword=***", text);
            Assert.Contains("md5=00", text);
            foreach (string secret in new[] { "FAKECRASHID", "FAKECRASHPW", "FAKECRASHM", "FAKECRASHMPW" }) Assert.DoesNotContain(secret, text);
        }
    }
}
