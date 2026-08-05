using System;
using System.IO;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Galaxia
{
    // The one spelling of what a ROM's save files are called - see EmuSen_Galaxia.md §5.
    public class SaveLibraryTests : IDisposable
    {
        public void Dispose() => DataStore.OverrideDirectory = null;

        [Fact]
        public void Sram_lands_in_Saves_named_after_the_rom()
        {
            Assert.Equal(
                Path.Combine(DataStore.Saves, "Super Metroid.srm"),
                SaveLibrary.SramPathFor(Path.Combine("anywhere", "Super Metroid.smc")));
        }

        // The ROM's own folder never appears in the answer, whatever it was.
        [Fact]
        public void Sram_ignores_where_the_rom_itself_lives()
        {
            string fromGames = SaveLibrary.SramPathFor(Path.Combine("Usr", "Home", "Games", "SNES", "ALTTP.smc"));
            string fromElsewhere = SaveLibrary.SramPathFor(Path.Combine("tmp", "ALTTP.smc"));

            Assert.Equal(fromGames, fromElsewhere);
            Assert.Equal(DataStore.Saves, Path.GetDirectoryName(fromGames));
        }

        // Slot 1 keeps the plain <rom>.state name every frontend already wrote.
        [Fact]
        public void Slot_one_is_the_unsuffixed_state_name()
        {
            Assert.Equal(
                Path.Combine(DataStore.SaveStates, "ALTTP.state"),
                SaveLibrary.StatePathFor(Path.Combine("Games", "ALTTP.smc")));
        }

        [Theory]
        [InlineData(2, "ALTTP.slot2.state")]
        [InlineData(9, "ALTTP.slot9.state")]
        public void Later_slots_are_suffixed(int slot, string expected)
        {
            Assert.Equal(
                Path.Combine(DataStore.SaveStates, expected),
                SaveLibrary.StatePathFor(Path.Combine("Games", "ALTTP.smc"), slot));
        }

        [Fact]
        public void A_directory_override_replaces_the_default_but_not_the_name()
        {
            string custom = Path.Combine(Path.GetTempPath(), "EmuSenStates");

            Assert.Equal(
                Path.Combine(custom, "ALTTP.slot3.state"),
                SaveLibrary.StatePathFor("ALTTP.smc", 3, custom));
        }

        // Preferences leaves the field blank when the user has not chosen one.
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void A_blank_override_falls_back_to_the_default_directory(string? blank)
        {
            Assert.Equal(
                Path.Combine(DataStore.SaveStates, "ALTTP.state"),
                SaveLibrary.StatePathFor("ALTTP.smc", 1, blank));
        }

        [Fact]
        public void Both_kinds_follow_the_data_store_override()
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenData_" + Guid.NewGuid().ToString("N"));
            DataStore.OverrideDirectory = dir;

            Assert.Equal(Path.Combine(dir, "Saves", "ALTTP.srm"), SaveLibrary.SramPathFor("ALTTP.smc"));
            Assert.Equal(
                Path.Combine(dir, "Saves", "Save States", "ALTTP.state"),
                SaveLibrary.StatePathFor("ALTTP.smc"));
        }
    }
}
