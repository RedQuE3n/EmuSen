using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // XxdCommand (EmuSen.DianaOS/Commands/XxdCommand.cs) - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
    public class XxdCommandTests
    {
        private static DianaShellFixtures.ScratchDir TempDir() => new("WiseManXxdCommandTests");

        [Fact]
        public void Xxd_dumps_a_files_raw_bytes_with_address_hex_and_ascii_columns()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "sample.bin");
            File.WriteAllBytes(file, new byte[] { 0x41, 0x42, 0x43 }); // "ABC"
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"xxd \"{file}\"");

            Assert.Contains("000000:", result.Output);
            Assert.Contains("41 42 43", result.Output);
            Assert.Contains("ABC", result.Output);
        }

        [Fact]
        public void Xxd_does_not_corrupt_bytes_that_are_not_valid_utf8()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "binary.bin");
            byte[] raw = { 0x00, 0xFF, 0x80, 0xFE };
            File.WriteAllBytes(file, raw);
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"xxd \"{file}\"");

            Assert.Contains("00 FF 80 FE", result.Output);
        }

        [Fact]
        public void Xxd_on_an_empty_file_returns_empty_output()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "empty.bin");
            File.WriteAllBytes(file, Array.Empty<byte>());
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"xxd \"{file}\"");

            Assert.Equal("", result.Output);
        }

        [Fact]
        public void Xxd_on_a_missing_file_fails_cleanly()
        {
            using var dir = TempDir();
            string missing = Path.Combine(dir.Path, "missing.bin");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"xxd \"{missing}\"");

            Assert.Contains("no such file", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Xxd_rejects_a_path_outside_the_sandbox()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            // '..'-climbing, not a leading '/' one - see `man hier` on why a real OS absolute path no longer means "outside".
            string outside = string.Concat(Enumerable.Repeat("../", 15)) + "outside.bin";

            var result = shell.Submit($"xxd \"{outside}\"");

            Assert.Contains("outside the project sandbox", result.Output);
        }

        [Fact]
        public void Xxd_with_no_path_hexdumps_piped_stdin_as_utf8_bytes()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("echo hi | xxd");

            // "hi" -> 0x68 0x69, then the trailing space `echo` joins with
            // is part of the piped text too (single word here, so no
            // extra separator byte) - just confirm the two ASCII bytes
            // show up correctly, not the exact row padding.
            Assert.Contains("68 69", result.Output);
            Assert.Contains("hi", result.Output);
        }

        [Fact]
        public void Hexdump_is_a_plain_alias_for_xxd()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "sample.bin");
            File.WriteAllBytes(file, new byte[] { 0x41, 0x42, 0x43 });
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal(shell.Submit($"xxd \"{file}\"").Output, shell.Submit($"hexdump \"{file}\"").Output);
        }
    }
}
