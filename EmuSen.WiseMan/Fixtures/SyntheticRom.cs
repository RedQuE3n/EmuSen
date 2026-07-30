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
            foreach (var (offset, bytes) in patches)
            {
                Array.Copy(bytes, 0, rom, offset, bytes.Length);
            }
            return rom;
        }

        // Convenience for the common "just needs *a* loaded core, content
        // doesn't matter" case (e.g. the audio-drain tests).
        public static byte[] BuildBlank() => Build();

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
