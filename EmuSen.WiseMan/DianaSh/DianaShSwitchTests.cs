using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // DianaSh's switch in state 2 and each way it fails over to the C#: no library, a refused library, a shell that cannot be made - see EmuSen_DianaSh.md §13.10.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class DianaShSwitchTests : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenDianaShSwitch_" + Guid.NewGuid().ToString("N"));
        private readonly List<string> _reported = new();
        private readonly string? _logOverride = ErrorLog.DirectoryOverride;

        public DianaShSwitchTests()
        {
            Directory.CreateDirectory(_temp);
            ConfigDiagnostics.Reset();
            ConfigDiagnostics.Sink = _reported.Add;
            ErrorLog.DirectoryOverride = Path.Combine(_temp, "logs");
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = null;
            ConfigDiagnostics.Reset();
            ErrorLog.DirectoryOverride = _logOverride;
            if (Directory.Exists(_temp)) Directory.Delete(_temp, recursive: true);
        }

        private static string Line(string why) => $"{PlatformLibrary.FileName} is not in use ({why}); DianaSh runs on its C# implementation.";

        private string Logged() => string.Concat(Directory.GetFiles(Path.Combine(_temp, "logs")).Select(File.ReadAllText));

        // The switch over a load of the library's exports through a resolver, as the process makes it over the real one.
        private static PlatformSwitch Over(string? variable, Func<string, nint> export) =>
            DianaShNative.SwitchFor(() => variable, () => DianaShNative.Check(export, out _) is { } refused ? (false, refused) : (true, ""));

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static uint MajorTwo() => 2u << 16;

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint NoShell(void* config) => 0;

        [Fact]
        public void A_missing_library_leaves_the_csharp_shell_and_says_so_once()
        {
            string empty = Directory.CreateDirectory(Path.Combine(_temp, "no library")).FullName;
            int loads = 0;
            foreach (string? variable in new[] { null, "1", "0" })
            {
                PlatformSwitch chosen = DianaShNative.SwitchFor(() => variable, () => { loads++; var opened = PlatformLibrary.Open(empty); return (opened.Handle != 0, opened.Report); });
                Assert.All(Enumerable.Range(0, 3), _ => Assert.False(chosen.Active));
            }
            // Asked only where the variable is not 0, and once however often the switch is asked.
            Assert.Equal(2, loads);
            string expected = Line($"{PlatformLibrary.FileName} not found beside the assemblies");
            Assert.Equal(new[] { expected, expected }, _reported);
            Assert.Equal(2, Logged().Split(expected).Length - 1);
        }

        [Fact]
        public void A_library_of_another_major_is_refused_with_its_version()
        {
            if (!DianaShNative.Ready) return;
            nint major = (nint)(delegate* unmanaged[Cdecl]<uint>)&MajorTwo;
            PlatformSwitch chosen = Over(null, name => name == "dianash_abi_version" ? major : PlatformLibrary.Export(name));
            Assert.False(chosen.Active);
            Assert.Equal(new[] { Line($"{PlatformLibrary.FileName} speaks DianaSh host API 2.0, this build 1.x") }, _reported);
        }

        [Fact]
        public void A_library_missing_an_export_is_refused_with_its_name()
        {
            if (!DianaShNative.Ready) return;
            PlatformSwitch chosen = Over(null, name => name == "dianash_session_replace" ? 0 : PlatformLibrary.Export(name));
            Assert.False(chosen.Active);
            Assert.Equal(new[] { Line($"{PlatformLibrary.FileName} has no dianash_session_replace") }, _reported);
        }

        [Fact]
        public void A_library_that_cannot_make_a_shell_is_refused()
        {
            if (!DianaShNative.Ready) return;
            nint none = (nint)(delegate* unmanaged[Cdecl]<void*, nint>)&NoShell;
            PlatformSwitch chosen = Over("1", name => name == "dianash_new" ? none : PlatformLibrary.Export(name));
            Assert.False(chosen.Active);
            Assert.Equal(new[] { Line($"{PlatformLibrary.FileName} could not make a DianaSh shell") }, _reported);
            Assert.Contains("[platform]", Logged());
        }

        [Fact]
        public void The_library_is_chosen_unless_the_variable_is_0()
        {
            if (!DianaShNative.Ready) return;
            foreach (string? variable in new[] { null, "1", "yes", "0" })
                Assert.Equal(variable != "0", Over(variable, PlatformLibrary.Export).Active);
            Assert.Empty(_reported);
        }
    }
}
