using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity.Slang;
using Microsoft.Data.Sqlite;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The SPIR-V cache in SQLite and the passes built in parallel: same pictures, no stale row, no crash - see EmuSen_Serenity.md §9.4 and §9.5.
    public class SpirvCacheTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSpirvCache", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;
        private readonly SlangVulkan? _gpu;

        public SpirvCacheTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            _gpu = SlangVulkan.TryCreate(out string report, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800");
            _output.WriteLine($"device: {report}");
        }

        public void Dispose()
        {
            _gpu?.Dispose();
            SqliteConnection.ClearAllPools();
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Db(string name = "cache") => Path.Combine(_root, name + ".db");

        private const string Head = """
            #version 450
            #include "common.inc"
            layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
            #pragma stage vertex
            layout(location = 0) in vec4 Position;
            layout(location = 1) in vec2 TexCoord;
            layout(location = 0) out vec2 vTexCoord;
            void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }
            #pragma stage fragment
            layout(location = 0) in vec2 vTexCoord;
            layout(location = 0) out vec4 FragColor;
            layout(set = 0, binding = 2) uniform sampler2D Source;

            """;

        // Three passes that each change the picture, all including one shared file.
        private string ThreePasses(float gain = 0.5f)
        {
            File.WriteAllText(Path.Combine(_root, "common.inc"), FormattableString.Invariant($"const float GAIN = {gain:F2};\n"));
            File.WriteAllText(Path.Combine(_root, "scale.slang"), Head + "void main() { FragColor = vec4(texture(Source, vTexCoord).rgb * GAIN, 1.0); }\n");
            File.WriteAllText(Path.Combine(_root, "invert.slang"), Head + "void main() { FragColor = vec4(1.0 - texture(Source, vTexCoord).rgb, 1.0); }\n");
            File.WriteAllText(Path.Combine(_root, "swap.slang"), Head + "void main() { FragColor = vec4(texture(Source, vTexCoord).bgr, 1.0); }\n");
            string preset = Path.Combine(_root, "three.slangp");
            File.WriteAllText(preset, "shaders = 3\nshader0 = scale.slang\nshader1 = invert.slang\nshader2 = swap.slang\nfilter_linear0 = false\nfilter_linear1 = false\nfilter_linear2 = false\n");
            return preset;
        }

        private static byte[] Picture(int w, int h, int frame)
        {
            var rgba = new byte[w * h * 4];
            for (int i = 0; i < rgba.Length; i += 4) (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]) = ((byte)(i * 7 + frame), (byte)(i / 3), (byte)(200 - frame * 20), 255);
            return rgba;
        }

        // Three frames through the chain at a size of its own.
        private static List<byte[]> Frames(SlangChain chain, int w = 64, int h = 48, int vw = 128, int vh = 96)
        {
            var pictures = new List<byte[]>();
            for (int frame = 0; frame < 3; frame++)
            {
                chain.Advance(Picture(w, h, frame), w, h, 1);
                pictures.Add(chain.Render(vw, vh));
            }
            return pictures;
        }

        [Fact]
        public void The_cache_lives_beside_the_packs_in_galaxia_s_data_home()
        {
            Assert.Equal(Path.Combine(DataStore.Shaders, "spirv-cache.db"), SpirvCache.DefaultPath);
        }

        [Fact]
        public void A_second_build_compiles_nothing_and_draws_the_first_build_s_pictures()
        {
            if (_gpu is null) return;
            string preset = ThreePasses();
            List<byte[]> plain;
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(preset))) plain = Frames(chain);
            using var cache = new SpirvCache(Db());
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(preset), cache)) Assert.Equal(plain, Frames(chain));
            Assert.Equal((0, 6, 6), (cache.Hits, cache.Misses, cache.Stored));
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(preset), cache)) Assert.Equal(plain, Frames(chain));
            Assert.Equal((6, 6, 6), (cache.Hits, cache.Misses, cache.Stored));
            Assert.Null(cache.Problem);
        }

        // A pack update is new text, so a new key: the edited include is compiled again, for every stage that includes it, and drawn.
        [Fact]
        public void An_edited_include_is_compiled_again_and_its_change_is_drawn()
        {
            if (_gpu is null) return;
            using var cache = new SpirvCache(Db());
            List<byte[]> before, after, fresh;
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(ThreePasses(0.5f)), cache)) before = Frames(chain);
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(ThreePasses(0.25f)), cache)) after = Frames(chain);
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(ThreePasses(0.25f)))) fresh = Frames(chain);
            Assert.NotEqual(before, after);
            Assert.Equal(fresh, after);
            Assert.Equal(12, cache.Misses);
        }

        [Fact]
        public void Every_input_to_a_compile_is_in_its_key()
        {
            byte[] Key(string identity = "shaderc x", string text = "void main() {}", SlangStage stage = SlangStage.Vertex, string name = "a.slang") =>
                SpirvCache.Key(identity, text, stage, name);
            byte[] basis = Key();
            Assert.Equal(basis, Key());
            Assert.NotEqual(basis, Key(identity: "shaderc y"));
            Assert.NotEqual(basis, Key(text: "void main() { }"));
            Assert.NotEqual(basis, Key(stage: SlangStage.Fragment));
            Assert.NotEqual(basis, Key(name: "b.slang"));
            Assert.NotEqual(Key(name: "ab", text: "c"), Key(name: "a", text: "bc"));
            Assert.Contains(SlangCompiler.Options, SlangCompiler.Identity);
            Assert.DoesNotContain("unfound", SlangCompiler.Identity);
        }

        // A text valid as either stage compiles to different SPIR-V for each, so the stage must keep them apart.
        [Fact]
        public void One_text_compiled_as_both_stages_is_kept_as_two()
        {
            using var cache = new SpirvCache(Db());
            const string text = "#version 450\nvoid main() {}\n";
            byte[] vertex = cache.Compile(text, SlangStage.Vertex, "both.slang");
            byte[] fragment = cache.Compile(text, SlangStage.Fragment, "both.slang");
            Assert.Equal(SlangCompiler.Compile(text, SlangStage.Vertex, "both.slang"), vertex);
            Assert.Equal(SlangCompiler.Compile(text, SlangStage.Fragment, "both.slang"), fragment);
            Assert.NotEqual(vertex, fragment);
            Assert.Equal(SlangCompiler.Compile(text, SlangStage.Fragment, "both.slang"), cache.Compile(text, SlangStage.Fragment, "both.slang"));
        }

        // Another compiler, or another set of options, is another identity: nothing it left is served.
        [Fact]
        public void A_row_left_by_another_compiler_is_not_served()
        {
            const string text = "#version 450\nlayout(location = 0) out vec4 c;\nvoid main() { c = vec4(1.0); }\n";
            using (var old = new SpirvCache(Db(), compilerIdentity: "an older shaderc"))
            {
                old.Compile(text, SlangStage.Fragment, "f.slang");
                Assert.Equal(1, old.Stored);
            }
            using var current = new SpirvCache(Db());
            current.Compile(text, SlangStage.Fragment, "f.slang");
            Assert.Equal((0, 1), (current.Hits, current.Misses));
        }

        [Fact]
        public void A_damaged_row_is_compiled_again_and_never_returned()
        {
            const string text = "#version 450\nlayout(location = 0) out vec4 c;\nvoid main() { c = vec4(0.5); }\n";
            byte[] truth = SlangCompiler.Compile(text, SlangStage.Fragment, "d.slang");
            using (var cache = new SpirvCache(Db())) cache.Compile(text, SlangStage.Fragment, "d.slang");
            using (var raw = new SqliteConnection($"Data Source={Db()};Pooling=False"))
            {
                raw.Open();
                using SqliteCommand read = raw.CreateCommand();
                read.CommandText = "SELECT key, spirv FROM spirv";
                byte[] key, spirv;
                using (SqliteDataReader row = read.ExecuteReader()) { Assert.True(row.Read()); (key, spirv) = ((byte[])row[0], (byte[])row[1]); }
                spirv[40] ^= 0x01;
                using SqliteCommand flip = raw.CreateCommand();
                flip.CommandText = "UPDATE spirv SET spirv = $spirv WHERE key = $key";
                flip.Parameters.AddWithValue("$spirv", spirv);
                flip.Parameters.AddWithValue("$key", key);
                Assert.Equal(1, flip.ExecuteNonQuery());
            }
            using var again = new SpirvCache(Db());
            Assert.Equal(truth, again.Compile(text, SlangStage.Fragment, "d.slang"));
            Assert.Equal((1, 0, 1), (again.Damaged, again.Hits, again.Stored));
            Assert.Equal(truth, again.Compile(text, SlangStage.Fragment, "d.slang"));
            Assert.Equal(1, again.Hits);
        }

        [Fact]
        public void A_file_that_is_not_a_database_is_set_aside_and_the_build_goes_on()
        {
            File.WriteAllBytes(Db(), Enumerable.Range(0, 8192).Select(i => (byte)(i * 31)).ToArray());
            using var cache = new SpirvCache(Db());
            Assert.True(cache.Working, cache.Problem);
            Assert.True(File.Exists(Db() + ".damaged"));
            const string text = "#version 450\nvoid main() {}\n";
            Assert.Equal(SlangCompiler.Compile(text, SlangStage.Vertex, "v.slang"), cache.Compile(text, SlangStage.Vertex, "v.slang"));
            Assert.Equal(1, cache.Stored);
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(ThreePasses()), cache);
            Assert.Equal(3, chain.Preset.Passes.Count);
        }

        // A database another connection holds exclusively cannot be opened, and one whose writes are held is written around; either way every stage is compiled and the wait is bounded.
        [Fact]
        public void A_locked_database_is_compiled_around_whether_it_was_locked_first_or_part_way()
        {
            static string Text(int n) => $"#version 450\nlayout(location = 0) out vec4 c;\nvoid main() {{ c = vec4({n}.0); }}\n";
            using (var fill = new SpirvCache(Db())) fill.Compile("#version 450\nvoid main() {}\n", SlangStage.Vertex, "x.slang");
            var clock = Stopwatch.StartNew();

            using (var exclusive = new SqliteConnection($"Data Source={Db()};Pooling=False"))
            {
                exclusive.Open();
                using (SqliteCommand command = exclusive.CreateCommand())
                {
                    command.CommandText = "PRAGMA locking_mode = EXCLUSIVE; BEGIN EXCLUSIVE; DELETE FROM spirv WHERE bytes < 0; COMMIT;";
                    command.ExecuteNonQuery();
                }
                using var early = new SpirvCache(Db());
                _output.WriteLine($"{clock.ElapsedMilliseconds} ms: opened under an exclusive lock, working {early.Working}: {early.Problem}");
                Assert.False(early.Working);
                Assert.Equal(SlangCompiler.Compile(Text(0), SlangStage.Fragment, "l.slang"), early.Compile(Text(0), SlangStage.Fragment, "l.slang"));
            }

            using var late = new SpirvCache(Db());
            Assert.True(late.Working, late.Problem);
            using var writer = new SqliteConnection($"Data Source={Db()};Pooling=False");
            writer.Open();
            using (SqliteCommand command = writer.CreateCommand())
            {
                command.CommandText = "BEGIN IMMEDIATE; DELETE FROM spirv WHERE bytes < 0;";
                command.ExecuteNonQuery();
            }
            clock.Restart();
            for (int n = 1; n <= 6; n++)
            {
                Assert.Equal(SlangCompiler.Compile(Text(n), SlangStage.Fragment, "l.slang"), late.Compile(Text(n), SlangStage.Fragment, "l.slang"));
                _output.WriteLine($"{clock.ElapsedMilliseconds} ms: stored {late.Stored} skipped {late.Skipped} working {late.Working}");
            }
            Assert.Equal((0, 2), (late.Stored, late.Skipped));
            Assert.False(late.Working);
            Assert.NotNull(late.Problem);
            Assert.True(clock.ElapsedMilliseconds < 10000);
        }

        // Past its bound the least recently used rows go, to three quarters of it, and a row just used stays.
        [Fact]
        public void The_cache_keeps_to_its_bound_by_letting_the_least_recently_used_go()
        {
            string Shader(int n) => $"#version 450\nlayout(location = 0) out vec4 c;\nvoid main() {{ c = vec4({n}.0); }}\n";
            int one = SlangCompiler.Compile(Shader(0), SlangStage.Fragment, "s.slang").Length;
            long now = 1000;
            using var cache = new SpirvCache(Db(), limit: one * 4 + one / 2) { Clock = () => now, TouchAfter = 0 };
            for (int n = 0; n < 4; n++) { now++; cache.Compile(Shader(n), SlangStage.Fragment, "s.slang"); }
            now++;
            cache.Compile(Shader(0), SlangStage.Fragment, "s.slang");
            Assert.Equal(1, cache.Hits);
            now++;
            cache.Compile(Shader(4), SlangStage.Fragment, "s.slang");
            Assert.True(cache.Evicted >= 1);

            using var raw = new SqliteConnection($"Data Source={Db()};Pooling=False");
            raw.Open();
            using SqliteCommand sum = raw.CreateCommand();
            sum.CommandText = "SELECT SUM(bytes), COUNT(*) FROM spirv";
            using SqliteDataReader row = sum.ExecuteReader();
            row.Read();
            Assert.True(row.GetInt64(0) <= one * 4 + one / 2);
            int before = cache.Misses;
            cache.Compile(Shader(0), SlangStage.Fragment, "s.slang");
            cache.Compile(Shader(4), SlangStage.Fragment, "s.slang");
            Assert.Equal(before, cache.Misses);
            cache.Compile(Shader(1), SlangStage.Fragment, "s.slang");
            Assert.Equal(before + 1, cache.Misses);
        }

        // Two builds of one preset at once, each with its own connection as two processes would have: one row per stage, an intact file, both drawing what an uncached build draws.
        [Fact]
        public void Two_builds_of_one_preset_at_once_neither_damage_nor_duplicate_the_cache()
        {
            if (_gpu is null) return;
            string preset = Pack("crt/crt-royale.slangp") ?? ThreePasses();
            List<byte[]> plain;
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(preset), null, 1)) plain = Frames(chain);
            using var a = new SpirvCache(Db());
            using var b = new SpirvCache(Db());
            List<byte[]>? fromA = null, fromB = null;
            Parallel.Invoke(
                () => { using var chain = new SlangChain(_gpu, SlangPreset.Load(preset), a, 4); fromA = Frames(chain); },
                () => { using var chain = new SlangChain(_gpu, SlangPreset.Load(preset), b, 4); fromB = Frames(chain); });
            Assert.Equal(plain, fromA);
            Assert.Equal(plain, fromB);
            _output.WriteLine($"a: {a.Hits} hits {a.Stored} stored {a.Skipped} skipped; b: {b.Hits} hits {b.Stored} stored {b.Skipped} skipped");
            Assert.Null(a.Problem);
            Assert.Null(b.Problem);

            SlangPreset loaded = SlangPreset.Load(preset);
            var keys = new HashSet<string>();
            foreach (SlangPassSpec pass in loaded.Passes)
            {
                SlangSource source = SlangSource.Load(pass.ShaderPath);
                keys.Add(Convert.ToHexString(SpirvCache.Key(SlangCompiler.Identity, source.Vertex, SlangStage.Vertex, pass.ShaderPath)));
                keys.Add(Convert.ToHexString(SpirvCache.Key(SlangCompiler.Identity, source.Fragment, SlangStage.Fragment, pass.ShaderPath)));
            }
            using var raw = new SqliteConnection($"Data Source={Db()};Pooling=False");
            raw.Open();
            using SqliteCommand check = raw.CreateCommand();
            check.CommandText = "PRAGMA integrity_check";
            Assert.Equal("ok", check.ExecuteScalar());
            check.CommandText = "SELECT COUNT(*), COUNT(DISTINCT key) FROM spirv";
            using SqliteDataReader row = check.ExecuteReader();
            row.Read();
            Assert.Equal(keys.Count, row.GetInt32(0));
            Assert.Equal(keys.Count, row.GetInt32(1));
            Assert.Equal(keys.Count, a.Stored + b.Stored);
        }

        private static string? Pack(string relative)
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            return string.IsNullOrEmpty(pack) ? null : Path.Combine(pack, relative);
        }

        // Passes built at once draw what passes built one by one draw, with the same parameters, for the pack's heaviest presets.
        [Theory]
        [InlineData("crt/crt-royale.slangp")]
        [InlineData("crt/crt-guest-advanced.slangp")]
        [InlineData("bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV.slangp")]
        public void A_preset_built_in_parallel_draws_what_one_built_pass_by_pass_draws(string relative)
        {
            if (_gpu is null || Pack(relative) is not { } path) return;
            var clock = Stopwatch.StartNew();
            using var serial = new SlangChain(_gpu, SlangPreset.Load(path), null, 1);
            long serialMs = clock.ElapsedMilliseconds;
            clock.Restart();
            using var parallel = new SlangChain(_gpu, SlangPreset.Load(path), null, SlangChain.DefaultBuilders);
            _output.WriteLine($"serial {serialMs} ms, {SlangChain.DefaultBuilders} builders {clock.ElapsedMilliseconds} ms");
            Assert.Equal(serial.Parameters, parallel.Parameters);
            Assert.Equal(Frames(serial, 256, 224, 640, 480), Frames(parallel, 256, 224, 640, 480));
        }

        // The error a player is shown is the first broken pass's, whichever pass a parallel build found first.
        [Fact]
        public void A_parallel_build_that_fails_says_what_a_serial_build_would()
        {
            if (_gpu is null) return;
            ThreePasses();
            File.WriteAllText(Path.Combine(_root, "broken1.slang"), Head + "void main() { FragColor = one(); }\n");
            File.WriteAllText(Path.Combine(_root, "broken2.slang"), Head + "void main() { FragColor = two(); }\n");
            string preset = Path.Combine(_root, "broken.slangp");
            File.WriteAllText(preset, "shaders = 4\nshader0 = scale.slang\nshader1 = broken1.slang\nshader2 = broken2.slang\nshader3 = swap.slang\n");
            var serial = Assert.Throws<SlangCompileException>(() => new SlangChain(_gpu, SlangPreset.Load(preset), null, 1));
            for (int attempt = 0; attempt < 5; attempt++)
            {
                var parallel = Assert.Throws<SlangCompileException>(() => new SlangChain(_gpu, SlangPreset.Load(preset), null, 4));
                Assert.Equal(serial.Message, parallel.Message);
            }
            Assert.Contains("broken1", serial.Message);
        }
    }
}
