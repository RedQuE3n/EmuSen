using System.IO;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Phase 0's whole surface: three containers, one header, and no emulation - see Mars_Rom.md.
    public class MarsRomImageTests
    {
        [Fact]
        public void A_big_endian_image_parses_its_header()
        {
            var image = RomImage.FromImage(SyntheticN64Rom.Build(entryPoint: 0x80001234, title: "WISEMAN"));

            Assert.Equal(RomByteOrder.BigEndian, image.SourceByteOrder);
            Assert.Equal(0x80001234u, image.EntryPoint);
            Assert.Equal("WISEMAN", image.Title);
            Assert.Equal('N', image.CategoryCode);
            Assert.Equal("WM", image.UniqueCode);
            Assert.Equal('E', image.DestinationCode);
        }

        [Theory]
        [InlineData(RomByteOrder.ByteSwapped)]
        [InlineData(RomByteOrder.LittleEndian)]
        public void Every_container_normalises_to_the_same_bytes(RomByteOrder order)
        {
            byte[] bigEndian = SyntheticN64Rom.Build(entryPoint: 0x80001234, title: "WISEMAN");
            byte[] file = order == RomByteOrder.ByteSwapped
                ? SyntheticN64Rom.ToByteSwapped(bigEndian)
                : SyntheticN64Rom.ToLittleEndian(bigEndian);

            var image = RomImage.FromImage(file);

            Assert.Equal(order, image.SourceByteOrder);
            Assert.Equal(bigEndian, image.Rom);
            Assert.Equal(0x80001234u, image.EntryPoint);
        }

        // The survey's erratum: sources disagree on what each extension means, so the magic word decides.
        [Fact]
        public void A_z64_file_holding_byte_swapped_data_is_read_as_byte_swapped()
        {
            byte[] bigEndian = SyntheticN64Rom.Build();
            string path = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.ToByteSwapped(bigEndian), ".z64");

            try
            {
                var image = RomImage.Load(path);

                Assert.Equal(RomByteOrder.ByteSwapped, image.SourceByteOrder);
                Assert.Equal(bigEndian, image.Rom);
            }
            finally
            {
                File.Delete(path);
            }
        }

        [Fact]
        public void An_image_with_no_recognised_magic_is_refused()
        {
            var bytes = new byte[RomImage.MinimumLength];
            bytes[0] = 0x12;

            var error = Assert.Throws<InvalidDataException>(() => RomImage.FromImage(bytes));
            Assert.Contains("0x12000000", error.Message);
        }

        [Fact]
        public void An_image_shorter_than_a_magic_word_is_refused()
        {
            Assert.Throws<InvalidDataException>(() => RomImage.FromImage(new byte[] { 0x80, 0x37 }));
        }

        [Fact]
        public void An_image_shorter_than_the_boot_code_is_refused()
        {
            byte[] truncated = SyntheticN64Rom.Build()[..0x800];

            var error = Assert.Throws<InvalidDataException>(() => RomImage.FromImage(truncated));
            Assert.Contains("shorter than", error.Message);
        }

        // A half-swapped tail would be silent corruption, so the odd byte is an error rather than a best effort.
        [Fact]
        public void A_byte_swapped_image_of_odd_length_is_refused_rather_than_half_converted()
        {
            byte[] swapped = SyntheticN64Rom.ToByteSwapped(SyntheticN64Rom.Build());
            byte[] odd = swapped[..(swapped.Length - 1)];

            var error = Assert.Throws<InvalidDataException>(() => RomImage.FromImage(odd));
            Assert.Contains("not a multiple of 2", error.Message);
        }

        [Theory]
        [InlineData(N64SaveType.None)]
        [InlineData(N64SaveType.Eeprom4k)]
        [InlineData(N64SaveType.Eeprom16k)]
        [InlineData(N64SaveType.Sram256k)]
        [InlineData(N64SaveType.SramBanked768k)]
        [InlineData(N64SaveType.FlashRam)]
        [InlineData(N64SaveType.Sram1M)]
        public void A_homebrew_header_states_its_own_save_type(N64SaveType saveType)
        {
            var image = RomImage.FromImage(SyntheticN64Rom.BuildHomebrew(saveType));

            Assert.True(image.HasEd64Header);
            Assert.Equal(saveType, image.SaveType);
        }

        [Fact]
        public void A_homebrew_header_carries_the_clock_and_region_flags_beside_the_save_type()
        {
            var image = RomImage.FromImage(
                SyntheticN64Rom.BuildHomebrew(N64SaveType.FlashRam, realTimeClock: true, regionFree: true));

            Assert.Equal(N64SaveType.FlashRam, image.SaveType);
            Assert.True(image.HasRealTimeClock);
            Assert.True(image.IsRegionFree);
        }

        // The finding two emulator databases independently confirm: a commercial header cannot answer this.
        [Fact]
        public void A_commercial_header_states_no_save_type_at_all()
        {
            var image = RomImage.FromImage(SyntheticN64Rom.Build(uniqueCode: "SM"));

            Assert.False(image.HasEd64Header);
            Assert.Equal(N64SaveType.Unknown, image.SaveType);
        }

        [Theory]
        [InlineData('E', false)]
        [InlineData('J', false)]
        [InlineData('P', true)]
        [InlineData('D', true)]
        [InlineData('U', true)]
        public void The_destination_code_carries_the_video_standard_by_convention(char destination, bool pal)
        {
            var image = RomImage.FromImage(SyntheticN64Rom.Build(destinationCode: destination));

            Assert.Equal(pal, image.IsPal);
        }

        [Fact]
        public void A_space_padded_title_loses_its_padding()
        {
            var image = RomImage.FromImage(SyntheticN64Rom.Build(title: "SUPER GAME"));

            Assert.Equal("SUPER GAME", image.Title);
        }

        // The corpus ROM if it is present, which is the first real N64 file this project has read.
        [Fact]
        public void The_hardware_corpus_rom_parses_when_it_is_installed()
        {
            string? path = N64TestRomLibrary.FindSystemTest();
            if (path is null) return;

            var image = RomImage.Load(path);

            Assert.Equal(RomByteOrder.BigEndian, image.SourceByteOrder);
            Assert.Equal(0x80006CF0u, image.EntryPoint);
            Assert.Equal("n64-systemtest", image.Title);
        }
    }
}
