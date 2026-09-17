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

        private static bool Installed(string name, out MarsBus? bus, out Cpu? cpu)
        {
            bus = null;
            cpu = null;

            string path = Path.Combine(N64TestRomLibrary.Root, name);
            if (!File.Exists(path)) return false;

            bus = new MarsBus();
            cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.Load(path));

            return true;
        }
    }
}
