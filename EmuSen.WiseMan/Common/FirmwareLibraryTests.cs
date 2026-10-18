using EmuSen.Common.Firmware;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Common
{
    // The core-agnostic firmware store: discovery, size validation, and what
    // happens to a file the player picks. Nothing here knows what an SNES is.
    // See EmuSen_Firmware.md §2.
    // Serial: FirmwareLibrary.Directory is a process global - §3.57.
    [Collection(TestCollections.ProcessGlobals)]
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

        // A file recognised by its content is named in the session's line by the player's own name - see EmuSen_Firmware.md §3.1 and §7.
        [Fact]
        public void The_session_line_names_a_file_recognised_by_its_content()
        {
            var entry = new EmuSen.Cores.Native.CoreFirmware(2, "widget.rom", "The widget's ROM", 64, false, []);
            var used = new[] { new EmuSen.Cores.Native.CoreFirmwareSource(2, "file") };
            var byName = new Dictionary<uint, FirmwareLocated> { [2] = new(_dir, ["widget.rom"], true) };
            var byContent = new Dictionary<uint, FirmwareLocated> { [2] = new(_dir, ["PSX - COPY.BIN"], true, ByContent: true) };
            Assert.Equal([$"[Firmware] The widget's ROM: the player's widget.rom, in {_dir}"], EmuSen.Cores.Native.CoreEngine.FirmwareLines("Core", used, [entry], byName));
            Assert.Equal([$"[Firmware] The widget's ROM: the player's PSX - COPY.BIN, recognised by its content, in {_dir}"], EmuSen.Cores.Native.CoreEngine.FirmwareLines("Core", used, [entry], byContent));
        }

        // Through the public calls, a renamed file of a listed checksum is found and loaded, and Install still writes only the canonical name.
        [Fact]
        public void A_renamed_image_is_found_by_its_checksum_through_the_public_calls()
        {
            string renamed = WriteFile("A PLAYER COPY.BIN", 64);
            var listed = Request with { Md5 = [Convert.ToHexStringLower(System.Security.Cryptography.MD5.HashData(File.ReadAllBytes(renamed)))] };
            Assert.Null(FirmwareLibrary.TryLoad(Request));
            Assert.Equal(File.ReadAllBytes(renamed), FirmwareLibrary.TryLoad(listed));
            Assert.True(FirmwareLibrary.IsInstalled(listed));
            Assert.Equal((true, "A PLAYER COPY.BIN"), (FirmwareLibrary.Find(listed)!.ByContent, FirmwareLibrary.Find(listed)!.Names[0]));
            Assert.Equal(["A PLAYER COPY.BIN"], Directory.GetFiles(_dir).Select(Path.GetFileName));
        }

        // --- Discovery ---

        // A file given as a symbolic link is measured by what it links to, which is what is read - see EmuSen_Backend.md §15.1.
        [Fact]
        public void An_image_given_as_a_link_is_measured_by_its_target()
        {
            string target = Path.Combine(Directory.CreateDirectory(Path.Combine(_dir, "elsewhere")).FullName, "dump.bin");
            File.WriteAllBytes(target, new byte[64]);
            File.CreateSymbolicLink(Path.Combine(_dir, "widget.rom"), target);
            Assert.Equal(64, FirmwareLibrary.TryLoad(Request)?.Length);
            Assert.Empty(FirmwareLibrary.MissingFrom([Request]));
        }

        // A link's own length is the length of the path it holds; one of 64 characters must not pass off a file of 37 bytes as the image.
        [Fact]
        public void A_link_whose_own_length_is_the_size_does_not_pass_off_another_size()
        {
            string name = new string('a', 60) + ".bin";
            File.WriteAllBytes(Path.Combine(_dir, name), new byte[37]);
            File.CreateSymbolicLink(Path.Combine(_dir, "widget.rom"), name);
            Assert.Null(FirmwareLibrary.Find(Request));
            Assert.Single(FirmwareLibrary.MissingFrom([Request]));
        }

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
