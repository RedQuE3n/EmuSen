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

        // Where IPL2 was running in IMEM when it jumped to IPL3 - see Mars_Boot.md §6.3.
        public const ulong Ipl2ReturnAddress = 0xFFFF_FFFF_A400_1550;

        // IPL2's first eight words, still in IMEM when IPL3 starts; the 6105's IPL3 decrypts with them - see Mars_Boot.md §6.4.
        public static readonly uint[] Ipl2Head =
        {
            0x3C0DBFC0, 0x8DA807FC, 0x25AD07C0, 0x31080080, 0x5500FFFC, 0x3C0DBFC0, 0x8DA80024, 0x3C0BB000,
        };

        public static void HandOff(MemoryBus bus, Cpu.Core.Cpu cpu, RomImage rom)
        {
            bus.Cart = rom;

            for (int i = 0; i < BootCodeLength && i < rom.Rom.Length; i++) bus.SpDmem[i] = rom.Rom[i];

            for (int i = 0; i < Ipl2Head.Length; i++)
            {
                uint word = Ipl2Head[i];
                bus.SpImem[i * 4] = (byte)(word >> 24);
                bus.SpImem[i * 4 + 1] = (byte)(word >> 16);
                bus.SpImem[i * 4 + 2] = (byte)(word >> 8);
                bus.SpImem[i * 4 + 3] = (byte)word;
            }

            cpu.Pc = EntryPoint;
            cpu.NextPc = EntryPoint + 4;

            // The register state the boot code is entitled to find - see Mars_Boot.md §2 and §6.
            cpu.Gpr[11] = EntryPoint;
            cpu.Gpr[29] = StackPointer;
            cpu.Gpr[31] = Ipl2ReturnAddress;
            cpu.Gpr[20] = rom.IsPal ? 0UL : 1UL;
            cpu.Gpr[22] = Cic.Seed(rom.CicChip);

            cpu.Cop0[Cpu.Core.Cpu.StatusRegister] = 0x3400_0000;
            cpu.Cop0[Cpu.Core.Cpu.CompareRegister] = 0xFFFF_FFFF;
            cpu.Cop0[Cpu.Core.Cpu.ProcessorIdRegister] = Cpu.Core.Cpu.ProcessorId;
            cpu.Cop0[Cpu.Core.Cpu.ConfigRegister] = Cpu.Core.Cpu.ConfigAtReset;
            cpu.Cop0Written();
        }
    }
}
