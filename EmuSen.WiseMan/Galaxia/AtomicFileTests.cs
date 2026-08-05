using System;
using System.IO;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Galaxia
{
    // The durability contract every .srm and .state write now goes through - see EmuSen_Galaxia.md §4.
    public class AtomicFileTests : IDisposable
    {
        private readonly string _dir = Path.Combine(
            Path.GetTempPath(), "EmuSenAtomic_" + Guid.NewGuid().ToString("N"));

        private string PathFor(string name) => Path.Combine(_dir, name);

        public void Dispose()
        {
            ConfigDiagnostics.Reset();
            if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true);
        }

        [Fact]
        public void Write_creates_the_directory_it_needs()
        {
            Assert.True(AtomicFile.Write(PathFor("new.srm"), new byte[] { 1, 2, 3 }));
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(PathFor("new.srm")));
        }

        [Fact]
        public void Write_replaces_the_previous_contents()
        {
            AtomicFile.Write(PathFor("save.srm"), new byte[] { 1, 1, 1, 1 });
            AtomicFile.Write(PathFor("save.srm"), new byte[] { 9, 9 });

            Assert.Equal(new byte[] { 9, 9 }, File.ReadAllBytes(PathFor("save.srm")));
        }

        // The temp file absorbs the truncation risk, and nothing ever reads it.
        [Fact]
        public void Write_leaves_no_temp_file_behind()
        {
            AtomicFile.Write(PathFor("save.srm"), new byte[] { 4, 5, 6 });

            Assert.False(File.Exists(PathFor("save.srm") + AtomicFile.TempSuffix));
        }

        // A leftover .tmp from a killed process must not be mistaken for the save.
        [Fact]
        public void An_abandoned_temp_file_never_becomes_the_live_file()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllBytes(PathFor("save.srm"), new byte[] { 7, 7, 7 });
            File.WriteAllBytes(PathFor("save.srm") + AtomicFile.TempSuffix, new byte[] { 0 });

            Assert.Equal(new byte[] { 7, 7, 7 }, AtomicFile.TryRead(PathFor("save.srm")));
        }

        [Fact]
        public void TryRead_returns_null_for_a_missing_file_without_reporting()
        {
            Assert.Null(AtomicFile.TryRead(PathFor("absent.srm")));
            Assert.Null(ConfigDiagnostics.LastMessage);
        }

        [Fact]
        public void TryRead_round_trips_what_Write_wrote()
        {
            byte[] payload = { 0xDE, 0xAD, 0xBE, 0xEF };
            AtomicFile.Write(PathFor("round.srm"), payload);

            Assert.Equal(payload, AtomicFile.TryRead(PathFor("round.srm")));
        }

        // Best-effort by design - a full disk must not take the program down mid-frame.
        [Fact]
        public void An_unwritable_path_reports_and_returns_false_instead_of_throwing()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllBytes(PathFor("blocker"), Array.Empty<byte>());

            Assert.False(AtomicFile.Write(Path.Combine(PathFor("blocker"), "save.srm"), new byte[] { 1 }));
            Assert.NotNull(ConfigDiagnostics.LastMessage);
        }
    }
}
