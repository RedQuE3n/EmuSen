namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Mode $21. Full 64KB ROM banks at $C0-$FF (mirrored into $40-$7D), the upper half of those same - see Venus_Memory.md §2.1a.
    public sealed class HiRomMapper : ICartridgeMapper
    {
        public string Name => "HiROM";

        public CartridgeAddress Resolve(byte bank, ushort offset)
        {
            bool sramBank = (bank >= 0x20 && bank <= 0x3F) || (bank >= 0xA0 && bank <= 0xBF);
            if (sramBank && offset >= 0x6000 && offset <= 0x7FFF)
            {
                return CartridgeAddress.Sram(((bank & 0x1F) * 0x2000) + (offset - 0x6000));
            }

            // One formula for both windows: the $00-$3F view of a bank is literally the top half of the $C0-$FF.
            bool fullBank = bank >= 0xC0 || (bank >= 0x40 && bank <= 0x7D);
            if (fullBank || offset >= 0x8000) return CartridgeAddress.Rom(((bank & 0x3F) << 16) | offset);

            return CartridgeAddress.Unmapped;
        }
    }
}
