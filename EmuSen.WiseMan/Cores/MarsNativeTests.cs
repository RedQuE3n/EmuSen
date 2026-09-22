using EmuSen.Cores.Nintendo.Mars.Native;

namespace EmuSen.WiseMan.Cores
{
    // The Rust library is built by the core's build and loaded by it - see Mars_Native.md §1.
    public class MarsNativeTests
    {
        [Fact]
        public void The_native_library_is_built_beside_the_assemblies_and_speaks_this_builds_interface()
        {
            Assert.True(MarsNative.Available, MarsNative.Report);
            Assert.NotEqual(0, MarsNative.Export("emusen_native_interface_version"));
            Assert.Equal(0, MarsNative.Export("no_such_export"));
        }
    }
}
