using System;
using System.IO;

namespace EmuSen.Memory
{
    public class Cartridge
    {
        [EmuSen.Common.SkipInState] private byte[] _rom;
        private byte[] _sram;
        public int SramSize => _sram.Length;

        // Saves/<rom-name>.srm - a dedicated folder sibling to Roms/ and Logs/,
        // not derived from the ROM's own directory (which could be anywhere on
        // disk, possibly read-only, and isn't necessarily "ours" to write into -
        // ROMs can be loaded from any path via the CLI arg or the Avalonia
        // frontend's file picker). Only the save FILENAME comes from the ROM;
        // the folder itself is always relative to where the emulator runs from,
        // same convention Logs/ already uses.
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

            // RAM Size byte lives at SNES address $00:FFD8, which for LoROM
            // maps to ROM file offset $7FD8 (bank 0, offset 0xFFD8 - 0x8000).
            // 0 means "no SRAM"; any other value N means the cart has
            // 1KB << N of SRAM (verified via SNESdev wiki's ROM header page
            // and a WLA-DX header example using SRAMSIZE $00 for "no SRAM").
            // Previously this was hardcoded to 2KB for every game, which
            // happens to match SMW but is wrong for anything else - Super
            // Metroid's RAM Size byte is 3 (8KB), and its anti-piracy boot
            // check specifically writes/reads across the $702000-$703FFF
            // vs $700000-$701FFF mirror that only exists at the *real*
            // chip size, so an undersized array made that check fail.
            int sramSize = 0;
            if (_rom.Length > 0x7FD8)
            {
                int ramSizeExponent = _rom[0x7FD8];
                if (ramSizeExponent > 0)
                {
                    // Clamp defensively - SnesLab documents 7 (512KB) as the
                    // maximum value real hardware/games use; a corrupt or
                    // unusual header shouldn't be able to make us allocate
                    // something absurd.
                    ramSizeExponent = Math.Min(ramSizeExponent, 7);
                    sramSize = 1024 << ramSizeExponent;
                }
            }
            _sram = new byte[sramSize];

            string saveDir = Path.Combine(Directory.GetCurrentDirectory(), "Saves");
            string romName = Path.GetFileNameWithoutExtension(romPath);
            SavePath = Path.Combine(saveDir, romName + ".srm");
            LoadSram();

            Console.WriteLine("=== Cartridge Loaded ===");
            Console.WriteLine($"ROM Size: {_rom.Length / 1024} KB");
            Console.WriteLine($"SRAM Size: {_sram.Length / 1024} KB");
            Console.WriteLine($"Save Path: {SavePath}");
            Console.WriteLine("========================");
        }

        // Reads an existing .srm into _sram if one exists. Tolerant of a
        // save file that doesn't exactly match the allocated SRAM size
        // (copies whichever is smaller) rather than failing outright - a
        // mismatch most likely means this ROM's header-reported SRAM size
        // differs from whatever created the file (e.g. re-dumped with a
        // different header, or an old save from before header-based sizing),
        // not a corrupted save.
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

        // Writes the current SRAM contents to disk. Safe to call frequently -
        // SRAM is a few KB at most, and this is a plain overwrite, not an
        // append. Called periodically and on shutdown (see Program.cs /
        // EmulatorSession.cs) rather than on every single SRAM write, since
        // some games touch SRAM quite often during normal play.
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
            
            // --- LoROM SRAM Mapping ---
            // SRAM is mapped to the lower 32KB ($0000-$7FFF) of banks $70-$7D and $F0-$FF.
            // Real SRAM chips only decode as many address lines as their actual
            // size needs, so a chip smaller than the full 32KB window mirrors
            // repeatedly within it - modulo addressing reproduces that for free.
            // This matters beyond just correctness: Super Metroid's boot-time
            // anti-piracy check deliberately writes a test pattern through one
            // mirror address and reads it back through another, specifically to
            // detect cartridges/emulators that DON'T mirror correctly (see
            // tcrf.net/Super_Metroid). Previously this used a hard range check
            // against the array length instead of wrapping, so anything past
            // the (also wrong, hardcoded-2KB) array size silently read as open
            // bus rather than mirroring - failing that check.
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

            // --- LoROM SRAM Mapping ---
            // ROM is read-only, so we only need to handle writes to SRAM.
            // Same mirroring rationale as Read8 above - modulo instead of a
            // hard size cutoff, and an explicit offset<0x8000 bound since
            // only the lower 32KB of these banks is SRAM.
            if (offset < 0x8000 && ((bank >= 0x70 && bank <= 0x7D) || (bank >= 0xF0 && bank <= 0xFF)) && _sram.Length > 0)
            {
                _sram[offset % _sram.Length] = data;
            }
        }
    }
}