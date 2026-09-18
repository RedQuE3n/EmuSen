using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Commercial cartridges, run headless as far as a machine with no RCP can take them - see Mars_FpuMath.md §10.
    public class MarsCommercialRomTests
    {
        private const string Mario = "Super Mario 64 (Europe) (En,Fr,De).z64";
        private const string WaveRace = "Wave Race 64 (USA) (Rev A).z64";
        private const string Ocarina = "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A).z64";

        private const int Budget = 2_000_000;


        // Neither game is playable and neither is meant to be: what is asserted is that nothing faults.
        [Theory]
        [InlineData(Mario)]
        [InlineData(WaveRace)]
        public void A_commercial_cartridge_runs_its_own_code_without_faulting(string name)
        {
            if (!Installed(name, out var bus, out var cpu)) return;

            cpu!.Run(Budget);

            // The timer keeps firing, so an interrupt is expected; anything else is a defect.
            Assert.True(cpu.LastException is null or { Code: ExceptionCode.Interrupt },
                $"{name} faulted with {cpu.LastException?.Code} at {cpu.LastException?.Address:X16}");

            Assert.Equal(4UL, (cpu.CurrentPc >> 29) & 7);
        }

        // Booted and waiting for a video interrupt nothing raises; the loop keeps tightening, so only its size bound is asserted - see Mars_FpuMath.md §10.
        [Fact]
        public void Super_Mario_64_settles_into_a_wait_loop_for_hardware_that_does_not_exist()
        {
            if (!Installed(Mario, out var bus, out var cpu)) return;

            cpu!.Run(Budget);

            var visited = new HashSet<ulong>();
            for (int i = 0; i < 4_000; i++)
            {
                cpu.Step();
                visited.Add(cpu.CurrentPc);
            }

            Assert.InRange(visited.Count, 1, 64);
        }

        // A 6105 cartridge, whose IPL3 reads t3 and ra before writing them and checksums the game from the 0x91 seed - see Mars_Boot.md §8.
        [Fact]
        public void Ocarina_of_Time_passes_its_own_boot_code_and_reaches_its_entry_point()
        {
            if (!Installed(Ocarina, out var bus, out var cpu)) return;

            RomImage rom = bus!.Cart!;
            Assert.Equal(CicChip.Nus6105, rom.CicChip);

            ulong entry = 0xFFFF_FFFF_0000_0000 | rom.EntryPoint;
            for (int i = 0; i < 10_000_000 && cpu!.CurrentPc != entry; i++)
            {
                cpu.Step();
                Assert.True(cpu.LastException is null, $"IPL3 faulted with {cpu.LastException?.Code} at {cpu.CurrentPc:X16}");
            }

            Assert.Equal(entry, cpu!.CurrentPc);

            // The game is where IPL3 loaded it, so the processor did not merely slide there through empty memory.
            uint first = (uint)((rom.Rom[0x1000] << 24) | (rom.Rom[0x1001] << 16) | (rom.Rom[0x1002] << 8) | rom.Rom[0x1003]);
            Assert.Equal(first, bus.Read32(rom.EntryPoint & 0x1FFF_FFFF));
        }

        private static bool Installed(string name, out MemoryBus? bus, out Cpu? cpu)
        {
            bus = null;
            cpu = null;

            string path = Path.Combine(N64TestRomLibrary.Root, name);
            if (!File.Exists(path)) return false;

            bus = new MemoryBus();
            cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.Load(path));

            return true;
        }
    }
}
