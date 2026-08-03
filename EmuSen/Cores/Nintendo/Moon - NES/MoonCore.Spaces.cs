using System;

namespace EmuSen.Cores.Nintendo.Moon
{
    // The named address spaces cheats, the frame log and the debug target all share - see Moon_Core.md §4.
    public partial class MoonCore
    {
        public const string SpaceRam = "RAM";
        public const string SpacePrgRom = "PRGROM";
        public const string SpacePrgRam = "PRGRAM";
        public const string SpaceChr = "CHR";
        public const string SpaceCiram = "CIRAM";
        public const string SpaceOam = "OAM";
        public const string SpacePalette = "PALETTE";
        public const string SpaceCpuBus = "CPUBUS";

        public byte ReadSpace(string spaceName, int address)
        {
            if (Cart is null || Bus is null || Ppu is null) return 0;

            switch (spaceName)
            {
                case SpaceRam:
                    return Bus.Ram[address & 0x07FF];
                case SpacePrgRom:
                    return Cart.PrgRom.Length == 0 ? (byte)0 : Cart.PrgRom[address % Cart.PrgRom.Length];
                case SpacePrgRam:
                    return Cart.PrgRam[address & 0x1FFF];
                case SpaceChr:
                    return Cart.Chr.Length == 0 ? (byte)0 : Cart.Chr[address % Cart.Chr.Length];
                case SpaceCiram:
                    return Ppu.Ciram[address & 0x0FFF];
                case SpaceOam:
                    return Ppu.Oam[address & 0xFF];
                case SpacePalette:
                    return Ppu.PaletteRam[address & 0x1F];
                case SpaceCpuBus:
                    return Bus.Read((ushort)(address & 0xFFFF));
                default:
                    return 0;
            }
        }

        public void WriteSpace(string spaceName, int address, byte value)
        {
            if (Cart is null || Bus is null || Ppu is null) return;

            switch (spaceName)
            {
                case SpaceRam:
                    Bus.Ram[address & 0x07FF] = value;
                    break;
                case SpacePrgRam:
                    Cart.PrgRam[address & 0x1FFF] = value;
                    break;
                case SpaceChr:
                    // Writable only when the board shipped RAM here; CHR ROM silently refuses.
                    if (Cart.ChrIsRam && Cart.Chr.Length != 0) Cart.Chr[address % Cart.Chr.Length] = value;
                    break;
                case SpaceCiram:
                    Ppu.Ciram[address & 0x0FFF] = value;
                    break;
                case SpaceOam:
                    Ppu.Oam[address & 0xFF] = value;
                    break;
                case SpacePalette:
                    Ppu.PaletteRam[address & 0x1F] = value;
                    break;
                case SpaceCpuBus:
                    Bus.Write((ushort)(address & 0xFFFF), value);
                    break;
            }
        }

        public int SpaceSize(string spaceName) => spaceName switch
        {
            SpaceRam => Memory.MemoryBus.RamSize,
            SpacePrgRom => Cart?.PrgRom.Length ?? 0,
            SpacePrgRam => Cart?.PrgRam.Length ?? 0,
            SpaceChr => Cart?.Chr.Length ?? 0,
            SpaceCiram => 0x1000,
            SpaceOam => Video.Ppu.OamSize,
            SpacePalette => 0x20,
            SpaceCpuBus => 0x10000,
            _ => 0,
        };

        // Little-endian, matching how every 6502 address in memory is stored.
        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value |= (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        private byte ReadForCheat(string spaceName, int address) => ReadSpace(spaceName, address);

        private void WriteForCheat(string spaceName, int address, byte value) => WriteSpace(spaceName, address, value);
    }
}
