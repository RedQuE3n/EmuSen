using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // The two-register window the S-CPU sees, and the banks it sees it in - see Venus_NecDSP.md §3.
    public sealed partial class NecDsp
    {
        // SR bits. Only RQM and DRS are load-bearing here; the rest exist because the firmware branches on - see Venus_NecDSP.md §3.2.
        private const ushort RequestForMaster = 0x8000;
        private const ushort DataRegStatus = 0x1000;
        private const ushort DataRegControl = 0x0400;
        private const ushort SerialOutControl = 0x0200;
        private const ushort SerialInControl = 0x0100;

        private static ushort SelectRegisterMask(NecDspVariant variant, bool hiRom)
        {
            if (NecDspProfile.IsSt01x(variant)) return 0x0001;
            return hiRom ? (ushort)0x1000 : (ushort)0x4000;
        }

        // The full 24-bit address is kept in the offset, because the ST01x needs the bank to tell its RAM.
        public CartridgeAddress ResolveScpu(byte bank, ushort offset)
        {
            int address = (bank << 16) | offset;

            if (NecDspProfile.IsSt01x(_variant))
            {
                if (offset > 0x0FFF) return CartridgeAddress.Unmapped;

                // $60/$E0 is the register pair; $68-$6F/$E8-$EF is the 4KB battery-backed RAM the same chip holds.
                int mirrored = bank & 0x7F;
                bool st01xBank = mirrored == 0x60 || (mirrored >= 0x68 && mirrored <= 0x6F);
                return st01xBank ? CartridgeAddress.CoprocessorRegister(address) : CartridgeAddress.Unmapped;
            }

            if (_hiRomMap)
            {
                // HiROM: a $6000-$7FFF window below the cartridge's own SRAM banks.
                bool hiRomBank = bank <= 0x1F || (bank >= 0x80 && bank <= 0x9F);
                if (hiRomBank && offset >= 0x6000) return CartridgeAddress.CoprocessorRegister(address);
                return CartridgeAddress.Unmapped;
            }

            // LoROM: the upper half of $30-$3F/$B0-$BF, plus the lower half of $60-$6F/$E0-$EF that Super Bases.
            bool loRomBank = (bank >= 0x30 && bank <= 0x3F) || (bank >= 0xB0 && bank <= 0xBF);
            if (loRomBank && offset >= 0x8000) return CartridgeAddress.CoprocessorRegister(address);

            bool altBank = (bank >= 0x60 && bank <= 0x6F) || (bank >= 0xE0 && bank <= 0xEF);
            if (altBank && offset < 0x8000) return CartridgeAddress.CoprocessorRegister(address);

            return CartridgeAddress.Unmapped;
        }

        public byte ReadRegister(int address)
        {
            if (IsSt01xRam(address))
            {
                ushort word = ReadRam((ushort)(address >> 1));
                return (address & 0x01) != 0 ? (byte)(word >> 8) : (byte)word;
            }

            // SR is read as its high byte alone; the low byte is firmware-private.
            if ((address & _registerMask) != 0) return (byte)(_sr >> 8);

            _inRqmLoop = false;

            if ((_sr & DataRegControl) != 0)
            {
                _sr &= unchecked((ushort)~RequestForMaster);
                return (byte)_dr;
            }

            // In 16-bit mode DRS tracks which half of DR the host is on, and only the second half clears RQM - see Venus_NecDSP.md §3.2.
            if ((_sr & DataRegStatus) != 0)
            {
                _sr &= unchecked((ushort)~(RequestForMaster | DataRegStatus));
                return (byte)(_dr >> 8);
            }

            _sr |= DataRegStatus;
            return (byte)_dr;
        }

        public void WriteRegister(int address, byte value)
        {
            if (IsSt01xRam(address))
            {
                ushort addr = (ushort)(address >> 1);
                ushort word = ReadRam(addr);
                WriteRam(addr, (address & 0x01) != 0
                    ? (ushort)((word & 0x00FF) | (value << 8))
                    : (ushort)((word & 0xFF00) | value));
                return;
            }

            // Writes to SR are ignored; the host can only drive DR.
            if ((address & _registerMask) != 0) return;

            _inRqmLoop = false;

            if ((_sr & DataRegControl) != 0)
            {
                _sr &= unchecked((ushort)~RequestForMaster);
                _dr = (ushort)((_dr & 0xFF00) | value);
                return;
            }

            if ((_sr & DataRegStatus) != 0)
            {
                _sr &= unchecked((ushort)~(RequestForMaster | DataRegStatus));
                _dr = (ushort)((_dr & 0x00FF) | (value << 8));
                return;
            }

            _sr |= DataRegStatus;
            _dr = (ushort)((_dr & 0xFF00) | value);
        }

        // Banks $68-$6F/$E8-$EF on an ST010/ST011 are the DSP's data RAM rather than its register pair.
        private bool IsSt01xRam(int address) => NecDspProfile.IsSt01x(_variant) && (address & 0x0F0000) >= 0x080000;

        // The .srm image of the uPD96050's data RAM, little-endian - see Venus_NecDSP.md §6.
        public byte[] ExportBatteryRam()
        {
            byte[] bytes = new byte[Ram.Length * 2];
            for (int i = 0; i < Ram.Length; i++)
            {
                bytes[i * 2] = (byte)Ram[i];
                bytes[(i * 2) + 1] = (byte)(Ram[i] >> 8);
            }
            return bytes;
        }

        public void ImportBatteryRam(byte[] bytes)
        {
            for (int i = 0; i < Ram.Length && (i * 2) + 1 < bytes.Length; i++)
            {
                Ram[i] = (ushort)(bytes[i * 2] | (bytes[(i * 2) + 1] << 8));
            }
        }
    }
}
