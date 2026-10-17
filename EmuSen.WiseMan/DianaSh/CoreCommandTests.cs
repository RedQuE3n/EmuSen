using EmuSen.Cores;
using EmuSen.Crystal.Debugger;
using System.Collections.Generic;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Systems;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;

namespace EmuSen.WiseMan.DianaSh
{
    // CoreCommand's own validation logic (unknown core name, missing
    // file, wrong extension) - pure string/file-existence checking, same
    // as EmuSen.Shell/Program.cs's own TryResolveCoreCommand, so it's
    // directly testable with no window/ICore needed. Content of the
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
            System.IO.Directory.CreateDirectory(DianaShSandbox.ScratchDirectory);
            string path = System.IO.Path.Combine(DianaShSandbox.ScratchDirectory, $"wiseman_core_{System.Guid.NewGuid():N}{extension}");
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

                DianaShResult result = command.Execute(null, new[] { "core", "venus", path }, null);

                var loadCore = Assert.IsType<HostAction.LoadCore>(result.Action);
                Assert.Equal("venus", loadCore.CoreName);
                Assert.Equal(path, loadCore.RomPath);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }

        // `core <name> --console` asks the host to start the console empty, where the host says it can - see `man core`.
        [Fact]
        public void Console_flag_signals_StartConsole_only_for_a_console_that_starts_empty()
        {
            var registry = new Dictionary<string, CoreDescriptor>(System.StringComparer.OrdinalIgnoreCase)
            {
                ["playstation"] = new CoreDescriptor("PlayStation", new[] { ".cue" }),
                ["venus"] = new CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
            };
            var command = new CoreCommand(registry, console => console == "PlayStation");

            var start = Assert.IsType<HostAction.StartConsole>(command.Execute(null, new[] { "core", "playstation", "--console" }, null).Action);
            Assert.Equal(("playstation", "PlayStation"), (start.CoreName, start.Console));

            DianaShResult refused = command.Execute(null, new[] { "core", "venus", "--console" }, null);
            Assert.Null(refused.Action);
            Assert.Contains("no core in this build starts the SNES (Venus) with nothing in it", refused.Output);
            Assert.Null(new CoreCommand(registry).Execute(null, new[] { "core", "playstation", "--console" }, null).Action);
        }

        [Fact]
        public void Wrong_argument_count_reports_usage_and_no_action()
        {
            var command = new CoreCommand(Registry());

            DianaShResult result = command.Execute(null, new[] { "core", "venus" }, null);

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

                DianaShResult result = command.Execute(null, new[] { "core", "nes", path }, null);

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

            DianaShResult result = command.Execute(null, new[] { "core", "venus", "/tmp/does_not_exist_wiseman.smc" }, null);

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

                DianaShResult result = command.Execute(null, new[] { "core", "venus", path }, null);

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
                var shell = DianaShInterpreter.CreateDefault(null, new IDianaShCommand[] { new CoreCommand(Registry()) });

                var result = shell.Submit($"core snes \"{path}\"");

                Assert.True(result.Action is HostAction.LoadCore, result.Output);
            }
            finally
            {
                System.IO.File.Delete(path);
            }
        }
    }
}
