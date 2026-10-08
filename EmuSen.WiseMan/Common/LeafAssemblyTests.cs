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

        // The core-agnostic Avalonia layer; its telemetry contracts come from DianaOS since the fold - see EmuSen_Debugging_Tools_Reference_v5.md §3.66.1.1.
        [Fact]
        public void Serenity_references_only_core_free_assemblies()
        {
            Assembly serenity = typeof(EmuSen.Serenity.GameFrameControl).Assembly;

            Assert.Equal(new[] { "EmuSen.DianaOS", "EmuSen.Galaxia", "EmuSen.LunaP" }, EmuSenReferencesOf(serenity));
        }

        // LunaP is a package from another repository now, and this is what would notice the split quietly regressing - see EmuSen_LunaP.md §4.
        [Fact]
        public void LunaP_references_nothing_of_EmuSen()
        {
            Assembly lunaP = typeof(EmuSen.LunaP.Controls.MeterRow).Assembly;

            Assert.Empty(EmuSenReferencesOf(lunaP));
        }

        // The telemetry contracts live in DianaOS, which references Galaxia alone, so Serenity inherits no core and closes no cycle through it.
        [Fact]
        public void DianaOS_holds_the_telemetry_contracts_and_references_only_Galaxia()
        {
            Assembly dianaOS = typeof(EmuSen.Cauldron.ICoreTelemetry).Assembly;

            Assert.Equal("EmuSen.DianaOS", dianaOS.GetName().Name);
            Assert.Equal(new[] { "EmuSen.Galaxia" }, EmuSenReferencesOf(dianaOS));
        }

        // What the leaf enforced by construction is now this rule - see EmuSen_Debugging_Tools_Reference_v5.md §3.66.1.
        [Fact]
        public void The_telemetry_contracts_name_nothing_of_the_debugger()
        {
            const BindingFlags all = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly;
            Type[] contracts = typeof(EmuSen.Cauldron.ICoreTelemetry).Assembly.GetTypes()
                .Where(t => t.Namespace == "EmuSen.Cauldron")
                .ToArray();
            Assert.Contains(typeof(EmuSen.Cauldron.HistoryProvider<>), contracts);

            foreach (Type contract in contracts)
            {
                var named = new[] { contract.BaseType }
                    .Concat(contract.GetInterfaces())
                    .Concat(contract.GetFields(all).Select(f => f.FieldType))
                    .Concat(contract.GetProperties(all).Select(p => p.PropertyType))
                    .Concat(contract.GetMethods(all).SelectMany(m => m.GetParameters().Select(p => p.ParameterType).Append(m.ReturnType)))
                    .Concat(contract.GetConstructors(all).SelectMany(c => c.GetParameters().Select(p => p.ParameterType)));

                foreach (Type type in named.Where(t => t is not null).SelectMany(t => Flatten(t!)))
                    Assert.False(type.Namespace?.StartsWith("EmuSen.DianaOS", StringComparison.Ordinal) ?? false, $"{contract.FullName} names {type.FullName}");
            }
        }

        // A type with every type it is built from: an array's or reference's element, and each generic argument.
        private static System.Collections.Generic.IEnumerable<Type> Flatten(Type type)
        {
            yield return type;
            if (type.HasElementType)
                foreach (Type inner in Flatten(type.GetElementType()!)) yield return inner;
            foreach (Type argument in type.GetGenericArguments())
                foreach (Type inner in Flatten(argument)) yield return inner;
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
