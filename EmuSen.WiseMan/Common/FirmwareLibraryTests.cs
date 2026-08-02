using EmuSen.Common.Firmware;

namespace EmuSen.WiseMan.Common
{
    // The core-agnostic firmware store: discovery, size validation, and what
    // happens to a file the user picks. Nothing here knows what an SNES is.
    // See EmuSen_Firmware.md §2.
    public class FirmwareLibraryTests : IDisposable
    {
        private readonly string _dir;

        private static readonly FirmwareRequest Request =
            new("TESTCORE", "WIDGET", "widget.rom", Size: 64, "a made-up chip's mask ROM");

        public FirmwareLibraryTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), $"wiseman_fw_{Guid.NewGuid():N}");
            Directory.CreateDirectory(_dir);
            FirmwareLibrary.Directory = _dir;
        }

        public void Dispose()
        {
            FirmwareLibrary.ResetDirectory();
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        private string WriteFile(string name, int size)
        {
            string path = Path.Combine(_dir, name);
            byte[] bytes = new byte[size];
            for (int i = 0; i < size; i++) bytes[i] = (byte)i;
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // --- Discovery ---

        [Fact]
        public void An_empty_library_reports_the_image_as_missing()
        {
            Assert.False(FirmwareLibrary.IsInstalled(Request));
            Assert.Null(FirmwareLibrary.TryLoad(Request));
        }

        [Fact]
        public void An_image_of_the_right_size_under_the_canonical_name_is_found()
        {
            WriteFile("widget.rom", 64);

            Assert.True(FirmwareLibrary.IsInstalled(Request));
            Assert.Equal(64, FirmwareLibrary.TryLoad(Request)!.Length);
        }

        [Theory]
        [InlineData(63)]
        [InlineData(65)]
        [InlineData(0)]
        public void An_image_of_the_wrong_size_counts_as_missing(int size)
        {
            WriteFile("widget.rom", size);

            // Loading it would run as garbage, which is far harder to
            // diagnose than a clean "not found".
            Assert.False(FirmwareLibrary.IsInstalled(Request));
        }

        [Fact]
        public void Alternate_names_are_checked_after_the_canonical_one()
        {
            var withAlternates = Request with { AlternateNames = new[] { "widget_alt.bin" } };
            WriteFile("widget_alt.bin", 64);

            Assert.True(FirmwareLibrary.IsInstalled(withAlternates));
            Assert.False(FirmwareLibrary.IsInstalled(Request));
        }

        [Fact]
        public void The_canonical_name_wins_over_an_alternate()
        {
            var withAlternates = Request with { AlternateNames = new[] { "widget_alt.bin" } };
            File.WriteAllBytes(Path.Combine(_dir, "widget.rom"), Enumerable.Repeat((byte)0xAA, 64).ToArray());
            File.WriteAllBytes(Path.Combine(_dir, "widget_alt.bin"), Enumerable.Repeat((byte)0xBB, 64).ToArray());

            Assert.Equal(0xAA, FirmwareLibrary.TryLoad(withAlternates)![0]);
        }

        [Fact]
        public void Missing_from_filters_a_batch_down_to_what_is_absent()
        {
            var other = new FirmwareRequest("TESTCORE", "GADGET", "gadget.rom", 32, "another chip");
            WriteFile("widget.rom", 64);

            var missing = FirmwareLibrary.MissingFrom(new[] { Request, other });

            Assert.Equal(new[] { other }, missing);
        }

        // --- Install ---

        [Fact]
        public void Installing_copies_the_users_file_under_the_canonical_name()
        {
            string source = Path.Combine(Path.GetTempPath(), $"picked_{Guid.NewGuid():N}.bin");
            File.WriteAllBytes(source, Enumerable.Repeat((byte)0x5A, 64).ToArray());

            Assert.True(FirmwareLibrary.Install(Request, source));

            // Found from now on without asking again.
            Assert.True(FirmwareLibrary.IsInstalled(Request));
            Assert.Equal(0x5A, FirmwareLibrary.TryLoad(Request)![0]);
            Assert.True(File.Exists(FirmwareLibrary.PathFor(Request)));
            File.Delete(source);
        }

        [Fact]
        public void Installing_the_wrong_file_fails_without_writing_anything()
        {
            string source = Path.Combine(Path.GetTempPath(), $"picked_{Guid.NewGuid():N}.bin");
            File.WriteAllBytes(source, new byte[10]);

            Assert.False(FirmwareLibrary.Install(Request, source));
            Assert.False(File.Exists(FirmwareLibrary.PathFor(Request)));
            File.Delete(source);
        }

        [Fact]
        public void Installing_a_file_that_does_not_exist_fails_quietly()
        {
            Assert.False(FirmwareLibrary.Install(Request, Path.Combine(_dir, "nope.rom")));
        }

        [Fact]
        public void Installing_creates_the_library_directory_if_it_is_gone()
        {
            Directory.Delete(_dir, recursive: true);
            string source = Path.Combine(Path.GetTempPath(), $"picked_{Guid.NewGuid():N}.bin");
            File.WriteAllBytes(source, new byte[64]);

            Assert.True(FirmwareLibrary.Install(Request, source));
            File.Delete(source);
        }

        [Fact]
        public void A_request_describes_itself_for_a_picker_title()
        {
            Assert.Equal("TESTCORE WIDGET firmware - widget.rom, 64 bytes", Request.Describe());
        }
    }
}
