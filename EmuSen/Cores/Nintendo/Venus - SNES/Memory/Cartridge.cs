using System;
using System.IO;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    public class Cartridge
    {
        [EmuSen.Common.SkipInState] private byte[] _rom;
        private byte[] _sram;
        public int SramSize => _sram.Length;

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

            // SRAM size from the ROM header ($00:FFD8) - see Venus_Memory.md §2.2.
            int sramSize = 0;
            if (_rom.Length > 0x7FD8)
            {
                int ramSizeExponent = _rom[0x7FD8];
                if (ramSizeExponent > 0)
                {
                    // Clamp to SnesLab's documented real-hardware max (512KB).
                    ramSizeExponent = Math.Min(ramSizeExponent, 7);
                    sramSize = 1024 << ramSizeExponent;
                }
            }
            _sram = new byte[sramSize];

            string saveDir = Path.Combine(Directory.GetCurrentDirectory(), "var", "games");
            string romName = Path.GetFileNameWithoutExtension(romPath);
            SavePath = Path.Combine(saveDir, romName + ".srm");
            LoadSram();

            Console.WriteLine("=== Cartridge Loaded ===");
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
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);

            // --- LoROM ROM Mapping ---
            // ROM is mapped to the upper 32KB ($8000-$FFFF) of banks $00-$3F and $80-$BF
            if (offset >= 0x8000)
            {
                // Masking the bank to 0x7F handles the mirror between the lower and upper banks
                uint romAddr = (uint)(((bank & 0x7F) * 0x8000) + (offset - 0x8000));
                
                if (romAddr < _rom.Length)
                {
                    return _rom[romAddr];
                }
            }
            
            // SRAM mirroring (modulo, not a hard range check) - see
            // Venus_Memory.md §2.3 for why this matters beyond correctness.
            if (offset < 0x8000 && ((bank >= 0x70 && bank <= 0x7D) || (bank >= 0xF0 && bank <= 0xFF)) && _sram.Length > 0)
            {
                return _sram[offset % _sram.Length];
            }

            // Unmapped memory (Open Bus)
            return 0x00; 
        }

        public void Write8(uint address, byte data)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);

            // ROM is read-only; only SRAM is writable. Same mirroring as Read8.
            if (offset < 0x8000 && ((bank >= 0x70 && bank <= 0x7D) || (bank >= 0xF0 && bank <= 0xFF)) && _sram.Length > 0)
            {
                _sram[offset % _sram.Length] = data;
            }
        }
    }
}