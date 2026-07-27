using System.Collections.Generic;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.WiseMan.DianaOS
{
    // CoreCommand's own validation logic (unknown core name, missing
    // file, wrong extension) - pure string/file-existence checking, same
    // as EmuSen.Hotaru/Program.cs's own TryResolveCoreCommand, so it's
    // directly testable with no window/VenusCore needed. Content of the
    // "ROM" file doesn't matter here - CoreCommand only ever checks
    // File.Exists and the extension, never opens it.
    public class CoreCommandTests
    {
        private static IReadOnlyDictionary<string, CoreDescriptor> Registry() => new Dictionary<string, CoreDescriptor>(System.StringComparer.OrdinalIgnoreCase)
        {
            ["venus"] = new CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
            ["snes"] = new CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
        };

        private static string TempRomPath(string extension)
        {
            string path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"wiseman_core_{System.Guid.NewGuid():N}{extension}");
            System.IO.File.WriteAllBytes(path, new byte[16]);
            return path;
        }

        [Fact]
        public void Valid_core_and_extension_signals_LoadCore()
        {
            string path = TempRomPath(".smc");
            try
            {
                var command = new CoreCommand(Registry());

                DianaOSResult result = command.Execute(null, new[] { "core", "venus", path }, null);

                var loadCore = Assert.IsType<HostAction.LoadCore>(result.Action);
                Assert.Equal("venus", loadCore.CoreName);
                Assert.Equal(path, loadCore.RomPath);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }

        [Fact]
        public void Wrong_argument_count_reports_usage_and_no_action()
        {
            var command = new CoreCommand(Registry());

            DianaOSResult result = command.Execute(null, new[] { "core", "venus" }, null);

            Assert.Contains("Usage", result.Output);
            Assert.Null(result.Action);
        }

        [Fact]
        public void Unknown_core_name_fails_cleanly()
        {
            string path = TempRomPath(".smc");
            try
            {
                var command = new CoreCommand(Registry());

                DianaOSResult result = command.Execute(null, new[] { "core", "nes", path }, null);

                Assert.Contains("unknown core", result.Output);
                Assert.Null(result.Action);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }

        [Fact]
        public void Missing_file_fails_cleanly()
        {
            var command = new CoreCommand(Registry());

            DianaOSResult result = command.Execute(null, new[] { "core", "venus", "/tmp/does_not_exist_wiseman.smc" }, null);

            Assert.Contains("ROM not found", result.Output);
            Assert.Null(result.Action);
        }

        [Fact]
        public void Unsupported_extension_fails_cleanly()
        {
            string path = TempRomPath(".nes");
            try
            {
                var command = new CoreCommand(Registry());

                DianaOSResult result = command.Execute(null, new[] { "core", "venus", path }, null);

                Assert.Contains("not a supported ROM type", result.Output);
                Assert.Null(result.Action);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }

        [Fact]
        public void Reachable_through_a_real_interpreter_via_extraCommands()
        {
            string path = TempRomPath(".sfc");
            try
            {
                var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new CoreCommand(Registry()) });

                var result = shell.Submit($"core snes {path}");

                Assert.IsType<HostAction.LoadCore>(result.Action);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }
    }
}
