using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // A leftover EMUSEN_<NAME>_NATIVE=0 after a component's C# half was deleted: ignored, and said once in one line - see EmuSen_Backend.md §40.
    [Collection(TestCollections.ProcessGlobals)]
    public unsafe class RetiredSwitchTests : IDisposable
    {
        private readonly string _logs = Path.Combine(Path.GetTempPath(), "EmuSenRetiredSwitch_" + Guid.NewGuid().ToString("N"));
        private readonly string? _logOverride = ErrorLog.DirectoryOverride;
        private readonly Action<string>? _sink = ConfigDiagnostics.Sink;
        private readonly List<string> _said = new();

        public RetiredSwitchTests()
        {
            ErrorLog.DirectoryOverride = _logs;
            ConfigDiagnostics.Sink = _said.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = _sink;
            ErrorLog.DirectoryOverride = _logOverride;
            Environment.SetEnvironmentVariable("EMUSEN_RETIREDTEST_NATIVE", null);
            Environment.SetEnvironmentVariable("EMUSEN_RETIREDTEST2_NATIVE", null);
            Environment.SetEnvironmentVariable(DianaShNative.Variable, null);
            try { Directory.Delete(_logs, recursive: true); } catch (IOException) { }
        }

        [Fact]
        public void A_leftover_zero_is_said_once_and_any_other_value_not_at_all()
        {
            Environment.SetEnvironmentVariable("EMUSEN_RETIREDTEST2_NATIVE", "1");
            RetiredSwitch.Note("EMUSEN_RETIREDTEST2_NATIVE", "the test's component");
            Assert.Empty(_said);
            Environment.SetEnvironmentVariable("EMUSEN_RETIREDTEST_NATIVE", "0");
            RetiredSwitch.Note("EMUSEN_RETIREDTEST_NATIVE", "the test's component");
            RetiredSwitch.Note("EMUSEN_RETIREDTEST_NATIVE", "the test's component");
            string line = Assert.Single(_said);
            Assert.Equal($"EMUSEN_RETIREDTEST_NATIVE=0 is ignored: the test's component has had no C# implementation since 2026-11-04 and runs on {PlatformLibrary.FileName}.", line);
        }

        [Fact]
        public void A_component_retired_later_says_its_own_date()
        {
            Environment.SetEnvironmentVariable("EMUSEN_RETIREDTEST2_NATIVE", "0");
            RetiredSwitch.Note("EMUSEN_RETIREDTEST2_NATIVE", "Endymion", "2026-11-05");
            Assert.Equal($"EMUSEN_RETIREDTEST2_NATIVE=0 is ignored: Endymion has had no C# implementation since 2026-11-05 and runs on {PlatformLibrary.FileName}.", Assert.Single(_said));
        }

        [Fact]
        public void Endymion_without_its_library_or_sdl_refuses_in_words()
        {
            Assert.Equal($"{PlatformLibrary.FileName} is not in use for Endymion (not found); Endymion has had no C# implementation since 2026-11-05.", EndymionNative.Refusal("not found"));
            Assert.True(DeviceNative.Ready, DeviceNative.Report);
            string? why = DeviceNative.Lend("EmuSenNoSuchSdl3");
            Assert.Equal("EmuSenNoSuchSdl3 could not be loaded", why);
            Assert.Equal($"{PlatformLibrary.FileName} could not take this process's SDL (EmuSenNoSuchSdl3 could not be loaded); Endymion's pads and sound have had no C# implementation since 2026-11-05.", DeviceNative.Refusal(why!));
        }

        [Fact]
        public void Serenity_without_its_library_refuses_in_words()
        {
            Assert.Equal($"{PlatformLibrary.FileName} is not in use for Serenity (not found); Serenity's slang half has had no C# implementation since 2026-11-05.", EmuSen.Serenity.Native.SerenityNative.Refusal("not found"));
            Assert.True(EmuSen.Serenity.Native.SerenityNative.Ready, EmuSen.Serenity.Native.SerenityNative.Report);
        }

        [Fact]
        public void The_library_without_its_platform_library_refuses_in_words()
        {
            Assert.Equal($"{PlatformLibrary.FileName} is not in use for the library (not found); the library has had no C# implementation since 2026-11-05.", EmuSen.Crystal.Client.Library.GameLibrary.Refusal("not found"));
            Assert.True(EmuSen.Crystal.Client.Library.GameLibrary.Ready, EmuSen.Crystal.Client.Library.GameLibrary.Report);
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static uint MajorTwo() => 2u << 16;

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint NoShell(void* config) => 0;

        [Fact]
        public void DianaSh_without_its_library_refuses_in_words_naming_why()
        {
            Assert.Equal($"{PlatformLibrary.FileName} is not in use for DianaSh (not found); DianaSh's shell has had no C# implementation since 2026-11-06.", DianaShNative.Refusal("not found"));
            Assert.True(DianaShNative.Ready, DianaShNative.Report);
            nint major = (nint)(delegate* unmanaged[Cdecl]<uint>)&MajorTwo;
            nint none = (nint)(delegate* unmanaged[Cdecl]<void*, nint>)&NoShell;
            Assert.Equal($"{PlatformLibrary.FileName} speaks DianaSh host API 2.0, this build 1.x", DianaShNative.Check(n => n == "dianash_abi_version" ? major : PlatformLibrary.Export(n), out _));
            Assert.Equal($"{PlatformLibrary.FileName} has no dianash_session_replace", DianaShNative.Check(n => n == "dianash_session_replace" ? 0 : PlatformLibrary.Export(n), out _));
            Assert.Equal($"{PlatformLibrary.FileName} could not make a DianaSh shell", DianaShNative.Check(n => n == "dianash_new" ? none : PlatformLibrary.Export(n), out _));
            Assert.Null(DianaShNative.Check(PlatformLibrary.Export, out _));
        }

        [Fact]
        public void A_leftover_dianash_zero_still_makes_the_rust_shell_and_is_said_once()
        {
            Environment.SetEnvironmentVariable(DianaShNative.Variable, "0");
            var shell = EmuSen.DianaSh.DianaSh.Bin.DianaShInterpreter.CreateDefault(null);
            EmuSen.DianaSh.DianaSh.Bin.DianaShInterpreter.CreateDefault(null);
            Assert.Equal("hello", shell.Submit("echo hello").Output);
            Assert.Equal($"{DianaShNative.Variable}=0 is ignored: DianaSh has had no C# implementation since 2026-11-06 and runs on {PlatformLibrary.FileName}.", Assert.Single(_said));
        }
    }
}
