using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // Both sides see the same ROM and Game Pak RAM through the same decode -
    // unlike the SA-1, the SuperFX has no separate map per side. What differs
    // is which banks each one reaches it through. See Venus_SuperFX.md §3.
    public sealed partial class SuperFx
    {
        private static bool IsLowBank(byte bank) => bank <= 0x3F || (bank >= 0x80 && bank <= 0xBF);

        // $00-$3F/$80-$BF are LoROM-style; $40-$5F/$C0-$DF are linear 64KB
        // banks. Bit 15 of the address is ignored in the LoROM view, so
        // $00:0000 and $00:8000 are the same ROM byte.
        private static int RomOffset(byte bank, ushort offset)
        {
            if ((bank & 0x60) == 0x40) return ((bank & 0x1F) << 16) | offset;
            return ((bank & 0x3F) << 15) | (offset & 0x7FFF);
        }

        private static int RamOffsetLinear(byte bank, ushort offset) => ((bank & 0x1F) << 16) | offset;

        // The S-CPU's 8KB window is packed one block per bank.
        // The $6000-$7FFF window is the FIRST 8KB of Game Pak RAM mirrored into
        // every low bank, not a bank-indexed slice - see Venus_SuperFX.md §5.1.
        private static int RamOffsetWindow(byte bank, ushort offset) => offset & 0x1FFF;

        // S-CPU view. MemoryBus has already claimed WRAM, the PPU and the CPU
        // registers before the cartridge is consulted.
        // Bus arbitration is not enforced, so this counts the violations the
        // hardware would have stalled - see Venus_SuperFX.md §2.1.
        public long DebugScpuRamWhileRunning { get; private set; }

        public CartridgeAddress ResolveScpu(byte bank, ushort offset)
        {
            if (IsLowBank(bank))
            {
                if (offset >= 0x3000 && offset <= 0x32FF) return CartridgeAddress.CoprocessorRegister(offset);
                if (offset >= 0x6000 && offset <= 0x7FFF) { if (Running) DebugScpuRamWhileRunning++; return CartridgeAddress.Sram(RamOffsetWindow(bank, offset)); }
                if (offset >= 0x8000) return CartridgeAddress.Rom(RomOffset(bank, offset));
                return CartridgeAddress.Unmapped;
            }

            // $7E/$7F never arrive here - MemoryBus decodes WRAM first.
            if ((bank >= 0x40 && bank <= 0x5F) || (bank >= 0xC0 && bank <= 0xDF)) return CartridgeAddress.Rom(RomOffset(bank, offset));
            if (bank >= 0x60) { if (Running) DebugScpuRamWhileRunning++; return CartridgeAddress.Sram(RamOffsetLinear(bank, offset)); }
            return CartridgeAddress.Unmapped;
        }

        // --- GSU-side access ---

        private byte ReadRom(byte bank, ushort offset)
        {
            int index = RomOffset(bank, offset);
            return index < _rom.Length ? _rom[index] : (byte)0x00;
        }

        public byte ReadRam(int offset) => _ram.Length == 0 ? (byte)0x00 : _ram[(offset & 0x7FFFFF) % _ram.Length];

        public void WriteRam(int offset, byte data)
        {
            if (_ram.Length == 0) return;
            int index = (offset & 0x7FFFFF) % _ram.Length;

            // See Venus_SuperFX.md §8.
            if (EmuSen.Debug.DebugSettings.SuperFxRamWriteTraceCountdown > 0
                && index == EmuSen.Debug.DebugSettings.SuperFxRamWriteTraceAddr)
            {
                EmuSen.Debug.DebugSettings.SuperFxRamWriteTraceCountdown--;
                System.Console.WriteLine($"[GSUW] {index:X6} = {data:X2} pc={_pbr:X2}:{R[15]:X4}");
            }

            _ram[index] = data;
        }

        // Program fetch. The GSU can run out of ROM ($00-$5F) or, for code the
        // S-CPU has staged there, out of Game Pak RAM ($70-$71).
        private byte ReadProgramMemory(byte bank, ushort offset)
        {
            if (bank >= 0x60) return ReadRam(RamOffsetLinear(bank, offset));
            return ReadRom(bank, offset);
        }

        // Instruction fetch goes through the 512-byte cache whenever the
        // address falls inside its window - see Venus_SuperFX.md §4.3.
        private byte FetchProgramByte(ushort address)
        {
            int inCache = (ushort)(address - _cbr);
            if (inCache < CacheSize)
            {
                int line = inCache >> 4;
                if (!_cacheValid[line]) FillCacheLine(line);
                return Cache[inCache];
            }

            return ReadProgramMemory(_pbr, address);
        }

        private void FillCacheLine(int line)
        {
            ushort baseAddress = (ushort)(_cbr + (line << 4));
            for (int i = 0; i < 16; i++)
            {
                Cache[(line << 4) + i] = ReadProgramMemory(_pbr, (ushort)(baseAddress + i));
            }
            _cacheValid[line] = true;
        }

        // GSU data access to ROM, through ROMBR:R14 - see Venus_SuperFX.md §5.2.
        private byte ReadRomBuffer() => ReadRom(_rombr, R[14]);
    }
}
