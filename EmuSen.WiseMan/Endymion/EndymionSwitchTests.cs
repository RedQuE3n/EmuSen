using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Endymion
{
    // Endymion's switch in state 2, made as the process makes it over a folder with no library: the C#, said once to the sink and the log - see EmuSen_RustPlatform.md §19.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class EndymionSwitchTests : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenEndymionSwitch_" + Guid.NewGuid().ToString("N"));
        private readonly List<string> _reported = new();
        private readonly string? _logOverride = ErrorLog.DirectoryOverride;

        public EndymionSwitchTests()
        {
            Directory.CreateDirectory(_temp);
            ConfigDiagnostics.Reset();
            ConfigDiagnostics.Sink = _reported.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = null;
            ConfigDiagnostics.Reset();
            ErrorLog.DirectoryOverride = _logOverride;
            if (Directory.Exists(_temp)) Directory.Delete(_temp, recursive: true);
        }

        [Fact]
        public void A_missing_library_leaves_endymion_on_the_csharp_and_says_so_once_in_the_log()
        {
            string logs = Path.Combine(_temp, "logs");
            ErrorLog.DirectoryOverride = logs;
            string empty = Directory.CreateDirectory(Path.Combine(_temp, "no library")).FullName;
            int loads = 0;
            foreach (string? variable in new[] { null, "1", "yes", "0" })
            {
                PlatformSwitch chosen = EndymionNative.SwitchFor(() => variable, () => { loads++; var opened = PlatformLibrary.Open(empty); return (opened.Handle != 0, opened.Report); });
                Assert.All(Enumerable.Range(0, 5), _ => Assert.False(chosen.Active));
            }
            // Asked only where the variable is not 0, and once each however often the switch is asked.
            Assert.Equal(3, loads);
            string expected = $"{PlatformLibrary.FileName} is not in use ({PlatformLibrary.FileName} not found beside the assemblies); Endymion runs on its C# implementation.";
            Assert.Equal(new[] { expected, expected, expected }, _reported);
            string written = string.Concat(Directory.GetFiles(logs).Select(File.ReadAllText));
            Assert.Equal(3, written.Split(expected).Length - 1);
            Assert.Contains("[platform]", written);

            // A load that throws is a fallback too, never a failure to start.
            _reported.Clear();
            PlatformSwitch thrown = EndymionNative.SwitchFor(() => null, () => throw new DllNotFoundException("refused"));
            Assert.False(thrown.Active);
            Assert.Equal(new[] { $"{PlatformLibrary.FileName} is not in use (DllNotFoundException: refused); Endymion runs on its C# implementation." }, _reported);
        }

        [Fact]
        public void A_library_that_loads_is_chosen_unless_the_variable_is_0()
        {
            foreach (string? variable in new[] { null, "1", "true", "0" })
            {
                PlatformSwitch chosen = EndymionNative.SwitchFor(() => variable, () => (true, ""));
                Assert.Equal(variable != "0", chosen.Active);
            }
            Assert.Empty(_reported);
        }
    }
}
