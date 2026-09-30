using System;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon.Cheats;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.Cores
{
    // The first increment of the generic native host: one loader, one handle, one patch table - see EmuSen_Settings_Reference.md §4.85.7.
    public class NativeHostTests
    {
        // The triples both 8-bit shims send, in ResolveRomPatches' order and within the console's range; no compare is uint.MaxValue.
        [Fact]
        public void The_rom_patch_triples_keep_the_registrys_order_within_their_bounds()
        {
            var cheats = new CheatRegistry();
            cheats.AddRomPatch(0x9123, 0x42, 0x07, "compared");
            cheats.AddRomPatch(0x9123, 0x55, null, "later");
            cheats.AddRomPatch(0x3FFF, 0x01, null, "below");

            uint[] nes = NativeRtCore<MoonMachine>.RomPatchTriples(cheats, 0x4020, 0xFFFF);
            Assert.Equal(new uint[] { 0x9123, 0x42, 0x07, 0x9123, 0x55, uint.MaxValue }, nes);
            uint[] gb = NativeRtCore<MoonMachine>.RomPatchTriples(cheats, 0, 0x7FFF);
            Assert.Equal(new uint[] { 0x3FFF, 0x01, uint.MaxValue }, gb);
            Assert.Empty(NativeRtCore<MoonMachine>.RomPatchTriples(new CheatRegistry(), 0, 0xFFFF));
        }

        // One loader, three facades, and their switches and crash-log names; MoonRT and MercuryRT on the common interface, MarsRT on its own until its step.
        [Fact]
        public void The_three_loaders_keep_their_switches_and_crash_logs()
        {
            Assert.Equal(("moonrt", "EMUSEN_MOON_NATIVE", "moonrt_crash"), (MoonNative.Library.Crate, MoonNative.Library.Variable, MoonNative.Library.CrashLogStem));
            Assert.Equal(("mercuryrt", "EMUSEN_MERCURY_NATIVE", "mercuryrt_crash"), (MercuryNative.Library.Crate, MercuryNative.Library.Variable, MercuryNative.Library.CrashLogStem));
            Assert.Equal(("marsrt", "EMUSEN_MARS_NATIVE", "native_crash"), (MarsNative.Library.Crate, MarsNative.Library.Variable, MarsNative.Library.CrashLogStem));
            Assert.True(MarsNative.Available, MarsNative.Report);
            Assert.NotEqual(0, MarsNative.Export("mars_machine_new"));
            Assert.Equal(0, MarsNative.Export("no_such_export"));

            var off = NativeCoreLibrary.Common("moonrt", "EMUSEN_CLEANUP_TEST_OFF", MoonNative.CoreVersion, MoonNative.RequiredCapabilities);
            Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", "0");
            try
            {
                Assert.False(off.Available);
                Assert.Equal("turned off by EMUSEN_CLEANUP_TEST_OFF=0", off.Report);
                Assert.Equal(0, off.Export("emusen_native_create"));
            }
            finally { Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", null); }

            var wrong = NativeCoreLibrary.Common("moonrt", "EMUSEN_CLEANUP_TEST_UNSET", 999, MoonNative.RequiredCapabilities);
            Assert.False(wrong.Available);
            Assert.EndsWith($"speaks common interface 1 core {MoonNative.CoreVersion}, this build common 1 core 999", wrong.Report);
        }

        // A MercuryRT library that is not there is refused with the words the notice carries, and its capabilities agree with its exports.
        [Fact]
        public void MercuryRTs_library_is_on_the_common_interface_and_a_missing_one_is_refused()
        {
            var missing = NativeCoreLibrary.Common("mercuryrt_missing", "EMUSEN_MERCURYRT_MISSING_TEST", MercuryNative.CoreVersion, MercuryNative.RequiredCapabilities);
            Assert.False(missing.Available);
            Assert.Contains("not found beside the assemblies", missing.Report);

            Assert.True(MercuryNative.Available, MercuryNative.Report);
            Assert.Equal(MercuryNative.RequiredCapabilities, MercuryNative.Library.Capabilities);
            nint handle = System.Runtime.InteropServices.NativeLibrary.Load(System.IO.Path.Combine(AppContext.BaseDirectory, MercuryNative.Library.FileName));
            foreach (string name in NativeInterface.Required) Assert.True(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, name, out _), name);
            foreach (var (bit, name, exports) in NativeInterface.Optional)
                foreach (string export in exports)
                    Assert.True(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, export, out _) == ((MercuryNative.Library.Capabilities & bit) != 0), $"{name}: {export}");
            Assert.False(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, "mercury_machine_new", out _), "the old mercury_* exports are retired");
        }

        // The shared status table and each console's band, in the words the shims used before.
        [Fact]
        public void Each_machine_describes_the_shared_codes_and_its_own()
        {
            Assert.Equal("not a Moon save state", MoonMachine.Describe(-3));
            Assert.Equal("not a Mercury save state", MercuryMachine.Describe(-3));
            Assert.Equal("not a Mars save state", MarsMachine.Describe(-3));
            Assert.Equal("the buffer is too small", MarsMachine.Describe(-7));
            Assert.Equal("a mapper no board implements", MoonMachine.Describe(-10));
            Assert.Equal("a cartridge type no board implements", MercuryMachine.Describe(-10));
            Assert.Equal("an RDRAM size that is neither 4 MB nor 8 MB", MarsMachine.Describe(-12));
            Assert.Equal("status -99", MoonMachine.Describe(-99));
        }

        // MarsRT's handle has only its lifecycle in the shared table, so the 8-bit calls refuse rather than call a function of another shape.
        [Fact]
        public void MarsRTs_handle_refuses_the_calls_its_library_shapes_differently()
        {
            using var machine = new MarsMachine(4 * 1024 * 1024);
            Assert.Throws<NotSupportedException>(() => machine.TotalFrames);
            Assert.Throws<NotSupportedException>(() => machine.Save());
            Assert.NotEmpty(machine.Save(snapshot: false));
        }
    }
}
