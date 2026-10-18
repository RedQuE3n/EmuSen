using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace EmuSen.WiseMan.Crystal
{
    // SR2: a frontend on the frontend API names no type of the runtime's own assemblies, read from its metadata, so a type reached through a transitive reference fails too - see EmuSen_Backend.md §11 and §14.
    public class FrontendReferenceTests
    {
        private static readonly string[] Runtime = { "EmuSen", "EmuSen.Crystal" };

        // Every type the assembly's metadata references, by the assembly it resolves to.
        public static IReadOnlyList<(string Assembly, string Type)> TypeReferences(Assembly assembly)
        {
            using var stream = File.OpenRead(assembly.Location);
            using var pe = new PEReader(stream);
            MetadataReader md = pe.GetMetadataReader();
            var found = new List<(string, string)>();
            foreach (TypeReferenceHandle handle in md.TypeReferences)
            {
                TypeReference t = md.GetTypeReference(handle);
                EntityHandle scope = t.ResolutionScope;
                while (scope.Kind == HandleKind.TypeReference) scope = md.GetTypeReference((TypeReferenceHandle)scope).ResolutionScope;
                if (scope.Kind != HandleKind.AssemblyReference) continue;
                string owner = md.GetString(md.GetAssemblyReference((AssemblyReferenceHandle)scope).Name);
                found.Add((owner, $"{md.GetString(t.Namespace)}.{md.GetString(t.Name)}"));
            }
            return found;
        }

        public static void AssertNamesNoRuntimeType(Assembly frontend)
        {
            string[] referenced = frontend.GetReferencedAssemblies().Select(a => a.Name ?? "").ToArray();
            Assert.DoesNotContain(referenced, Runtime.Contains);
            var reached = TypeReferences(frontend).Where(r => Runtime.Contains(r.Assembly)).Select(r => $"{r.Type} ({r.Assembly})").ToArray();
            Assert.True(reached.Length == 0, $"{frontend.GetName().Name} names the runtime's types: {string.Join(", ", reached)}");
            Assert.Contains("EmuSen.Crystal.Client", referenced);
        }

        [Fact]
        public void Pharaoh_names_no_runtime_type() => AssertNamesNoRuntimeType(typeof(EmuSen.Pharaoh.FrameRunner).Assembly);

        // The check sees a frontend that does name the runtime's types: Hotaru, not yet moved, names ICore.
        [Fact]
        public void The_check_sees_a_frontend_that_names_the_runtime()
        {
            var reached = TypeReferences(typeof(EmuSen.Hotaru.HotaruSession).Assembly);
            Assert.Contains(reached, r => r == ("EmuSen", "EmuSen.Cores.ICore"));
        }
    }
}
