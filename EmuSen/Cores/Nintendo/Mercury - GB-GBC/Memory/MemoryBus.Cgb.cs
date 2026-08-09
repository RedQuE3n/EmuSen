namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // The registers a DMG does not have: VRAM/WRAM banking, KEY1 and HDMA - see Mercury_Cgb.md.
    public sealed partial class MemoryBus
    {
        public int VramBank;

        // 0 and 1 both select bank 1; there is no second copy of the fixed bank.
        public int WramBank = 1;

        public bool DoubleSpeed;
        public bool SpeedSwitchArmed;

        // Halves the LCD's clock relative to the CPU's while double speed is on.
        private bool _ppuHalfCycle;

        public ushort HdmaSource;
        public ushort HdmaDestination;

        // Blocks of 16 bytes still owed, or 0 when nothing is running.
        public int HdmaBlocksLeft;

        public bool HdmaIsHBlankDriven;

        private int VramOffset(int address) => (VramBank * VramBankSize) + (address - 0x8000);

        // $C000-$DFFF and its $E000 echo fold to the same 8K; the top half is the banked one.
        private int WramOffset(int address)
        {
            int offset = address & 0x1FFF;
            return offset < WramBankSize ? offset : (WramBank * WramBankSize) + (offset - WramBankSize);
        }

        private byte ReadCgbIo(ushort address) => address switch
        {
            0xFF4D => (byte)(0x7E | (DoubleSpeed ? 0x80 : 0x00) | (SpeedSwitchArmed ? 0x01 : 0x00)),
            0xFF4F => (byte)(0xFE | VramBank),
            0xFF51 => (byte)(HdmaSource >> 8),
            0xFF52 => (byte)(HdmaSource & 0xF0),
            0xFF53 => (byte)((HdmaDestination >> 8) & 0x1F),
            0xFF54 => (byte)(HdmaDestination & 0xF0),

            // Bit 7 set means idle, and the low bits are the blocks still owed minus one.
            0xFF55 => HdmaBlocksLeft == 0 ? (byte)0xFF : (byte)(HdmaBlocksLeft - 1),

            0xFF68 => Ppu.ReadBgPaletteIndex(),
            0xFF69 => Ppu.ReadBgPaletteData(),
            0xFF6A => Ppu.ReadObjPaletteIndex(),
            0xFF6B => Ppu.ReadObjPaletteData(),
            0xFF70 => (byte)(0xF8 | WramBank),
            _ => Io[address - 0xFF00],
        };

        // True when this address belongs to a colour register, so the generic $FF00 array is left alone.
        private bool WriteCgbIo(ushort address, byte data)
        {
            switch (address)
            {
                case 0xFF4D:
                    SpeedSwitchArmed = (data & 0x01) != 0;
                    return true;

                case 0xFF4F:
                    VramBank = data & 0x01;
                    return true;

                // The source's low four bits are not wired; a transfer always starts 16-byte aligned.
                case 0xFF51:
                    HdmaSource = (ushort)((data << 8) | (HdmaSource & 0xF0));
                    return true;

                case 0xFF52:
                    HdmaSource = (ushort)((HdmaSource & 0xFF00) | (data & 0xF0));
                    return true;

                // The destination is always inside VRAM, so only bits 12-4 of it exist.
                case 0xFF53:
                    HdmaDestination = (ushort)(((data & 0x1F) << 8) | (HdmaDestination & 0xF0));
                    return true;

                case 0xFF54:
                    HdmaDestination = (ushort)((HdmaDestination & 0x1F00) | (data & 0xF0));
                    return true;

                case 0xFF55:
                    StartHdma(data);
                    return true;

                case 0xFF68:
                    Ppu.WriteBgPaletteIndex(data);
                    return true;

                case 0xFF69:
                    Ppu.WriteBgPaletteData(data);
                    return true;

                case 0xFF6A:
                    Ppu.WriteObjPaletteIndex(data);
                    return true;

                case 0xFF6B:
                    Ppu.WriteObjPaletteData(data);
                    return true;

                // OPRI: the CGB boot ROM writes it, and Mercury never runs in the DMG-priority mode it selects.
                case 0xFF6C:
                    return true;

                case 0xFF70:
                    WramBank = (data & 0x07) == 0 ? 1 : data & 0x07;
                    return true;

                default:
                    return false;
            }
        }

        // STOP performs the switch KEY1 armed; it is not a stop at all on a CGB - see Mercury_Cgb.md §5.
        public void Stop()
        {
            if (!Cgb || !SpeedSwitchArmed) return;

            DoubleSpeed = !DoubleSpeed;
            SpeedSwitchArmed = false;
            _ppuHalfCycle = false;
        }

        private void StartHdma(byte data)
        {
            // Clearing bit 7 during an hblank transfer cancels it rather than starting a new one.
            if (HdmaIsHBlankDriven && HdmaBlocksLeft > 0 && (data & 0x80) == 0)
            {
                HdmaBlocksLeft = 0;
                HdmaIsHBlankDriven = false;
                return;
            }

            HdmaBlocksLeft = (data & 0x7F) + 1;
            HdmaIsHBlankDriven = (data & 0x80) != 0;

            if (HdmaIsHBlankDriven) return;

            // General-purpose DMA runs to completion here rather than stalling the CPU - see Mercury_Cgb.md §4.
            while (HdmaBlocksLeft > 0) TransferHdmaBlock();
        }

        // Called by the PPU when a visible line enters mode 0 - see Mercury_Cgb.md §4.
        public void OnHBlankStarted()
        {
            if (HdmaIsHBlankDriven && HdmaBlocksLeft > 0) TransferHdmaBlock();
        }

        private void TransferHdmaBlock()
        {
            for (int i = 0; i < 16; i++)
            {
                byte value = Read(HdmaSource);
                Vram[(VramBank * VramBankSize) + (HdmaDestination & 0x1FFF)] = value;

                HdmaSource++;
                HdmaDestination = (ushort)((HdmaDestination + 1) & 0x1FFF);
            }

            if (--HdmaBlocksLeft == 0) HdmaIsHBlankDriven = false;
        }

        private void ResetCgb()
        {
            VramBank = 0;
            WramBank = 1;
            DoubleSpeed = false;
            SpeedSwitchArmed = false;
            _ppuHalfCycle = false;
            HdmaSource = 0;
            HdmaDestination = 0;
            HdmaBlocksLeft = 0;
            HdmaIsHBlankDriven = false;
        }
    }
}
