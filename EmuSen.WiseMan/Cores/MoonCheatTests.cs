using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Cheats;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Cheats reaching a running NES core - the decode itself is pinned separately
    // in NesGameGenieCodecTests. See Moon_Cheats.md §3.
    public class MoonCheatTests : IDisposable
    {
        // A recognisable byte at PRG offset 0, which a 16K NROM maps to both $8000 and $C000.
        private const byte Original = 0xCE;
        private const int Address = 0x8000;

        private readonly string _romPath;
        private readonly MoonCore _core = new();

        public MoonCheatTests()
        {
            _romPath = SyntheticNesRom.WriteTemp(
                SyntheticNesRom.Build(patches: (0, new byte[] { Original })));
            _core.LoadRom(_romPath);
        }

        public void Dispose()
        {
            try { File.Delete(_romPath); } catch { }
        }

        private byte Read() => _core.Bus!.Read(Address);

        [Fact]
        public void An_unpatched_read_returns_the_cartridge_byte()
        {
            Assert.Equal(Original, Read());
        }

        [Fact]
        public void A_rom_patch_changes_what_the_cpu_reads()
        {
            _core.Cheats.AddRomPatch(Address, 0xAD, null, "patch");

            Assert.Equal(0xAD, Read());
        }

        // Disabling has to restore the real byte, not merely stop re-applying.
        [Fact]
        public void Disabling_a_rom_patch_restores_the_byte()
        {
            int id = _core.Cheats.AddRomPatch(Address, 0xAD, null, "patch");
            _core.Cheats.SetEnabled(id, false);

            Assert.Equal(Original, Read());
        }

        [Fact]
        public void The_master_switch_suspends_a_rom_patch()
        {
            _core.Cheats.AddRomPatch(Address, 0xAD, null, "patch");
            _core.Cheats.MasterEnabled = false;

            Assert.Equal(Original, Read());

            _core.Cheats.MasterEnabled = true;
            Assert.Equal(0xAD, Read());
        }

        // The compare byte is what makes a code target one bank rather than every
        // bank that maps to the same CPU address.
        [Fact]
        public void A_matching_compare_patches_and_a_mismatched_one_does_not()
        {
            _core.Cheats.AddRomPatch(Address, 0xAD, Original, "right");
            Assert.Equal(0xAD, Read());

            _core.Cheats.Clear();
            _core.Cheats.AddRomPatch(Address, 0xAD, (byte)(Original ^ 0xFF), "wrong");
            Assert.Equal(Original, Read());
        }

        // RAM pokes are re-applied at every frame boundary, not once when added.
        [Fact]
        public void A_ram_poke_lands_on_the_next_frame()
        {
            _core.Cheats.AddRamPoke("RAM", 0x0075, 0x09, "lives");

            _core.RunFrame();

            Assert.Equal(0x09, _core.Bus!.Ram[0x0075]);
        }

        // Nothing below the cartridge window reaches a Game Genie, so a patch there must be inert.
        [Fact]
        public void A_rom_patch_never_intercepts_a_ram_read()
        {
            _core.Bus!.Ram[0x0075] = 0x11;
            _core.Cheats.AddRomPatch(0x0075, 0xAD, null, "out of reach");

            Assert.Equal(0x11, _core.Bus.Read(0x0075));
        }

        [Fact]
        public void The_factory_gives_the_nes_both_codecs()
        {
            CoreBundle bundle = CoreFactory.Load(_romPath);

            Assert.IsType<NesRawCheatCodec>(bundle.CheatAutoDetectCodec);
            Assert.IsType<NesGameGenieCheatCodec>(bundle.CheatExplicitCodec);
        }

        // The whole chain: a published code, decoded, applied, seen by the CPU.
        [Fact]
        public void A_game_genie_code_decodes_and_patches_end_to_end()
        {
            ICheatCodeCodec codec = new NesGameGenieCheatCodec();
            // Decodes to $91D9, so this fixture patches the byte the code names rather than the one it has.
            (int address, byte value) = codec.Decode("SXIOPO");
            Assert.Equal(0x91D9, address);
            Assert.Equal(0xAD, value);

            _core.Cheats.AddRomPatch(address, value, codec.DecodeCompare("SXIOPO"), "SXIOPO");

            Assert.Equal(0xAD, _core.Bus!.Read((ushort)address));
        }
    }
}
