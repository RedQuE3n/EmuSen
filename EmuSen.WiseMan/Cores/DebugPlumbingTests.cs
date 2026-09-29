using System;
using System.Linq;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.Cores
{
    // One debugger memory space for every core, and one N64 address map for both engines - see EmuSen_Settings_Reference.md §4.85.7.
    public class DebugPlumbingTests
    {
        // Wrapping both ways, negatives included, a live size, and zero for an empty space.
        [Fact]
        public void A_debug_space_wraps_reads_and_writes_modulo_its_live_size()
        {
            var data = new byte[8];
            int size = 8;
            var space = new DelegateDebugMemorySpace("T", () => size, a => data[a], (a, v) => data[a] = v);

            space.Write(9, 0x11);
            space.Write(-1, 0x22);
            Assert.Equal(0x11, data[1]);
            Assert.Equal(0x22, data[7]);
            Assert.Equal(0x22, space.Read(15));

            size = 4;
            Assert.Equal(4, space.Size);
            Assert.Equal(data[1], space.Read(5));

            size = 0;
            Assert.Equal(data[0], space.Read(123));
            Assert.True(space.IsWritable);
        }

        // A bus or a processor's view: every address as given.
        [Fact]
        public void A_bus_space_passes_every_address_through_and_one_without_a_write_is_read_only()
        {
            int seen = 0;
            var bus = new DelegateDebugMemorySpace("B", 16, a => { seen = a; return 0; }, wrap: false);
            bus.Read(-5);
            Assert.Equal(-5, seen);
            bus.Read(1000);
            Assert.Equal(1000, seen);
            Assert.False(bus.IsWritable);

            var empty = DelegateDebugMemorySpace.Over("E", Array.Empty<byte>());
            Assert.Equal(0, empty.Read(3));
            empty.Write(3, 1);
            Assert.Equal(0, empty.Size);
        }

        // Venus's arrays used to drop a write past the end and wrap a read there; both wrap now, as on every other core.
        [Fact]
        public void A_venus_write_past_the_end_of_an_array_space_wraps_as_its_read_does()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = (IDebugTarget)EmuSen.Cores.CoreFactory.Bundle(core).DebugTarget;
            IDebugMemorySpace wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            wram.Write(wram.Size + 5, 0x5A);
            Assert.Equal(0x5A, core.Bus!.Ram[5]);
            Assert.Equal(0x5A, wram.Read(-wram.Size + 5));
        }

        // One map of physical addresses for both N64 engines.
        [Fact]
        public void Resolve_names_the_same_memory_for_either_engine()
        {
            Assert.Equal(MarsDebugSpaces.Rdram, MarsDebugSpaces.Resolve(0x100, 8 * 1024 * 1024)!.Value.Space);
            Assert.False(MarsDebugSpaces.Resolve(0x0050_0000, 4 * 1024 * 1024)!.Value.IsAddressable);
            Assert.Equal(MarsDebugSpaces.Imem, MarsDebugSpaces.Resolve(0x0400_1010, 4 * 1024 * 1024)!.Value.Space);
        }
    }
}
