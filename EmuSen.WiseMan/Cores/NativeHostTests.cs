using EmuSen.Crystal.Systems.N64;
using EmuSen.WiseMan.Fixtures;
using System;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Crystal.Cheats;

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

            uint[] nes = CoreEngine.RomPatchTriples(cheats, 0x4020, 0xFFFF);
            Assert.Equal(new uint[] { 0x9123, 0x42, 0x07, 0x9123, 0x55, uint.MaxValue }, nes);
            uint[] gb = CoreEngine.RomPatchTriples(cheats, 0, 0x7FFF);
            Assert.Equal(new uint[] { 0x3FFF, 0x01, uint.MaxValue }, gb);
            Assert.Empty(CoreEngine.RomPatchTriples(new CheatRegistry(), 0, 0xFFFF));
        }

        // MercuryRT's pre-stable half, which no host loads since every shim on it retired, still answers the common interface until that interface goes.
        private const ushort MercuryCoreVersion = 5;
        private const ulong MercuryCapabilities = NativeInterface.Mutes | NativeInterface.RomPatches | NativeInterface.Debug | NativeInterface.DebugStack;
        private static readonly NativeCoreLibrary Mercury = NativeCoreLibrary.Common("mercuryrt", "EMUSEN_MERCURY_NATIVE", MercuryCoreVersion, MercuryCapabilities);

        // The loaders' switches and crash-log names; MercuryRT's pre-stable half on the common interface, every engine discovered from its sidecar since the shims retired, MarsRT's component exports gone.
        [Fact]
        public void The_three_loaders_keep_their_switches_and_crash_logs()
        {
            Assert.Equal(("mercuryrt", "EMUSEN_MERCURY_NATIVE", "mercuryrt_crash"), (Mercury.Crate, Mercury.Variable, Mercury.CrashLogStem));
            Assert.True(MarsRt.Available, MarsRt.Report);
            Assert.NotEqual(0, MarsRt.Export("emusen_core_create"));
            Assert.Equal(0, MarsRt.Export("mars_rsp_new"));
            Assert.Equal(0, MarsRt.Export("mars_machine_new"));
            Assert.Equal(0, MarsRt.Export("no_such_export"));

            var off = NativeCoreLibrary.Common("mercuryrt", "EMUSEN_CLEANUP_TEST_OFF", MercuryCoreVersion, MercuryCapabilities);
            Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", "0");
            try
            {
                Assert.False(off.Available);
                Assert.Equal("turned off by EMUSEN_CLEANUP_TEST_OFF=0", off.Report);
                Assert.Equal(0, off.Export("emusen_native_create"));
            }
            finally { Environment.SetEnvironmentVariable("EMUSEN_CLEANUP_TEST_OFF", null); }

            var wrong = NativeCoreLibrary.Common("mercuryrt", "EMUSEN_CLEANUP_TEST_UNSET", 999, MercuryCapabilities);
            Assert.False(wrong.Available);
            Assert.EndsWith($"speaks common interface 1 core {MercuryCoreVersion}, this build common 1 core 999", wrong.Report);
        }

        // A MercuryRT library that is not there is refused with the words the notice carries, and its capabilities agree with its exports.
        [Fact]
        public void MercuryRTs_library_is_on_the_common_interface_and_a_missing_one_is_refused()
        {
            var missing = NativeCoreLibrary.Common("mercuryrt_missing", "EMUSEN_MERCURYRT_MISSING_TEST", MercuryCoreVersion, MercuryCapabilities);
            Assert.False(missing.Available);
            Assert.Contains("not found beside the assemblies", missing.Report);

            Assert.True(Mercury.Available, Mercury.Report);
            Assert.Equal(MercuryCapabilities, Mercury.Capabilities);
            nint handle = System.Runtime.InteropServices.NativeLibrary.Load(System.IO.Path.Combine(AppContext.BaseDirectory, Mercury.FileName));
            foreach (string name in NativeInterface.Required) Assert.True(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, name, out _), name);
            foreach (var (bit, name, exports) in NativeInterface.Optional)
                foreach (string export in exports)
                    Assert.True(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, export, out _) == ((Mercury.Capabilities & bit) != 0), $"{name}: {export}");
            Assert.False(System.Runtime.InteropServices.NativeLibrary.TryGetExport(handle, "mercury_machine_new", out _), "the old mercury_* exports are retired");
        }

        // The shared status table and each console's band, in the words the shims used before.
        [Fact]
        public void Each_machine_describes_the_shared_codes_and_its_own()
        {
            Assert.Equal("not a Mercury save state", NativeMachine.Shared(-3, "Mercury"));
            Assert.Equal("not a Mars save state", MarsMachine.Describe(-3));
            Assert.Equal("the buffer is too small", MarsMachine.Describe(-7));
            Assert.Equal("an RDRAM size that is neither 4 MB nor 8 MB", MarsMachine.Describe(-12));
            Assert.Null(NativeMachine.Shared(-99, "Mercury"));
        }

        // MarsRT's state handle is a v1 machine: it writes either kind, has loaded nothing, and is the size it was asked for.
        [Fact]
        public void MarsRTs_state_handle_is_a_v1_machine_of_the_size_asked()
        {
            using var machine = new MarsMachine(4 * 1024 * 1024);
            Assert.Equal((0, 4 * 1024 * 1024), (machine.LoadedKind, machine.RdramBytes));
            Assert.NotEmpty(machine.Save(snapshot: false));
            Assert.NotEmpty(machine.Save(snapshot: true));
        }
    }
}
