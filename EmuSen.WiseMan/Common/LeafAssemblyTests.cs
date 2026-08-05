using System;
using System.Linq;
using System.Reflection;

namespace EmuSen.WiseMan.Common
{
    // The device/presentation layers must not reach back into the core - see EmuSen_Multicore.md §9.
    public class LeafAssemblyTests
    {
        private static string[] EmuSenReferencesOf(Assembly assembly) =>
            assembly.GetReferencedAssemblies()
                .Select(a => a.Name ?? "")
                .Where(n => n.StartsWith("EmuSen", StringComparison.Ordinal))
                .OrderBy(n => n, StringComparer.Ordinal)
                .ToArray();

        // Endymion takes PCM and a rate; it has no reason to know a core exists.
        [Fact]
        public void Endymion_references_no_other_EmuSen_assembly()
        {
            Assembly endymion = typeof(EmuSen.Endymion.AudioPlayer).Assembly;

            Assert.Equal("EmuSen.Endymion", endymion.GetName().Name);
            Assert.Empty(EmuSenReferencesOf(endymion));
        }

        // Nehellania reports PadButton and persists a binding map - Galaxia covers both.
        [Fact]
        public void Nehellania_references_only_Galaxia()
        {
            Assembly nehellania = typeof(EmuSen.Nehellania.Input.GamepadManager).Assembly;

            Assert.Equal(new[] { "EmuSen.Galaxia" }, EmuSenReferencesOf(nehellania));
        }

        // The model both of the above are held to - see EmuSen_Audio_Sync.md §7.
        [Fact]
        public void Serenity_references_only_Galaxia()
        {
            Assembly serenity = typeof(EmuSen.Serenity.GameFrameControl).Assembly;

            Assert.Equal(new[] { "EmuSen.Galaxia" }, EmuSenReferencesOf(serenity));
        }

        // The launcher's whole value is browsing a library with no core loaded - see EmuSen_LunaP.md §1.
        [Fact]
        public void LunaP_references_only_Galaxia()
        {
            Assembly lunaP = typeof(EmuSen.LunaP.Controls.MeterRow).Assembly;

            Assert.Equal(new[] { "EmuSen.Galaxia" }, EmuSenReferencesOf(lunaP));
        }

        [Fact]
        public void Galaxia_is_the_root_leaf()
        {
            Assembly galaxia = typeof(EmuSen.Galaxia.Input.PadButton).Assembly;

            Assert.Equal("EmuSen.Galaxia", galaxia.GetName().Name);
            Assert.Empty(EmuSenReferencesOf(galaxia));
        }

        // Whatever else moves, no leaf may pull in a console core.
        [Theory]
        [InlineData(typeof(EmuSen.Endymion.AudioPlayer))]
        [InlineData(typeof(EmuSen.Nehellania.Input.GamepadManager))]
        [InlineData(typeof(EmuSen.Serenity.GameFrameControl))]
        [InlineData(typeof(EmuSen.LunaP.Controls.MeterRow))]
        [InlineData(typeof(EmuSen.Galaxia.Input.PadButton))]
        public void No_leaf_reaches_the_core_assembly(Type witness)
        {
            Assert.DoesNotContain("EmuSen", EmuSenReferencesOf(witness.Assembly));
        }
    }
}
