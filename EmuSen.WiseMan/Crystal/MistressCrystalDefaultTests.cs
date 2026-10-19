using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.Loader;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The file a Mistress process enters through carries the Rust Crystal's key, read from the built assembly without opening a window - see EmuSen_Backend.md §29.29.
    public class MistressCrystalDefaultTests
    {
        private const string Key = "EmuSen.CrystalNative";

        // Mistress's built entry assembly beside the tests, and its own project's output where the same configuration built one.
        private static IEnumerable<string> Builds()
        {
            string beside = Path.Combine(AppContext.BaseDirectory, "EmuSen.Mistress.dll");
            Assert.True(File.Exists(beside), $"no Mistress built beside the tests at {beside}");
            yield return beside;
            var output = new DirectoryInfo(AppContext.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar));
            DirectoryInfo? repository = output;
            while (repository is not null && !File.Exists(Path.Combine(repository.FullName, "EmuSen.sln"))) repository = repository.Parent;
            string own = Path.Combine(repository!.FullName, "EmuSen.Mistress", "bin", output.Parent!.Name, output.Name, "EmuSen.Mistress.dll");
            if (File.Exists(own)) yield return own;
        }

        // Each AssemblyMetadata value the file's assembly carries under the key, read from its metadata tables alone.
        private static List<string?> Values(string path)
        {
            using FileStream stream = File.OpenRead(path);
            using var pe = new PEReader(stream);
            MetadataReader md = pe.GetMetadataReader();
            var values = new List<string?>();
            foreach (CustomAttributeHandle handle in md.GetAssemblyDefinition().GetCustomAttributes())
            {
                CustomAttribute attribute = md.GetCustomAttribute(handle);
                if (attribute.Constructor.Kind != HandleKind.MemberReference) continue;
                MemberReference ctor = md.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
                if (ctor.Parent.Kind != HandleKind.TypeReference || md.GetString(md.GetTypeReference((TypeReferenceHandle)ctor.Parent).Name) != nameof(AssemblyMetadataAttribute)) continue;
                BlobReader blob = md.GetBlobReader(attribute.Value);
                Assert.Equal(1, blob.ReadUInt16());
                if (blob.ReadSerializedString() == Key) values.Add(blob.ReadSerializedString());
            }
            return values;
        }

        [Fact]
        public void Mistress_s_built_entry_assembly_carries_the_key_with_the_value_1()
        {
            foreach (string build in Builds()) Assert.Equal(new string?[] { "1" }, Values(build));
        }

        // The binding's own reader, over the file loaded apart from the tests' copy.
        [Fact]
        public void The_binding_reads_Mistress_s_built_file_as_defaulting_to_the_rust()
        {
            foreach (string build in Builds())
            {
                var context = new AssemblyLoadContext("MistressDefault", isCollectible: true);
                try { Assert.True(CrystalContract.DefaultsToRust(context.LoadFromAssemblyPath(build)), build); }
                finally { context.Unload(); }
            }
        }
    }
}
