using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The Model setting: each cartridge on each console, the Color's palettes for Game Boy games, and states across consoles - see Mercury_Model.md.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryModelTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "mercury_model_" + Guid.NewGuid().ToString("N"));

        public MercuryModelTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_dir);
        }

        public void Dispose()
        {
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        // TCRF's table of the palettes the Color gives each listed game, RGB888 as TCRF converts RGB555 (round(c * 255 / 31)); its Radar Mission row is tested apart.
        public static TheoryData<int, char?, string> TcrfRows => new()
        {
            { 0xB3, 'U', "FFFFFF ADAD84 42737B 000000|FFFFFF FF7300 944200 000000|FFFFFF FF7300 944200 000000" }, // Moguranya (Japan)
            { 0xC6, 'A', "FFFFFF ADAD84 42737B 000000|FFFFFF FF7300 944200 000000|FFFFFF 5ABDFF FF0000 0000FF" }, // Game Boy Wars (Japan)
            { 0xA8, null, "FFFF9C 94B5FF 639473 003A3A|FFC542 FFD600 943A00 4A0000|FFFFFF FF8484 943A3A 000000" }, // Donkey Kong Land (USA, Europe)
            { 0xBF, 'C', "6BFF00 FFFFFF FF524A 000000|FFFFFF FFFFFF 63A5FF 0000FF|FFFFFF FFAD63 843100 000000" }, // Soccer (Europe) (En,Fr,De)
            { 0xCE, null, "6BFF00 FFFFFF FF524A 000000|FFFFFF FFFFFF 63A5FF 0000FF|FFFFFF FFAD63 843100 000000" }, // Soccer (Europe) (En,Fr,De)
            { 0xD1, null, "6BFF00 FFFFFF FF524A 000000|FFFFFF FFFFFF 63A5FF 0000FF|FFFFFF FFAD63 843100 000000" }, // Soccer (Europe) (En,Fr,De)
            { 0xF0, null, "6BFF00 FFFFFF FF524A 000000|FFFFFF FFFFFF 63A5FF 0000FF|FFFFFF FFAD63 843100 000000" }, // Soccer (Europe) (En,Fr,De)
            { 0x36, null, "52DE00 FF8400 FFFF00 FFFFFF|FFFFFF FFFFFF 63A5FF 0000FF|FFFFFF FF8484 943A3A 000000" }, // Baseball (World)
            { 0x34, null, "FFFFFF 7BFF00 B57300 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Game & Watch Gallery (Europe)
            { 0x66, 'E', "FFFFFF 7BFF00 B57300 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Game & Watch Gallery (Europe)
            { 0xF4, ' ', "FFFFFF 7BFF00 B57300 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Game & Watch Gallery (Europe)
            { 0x3D, null, "FFFFFF 52FF00 FF4200 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Mario & Yoshi (Europe)
            { 0x6A, 'I', "FFFFFF 52FF00 FF4200 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Mario & Yoshi (Europe)
            { 0xB3, 'R', "FFFFFF 52FF00 FF4200 000000|FFFFFF 52FF00 FF4200 000000|FFFFFF 5ABDFF FF0000 0000FF" }, // Tetris Attack (USA)
            { 0x71, null, "FFFFFF FF9C00 FF0000 000000|FFFFFF FF9C00 FF0000 000000|FFFFFF FF9C00 FF0000 000000" }, // Balloon Kid (USA, Europe)
            { 0xFF, null, "FFFFFF FF9C00 FF0000 000000|FFFFFF FF9C00 FF0000 000000|FFFFFF FF9C00 FF0000 000000" }, // Balloon Kid (USA, Europe)
            { 0xE0, null, "FFFFFF FF9C00 FF0000 000000|FFFFFF FF9C00 FF0000 000000|FFFFFF 5ABDFF FF0000 0000FF" }, // Yoshi no Cookie (Japan)
            { 0x15, null, "FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000" }, // Pocket Monsters - Pikachu (Japan)
            { 0xDB, null, "FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000" }, // Pocket Monsters - Pikachu (Japan)
            { 0x69, null, "FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000|FFFFFF 5ABDFF FF0000 0000FF" }, // Qix (World)
            { 0xF2, null, "FFFFFF FFFF00 FF0000 000000|FFFFFF FFFF00 FF0000 000000|FFFFFF 5ABDFF FF0000 0000FF" }, // Qix (World)
            { 0x88, null, "A59CFF FFFF00 006300 000000|A59CFF FFFF00 006300 000000|A59CFF FFFF00 006300 000000" }, // Alleyway (World)
            { 0x49, null, "A59CFF FFFF00 006300 000000|FF6352 D60000 630000 000000|0000FF FFFFFF FFFF7B 0084FF" }, // Hoshi no Kirby (Japan)
            { 0x5C, null, "A59CFF FFFF00 006300 000000|FF6352 D60000 630000 000000|0000FF FFFFFF FFFF7B 0084FF" }, // Hoshi no Kirby (Japan)
            { 0xB3, 'B', "A59CFF FFFF00 006300 000000|FF6352 D60000 630000 000000|0000FF FFFFFF FFFF7B 0084FF" }, // Hoshi no Kirby (Japan)
            { 0xC9, null, "FFFFCE 63EFEF 9C8431 5A5A5A|FFFFFF FF7300 944200 000000|FFFFFF 63A5FF 0000FF 000000" }, // Super Mario Land 2 - 6 Golden Coins
            { 0x46, 'E', "B5B5FF FFFF94 AD5A42 000000|000000 FFFFFF FF8484 943A3A|000000 FFFFFF FF8484 943A3A" }, // Super Mario Land (World)
            { 0x61, 'E', "FFFFFF 63A5FF 0000FF 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 63A5FF 0000FF 000000" }, // Pocket Monsters Ao (Japan)
            { 0x9C, null, "FFFFFF 8C8CDE 52528C 000000|FFFFFF 8C8CDE 52528C 000000|FFC542 FFD600 943A00 4A0000" }, // Pinocchio (Europe)
            { 0x6A, 'K', "FFFFFF 8C8CDE 52528C 000000|FFC542 FFD600 943A00 4A0000|FFFFFF 5ABDFF FF0000 0000FF" }, // Donkey Kong Land (Japan)
            { 0x6B, null, "FFFFFF 8C8CDE 52528C 000000|FFC542 FFD600 943A00 4A0000|FFFFFF 5ABDFF FF0000 0000FF" }, // Donkey Kong Land (Japan)
            { 0xD3, 'R', "FFFFFF 8C8CDE 52528C 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 8C8CDE 52528C 000000" }, // Kaeru no Tame ni Kane wa Naru (Japan)
            { 0x28, 'F', "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 1
            { 0x4B, null, "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 1
            { 0x90, null, "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 1
            { 0x9A, null, "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 1
            { 0xBD, null, "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 1
            { 0x27, 'N', "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 63A5FF 0000FF 000000" }, // Magnetic Soccer (Europe)
            { 0x61, 'A', "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 63A5FF 0000FF 000000" }, // Magnetic Soccer (Europe)
            { 0x8B, null, "FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 63A5FF 0000FF 000000" }, // Magnetic Soccer (Europe)
            { 0x39, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 63A5FF 0000FF 000000" }, // Chessmaster, The (Europe)
            { 0x43, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 63A5FF 0000FF 000000" }, // Chessmaster, The (Europe)
            { 0x97, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 63A5FF 0000FF 000000" }, // Chessmaster, The (Europe)
            { 0x10, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x29, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x52, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x5D, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x68, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x6D, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0xF6, null, "FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000|FFFFFF 7BFF31 008400 000000" }, // Adventures of Lolo (Europe)
            { 0x14, null, "FFFFFF FF8484 943A3A 000000|FFFFFF 7BFF31 008400 000000|FFFFFF FF8484 943A3A 000000" }, // Game Boy Camera Gold (USA)
            { 0x70, null, "FFFFFF FF8484 943A3A 000000|FFFFFF 00FF00 318400 004A00|FFFFFF 63A5FF 0000FF 000000" }, // Link's Awakening
            { 0x0C, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x16, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x35, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x67, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x75, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x92, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0x99, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0xB7, null, "FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000|FFFFFF FFAD63 843100 000000" }, // F-1 Race (World)
            { 0xF7, null, "FFFFFF FFAD63 843100 000000|FFFFFF 7BFF31 008400 000000|FFFFFF 63A5FF 0000FF 000000" }, // A Boy and His Blob (Europe)
            { 0x28, 'A', "000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF" }, // Arcade Classic No. 3
            { 0xA5, 'A', "000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF" }, // Arcade Classic No. 3
            { 0xE8, null, "000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF|000000 008484 FFDE00 FFFFFF" }, // Arcade Classic No. 3
            { 0x46, 'R', "FFFFFF 63A5FF 0000FF 000000|FFFF00 FF0000 630000 000000|FFFFFF 7BFF31 008400 000000" }, // Metroid II (World)
            { 0xD3, 'I', "FFFFFF ADAD84 42737B 000000|FFFFFF FFAD63 843100 000000|FFFFFF 63A5FF 0000FF 000000" }, // Wario Land II (USA, Europe)
            { 0x58, null, "FFFFFF A5A5A5 525252 000000|FFFFFF A5A5A5 525252 000000|FFFFFF A5A5A5 525252 000000" }, // X (Japan)
            { 0x6F, null, "FFFFFF FFCE00 9C6300 000000|FFFFFF FFCE00 9C6300 000000|FFFFFF FFCE00 9C6300 000000" }, // Pocket Camera (Japan)
            { 0xAA, null, "FFFFFF 7BFF31 0063C5 000000|FFFFFF FF8484 943A3A 000000|FFFFFF 7BFF31 0063C5 000000" }, // James Bond 007 (USA, Europe)
            { 0x18, 'I', "FFFFFF 7BFF31 0063C5 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 2
            { 0x3F, null, "FFFFFF 7BFF31 0063C5 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 2
            { 0x66, 'L', "FFFFFF 7BFF31 0063C5 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 2
            { 0xC6, ' ', "FFFFFF 7BFF31 0063C5 000000|FFFFFF FF8484 943A3A 000000|FFFFFF FF8484 943A3A 000000" }, // Arcade Classic No. 2
        };

        // A header whose sixteen title bytes sum to the checksum with the fourth letter in place, under the licensee given.
        private static byte[] Header(int checksum, char? fourth, byte licensee = 0x01, byte cgb = 0x00, string newLicensee = "\0\0")
        {
            byte[] rom = SyntheticGbRom.Build(cgbFlag: cgb, title: "");
            Array.Clear(rom, 0x134, 16);
            rom[0x143] = cgb;
            rom[0x137] = (byte)(fourth ?? '\0');
            rom[0x134] = (byte)(checksum - rom[0x137] - cgb);
            rom[0x144] = (byte)newLicensee[0];
            rom[0x145] = (byte)newLicensee[1];
            rom[0x14B] = licensee;
            rom[SyntheticGbRom.HeaderChecksumAddress] = SyntheticGbRom.HeaderChecksum(rom);
            return rom;
        }

        private static string Rgb888(ushort[] palette) =>
            string.Join(' ', palette.Select(c => $"{Scale(c & 0x1F):X2}{Scale((c >> 5) & 0x1F):X2}{Scale((c >> 10) & 0x1F):X2}"));

        private static int Scale(int five) => (int)Math.Round(five * 255 / 31.0, MidpointRounding.AwayFromZero);

        // What a Game Boy game on the Color holds in colour RAM before its first instruction.
        private static string PalettesOn(byte[] rom)
        {
            using MercuryRig core = MercuryRig.Create(rom, GbModel.GameBoyColor);
            byte[] bg = core.FieldBytes("Bus.Ppu.BgPaletteRam"), obj = core.FieldBytes("Bus.Ppu.ObjPaletteRam");
            ushort[] Entries(byte[] ram, int at) => Enumerable.Range(0, 4).Select(i => (ushort)(ram[at + i * 2] | (ram[at + i * 2 + 1] << 8))).ToArray();
            return $"{Rgb888(Entries(bg, 0))}|{Rgb888(Entries(obj, 0))}|{Rgb888(Entries(obj, 8))}";
        }

        // Each listed game's palettes in TCRF's words, as the machine's colour RAM holds them - see Mercury_Native.md §12.1.
        [Theory]
        [MemberData(nameof(TcrfRows))]
        public void The_colour_palette_for_a_listed_game_is_the_one_tcrf_documents(int checksum, char? fourth, string expected)
        {
            Assert.Equal(expected, PalettesOn(Header(checksum, fourth)));
        }

        // TCRF lists Radar Mission ($8C) as entry $01 with flags $00; the boot ROM's own tables give $8C palette number 8, entry $00 with flags $01 - see Mercury_Model.md §3.3.
        [Fact]
        public void Radar_mission_follows_the_boot_roms_table_where_tcrf_disagrees()
        {
            string loaded = PalettesOn(Header(0x8C, null));
            Assert.Equal("FFFFFF ADAD84 42737B 000000|FFFFFF FF7300 944200 000000|FFFFFF ADAD84 42737B 000000", loaded);
            Assert.Equal(FrozenPalettes()[8].Palettes, loaded);
            Assert.NotEqual("FFFF9C 94B5FF 639473 003A3A|FFFF9C 94B5FF 639473 003A3A|FFFF9C 94B5FF 639473 003A3A", loaded);
        }

        // Pan Docs' default: a game not Nintendo's, not listed, or listed as ambiguous with no matching letter gets palette 0; the machine loads the number's palettes.
        [Theory]
        [InlineData(0x70, null, 0x00, "\0\0", 0)]
        [InlineData(0x70, null, 0x33, "08", 0)]
        [InlineData(0x70, null, 0x33, "01", 15)]
        [InlineData(0x70, null, 0x01, "\0\0", 15)]
        [InlineData(0x01, null, 0x01, "\0\0", 55)]
        [InlineData(0x02, null, 0x01, "\0\0", 0)]
        [InlineData(0xB3, 'Z', 0x01, "\0\0", 0)]
        [InlineData(0xB3, 'B', 0x01, "\0\0", 65)]
        [InlineData(0xB3, 'U', 0x01, "\0\0", 79)]
        [InlineData(0xB3, 'R', 0x01, "\0\0", 93)]
        [InlineData(0x46, 'R', 0x01, "\0\0", 80)]
        public void The_palette_number_follows_the_licensee_the_checksum_and_the_fourth_letter(int checksum, char? fourth, byte licensee, string newLicensee, int number)
        {
            Assert.Equal(FrozenPalettes()[number].Palettes, PalettesOn(Header(checksum, fourth, licensee, newLicensee: newLicensee)));
        }

        [Fact]
        public void The_default_palette_is_the_one_pan_docs_gives()
        {
            ushort[] bg = { 0x7FFF, 0x1BEF, 0x6180, 0x0000 }, obj = { 0x7FFF, 0x421F, 0x1CF2, 0x0000 };
            Assert.Equal($"{Rgb888(bg)}|{Rgb888(obj)}|{Rgb888(obj)}", FrozenPalettes()[0].Palettes);
            Assert.Equal(FrozenPalettes()[0].Palettes, PalettesOn(Header(0x70, null, licensee: 0x00)));
        }

        public static string FrozenPalettesPath([System.Runtime.CompilerServices.CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(here)!, "MercuryGoldens", "palettes.tsv");

        // The Color's 94 palettes for Game Boy games, each with a Nintendo header that selects it, frozen from C# Mercury's table before it left - see Mercury_Native.md §14.6.
        public static IReadOnlyList<(int Checksum, char Fourth, string Palettes)> FrozenPalettes() => File.ReadAllLines(FrozenPalettesPath())
            .Where(l => l.Length > 0 && l[0] != '#').Skip(1).Select(l => l.Split('\t'))
            .Select(c => (Convert.ToInt32(c[1], 16), (char)Convert.ToInt32(c[2], 16), c[3])).ToArray();

        // Each frozen palette, selected by its header, is what MercuryRT loads into colour RAM.
        [Fact]
        public void Every_frozen_palette_is_what_mercuryrt_loads()
        {
            var rows = FrozenPalettes();
            Assert.Equal(94, rows.Count);
            for (int number = 0; number < rows.Count; number++)
                Assert.True(rows[number].Palettes == PalettesOn(Header(rows[number].Checksum, rows[number].Fourth == '\0' ? null : rows[number].Fourth)), $"palette {number}");
        }

        // Stores A, B, C, D, E, H, L and F at $C000, then reads the colour registers, tries a speed switch and a WRAM bank, and draws four shades.
        private static readonly byte[] Probe =
        {
            0xEA, 0x00, 0xC0, 0x78, 0xEA, 0x01, 0xC0, 0x79, 0xEA, 0x02, 0xC0, 0x7A, 0xEA, 0x03, 0xC0, 0x7B, 0xEA, 0x04, 0xC0,
            0x7C, 0xEA, 0x05, 0xC0, 0x7D, 0xEA, 0x06, 0xC0, 0xF5, 0xE1, 0x7D, 0xEA, 0x07, 0xC0,
            0xF0, 0x4C, 0xEA, 0x10, 0xC0, 0xF0, 0x4D, 0xEA, 0x11, 0xC0, 0xF0, 0x4F, 0xEA, 0x12, 0xC0, 0xF0, 0x55, 0xEA, 0x13, 0xC0,
            0xF0, 0x68, 0xEA, 0x14, 0xC0, 0xF0, 0x69, 0xEA, 0x15, 0xC0, 0xF0, 0x6A, 0xEA, 0x16, 0xC0, 0xF0, 0x6B, 0xEA, 0x17, 0xC0,
            0xF0, 0x6C, 0xEA, 0x18, 0xC0, 0xF0, 0x70, 0xEA, 0x19, 0xC0, 0xF0, 0x02, 0xEA, 0x1A, 0xC0,
            0x3E, 0x01, 0xE0, 0x4D, 0x10, 0x00, 0xF0, 0x4D, 0xEA, 0x1B, 0xC0,
            0x3E, 0x02, 0xE0, 0x70, 0xF0, 0x70, 0xEA, 0x1C, 0xC0,
            0xAF, 0xE0, 0x40, 0x21, 0x00, 0x80, 0x06, 0x08, 0x3E, 0x55, 0x22, 0x3E, 0x33, 0x22, 0x05, 0x20, 0xF7,
            0x3E, 0xE4, 0xE0, 0x47, 0x3E, 0x91, 0xE0, 0x40, 0x18, 0xFE,
        };

        private static byte[] ProbeRom(byte cgb, int checksum = 0, byte licensee = 0x00)
        {
            byte[] rom = Header(checksum, null, licensee, cgb);
            Probe.CopyTo(rom, SyntheticGbRom.EntryPoint);
            rom[SyntheticGbRom.HeaderChecksumAddress] = SyntheticGbRom.HeaderChecksum(rom);
            return rom;
        }

        public static TheoryData<byte, GbModel> Cases
        {
            get
            {
                var cases = new TheoryData<byte, GbModel>();
                foreach (byte cgb in new byte[] { 0x00, 0x80, 0xC0 })
                    foreach (GbModel model in Enum.GetValues<GbModel>())
                        cases.Add(cgb, model);
                return cases;
            }
        }

        // What each console hands the cartridge and shows it - see Mercury_Model.md §2.
        [Theory]
        [MemberData(nameof(Cases))]
        public void Each_cartridge_on_each_console_runs_as_that_console(byte cgb, GbModel model)
        {
            using MercuryRig rig = MercuryRig.Create(ProbeRom(cgb), model);
            for (int f = 0; f < 10; f++) rig.RunFrame();
            byte[] wram = rig.FieldBytes("Bus.Wram");
            bool colourConsole = model == GbModel.GameBoyColor || (model == GbModel.Auto && cgb != 0);
            string kind = !colourConsole ? "Game Boy" : cgb == 0 ? "Color, compatibility" : "Color";
            _output.WriteLine($"${cgb:X2} on {model}: {kind}; A {wram[0]:X2} F {wram[7]:X2} BC {wram[1]:X2}{wram[2]:X2} DE {wram[3]:X2}{wram[4]:X2} HL {wram[5]:X2}{wram[6]:X2}; FF4C-FF70 {string.Join(" ", wram[0x10..0x1D].Select(b => b.ToString("X2")))}");

            Assert.Equal(colourConsole, rig.Flag("Console"));
            Assert.Equal(colourConsole ? "GBC" : "GB", rig.CoreName);
            Assert.Equal(colourConsole ? 0x4000 : 0x2000, rig.FieldBytes("Bus.Vram").Length);

            byte[] registers = wram[0..8];
            byte[] frame = rig.GetFrameBufferRgba();
            if (!colourConsole)
            {
                Assert.Equal(new byte[] { 0x01, 0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D, 0xB0 }, registers);
                Assert.False(rig.Flag("Bus.DoubleSpeed"));
                Assert.Equal(new byte[] { 0xFF, 0xAA, 0x55, 0x00 }, Enumerable.Range(0, 4).Select(x => frame[x * 4]).ToArray());
            }
            else if (cgb != 0)
            {
                Assert.Equal(new byte[] { 0x11, 0x00, 0x00, 0xFF, 0x56, 0x00, 0x0D, 0x80 }, registers);
                Assert.True(rig.Flag("Bus.DoubleSpeed"), "a colour cartridge on the Color switches speed");
            }
            else
            {
                Assert.Equal(new byte[] { 0x11, 0x00, 0x00, 0x00, 0x08, 0x00, 0x7C, 0x80 }, registers);
                Assert.Equal(new byte[] { 0xFF, 0xFF, 0xFE, 0xFF, 0xC8, 0xFF, 0xD0, 0xFF, 0xFF, 0xFF, 0x7E, 0xFF, 0xFF }, wram[0x10..0x1D]);
                Assert.False(rig.Flag("Bus.DoubleSpeed"), "compatibility mode has no speed switch");
                Assert.Equal(1, rig.Field("Bus.WramBank"));

                // Palette 0's background colours, 7FFF 1BEF 6180 0000, through the 5-to-8-bit expansion, for shades 0 to 3.
                Assert.Equal(new[] { "FFFFFF", "7BFF31", "0063C6", "000000" },
                    Enumerable.Range(0, 4).Select(x => $"{frame[x * 4]:X2}{frame[x * 4 + 1]:X2}{frame[x * 4 + 2]:X2}").ToArray());
            }
        }

        // A Nintendo game's checksum reaches B, and the logo map's two checksums leave HL at $991A - see Mercury_Model.md §3.2.
        [Theory]
        [InlineData(0x70, 0x007C)]
        [InlineData(0x58, 0x991A)]
        [InlineData(0x43, 0x991A)]
        public void A_nintendo_game_on_the_color_hands_off_its_checksum(int checksum, int hl)
        {
            using MercuryRig rig = MercuryRig.Create(ProbeRom(0x00, checksum, licensee: 0x01), GbModel.GameBoyColor);
            for (int f = 0; f < 3; f++) rig.RunFrame();
            byte[] wram = rig.FieldBytes("Bus.Wram");
            Assert.Equal(checksum, wram[1]);
            Assert.Equal(hl, (wram[5] << 8) | wram[6]);
        }

        // A state made on one console, loaded into a machine running as the other, rebuilds it as the state's console and runs on as the machine that made it.
        [Theory]
        [InlineData((byte)0x00, GbModel.GameBoyColor, GbModel.Auto)]
        [InlineData((byte)0x00, GbModel.Auto, GbModel.GameBoyColor)]
        [InlineData((byte)0x80, GbModel.GameBoy, GbModel.Auto)]
        [InlineData((byte)0xC0, GbModel.Auto, GbModel.GameBoy)]
        public void A_state_resumes_on_the_console_it_was_made_on(byte cgb, GbModel madeOn, GbModel loadedOn)
        {
            byte[] rom = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x03, ramSizeCode: 0x02, cgbFlag: cgb, patches: (0, MercuryPrograms.Busy));
            using MercuryRig first = MercuryRig.Create(rom, madeOn);
            for (int f = 0; f < 120; f++) first.RunFrame();
            byte[] saved = first.State();

            using MercuryRig second = MercuryRig.Create(rom, loadedOn);
            Assert.NotEqual(first.CoreName, second.CoreName);
            second.SetButton(0, EmuSen.Galaxia.Input.PadButton.A, true);
            second.LoadState(new MemoryStream(saved));
            Assert.Equal(saved, second.State());
            Assert.Equal(first.CoreName, second.CoreName);

            for (int f = 0; f < 60; f++)
            {
                first.RunFrame();
                second.RunFrame();
            }
            Assert.True(first.State().AsSpan().SequenceEqual(second.State()), "the rebuilt machine ran on differently from the one that made the state");

            // The button held before the load is still held after the rebuild: the buttons row reads A down.
            second.Write(0xFF00, 0x10);
            Assert.Equal(0xDE, second.Read(0xFF00));
        }

        // The debugger's palette 0 is the four colours the probe draws, its object palettes the registers through colour RAM, on the console the game runs on - see Mercury_Model.md §5.
        [Theory]
        [InlineData((byte)0x00, GbModel.Auto)]
        [InlineData((byte)0x00, GbModel.GameBoyColor)]
        [InlineData((byte)0x80, GbModel.GameBoy)]
        public void The_debugger_shows_the_console_the_game_runs_on(byte cgb, GbModel model)
        {
            string path = Path.Combine(_dir, $"debug-{cgb:X2}-{model}.gb");
            File.WriteAllBytes(path, ProbeRom(cgb, 0x70, licensee: 0x01));
            using var rt = new MercuryRtCore { Options = TestCores.NoBattery, Model = model };
            rt.LoadRom(path);
            var target = rt.CreateDebugTarget();
            for (int f = 0; f < 10; f++) rt.RunFrame();
            target.RefreshProviders();

            static string Show(IReadOnlyList<EmuSen.Crystal.Telemetry.DebugPaletteInfo> palettes) =>
                string.Join("|", palettes.Select(p => string.Join(" ", p.Colors.Select(c => $"{c.r:X2}{c.g:X2}{c.b:X2}"))));
            string shown = Show(target.Palettes.Current);
            _output.WriteLine($"${cgb:X2} on {model}: {shown}");
            Assert.Equal(3, target.Palettes.Current.Count);

            byte[] frame = rt.GetFrameBufferRgba();
            Assert.Equal(Enumerable.Range(0, 4).Select(x => $"{frame[x * 4]:X2}{frame[x * 4 + 1]:X2}{frame[x * 4 + 2]:X2}"),
                target.Palettes.Current[0].Colors.Select(c => $"{c.r:X2}{c.g:X2}{c.b:X2}"));
            bool compatibility = cgb == 0 && model == GbModel.GameBoyColor;
            Assert.Equal(compatibility ? "GBC" : "GB", rt.CoreName);
            if (!compatibility) return;

            // The object registers through object palettes 0 and 1 of colour RAM, which the boot ROM filled.
            var (state, layout) = rt.Snapshot();
            byte[] obj = layout.Bytes(state, "Bus.Ppu.ObjPaletteRam").ToArray(), bg = layout.Bytes(state, "Bus.Ppu.BgPaletteRam").ToArray();
            static int Expand(int five) => (five << 3) | (five >> 2);
            static string Through(long register, byte[] ram, int palette) => string.Join(" ", Enumerable.Range(0, 4)
                .Select(c => ram[palette * 8 + (int)((register >> (c * 2)) & 3) * 2] | (ram[palette * 8 + (int)((register >> (c * 2)) & 3) * 2 + 1] << 8))
                .Select(v => $"{Expand(v & 0x1F):X2}{Expand((v >> 5) & 0x1F):X2}{Expand(v >> 10):X2}"));
            Assert.Equal($"{Through(layout.Get(state, "Bus.Ppu.Bgp"), bg, 0)}|{Through(layout.Get(state, "Bus.Ppu.Obp0"), obj, 0)}|{Through(layout.Get(state, "Bus.Ppu.Obp1"), obj, 1)}", shown);
        }

        // A rebuild for another console's state keeps the debugger's call stack following the game - see Mercury_Model.md §5.
        [Theory]
        [InlineData((byte)0x00, GbModel.GameBoyColor, GbModel.Auto)]
        [InlineData((byte)0x80, GbModel.GameBoy, GbModel.Auto)]
        public void A_rebuild_for_a_state_keeps_the_debuggers_call_stack(byte cgb, GbModel madeOn, GbModel loadedOn)
        {
            byte[] rom = SyntheticGbRom.Build(cgbFlag: cgb, patches: new[] { (0, new byte[] { 0xCD, 0x00, 0x02, 0x18, 0xFE }), (0xB0, new byte[] { 0x18, 0xFE }) });
            string path = Path.Combine(_dir, $"calls-{cgb:X2}.gb");
            File.WriteAllBytes(path, rom);
            using var maker = new MercuryRtCore { Options = TestCores.NoBattery, Model = madeOn };
            maker.LoadRom(path);
            using var saved = new MemoryStream();
            maker.SaveState(saved);

            using var rt = new MercuryRtCore { Options = TestCores.NoBattery, Model = loadedOn };
            rt.LoadRom(path);
            rt.Breakpoints.AddBreakpoint(0x7000);
            Assert.NotEqual(maker.CoreName, rt.CoreName);
            rt.LoadState(new MemoryStream(saved.ToArray()));
            Assert.Equal(maker.CoreName, rt.CoreName);

            rt.RunFrame();
            Assert.Equal(new[] { 0x0200 }, rt.CallStack.Frames.Select(f => (int)f.Target));
        }

        // The setting through ICoreSettings: read, refused, and before the first frame a change reloads the game at once.
        [Fact]
        public void The_model_setting_is_honoured_before_the_first_frame_and_otherwise_at_the_next_load()
        {
            string path = Path.Combine(_dir, "model.gb");
            File.WriteAllBytes(path, ProbeRom(0x00));
            using var core = new MercuryRtCore() { Options = TestCores.NoBattery };
            var settings = (ICoreSettings)core;
            Assert.Equal(CoreCatalog.SettingsFor("GB"), settings.Settings);
            Assert.Equal(MercuryRtCore.ModelAuto, settings.Get(MercuryRtCore.ModelKey));

            core.LoadRom(path);
            Assert.Equal("GB", core.CoreName);
            settings.Set(MercuryRtCore.ModelKey, MercuryRtCore.ModelGameBoyColor);
            Assert.Equal("GBC", core.CoreName);
            core.RunFrame();
            settings.Set(MercuryRtCore.ModelKey, MercuryRtCore.ModelGameBoy);
            Assert.Equal("GBC", core.CoreName);
            core.LoadRom(path);
            Assert.Equal("GB", core.CoreName);
            Assert.Throws<ArgumentException>(() => settings.Set(MercuryRtCore.ModelKey, "Super Game Boy"));
            Assert.Throws<ArgumentException>(() => settings.Get("Palette"));
            Assert.Equal(MercuryRtCore.ModelGameBoy, settings.Get(MercuryRtCore.ModelKey));
        }

        // mooneye's misc/boot_regs-cgb, verified on a CGB: a DMG header on the Color must hand over A=$11 F=$80 BC=$0000 DE=$0008 HL=$007C.
        [Theory]
        [InlineData(GbModel.Auto, TestRomVerdict.Failed)]
        [InlineData(GbModel.GameBoyColor, TestRomVerdict.Passed)]
        public void Mooneyes_cgb_boot_register_rom_passes_on_the_color_and_fails_on_the_game_boy(GbModel model, TestRomVerdict expected)
        {
            string? rom = HardwareTestRomLibrary.Find().FirstOrDefault(p => Path.GetFileName(p) == "boot_regs-cgb.gb");
            if (rom is null)
            {
                _output.WriteLine("the corpus or boot_regs-cgb.gb is absent, not run");
                return;
            }
            using MercuryRig rig = MercuryRig.Open(rom, model);
            rig.SkipRendering = true;
            var verdict = TestRomVerdict.NoVerdict;
            for (int f = 0; f < 600 && verdict == TestRomVerdict.NoVerdict; f++)
            {
                rig.RunFrame();
                verdict = HardwareTestRomLibrary.ReadVerdict(rig.SerialLog);
            }
            Assert.Equal(expected, verdict);
        }
    }
}
