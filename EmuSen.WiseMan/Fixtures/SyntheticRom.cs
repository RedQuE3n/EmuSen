using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a minimal, synthetic (not real game data - never a copyrighted
    // ROM) SNES LoROM for tests that need a genuinely loaded VenusCore/
    // EmulatorSession rather than a null IDebugTarget - Cartridge's
    // constructor insists on a real file on disk (LoadRom(path) takes a
    // path, not a byte[]), so there's no way to hand it ROM bytes directly.
    public static class SyntheticRom
    {
        public const int Size = 32768;

        // NOP ($EA, a 1-byte instruction) fill, not zero - zero decodes as
        // BRK (a 2-byte instruction on the 65816), and an odd-length gap
        // of BRKs between two placed instructions misaligns a linear
        // disassembly scan before it ever reaches the second one (the same
        // "may misalign through embedded data" caveat this project's own
        // docs already flag for disasm/callers/writers/readers) - NOP
        // padding sidesteps that ambiguity for any test placing more than
        // one instruction in the same ROM.
        //
        // <patches> are (file offset, raw bytes) pairs. Bank $00, offset
        // $8000-$FFFF maps 1:1 to file offset 0-$7FFF (see Cartridge.
        // Read8's own LoROM mapping), so a patch at file offset N lands at
        // CPU address $00:8000+N.
        public static byte[] Build(params (int Offset, byte[] Bytes)[] patches)
        {
            byte[] rom = new byte[Size];
            Array.Fill(rom, (byte)0xEA);

            // Boot stub: point reset at $008000 and enable auto-joypad read, the way every real game does - see Venus_Memory.md §4.4.
            Array.Copy(new byte[] { 0xA9, 0x01, 0x8D, 0x00, 0x42 }, 0, rom, 0, 5);
            rom[0x7FFC] = 0x00;
            rom[0x7FFD] = 0x80;

            foreach (var (offset, bytes) in patches)
            {
                Array.Copy(bytes, 0, rom, offset, bytes.Length);
            }
            return rom;
        }

        // Convenience for the common "just needs *a* loaded core, content
        // doesn't matter" case (e.g. the audio-drain tests).
        public static byte[] BuildBlank() => Build();

        // LoROM header offsets: title at +$00, map mode at +$15, cartridge
        // type at +$16, SRAM size at +$18, chip subtype at -$01.
        private const int CartNameOffset = 0x7FC0;
        private const int MapModeOffset = 0x7FD5;
        private const int CartTypeOffset = 0x7FD6;
        private const int SramSizeOffset = 0x7FD8;
        private const int ChipTypeOffset = 0x7FBF;

        // Cartridge type $15 is what makes Cartridge build a GSU - see Venus_SuperFX.md §1.
        public static byte[] BuildSuperFx(params (int Offset, byte[] Bytes)[] patches)
        {
            var withHeader = new List<(int, byte[])>
            {
                (MapModeOffset, new byte[] { 0x20 }),
                (MapModeOffset + 1, new byte[] { 0x15 }),
            };
            withHeader.AddRange(patches.Select(p => (p.Offset, p.Bytes)));
            return Build(withHeader.ToArray());
        }

        // Same ROM, but declaring the SA-1 map mode and 32KB of BW-RAM, which
        // is what makes Cartridge build the coprocessor - see Venus_SA1.md §1.
        public static byte[] BuildSa1(params (int Offset, byte[] Bytes)[] patches)
        {
            var withHeader = new List<(int, byte[])>
            {
                (MapModeOffset, new byte[] { 0x23 }),
                (SramSizeOffset, new byte[] { 0x05 }),
            };
            withHeader.AddRange(patches.Select(p => (p.Offset, p.Bytes)));
            return Build(withHeader.ToArray());
        }

        // Cartridge type $03 is what makes Cartridge look for a NEC DSP, and
        // the title alone picks the revision - see Venus_NecDSP.md §1. The
        // firmware is appended to the ROM the way a real dump carries it.
        public static byte[] BuildNecDsp(string cartName, byte[]? firmware = null, byte cartType = 0x03, byte chipType = 0x00)
        {
            byte[] rom = Build(
                (MapModeOffset, new byte[] { 0x20 }),
                (CartTypeOffset, new byte[] { cartType }),
                (ChipTypeOffset, new byte[] { chipType }),
                (CartNameOffset, PaddedCartName(cartName)));

            if (firmware == null || firmware.Length == 0) return rom;

            byte[] combined = new byte[rom.Length + firmware.Length];
            Array.Copy(rom, combined, rom.Length);
            Array.Copy(firmware, 0, combined, rom.Length, firmware.Length);
            return combined;
        }

        // Cartridge type $25 with 8KB of SRAM - the OBC1 needs the SRAM,
        // since that is the only memory it has - see Venus_OBC1.md §1.
        public static byte[] BuildObc1() => Build(
            (MapModeOffset, new byte[] { 0x20 }),
            (CartTypeOffset, new byte[] { 0x25 }),
            (SramSizeOffset, new byte[] { 0x03 }));

        // 21 bytes, space-padded, in the SNES's own single-byte encoding.
        private static byte[] PaddedCartName(string cartName)
        {
            byte[] name = new byte[0x15];
            Array.Fill(name, (byte)' ');
            byte[] encoded = System.Text.Encoding.Latin1.GetBytes(cartName);
            Array.Copy(encoded, name, Math.Min(encoded.Length, name.Length));
            return name;
        }

        // Writes <rom> to a fresh, uniquely-named temp file (avoids
        // collisions between concurrently-running tests reusing the same
        // filename) and loads a VenusCore from it.
        public static VenusCore LoadCore(byte[] rom)
        {
            var core = new VenusCore(headless: true);
            core.LoadRom(WriteTempFile(rom));
            return core;
        }

        public static EmulatorSession LoadSession(byte[] rom)
        {
            var session = new EmulatorSession();
            session.LoadRom(WriteTempFile(rom));
            return session;
        }

        private static string WriteTempFile(byte[] rom)
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_synth_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(path, rom);
            return path;
        }
    }
}
