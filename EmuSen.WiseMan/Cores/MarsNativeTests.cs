using EmuSen.Crystal.Systems.N64;
using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The Rust library is built by the core's build and loaded by it - see Mars_Native.md §1.
    public class MarsNativeTests
    {
        [Fact]
        public void The_native_library_is_built_beside_the_assemblies_and_speaks_this_builds_interface()
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            Assert.NotEqual(0, MarsRt.Export("emusen_core_abi_version"));
            Assert.Equal(0, MarsRt.Export("no_such_export"));
        }
    }
}
