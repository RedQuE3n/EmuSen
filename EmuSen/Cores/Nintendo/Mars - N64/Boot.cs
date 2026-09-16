using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;

namespace EmuSen.Cores.Nintendo.Mars
{
    // What the PIF would have done, done by us instead - see Mars_Boot.md §1.
    public static class Boot
    {
        // The cartridge's own boot code, which the real machine copies before running it.
        public const int BootCodeLength = 0x1000;

        public const ulong EntryPoint = 0xFFFF_FFFF_A400_0040;
        public const ulong StackPointer = 0xFFFF_FFFF_A400_1FF0;

        public static void HandOff(MarsBus bus, Cpu.Core.Cpu cpu, RomImage rom)
        {
            bus.Cart = rom;

            for (int i = 0; i < BootCodeLength && i < rom.Rom.Length; i++) bus.SpDmem[i] = rom.Rom[i];

            cpu.Pc = EntryPoint;
            cpu.NextPc = EntryPoint + 4;

            // The register state the boot code is entitled to find - see Mars_Boot.md §2.
            cpu.Gpr[29] = StackPointer;
            cpu.Gpr[20] = rom.IsPal ? 0UL : 1UL;
            cpu.Gpr[22] = 0x3F;

            cpu.Cop0[Cpu.Core.Cpu.StatusRegister] = 0x3400_0000;
            cpu.Cop0[Cpu.Core.Cpu.CompareRegister] = 0xFFFF_FFFF;
            cpu.Cop0[15] = 0x0000_0B22;
            cpu.Cop0[16] = 0x0006_E463;
        }
    }
}
