namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Mode $20. ROM in the upper half of every bank, SRAM at $70-$7D/$F0-$FF - see Venus_Memory.md §2.1a.
    public sealed class LoRomMapper : ICartridgeMapper
    {
        public string Name => "LoROM";

        public CartridgeAddress Resolve(byte bank, ushort offset)
        {
            // Masking the bank to 0x7F handles the $00-$3F / $80-$BF mirror.
            if (offset >= 0x8000) return CartridgeAddress.Rom(((bank & 0x7F) * 0x8000) + (offset - 0x8000));

            if ((bank >= 0x70 && bank <= 0x7D) || bank >= 0xF0) return CartridgeAddress.Sram(offset);

            return CartridgeAddress.Unmapped;
        }
    }
}
