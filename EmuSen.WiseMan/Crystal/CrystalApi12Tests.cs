using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // What 1.2 added to the frontend API: the config's lent SQLite, its cores directories and its development flag, honoured by the C# implementation - see EmuSen_Backend.md §29.11, Q1.
    [Collection(TestCollections.ProcessGlobals)]
    public unsafe class CrystalApi12Tests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            _f.Dispose();
        }

        private static string[] Engines(CrystalRuntime runtime) =>
            JsonDocument.Parse(runtime.Engines()).RootElement.EnumerateArray().Select(e => e.GetProperty("id").GetString()!).ToArray();

        // A tree of its own holding the plain core, its sidecar written as development or not.
        private string PlainTree(string name, bool development)
        {
            string dir = Path.Combine(_f.Root, name);
            Directory.CreateDirectory(dir);
            string source = CoreAdapterTests.LibraryPath("v1_plain_core");
            string copy = Path.Combine(dir, Path.GetFileName(source));
            File.Copy(source, copy);
            CoreSidecar.Write(copy, development);
            return dir;
        }

        [Fact]
        public void The_config_s_cores_name_the_directories_the_runtime_lists()
        {
            string plain = PlainTree("plain", development: false);
            Assert.Contains("v1-test-core", Engines(_f.Runtime));
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { plain } });
            Assert.Equal(new[] { "v1-plain-core" }, Engines(runtime));
        }

        [Fact]
        public void The_development_flag_lists_a_core_in_development()
        {
            string dev = PlainTree("dev", development: true);
            using (var hidden = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { dev } })) Assert.Empty(Engines(hidden));
            using var shown = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { dev }, Development = true });
            Assert.Equal(new[] { "v1-plain-core" }, Engines(shown));
        }

        private static (nint Runtime, string Words) New(CrystalConfig config, byte[]? cores = null)
        {
            fixed (byte* c = cores)
            {
                if (cores is not null)
                {
                    config.cores = c;
                    config.cores_len = (nuint)cores.Length;
                }
                nint rt = CrystalContract.New(&config);
                return (rt, rt != 0 ? "" : CrystalRuntime.Failure(0).Message);
            }
        }

        [Fact]
        public void A_config_of_1_0_s_size_is_taken_and_a_shorter_one_refused()
        {
            var (rt, _) = New(new CrystalConfig { size = 8 });
            Assert.NotEqual(0, rt);
            Assert.Equal(0, CrystalContract.Free(rt));
            var (refused, words) = New(new CrystalConfig { size = 4 });
            Assert.Equal(0, refused);
            Assert.Contains("version 1.0's is 8", words);
        }

        [Fact]
        public void A_flag_unknown_to_1_2_and_cores_that_are_no_list_are_refused()
        {
            var (flagged, flagWords) = New(new CrystalConfig { size = (uint)sizeof(CrystalConfig), flags = 2 });
            Assert.Equal(0, flagged);
            Assert.Contains("not known to 1.2", flagWords);
            var (listed, listWords) = New(new CrystalConfig { size = (uint)sizeof(CrystalConfig) }, Encoding.UTF8.GetBytes("{\"cores\":1}"));
            Assert.Equal(0, listed);
            Assert.Contains("JSON array of directories", listWords);
        }

        [Fact]
        public void The_config_is_32_bytes_and_its_fields_at_the_header_s_offsets()
        {
            Assert.Equal(32, sizeof(CrystalConfig));
            Assert.Equal((8, 16, 24), ((int)System.Runtime.InteropServices.Marshal.OffsetOf<CrystalConfig>("sqlite"), (int)System.Runtime.InteropServices.Marshal.OffsetOf<CrystalConfig>("cores"), (int)System.Runtime.InteropServices.Marshal.OffsetOf<CrystalConfig>("cores_len")));
            Assert.Equal(2u, CrystalAbi.Minor);
        }
    }
}
