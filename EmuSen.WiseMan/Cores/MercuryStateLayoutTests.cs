using EmuSen.Cores.Native;
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
    // The committed layout listings are the state format's specification: each is what MercuryRT's state_layout prints - see Mercury_Native.md §14.5.
    public class MercuryStateLayoutTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryStateLayoutTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<string> Listings
        {
            get
            {
                var data = new TheoryData<string>();
                foreach (string path in Directory.GetFiles(MercuryLayouts.Directory(), "*.layout").Order(StringComparer.Ordinal)) data.Add(Path.GetFileName(path));
                return data;
            }
        }

        // A cartridge of the listing's board, console and RAM size.
        private static byte[] RomFor(string listing)
        {
            string[] p = Path.GetFileNameWithoutExtension(listing).Split('-');
            int ram = int.Parse(p[2]["ram".Length..]);
            byte kind = (p[0], ram > 0) switch
            {
                ("none", false) => 0x00, ("none", true) => 0x08,
                ("mbc1", false) => 0x01, ("mbc1", true) => 0x03,
                ("mbc2", _) => 0x06,
                ("mbc3", false) => 0x11, ("mbc3", true) => 0x13,
                ("mbc5", false) => 0x19, ("mbc5", true) => 0x1B,
                _ => throw new ArgumentException(listing),
            };
            byte code = (byte)(p[0] == "mbc2" ? 0 : ram switch { 0 => 0, 2048 => 1, 8192 => 2, 32768 => 3, 131072 => 4, 65536 => 5, _ => throw new ArgumentException(listing) });
            return SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: code, cgbFlag: p[1] == "gbc" ? (byte)0xC0 : (byte)0x00);
        }

        [Theory]
        [MemberData(nameof(Listings))]
        public void A_committed_listing_is_what_mercuryrt_writes(string listing)
        {
            string committed = MercuryLayouts.ReadListing(Path.Combine(MercuryLayouts.Directory(), listing));
            byte[] rom = RomFor(listing);
            CoreOptions.BatteryRamDisabled = true;
            using MercuryRig rust = MercuryRig.Create(rom);
            rust.RunFrame();
            Assert.Equal(committed, rust.Core.Machine.Core.Layout());
            Assert.Equal(MercuryLayouts.FileName(MercuryLayouts.Board(rom[0x147]), listing.Contains("-gbc-"), StateLayout.FromText(listing, committed)["Cart.Ram"].Length), listing);
            Assert.Same(MercuryLayouts.Load(Path.Combine(MercuryLayouts.Directory(), listing)), MercuryLayouts.For(rom, rust.State()));
        }

        public static TheoryData<byte, byte, byte, GbModel> BoardsOnConsoles
        {
            get
            {
                var data = new TheoryData<byte, byte, byte, GbModel>();
                foreach (var (kind, ram, cgb) in MercuryPrograms.Boards)
                    foreach (GbModel model in Enum.GetValues<GbModel>()) data.Add(kind, ram, cgb, model);
                return data;
            }
        }

        // Every board of the programs on every console has its listing, so the reader never meets a shape the specification lacks.
        [Theory]
        [MemberData(nameof(BoardsOnConsoles))]
        public void Every_board_on_every_console_has_a_committed_listing(byte kind, byte ram, byte cgb, GbModel model)
        {
            CoreOptions.BatteryRamDisabled = true;
            using MercuryRig rust = MercuryRig.Create(SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: ram, cgbFlag: cgb), model);
            rust.RunFrame();
            Assert.Equal(MercuryLayouts.StateVersion, rust.Field("Version"));
        }

        [Fact]
        public void The_listings_describe_the_state_version_mercuryrt_writes()
        {
            Assert.Equal(MercuryRtCore.StateFormatVersion, MercuryLayouts.StateVersion);
            Assert.NotEmpty(Listings);
        }
    }
}
