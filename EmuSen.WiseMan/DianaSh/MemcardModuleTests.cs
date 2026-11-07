using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `memcard` is the platform library's emusen.memcard module in every front end's shell, its media the ones the build's engines declare - see EmuSen_Backend.md §40.11.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class MemcardModuleTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMemcardModule_" + Guid.NewGuid().ToString("N"));

        public MemcardModuleTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static string Said(DianaShInterpreter shell, string line) => shell.Submit(line).Output.TrimEnd('\n');

        [Fact]
        public void A_card_is_made_and_listed_through_a_front_end_s_shell()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateWithoutSession();
            Assert.Equal("No cards: nothing under Saves is a file of a medium a system pack names.", Said(shell, "memcard ls"));
            Assert.Equal("Blue.mcr made, a formatted memory card", Said(shell, "memcard new /Saves/PlayStation/Cards/Blue.mcr"));
            Assert.Equal("PlayStation\n  Saves/PlayStation/Cards/Blue.mcr  Memory Card  0 save(s), 15 of 15 blocks free", Said(shell, "memcard ls"));
        }

        // The words the C# command said from the packs, which a build with every engine says from their declarations.
        [Fact]
        public void The_media_known_are_the_build_s_engines_in_the_table_s_order() =>
            Assert.Equal("memcard: notes.txt is no card: the media known are .crm (Backup RAM Cartridge), .brm (Internal Backup RAM), .mcr (Memory Card), .mpk (Controller Pak), .ps2 (Memory Card (PS2))",
                Said(DianaShInterpreter.CreateWithoutSession(), "memcard ls /notes.txt"));
    }
}
