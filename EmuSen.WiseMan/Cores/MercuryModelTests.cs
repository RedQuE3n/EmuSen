using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.Mercury.Video;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The Model setting on both engines: each cartridge on each console, the Color's palettes for Game Boy games, and states across consoles - see Mercury_Model.md.
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
            CoreOptions.BatteryRamDisabled = true;
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
            rom[Cartridge.HeaderChecksumAddress] = Cartridge.ComputeHeaderChecksum(rom);
            return rom;
        }

        private static string Rgb888(ushort[] palette) =>
            string.Join(' ', palette.Select(c => $"{Scale(c & 0x1F):X2}{Scale((c >> 5) & 0x1F):X2}{Scale((c >> 10) & 0x1F):X2}"));

        private static int Scale(int five) => (int)Math.Round(five * 255 / 31.0, MidpointRounding.AwayFromZero);

        private static string Palettes(byte[] rom)
        {
            var (bg, obj0, obj1) = CompatibilityPalettes.ForNumber(CompatibilityPalettes.PaletteNumber(rom));
            return $"{Rgb888(bg)}|{Rgb888(obj0)}|{Rgb888(obj1)}";
        }

        [Theory]
        [MemberData(nameof(TcrfRows))]
        public void The_colour_palette_for_a_listed_game_is_the_one_tcrf_documents(int checksum, char? fourth, string expected)
        {
            byte[] rom = Header(checksum, fourth);
            Assert.Equal((byte)checksum, CompatibilityPalettes.TitleChecksum(rom));
            Assert.Equal(expected, Palettes(rom));
        }

        // TCRF lists Radar Mission ($8C) as entry $01 with flags $00; the boot ROM's own tables give $8C palette number 8, entry $00 with flags $01 - see Mercury_Model.md §3.3.
        [Fact]
        public void Radar_mission_follows_the_boot_roms_table_where_tcrf_disagrees()
        {
            byte[] rom = Header(0x8C, null);
            Assert.Equal(8, CompatibilityPalettes.PaletteNumber(rom));
            Assert.Equal("FFFFFF ADAD84 42737B 000000|FFFFFF FF7300 944200 000000|FFFFFF ADAD84 42737B 000000", Palettes(rom));
            Assert.NotEqual("FFFF9C 94B5FF 639473 003A3A|FFFF9C 94B5FF 639473 003A3A|FFFF9C 94B5FF 639473 003A3A", Palettes(rom));
        }

        // Pan Docs' default: a game not Nintendo's, not listed, or listed as ambiguous with no matching letter gets palette 0.
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
            Assert.Equal(number, CompatibilityPalettes.PaletteNumber(Header(checksum, fourth, licensee, newLicensee: newLicensee)));
        }

        [Fact]
        public void The_default_palette_is_the_one_pan_docs_gives()
        {
            var (bg, obj0, obj1) = CompatibilityPalettes.ForNumber(0);
            Assert.Equal(new ushort[] { 0x7FFF, 0x1BEF, 0x6180, 0x0000 }, bg);
            Assert.Equal(new ushort[] { 0x7FFF, 0x421F, 0x1CF2, 0x0000 }, obj0);
            Assert.Equal(new ushort[] { 0x7FFF, 0x421F, 0x1CF2, 0x0000 }, obj1);
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
            rom[Cartridge.HeaderChecksumAddress] = Cartridge.ComputeHeaderChecksum(rom);
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

        // What each console hands the cartridge and shows it, the two engines in lock-step - see Mercury_Model.md §2.
        [Theory]
        [MemberData(nameof(Cases))]
        public void Each_cartridge_on_each_console_runs_as_that_console_identically(byte cgb, GbModel model)
        {
            var pair = new MercuryRtPair(ProbeRom(cgb), skipRendering: false, model: model);
            pair.Run(10, null);
            MemoryBus bus = pair.Csharp.Bus!;
            byte[] wram = bus.Wram;
            bool colourConsole = model == GbModel.GameBoyColor || (model == GbModel.Auto && cgb != 0);
            string kind = !colourConsole ? "Game Boy" : cgb == 0 ? "Color, compatibility" : "Color";
            _output.WriteLine($"${cgb:X2} on {model}: {kind}; A {wram[0]:X2} F {wram[7]:X2} BC {wram[1]:X2}{wram[2]:X2} DE {wram[3]:X2}{wram[4]:X2} HL {wram[5]:X2}{wram[6]:X2}; FF4C-FF70 {string.Join(" ", wram[0x10..0x1D].Select(b => b.ToString("X2")))}; {pair.Summary}");

            Assert.Equal(colourConsole, bus.CgbHardware);
            Assert.Equal(colourConsole ? "GBC" : "GB", pair.Csharp.CoreName);
            Assert.Equal(colourConsole, pair.Rust.CgbHardware);
            Assert.Equal(colourConsole ? 0x4000 : 0x2000, bus.Vram.Length);

            byte[] registers = wram[0..8];
            byte[] frame = pair.Csharp.GetFrameBufferRgba();
            if (!colourConsole)
            {
                Assert.Equal(new byte[] { 0x01, 0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D, 0xB0 }, registers);
                Assert.False(bus.DoubleSpeed);
                Assert.Equal(new byte[] { 0xFF, 0xAA, 0x55, 0x00 }, Enumerable.Range(0, 4).Select(x => frame[x * 4]).ToArray());
            }
            else if (cgb != 0)
            {
                Assert.Equal(new byte[] { 0x11, 0x00, 0x00, 0xFF, 0x56, 0x00, 0x0D, 0x80 }, registers);
                Assert.True(bus.DoubleSpeed, "a colour cartridge on the Color switches speed");
            }
            else
            {
                Assert.Equal(new byte[] { 0x11, 0x00, 0x00, 0x00, 0x08, 0x00, 0x7C, 0x80 }, registers);
                Assert.Equal(new byte[] { 0xFF, 0xFF, 0xFE, 0xFF, 0xC8, 0xFF, 0xD0, 0xFF, 0xFF, 0xFF, 0x7E, 0xFF, 0xFF }, wram[0x10..0x1D]);
                Assert.False(bus.DoubleSpeed, "compatibility mode has no speed switch");
                Assert.Equal(1, bus.WramBank);

                // Palette 0's background colours, 7FFF 1BEF 6180 0000, through Mercury's 5-to-8-bit expansion, for shades 0 to 3.
                Assert.Equal(new[] { "FFFFFF", "7BFF31", "0063C6", "000000" },
                    Enumerable.Range(0, 4).Select(x => $"{frame[x * 4]:X2}{frame[x * 4 + 1]:X2}{frame[x * 4 + 2]:X2}").ToArray());
            }
        }

        // A Nintendo game's checksum reaches B, the logo map's two checksums leave HL at $991A, and its palette is in colour RAM - see Mercury_Model.md §3.2.
        [Theory]
        [InlineData(0x70, 0x007C)]
        [InlineData(0x58, 0x991A)]
        [InlineData(0x43, 0x991A)]
        public void A_nintendo_game_on_the_color_hands_off_its_checksum_identically(int checksum, int hl)
        {
            byte[] rom = ProbeRom(0x00, checksum, licensee: 0x01);
            var pair = new MercuryRtPair(rom, skipRendering: false, model: GbModel.GameBoyColor);
            pair.Run(3, null);
            byte[] wram = pair.Csharp.Bus!.Wram;
            Assert.Equal(checksum, wram[1]);
            Assert.Equal(hl, (wram[5] << 8) | wram[6]);

            var (bg, obj0, obj1) = CompatibilityPalettes.ForNumber(CompatibilityPalettes.PaletteNumber(rom));
            EmuSen.Cores.Nintendo.Mercury.Video.Ppu ppu = pair.Csharp.Bus!.Ppu;
            for (int i = 0; i < 4; i++)
            {
                Assert.Equal(bg[i], ppu.BgPaletteRam[i * 2] | (ppu.BgPaletteRam[i * 2 + 1] << 8));
                Assert.Equal(obj0[i], ppu.ObjPaletteRam[i * 2] | (ppu.ObjPaletteRam[i * 2 + 1] << 8));
                Assert.Equal(obj1[i], ppu.ObjPaletteRam[8 + i * 2] | (ppu.ObjPaletteRam[8 + i * 2 + 1] << 8));
            }
        }

        // Every one of the 94 palettes on both engines, by a title that selects it; the picture and colour RAM are compared each frame.
        [Fact]
        public void Every_palette_number_renders_identically_on_both_engines()
        {
            var found = new Dictionary<int, byte[]>();
            foreach (char fourth in "BEFAARBEKEK R-URAR INAILICE RZ".Distinct())
                for (int checksum = 0; checksum < 256; checksum++)
                {
                    byte[] rom = ProbeRom(0x00, checksum, licensee: 0x01);
                    rom[0x137] = (byte)fourth;
                    rom[0x134] = (byte)(checksum - fourth);
                    rom[Cartridge.HeaderChecksumAddress] = Cartridge.ComputeHeaderChecksum(rom);
                    found.TryAdd(CompatibilityPalettes.PaletteNumber(rom), rom);
                }
            Assert.Equal(CompatibilityPalettes.PaletteCount, found.Count);
            foreach (byte[] rom in found.Values)
            {
                var pair = new MercuryRtPair(rom, skipRendering: false, model: GbModel.GameBoyColor);
                pair.Run(2, null);
            }
        }

        // A state made on one console, loaded into a machine running as the other, rebuilds it as the state's console and runs on identically.
        [Theory]
        [InlineData((byte)0x00, GbModel.GameBoyColor, GbModel.Auto)]
        [InlineData((byte)0x00, GbModel.Auto, GbModel.GameBoyColor)]
        [InlineData((byte)0x80, GbModel.GameBoy, GbModel.Auto)]
        [InlineData((byte)0xC0, GbModel.Auto, GbModel.GameBoy)]
        public void A_state_resumes_on_the_console_it_was_made_on(byte cgb, GbModel madeOn, GbModel loadedOn)
        {
            byte[] rom = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x03, ramSizeCode: 0x02, cgbFlag: cgb, patches: (0, MercuryRtMachineTests.Busy));
            var first = new MercuryRtPair(rom, skipRendering: false, model: madeOn);
            first.Run(120, null);
            using var saved = new MemoryStream();
            first.Csharp.SaveState(saved);

            var second = new MercuryRtPair(rom, skipRendering: false, model: loadedOn);
            Assert.NotEqual(first.Csharp.Bus!.CgbHardware, second.Csharp.Bus!.CgbHardware);
            second.Csharp.SetButton(0, EmuSen.Galaxia.Input.PadButton.A, true);
            second.Rust.SetButtons(1u << 4);
            second.Csharp.LoadState(new MemoryStream(saved.ToArray()));
            second.Rust.Load(saved.ToArray());
            second.CompareState("after the load");
            Assert.Equal(first.Csharp.CoreName, second.Csharp.CoreName);
            Assert.Equal(first.Csharp.Bus!.CgbHardware, second.Rust.CgbHardware);

            first.Run(60, null);
            second.Run(60, null);
            using var a = new MemoryStream();
            using var b = new MemoryStream();
            first.Csharp.SaveState(a);
            second.Csharp.SaveState(b);
            Assert.True(a.ToArray().AsSpan().SequenceEqual(b.ToArray()), "the rebuilt machine ran on differently from the one that made the state");

            // The button held before the load is still held after the rebuild, on both engines: the buttons row reads A down.
            second.Csharp.Bus!.Write(0xFF00, 0x10);
            second.Rust.WriteSpace(6, 0xFF00, new byte[] { 0x10 });
            var p1 = new byte[1];
            second.Rust.ReadSpace(6, 0xFF00, p1);
            Assert.Equal(0xDE, second.Csharp.Bus!.Read(0xFF00));
            Assert.Equal(0xDE, p1[0]);
        }

        // The setting through ICoreSettings on both engines: read, refused, and before the first frame a change reloads the game at once.
        [Theory]
        [MemberData(nameof(Engines))]
        public void The_model_setting_is_honoured_before_the_first_frame_and_otherwise_at_the_next_load(string engine)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = Path.Combine(_dir, $"model-{engine}.gb");
            File.WriteAllBytes(path, ProbeRom(0x00));
            ICore core = engine == "C#" ? new MercuryCore() : new MercuryRtCore();
            var settings = (ICoreSettings)core;
            Assert.Equal(CoreCatalog.SettingsFor("GB"), settings.Settings);
            Assert.Equal(MercuryCore.ModelAuto, settings.Get(MercuryCore.ModelKey));

            core.LoadRom(path);
            Assert.Equal("GB", core.CoreName);
            settings.Set(MercuryCore.ModelKey, MercuryCore.ModelGameBoyColor);
            Assert.Equal("GBC", core.CoreName);
            core.RunFrame();
            settings.Set(MercuryCore.ModelKey, MercuryCore.ModelGameBoy);
            Assert.Equal("GBC", core.CoreName);
            core.LoadRom(path);
            Assert.Equal("GB", core.CoreName);
            Assert.Throws<ArgumentException>(() => settings.Set(MercuryCore.ModelKey, "Super Game Boy"));
            Assert.Throws<ArgumentException>(() => settings.Get("Palette"));
            Assert.Equal(MercuryCore.ModelGameBoy, settings.Get(MercuryCore.ModelKey));
            (core as IDisposable)?.Dispose();
        }

        public static TheoryData<string> Engines => new() { "C#", "Rust" };

        // The hardware corpus on each forced console, both engines in lock-step as MercuryRtCorpusTests runs it under Auto; absent, not run.
        [Theory]
        [InlineData(GbModel.GameBoy)]
        [InlineData(GbModel.GameBoyColor)]
        public void The_hardware_corpus_runs_identically_on_each_console(GbModel model)
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryRtMachineTests.CorpusVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MercuryRtMachineTests.CorpusVariable} unset, not run");
                return;
            }
            var files = Directory.GetFiles(folder, "*.gb*", SearchOption.AllDirectories).Where(f => f.EndsWith(".gb") || f.EndsWith(".gbc")).Order(StringComparer.Ordinal).ToList();
            int frames = 0, passed = 0;
            foreach (string path in files)
            {
                var pair = new MercuryRtPair(File.ReadAllBytes(path), skipRendering: true, stateEvery: 30, soundAndPicture: false, model: model);
                var verdict = TestRomVerdict.NoVerdict;
                for (int f = 0; f < 3600 && verdict == TestRomVerdict.NoVerdict; f++, frames++)
                {
                    if (!pair.Frame(null, f)) break;
                    verdict = HardwareTestRomLibrary.ReadVerdict(pair.Csharp.Bus!.SerialLog);
                    if (pair.Csharp.Cart!.Ram is { Length: >= 4 } ram && ram[1] == 0xDE && ram[2] == 0xB0 && ram[3] == 0x61 && ram[0] != 0x80) break;
                }
                pair.CompareState("the end");
                if (verdict == TestRomVerdict.Passed) passed++;
            }
            _output.WriteLine($"{model}: {files.Count} ROMs, {frames} frames identical in serial every frame and in state every 30 frames and at the end; {passed} passed by serial");
        }

        // The four games on the console they were not made for, both engines in lock-step with the bench's input; each last picture saved where EMUSEN_UI_DUMP says.
        [Theory]
        [InlineData(GbModel.GameBoyColor)]
        [InlineData(GbModel.GameBoy)]
        public void Real_games_run_identically_on_the_other_console(GbModel model)
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MercuryRtStateTests.RomsVariable} unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                var pair = new MercuryRtPair(rom, skipRendering: false, model: model, stateEvery: 10);
                pair.Run(1500, f => (f % 90) switch { < 5 => 1u << 7, >= 45 and < 50 => 1u << 4, _ => 0u });
                string console = pair.Csharp.CoreName;
                _output.WriteLine($"{Path.GetFileName(path)} on {model} runs as {console}, palette {CompatibilityPalettes.PaletteNumber(rom)}: {pair.Summary}");
                if (Environment.GetEnvironmentVariable("EMUSEN_UI_DUMP") is { Length: > 0 } dump)
                    EmuSen.Hotaru.Imaging.FrameImageWriter.SavePng(pair.Csharp.GetFrameBufferRgba(), 160, 144, Path.Combine(dump, $"{Path.GetFileNameWithoutExtension(path)}-{model}.png"));
            }
        }

        // mooneye's misc/boot_regs-cgb, verified on a CGB: a DMG header on the Color must hand over A=$11 F=$80 BC=$0000 DE=$0008 HL=$007C.
        [Theory]
        [InlineData(GbModel.Auto, TestRomVerdict.Failed)]
        [InlineData(GbModel.GameBoyColor, TestRomVerdict.Passed)]
        public void Mooneyes_cgb_boot_register_rom_passes_on_the_color_and_fails_on_the_game_boy(GbModel model, TestRomVerdict expected)
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryRtMachineTests.CorpusVariable);
            string? rom = folder is null ? null : Directory.GetFiles(folder, "boot_regs-cgb.gb", SearchOption.AllDirectories).FirstOrDefault();
            if (rom is null)
            {
                _output.WriteLine("the corpus or boot_regs-cgb.gb is absent, not run");
                return;
            }
            var pair = new MercuryRtPair(File.ReadAllBytes(rom), skipRendering: true, model: model);
            var verdict = TestRomVerdict.NoVerdict;
            for (int f = 0; f < 600 && verdict == TestRomVerdict.NoVerdict; f++)
            {
                pair.Frame(null, f);
                verdict = HardwareTestRomLibrary.ReadVerdict(pair.Csharp.Bus!.SerialLog);
            }
            Assert.Equal(expected, verdict);
        }
    }
}
