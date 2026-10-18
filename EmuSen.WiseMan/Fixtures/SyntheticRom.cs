using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Crystal.Debugger;

namespace EmuSen.WiseMan.Fixtures
{
    // A minimal synthetic SNES LoROM, never a copyrighted one, loaded as a frontend loads it - see VenusRT_Native.md §68.6.
    public static class SyntheticRom
    {
        public const int Size = 32768;

        // NOP fill, so a linear disassembly never misaligns; a patch at file offset N lands at CPU $00:8000+N - see VenusRT_Native.md §68.6.
        public static byte[] Build(params (int Offset, byte[] Bytes)[] patches)
        {
            byte[] rom = new byte[Size];
            Array.Fill(rom, (byte)0xEA);

            // Boot stub: reset at $008000 and auto-joypad read enabled, as every real game does - see VenusRT_Native.md §68.6.
            Array.Copy(new byte[] { 0xA9, 0x01, 0x8D, 0x00, 0x42 }, 0, rom, 0, 5);
            rom[0x7FFC] = 0x00;
            rom[0x7FFD] = 0x80;

            foreach (var (offset, bytes) in patches)
            {
                Array.Copy(bytes, 0, rom, offset, bytes.Length);
            }
            return rom;
        }

        // For a test that needs a loaded core and no particular program.
        public static byte[] BuildBlank() => Build();

        // LoROM header offsets: title at +$00, map mode at +$15, chipset at +$16, SRAM size at +$18, chip subtype at -$01.
        private const int CartNameOffset = 0x7FC0;
        private const int MapModeOffset = 0x7FD5;
        private const int CartTypeOffset = 0x7FD6;
        private const int SramSizeOffset = 0x7FD8;
        private const int ChipTypeOffset = 0x7FBF;

        // fullsnes: chipset $15 on map mode $20 is a GSU board.
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

        // fullsnes: an SA-1 board is chipset 32h-35h on map mode 23h; here with 32 KiB of BW-RAM.
        public static byte[] BuildSa1(params (int Offset, byte[] Bytes)[] patches)
        {
            var withHeader = new List<(int, byte[])>
            {
                (MapModeOffset, new byte[] { 0x23 }),
                (CartTypeOffset, new byte[] { 0x35 }),
                (SramSizeOffset, new byte[] { 0x05 }),
            };
            withHeader.AddRange(patches.Select(p => (p.Offset, p.Bytes)));
            return Build(withHeader.ToArray());
        }

        // fullsnes: chipset $03 is a NEC DSP and the title picks the revision; firmware, when given, is appended as a dump carries it.
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

        // fullsnes: chipset $25 is the OBC1, whose only memory is the 8 KiB of SRAM declared here.
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

        // The image on the SNES's engine, VenusRT, from a temp file of its own, so concurrent tests never share one.
        public static ICore LoadCore(byte[] rom, CoreOptions? options = null) => CoreFactory.Loaded(WriteTempFile(rom), options: options);

        // The engine's debug target, as a frontend's bundle gives it.
        public static IDebugTarget TargetOf(ICore core) => CoreFactory.Bundle(core).DebugTarget;

        public static IDebugTarget LoadTarget(byte[] rom) => TargetOf(LoadCore(rom));

        // A program that copies the auto-read's pad 1 and pad 2 words to $0000-$0003, forever.
        public static byte[] BuildPadReader() => Build((5, new byte[]
        {
            0xAD, 0x18, 0x42, 0x8D, 0x00, 0x00, 0xAD, 0x19, 0x42, 0x8D, 0x01, 0x00,
            0xAD, 0x1A, 0x42, 0x8D, 0x02, 0x00, 0xAD, 0x1B, 0x42, 0x8D, 0x03, 0x00,
            0x4C, 0x05, 0x80,
        }));

        // The two words BuildPadReader's program last copied, after two frames so an auto-read has latched what is held.
        public static (ushort One, ushort Two) Pads(ICore core, int frames = 2)
        {
            for (int i = 0; i < frames; i++) core.RunFrame();
            var engine = (EmuSen.Cores.Native.CoreEngine)core;
            ushort Word(int at) => (ushort)(engine.ReadSpace("WRAM", at) | engine.ReadSpace("WRAM", at + 1) << 8);
            return (Word(0), Word(2));
        }

        public static EmulatorSession LoadSession(byte[] rom, CoreOptions? options = null)
        {
            var session = new EmulatorSession { Options = options ?? CoreOptions.Default };
            session.LoadRom(WriteTempFile(rom));
            return session;
        }

        public static string WriteTempFile(byte[] rom)
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_synth_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(path, rom);
            return path;
        }
    }
}
