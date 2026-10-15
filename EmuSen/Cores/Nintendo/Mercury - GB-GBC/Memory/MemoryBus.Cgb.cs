namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // The registers a DMG does not have: VRAM/WRAM banking, KEY1 and HDMA - see Mercury_Cgb.md.
    public sealed partial class MemoryBus
    {
        public int VramBank;

        // SVBK as written; 0 and 1 both map bank 1, there being no second copy of the fixed bank - see Mercury_Native.md §13.8.
        public int WramBank = 1;

        public int MappedWramBank => WramBank == 0 ? 1 : WramBank;

        public bool DoubleSpeed;
        public bool SpeedSwitchArmed;

        // Halves the LCD's and the APU's clock relative to the CPU's while double speed is on.
        private bool _baseClockPhase;

        public ushort HdmaSource;
        public ushort HdmaDestination;

        // Blocks of 16 bytes still owed: running while hblank-driven, stopped with that many left otherwise, 0 when none - see Mercury_Native.md §13.9.
        public int HdmaBlocksLeft;

        public bool HdmaIsHBlankDriven;

        private int VramOffset(int address) => (VramBank * VramBankSize) + (address - 0x8000);

        // $C000-$DFFF and its $E000 echo fold to the same 8K; the top half is the banked one.
        private int WramOffset(int address)
        {
            int offset = address & 0x1FFF;
            return offset < WramBankSize ? offset : (MappedWramBank * WramBankSize) + (offset - WramBankSize);
        }

        private byte ReadCgbIo(ushort address) => address switch
        {
            0xFF4D => (byte)(0x7E | (DoubleSpeed ? 0x80 : 0x00) | (SpeedSwitchArmed ? 0x01 : 0x00)),
            0xFF4F => (byte)(0xFE | VramBank),

            // Bit 7 set means idle, and the low bits are the blocks still owed minus one, a stopped transfer's too; HDMA1-4 are write-only.
            0xFF55 => HdmaBlocksLeft == 0 ? (byte)0xFF : (byte)((HdmaIsHBlankDriven ? 0x00 : 0x80) | (HdmaBlocksLeft - 1)),

            // RP: no partner's light ever arrives, so bit 1 reads 1 whether or not reading is enabled.
            0xFF56 => (byte)(0x3E | Io[0x56]),
            0xFF68 => Ppu.ReadBgPaletteIndex(),
            0xFF69 => Ppu.ReadBgPaletteData(),
            0xFF6A => Ppu.ReadObjPaletteIndex(),
            0xFF6B => Ppu.ReadObjPaletteData(),
            0xFF6C => (byte)(0xFE | Io[0x6C]),
            0xFF70 => (byte)(0xF8 | WramBank),
            _ => ReadUndocumentedIo(address, colourMode: true),
        };

        // $FF72-$FF77 on the Color in either mode, $FF74 only in colour mode - see Mercury_Native.md §13.1.
        private byte ReadUndocumentedIo(ushort address, bool colourMode) => address switch
        {
            0xFF72 or 0xFF73 => Io[address - 0xFF00],
            0xFF74 => colourMode ? Io[0x74] : (byte)0xFF,
            0xFF75 => (byte)(0x8F | Io[0x75]),
            0xFF76 => Apu.Pcm12,
            0xFF77 => Apu.Pcm34,
            _ => 0xFF,
        };

        private void WriteUndocumentedIo(ushort address, byte data, bool colourMode)
        {
            switch (address)
            {
                case 0xFF72 or 0xFF73:
                    Io[address - 0xFF00] = data;
                    return;

                case 0xFF74:
                    if (colourMode) Io[0x74] = data;
                    return;

                case 0xFF75:
                    Io[0x75] = (byte)(data & 0x70);
                    return;
            }
        }

        private void WriteCgbIo(ushort address, byte data)
        {
            switch (address)
            {
                case 0xFF4D:
                    SpeedSwitchArmed = (data & 0x01) != 0;
                    return;

                case 0xFF4F:
                    VramBank = data & 0x01;
                    return;

                // The source's low four bits are not wired; a transfer always starts 16-byte aligned.
                case 0xFF51:
                    HdmaSource = (ushort)((data << 8) | (HdmaSource & 0xF0));
                    return;

                case 0xFF52:
                    HdmaSource = (ushort)((HdmaSource & 0xFF00) | (data & 0xF0));
                    return;

                // The destination is always inside VRAM, so only bits 12-4 of it exist.
                case 0xFF53:
                    HdmaDestination = (ushort)(((data & 0x1F) << 8) | (HdmaDestination & 0xF0));
                    return;

                case 0xFF54:
                    HdmaDestination = (ushort)((HdmaDestination & 0x1F00) | (data & 0xF0));
                    return;

                case 0xFF55:
                    StartHdma(data);
                    return;

                case 0xFF68:
                    Ppu.WriteBgPaletteIndex(data);
                    return;

                case 0xFF69:
                    Ppu.WriteBgPaletteData(data);
                    return;

                case 0xFF6A:
                    Ppu.WriteObjPaletteIndex(data);
                    return;

                case 0xFF6B:
                    Ppu.WriteObjPaletteData(data);
                    return;

                case 0xFF56:
                    Io[0x56] = (byte)(data & 0xC1);
                    return;

                // OPRI: kept for its read-back; the renderer's priority follows the mode, not this bit - see Mercury_Model.md §4.
                case 0xFF6C:
                    Io[0x6C] = (byte)(data & 0x01);
                    return;

                case 0xFF70:
                    WramBank = data & 0x07;
                    return;

                default:
                    WriteUndocumentedIo(address, data, colourMode: true);
                    return;
            }
        }

        // With a Game Boy cartridge the colour registers read $FF and ignore writes, except VRAM bank, the palette indices and $FF72-$FF77 - see Mercury_Model.md §4.2, Mercury_Native.md §13.1.
        private byte ReadCompatIo(ushort address) => address switch
        {
            0xFF4F => (byte)(0xFE | VramBank),
            0xFF68 => Ppu.ReadBgPaletteIndex(),
            0xFF6A => Ppu.ReadObjPaletteIndex(),
            _ => ReadUndocumentedIo(address, colourMode: false),
        };

        private void WriteCompatIo(ushort address, byte data)
        {
            switch (address)
            {
                case 0xFF4F:
                    VramBank = data & 0x01;
                    return;

                case 0xFF68:
                    Ppu.WriteBgPaletteIndex(data);
                    return;

                case 0xFF6A:
                    Ppu.WriteObjPaletteIndex(data);
                    return;

                default:
                    WriteUndocumentedIo(address, data, colourMode: false);
                    return;
            }
        }

        // STOP performs the switch KEY1 armed; it is not a stop at all on a CGB - see Mercury_Cgb.md §5.
        public void Stop()
        {
            if (!Cgb || !SpeedSwitchArmed) return;

            DoubleSpeed = !DoubleSpeed;
            SpeedSwitchArmed = false;
            _baseClockPhase = false;
        }

        private void StartHdma(byte data)
        {
            // Clearing bit 7 during an hblank transfer cancels it rather than starting a new one.
            if (HdmaIsHBlankDriven && HdmaBlocksLeft > 0 && (data & 0x80) == 0)
            {
                HdmaIsHBlankDriven = false;
                return;
            }

            HdmaBlocksLeft = (data & 0x7F) + 1;
            HdmaIsHBlankDriven = (data & 0x80) != 0;

            if (HdmaIsHBlankDriven) return;

            // General-purpose DMA copies everything at once and charges the CPU for it - see Mercury_Cgb.md §4.1.
            while (HdmaBlocksLeft > 0) TransferHdmaBlock();
        }

        // Called by the PPU when a visible line enters mode 0 - see Mercury_Cgb.md §4.
        public void OnHBlankStarted()
        {
            if (HdmaIsHBlankDriven && HdmaBlocksLeft > 0) TransferHdmaBlock();
        }

        // About 8 µs a block at either speed: 8 machine cycles, or 16 in double speed - see Mercury_Cgb.md §4.1, Mercury_Native.md §13.9.
        private void ChargeHdmaBlock() => _stallCycles += DoubleSpeed ? 64 : 32;

        private void TransferHdmaBlock()
        {
            ChargeHdmaBlock();

            for (int i = 0; i < 16; i++)
            {
                byte value = Read(HdmaSource);
                Vram[(VramBank * VramBankSize) + (HdmaDestination & 0x1FFF)] = value;

                HdmaSource++;
                HdmaDestination = (ushort)((HdmaDestination + 1) & 0x1FFF);
            }

            // A destination run off the end of VRAM ends the transfer - see Mercury_Native.md §13.9.
            if (HdmaDestination == 0) HdmaBlocksLeft = 1;
            if (--HdmaBlocksLeft == 0) HdmaIsHBlankDriven = false;
        }

        private void ResetCgb()
        {
            VramBank = 0;
            WramBank = 1;
            DoubleSpeed = false;
            SpeedSwitchArmed = false;
            _baseClockPhase = false;
            HdmaSource = 0;
            HdmaDestination = 0;
            HdmaBlocksLeft = 0;
            HdmaIsHBlankDriven = false;
        }
    }
}
