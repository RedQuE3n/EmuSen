using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // Address decode for both sides of the SA-1 - see Venus_SA1.md §3.
    public sealed partial class Sa1
    {
        private static bool IsLowBank(byte bank) => bank <= 0x3F || (bank >= 0x80 && bank <= 0xBF);

        // Bit 7 clear means "use the power-on bank for this slot" - see Venus_SA1.md §3.1.
        private static int SuperBank(byte reg, int fallback) => (reg & 0x80) != 0 ? (reg & 0x07) : fallback;

        // $00-$1F/$20-$3F/$80-$9F/$A0-$BF each project one 1MB super bank LoROM-style: 32 banks of the upper 32KB.
        private int LoRomStyleOffset(byte bank, ushort offset)
        {
            int super = bank <= 0x1F ? SuperBank(_cxb, 0)
                      : bank <= 0x3F ? SuperBank(_dxb, 1)
                      : bank <= 0x9F ? SuperBank(_exb, 2)
                                     : SuperBank(_fxb, 3);
            return (super << 20) | ((bank & 0x1F) << 15) | (offset & 0x7FFF);
        }

        // $C0-$FF project the same four super banks HiROM-style: 16 full banks each.
        private int HiRomStyleOffset(byte bank, ushort offset)
        {
            int super = bank <= 0xCF ? SuperBank(_cxb, 0)
                      : bank <= 0xDF ? SuperBank(_dxb, 1)
                      : bank <= 0xEF ? SuperBank(_exb, 2)
                                     : SuperBank(_fxb, 3);
            return (super << 20) | ((bank & 0x0F) << 16) | offset;
        }

        // The 8KB $6000-$7FFF window, one selectable block per side - see Venus_SA1.md §3.2.
        private static int BwRamWindowOffset(int block, ushort offset) => (block << 13) | (offset & 0x1FFF);

        // S-CPU view. WRAM, PPU and CPU registers never reach here - MemoryBus decodes those first and only.
        public CartridgeAddress ResolveScpu(byte bank, ushort offset)
        {
            if (IsLowBank(bank))
            {
                if (offset >= 0x2200 && offset <= 0x23FF) return CartridgeAddress.CoprocessorRegister(offset);
                if (offset >= 0x3000 && offset <= 0x37FF) return CartridgeAddress.IRam(offset - 0x3000);
                if (offset >= 0x6000 && offset <= 0x7FFF) return CartridgeAddress.Sram(BwRamWindowOffset(_bmaps & 0x1F, offset));

                if (offset >= 0x8000)
                {
                    // The SA-1 can substitute its own NMI/IRQ vectors for the S-CPU's, and does it by intercepting the - see Venus_SA1.md §4.3.
                    if (bank == 0x00)
                    {
                        if ((_scnt & 0x20) != 0 && offset >= 0xFFEA && offset <= 0xFFEB) return CartridgeAddress.Sa1Vector(offset - 0xFFEA);
                        if ((_scnt & 0x10) != 0 && offset >= 0xFFEE && offset <= 0xFFEF) return CartridgeAddress.Sa1Vector(offset - 0xFFEE + 2);
                    }
                    return CartridgeAddress.Rom(LoRomStyleOffset(bank, offset));
                }

                return CartridgeAddress.Unmapped;
            }

            if (bank >= 0x40 && bank <= 0x4F) return CartridgeAddress.Sram(((bank & 0x0F) << 16) | offset);
            if (bank >= 0xC0) return CartridgeAddress.Rom(HiRomStyleOffset(bank, offset));
            return CartridgeAddress.Unmapped;
        }

        // No WRAM and no PPU; I-RAM is mirrored low as well - see Venus_SA1.md §3.
        public CartridgeAddress ResolveSa1(byte bank, ushort offset)
        {
            if (IsLowBank(bank))
            {
                if (offset < 0x0800) return CartridgeAddress.IRam(offset);
                if (offset >= 0x2200 && offset <= 0x23FF) return CartridgeAddress.CoprocessorRegister(offset);
                if (offset >= 0x3000 && offset <= 0x37FF) return CartridgeAddress.IRam(offset - 0x3000);
                if (offset >= 0x6000 && offset <= 0x7FFF) return ResolveSa1BwRamWindow(offset);

                if (offset >= 0x8000)
                {
                    // The SA-1's own reset/NMI/IRQ vectors are always its own, with no enable bit.
                    if (bank == 0x00)
                    {
                        if (offset >= 0xFFEA && offset <= 0xFFEB) return CartridgeAddress.Sa1Vector(offset - 0xFFEA + 4);
                        if (offset >= 0xFFEE && offset <= 0xFFEF) return CartridgeAddress.Sa1Vector(offset - 0xFFEE + 6);
                        if (offset >= 0xFFFC && offset <= 0xFFFD) return CartridgeAddress.Sa1Vector(offset - 0xFFFC + 8);
                    }
                    return CartridgeAddress.Rom(LoRomStyleOffset(bank, offset));
                }

                return CartridgeAddress.Unmapped;
            }

            if (bank >= 0x40 && bank <= 0x4F) return CartridgeAddress.Sram(((bank & 0x0F) << 16) | offset);
            if (bank >= 0xC0) return CartridgeAddress.Rom(HiRomStyleOffset(bank, offset));
            return CartridgeAddress.Unmapped;
        }

        // BMAP bit 7 turns the window into a virtual bitmap, where one byte is one 2bpp or 4bpp pixel - see Venus_SA1.md §3.3.
        private CartridgeAddress ResolveSa1BwRamWindow(ushort offset)
        {
            if ((_bmap & 0x80) == 0) return CartridgeAddress.Sram(BwRamWindowOffset(_bmap & 0x1F, offset));

            int pixel = offset & 0x1FFF;
            bool twoBpp = (_bbf & 0x80) != 0;
            int block = _bmap & 0x7F;
            return twoBpp
                ? CartridgeAddress.Sram((block << 11) + (pixel >> 2))
                : CartridgeAddress.Sram((block << 12) + (pixel >> 1));
        }

        // Bitmap-mode reads/writes touch one sub-byte field of the resolved byte rather than the whole thing.
        private bool Sa1BitmapWindow(ushort offset, out int shift, out int mask)
        {
            if ((_bmap & 0x80) == 0) { shift = 0; mask = 0xFF; return false; }
            if ((_bbf & 0x80) != 0) { shift = (offset & 0x03) * 2; mask = 0x03; }
            else { shift = (offset & 0x01) * 4; mask = 0x0F; }
            return true;
        }

        // ReadSa1 with the register window peeked instead of read - see Venus_SA1.md §11.1.
        public byte DebugReadSa1(uint address) => ReadSa1Internal(address, forDebug: true);

        public byte ReadSa1(uint address) => ReadSa1Internal(address, forDebug: false);

        private byte ReadSa1Internal(uint address, bool forDebug)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)address;
            var mapped = ResolveSa1(bank, offset);

            switch (mapped.Region)
            {
                case CartridgeRegion.Rom:
                    return mapped.Offset < _rom.Length ? _rom[mapped.Offset] : (byte)0x00;

                case CartridgeRegion.Sram:
                    if (_bwRam.Length == 0) return 0x00;
                    byte raw = _bwRam[mapped.Offset % _bwRam.Length];
                    if (IsLowBank(bank) && offset >= 0x6000 && offset <= 0x7FFF
                        && Sa1BitmapWindow(offset, out int shift, out int mask))
                    {
                        return (byte)((raw >> shift) & mask);
                    }
                    return raw;

                case CartridgeRegion.IRam:
                    return IRam[mapped.Offset];

                case CartridgeRegion.CoprocessorRegister:
                    return forDebug
                        ? DebugPeekRegister((ushort)mapped.Offset)
                        : ReadRegister((ushort)mapped.Offset);

                case CartridgeRegion.Sa1Vector:
                    return VectorByte(mapped.Offset);

                default:
                    return 0x00;
            }
        }

        public void WriteSa1(uint address, byte data)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)address;
            var mapped = ResolveSa1(bank, offset);

            switch (mapped.Region)
            {
                case CartridgeRegion.Sram:
                    if (_bwRam.Length == 0) return;
                    int index = mapped.Offset % _bwRam.Length;
                    if (IsLowBank(bank) && offset >= 0x6000 && offset <= 0x7FFF
                        && Sa1BitmapWindow(offset, out int shift, out int mask))
                    {
                        _bwRam[index] = (byte)((_bwRam[index] & ~(mask << shift)) | ((data & mask) << shift));
                        WriteObserver?.OnCoprocessorWrite("BWRAM", index, _bwRam[index]);
                        return;
                    }
                    _bwRam[index] = data;
                    WriteObserver?.OnCoprocessorWrite("BWRAM", index, data);
                    return;

                case CartridgeRegion.IRam:
                    IRam[mapped.Offset] = data;
                    WriteObserver?.OnCoprocessorWrite("SA1IRAM", mapped.Offset, data);
                    return;

                case CartridgeRegion.CoprocessorRegister:
                    WriteRegister((ushort)mapped.Offset, data);
                    return;
            }
        }

        // Flat, device-relative BW-RAM access for Sa1Dma, which addresses the chip directly rather than.
        internal byte ReadBwRamByte(int offset) => _bwRam.Length == 0 ? (byte)0x00 : _bwRam[offset % _bwRam.Length];

        internal void WriteBwRamByte(int offset, byte data)
        {
            if (_bwRam.Length == 0) return;
            int index = offset % _bwRam.Length;
            _bwRam[index] = data;
            WriteObserver?.OnCoprocessorWrite("BWRAM", index, data);
        }

        // Offsets 0-3 are the S-CPU's overridden NMI/IRQ vectors, 4-9 the SA-1's own.
        public byte VectorByte(int index) => index switch
        {
            0 => (byte)_snv,
            1 => (byte)(_snv >> 8),
            2 => (byte)_siv,
            3 => (byte)(_siv >> 8),
            4 => (byte)_cnv,
            5 => (byte)(_cnv >> 8),
            6 => (byte)_civ,
            7 => (byte)(_civ >> 8),
            8 => (byte)_crv,
            _ => (byte)(_crv >> 8),
        };
    }
}
