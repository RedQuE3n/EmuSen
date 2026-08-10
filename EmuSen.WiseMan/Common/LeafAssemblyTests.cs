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

        // Endymion takes PCM and a rate, and reports PadButton - Galaxia covers both halves.
        [Fact]
        public void Endymion_references_only_Galaxia()
        {
            Assembly endymion = typeof(EmuSen.Endymion.AudioPlayer).Assembly;

            Assert.Equal(new[] { "EmuSen.Galaxia" }, EmuSenReferencesOf(endymion));
        }

        // Both halves of the SDL3 layer ship in one assembly - see EmuSen_Multicore.md §9.2.
        [Fact]
        public void Endymion_carries_the_gamepad_layer_too()
        {
            Assembly audio = typeof(EmuSen.Endymion.AudioPlayer).Assembly;
            Assembly input = typeof(EmuSen.Endymion.Input.GamepadManager).Assembly;

            Assert.Same(audio, input);
        }

        // The core-agnostic Avalonia layer: it took CoretopWindow when the toolkit had to stop naming EmuSen - see EmuSen_LunaP.md §19.3.
        [Fact]
        public void Serenity_references_only_core_free_leaves()
        {
            Assembly serenity = typeof(EmuSen.Serenity.GameFrameControl).Assembly;

            Assert.Equal(new[] { "EmuSen.Cauldron", "EmuSen.Galaxia", "EmuSen.LunaP" }, EmuSenReferencesOf(serenity));
        }

        // LunaP is a package from another repository now, and this is what would notice the split quietly regressing - see EmuSen_LunaP.md §4.
        [Fact]
        public void LunaP_references_nothing_of_EmuSen()
        {
            Assembly lunaP = typeof(EmuSen.LunaP.Controls.MeterRow).Assembly;

            Assert.Empty(EmuSenReferencesOf(lunaP));
        }

        // Cauldron is still a leaf, and Serenity now depends on that the way LunaP used to - see EmuSen_LunaP.md §16.
        [Fact]
        public void Cauldron_is_a_leaf_so_Serenity_inherits_nothing_through_it()
        {
            Assembly cauldron = typeof(EmuSen.Cauldron.ICoreTelemetry).Assembly;

            Assert.Equal("EmuSen.Cauldron", cauldron.GetName().Name);
            Assert.Empty(EmuSenReferencesOf(cauldron));
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
        [InlineData(typeof(EmuSen.Endymion.Input.GamepadManager))]
        [InlineData(typeof(EmuSen.Serenity.GameFrameControl))]
        [InlineData(typeof(EmuSen.LunaP.Controls.MeterRow))]
        [InlineData(typeof(EmuSen.Galaxia.Input.PadButton))]
        [InlineData(typeof(EmuSen.Cauldron.ICoreTelemetry))]
        public void No_leaf_reaches_the_core_assembly(Type witness)
        {
            Assert.DoesNotContain("EmuSen", EmuSenReferencesOf(witness.Assembly));
        }
    }
}
