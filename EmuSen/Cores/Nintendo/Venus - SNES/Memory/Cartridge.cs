using System;
using System.IO;
using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    public class Cartridge
    {
        [EmuSen.Common.SkipInState] private byte[] _rom;
        private byte[] _sram;
        public int SramSize => _sram.Length;

        // All three are derived from the ROM file, which LoadRom re-reads
        // before any state load, so none belong in a save state.
        [EmuSen.Common.SkipInState] private readonly ICartridgeMapper _mapper;
        [EmuSen.Common.SkipInState] private readonly bool _isHiRom;

        private const int LoRomHeader = 0x7FC0;
        private const int HiRomHeader = 0xFFC0;

        public string MapperName => _mapper.Name;

        // Saves/<rom-name>.srm - see Venus_Memory.md §2.4.
        public string SavePath { get; }

        public Cartridge(string romPath)
        {
            byte[] fileBytes = File.ReadAllBytes(romPath);
            
            // SNES ROM copiers often appended a 512-byte header to the file.
            // A standard LoROM without a header will divide cleanly by 32KB (32768 bytes).
            int headerSize = (fileBytes.Length % 32768 == 512) ? 512 : 0;
            
            if (headerSize > 0)
            {
                Console.WriteLine("[Cartridge] Copier header detected. Stripping 512 bytes...");
            }

            _rom = new byte[fileBytes.Length - headerSize];
            Array.Copy(fileBytes, headerSize, _rom, 0, _rom.Length);

            // Which of the two header locations is real decides the whole
            // memory map - see Venus_Memory.md §2.1a.
            _isHiRom = ScoreHeader(_rom, HiRomHeader, hiRom: true) > ScoreHeader(_rom, LoRomHeader, hiRom: false);
            _mapper = _isHiRom ? new HiRomMapper() : new LoRomMapper();
            int headerBase = _isHiRom ? HiRomHeader : LoRomHeader;

            // SRAM size from the ROM header (+$18) - see Venus_Memory.md §2.2.
            int sramSize = 0;
            if (_rom.Length > headerBase + 0x18)
            {
                int ramSizeExponent = _rom[headerBase + 0x18];
                if (ramSizeExponent > 0)
                {
                    // Clamp to SnesLab's documented real-hardware max (512KB).
                    ramSizeExponent = Math.Min(ramSizeExponent, 7);
                    sramSize = 1024 << ramSizeExponent;
                }
            }
            _sram = new byte[sramSize];

            string saveDir = DianaOSSandbox.SavesDirectory;
            string romName = Path.GetFileNameWithoutExtension(romPath);
            SavePath = Path.Combine(saveDir, romName + ".srm");
            LoadSram();

            Console.WriteLine("=== Cartridge Loaded ===");
            Console.WriteLine($"Mapper: {_mapper.Name}");
            Console.WriteLine($"ROM Size: {_rom.Length / 1024} KB");
            Console.WriteLine($"SRAM Size: {_sram.Length / 1024} KB");
            Console.WriteLine($"Save Path: {SavePath}");
            Console.WriteLine("========================");
        }

        // Tolerant of a save file that doesn't match the allocated SRAM
        // size - see Venus_Memory.md §2.4.
        private void LoadSram()
        {
            try
            {
                if (!File.Exists(SavePath)) return;

                byte[] saved = File.ReadAllBytes(SavePath);
                int count = Math.Min(saved.Length, _sram.Length);
                Array.Copy(saved, _sram, count);
                Console.WriteLine($"[Cartridge] Loaded save: {SavePath} ({count} bytes)");
            }
            catch (Exception ex)
            {
                // A save that fails to load shouldn't prevent the game itself
                // from booting - worst case the player starts without their
                // save data, not with a crashed emulator.
                Console.WriteLine($"[Cartridge] Failed to load save ({SavePath}): {ex.Message}");
            }
        }

        // Called periodically + on shutdown, not on every write - see
        // Venus_Memory.md §2.4.
        public void SaveSram()
        {
            try
            {
                string? dir = Path.GetDirectoryName(SavePath);
                if (!string.IsNullOrEmpty(dir)) Directory.CreateDirectory(dir);
                File.WriteAllBytes(SavePath, _sram);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[Cartridge] Failed to save ({SavePath}): {ex.Message}");
            }
        }

        public byte Read8(uint address)
        {
            var mapped = _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF));
            switch (mapped.Region)
            {
                case CartridgeRegion.Rom:
                    return mapped.Offset < _rom.Length ? _rom[mapped.Offset] : (byte)0x00;

                // SRAM mirroring (modulo, not a hard range check) - see
                // Venus_Memory.md §2.3 for why this matters beyond correctness.
                case CartridgeRegion.Sram:
                    return _sram.Length > 0 ? _sram[mapped.Offset % _sram.Length] : (byte)0x00;

                default:
                    return 0x00; // open bus
            }
        }

        public void Write8(uint address, byte data)
        {
            // ROM is read-only; only SRAM is writable.
            var mapped = _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF));
            if (mapped.Region == CartridgeRegion.Sram && _sram.Length > 0)
            {
                _sram[mapped.Offset % _sram.Length] = data;
            }
        }

        // Lets MemoryBus route a low-half address in a hardware bank here
        // instead of returning open bus - see Venus_Memory.md §2.1a.
        public bool MapsAddress(uint address)
        {
            return _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF)).Region != CartridgeRegion.Unmapped;
        }

        // Higher score wins - see Venus_Memory.md §2.1a.
        private static int ScoreHeader(byte[] rom, int baseAddr, bool hiRom)
        {
            if (rom.Length < baseAddr + 0x20) return -1;

            int score = 0;
            int mapMode = rom[baseAddr + 0x15] & 0x0F;
            bool modeMatches = hiRom ? (mapMode == 0x01 || mapMode == 0x05) : (mapMode == 0x00 || mapMode == 0x02 || mapMode == 0x03);
            if (modeMatches) score += 2;

            int complement = rom[baseAddr + 0x1C] | (rom[baseAddr + 0x1D] << 8);
            int checksum = rom[baseAddr + 0x1E] | (rom[baseAddr + 0x1F] << 8);
            if (checksum != 0 && (checksum ^ complement) == 0xFFFF) score += 2;

            return score;
        }
    }
}