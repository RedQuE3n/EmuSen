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
        // The table both 8-bit shims send: an unconditional patch fills all 256 entries, a compared one only its own.
        [Fact]
        public void The_rom_patch_table_flattens_try_patch_rom_within_its_bounds()
        {
            var cheats = new CheatRegistry();
            var (address, value) = NesGameGenieCodec.Decode("SXIOPO");
            cheats.AddRomPatch(address, value, NesGameGenieCodec.DecodeCompare("SXIOPO"), "six");
            cheats.AddRomPatch(0x9123, 0x42, 0x07, "compared");

            var (addresses, tables) = RomPatchTable.Build(cheats, 0x4020, 0xFFFF);
            Assert.Equal(addresses.Length * 256, tables.Length);
            int compared = Array.IndexOf(addresses, (ushort)0x9123);
            Assert.True(compared >= 0);
            for (int v = 0; v < 256; v++) Assert.Equal(v == 0x07 ? 0x142 : 0, tables[compared * 256 + v]);

            cheats.AddRomPatch(0x7FFF, 0x01, null, "last");
            var (low, _) = RomPatchTable.Build(cheats, 0, 0x7FFF);
            Assert.DoesNotContain((ushort)0x9123, low);
            Assert.Equal((ushort)0x7FFF, low[^1]);
            Assert.Empty(RomPatchTable.Build(new CheatRegistry(), 0, 0xFFFF).Addresses);
        }

        // One loader, three facades, and today's switches, crash-log names and reports kept exactly.
        [Fact]
        public void The_three_loaders_keep_their_switches_and_crash_logs()
        {
            Assert.Equal(("moonrt", "EMUSEN_MOON_NATIVE", "moonrt_crash"), (MoonNative.Library.Crate, MoonNative.Library.Variable, MoonNative.Library.CrashLogStem));
            Assert.Equal(("mercuryrt", "EMUSEN_MERCURY_NATIVE", "mercuryrt_crash"), (MercuryNative.Library.Crate, MercuryNative.Library.Variable, MercuryNative.Library.CrashLogStem));
            Assert.Equal(("marsrt", "EMUSEN_MARS_NATIVE", "native_crash"), (MarsNative.Library.Crate, MarsNative.Library.Variable, MarsNative.Library.CrashLogStem));
            Assert.True(MarsNative.Available, MarsNative.Report);
            Assert.NotEqual(0, MarsNative.Export("mars_machine_new"));
            Assert.Equal(0, MarsNative.Export("no_such_export"));

            var off = new NativeCoreLibrary("moonrt", "EMUSEN_CLEANUP_TEST_OFF", 2, "moon_interface_version", "moon_set_crash_log", "moonrt_crash");
            Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", "0");
            try
            {
                Assert.False(off.Available);
                Assert.Equal("turned off by EMUSEN_CLEANUP_TEST_OFF=0", off.Report);
                Assert.Equal(0, off.Export("moon_machine_new"));
            }
            finally { Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", null); }

            var wrong = new NativeCoreLibrary("moonrt", "EMUSEN_CLEANUP_TEST_UNSET", 999, "moon_interface_version", "moon_set_crash_log", "moonrt_crash");
            Assert.False(wrong.Available);
            Assert.EndsWith("this build 999", wrong.Report);
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
