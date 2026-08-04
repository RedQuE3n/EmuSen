using System;

namespace EmuSen.Cores.Nintendo.Mercury
{
    // The named windows `mem`, `watch` and the cheat engine address - see Mercury_Memory.md §8.
    public sealed partial class MercuryCore
    {
        public const string SpaceRom = "ROM";
        public const string SpaceVram = "VRAM";
        public const string SpaceCartRam = "CARTRAM";
        public const string SpaceWram = "WRAM";
        public const string SpaceOam = "OAM";
        public const string SpaceHram = "HRAM";
        public const string SpaceCpuBus = "CPUBUS";

        public byte ReadSpace(string spaceName, int address)
        {
            if (Bus is null || Cart is null) return 0;

            switch (spaceName)
            {
                case SpaceRom:
                    return address < Cart.Rom.Length ? Cart.Rom[address] : (byte)0xFF;

                case SpaceVram:
                    return Bus.Vram[Wrap(address, Bus.Vram.Length)];

                case SpaceCartRam:
                    return Cart.Ram.Length == 0 ? (byte)0xFF : Cart.Ram[Wrap(address, Cart.Ram.Length)];

                case SpaceWram:
                    return Bus.Wram[Wrap(address, Bus.Wram.Length)];

                case SpaceOam:
                    return Bus.Oam[Wrap(address, Bus.Oam.Length)];

                case SpaceHram:
                    return Bus.HighRam[Wrap(address, Bus.HighRam.Length)];

                // Reads here go through the real decode, side effects included - see Mercury_Memory.md §8.
                case SpaceCpuBus:
                    return Bus.Read((ushort)(address & 0xFFFF));

                default:
                    return 0;
            }
        }

        public void WriteSpace(string spaceName, int address, byte value)
        {
            if (Bus is null || Cart is null) return;

            switch (spaceName)
            {
                case SpaceVram:
                    Bus.Vram[Wrap(address, Bus.Vram.Length)] = value;
                    return;

                case SpaceCartRam:
                    if (Cart.Ram.Length > 0) Cart.Ram[Wrap(address, Cart.Ram.Length)] = value;
                    return;

                case SpaceWram:
                    Bus.Wram[Wrap(address, Bus.Wram.Length)] = value;
                    return;

                case SpaceOam:
                    Bus.Oam[Wrap(address, Bus.Oam.Length)] = value;
                    return;

                case SpaceHram:
                    Bus.HighRam[Wrap(address, Bus.HighRam.Length)] = value;
                    return;

                case SpaceCpuBus:
                    Bus.Write((ushort)(address & 0xFFFF), value);
                    return;

                // ROM is the cartridge mask; a debug write must not corrupt the loaded image.
                default:
                    return;
            }
        }

        public int SpaceSize(string spaceName) => spaceName switch
        {
            SpaceRom => Cart?.Rom.Length ?? 0,
            SpaceVram => Bus?.Vram.Length ?? 0,
            SpaceCartRam => Cart?.Ram.Length ?? 0,
            SpaceWram => Bus?.Wram.Length ?? 0,
            SpaceOam => Bus?.Oam.Length ?? 0,
            SpaceHram => Bus?.HighRam.Length ?? 0,
            SpaceCpuBus => 0x10000,
            _ => 0,
        };

        private static int Wrap(int address, int size) => size == 0 ? 0 : ((address % size) + size) % size;

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
