using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Crystal;
using Microsoft.Data.Sqlite;
using Silk.NET.Vulkan;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Serenity's device and chain in the platform library, given presets and frames and held byte for byte to the pictures and answers the C#'s gave before it was deleted, on the RX 6800 they were recorded on - see EmuSen_Backend.md §40.6 and EmuSen_RustPlatform.md §17.5.
    public sealed unsafe class SerenityChainParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSerenityChainParity", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";
        private readonly SlangVulkan? _gpu;
        private readonly Dictionary<string, ParityGolden> _goldens = new(StringComparer.Ordinal);

        public SerenityChainParityTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
            Directory.CreateDirectory(_root);
            _gpu = SlangVulkan.TryCreate(out string report, _wanted, null);
            _output.WriteLine(report);
        }

        public void Dispose()
        {
            _gpu?.Dispose();
            SqliteConnection.ClearAllPools();
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The device, or none where the machine has no such card and the case passes by.
        private bool Gpu => _gpu is not null;

        private ParityGolden Golden(string test)
        {
            if (!_goldens.TryGetValue(test, out ParityGolden? golden)) _goldens[test] = golden = new ParityGolden("SerenityChainParity." + test);
            return golden;
        }

        private static string Digest(byte[] bytes) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))[..16];

        // The test's folder written plainly, so an answer holds no machine's path.
        private string Plainly(string text) => text.Replace(_root, "<root>", StringComparison.Ordinal);

        private const string Vertex = """
            #pragma stage vertex
            layout(location = 0) in vec4 Position;
            layout(location = 1) in vec2 TexCoord;
            layout(location = 0) out vec2 vTexCoord;
            void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }
            #pragma stage fragment
            layout(location = 0) in vec2 vTexCoord;
            layout(location = 0) out vec4 FragColor;

            """;

        private const string Plain = """
            #version 450
            layout(push_constant) uniform Push { vec4 SourceSize; vec4 OutputSize; uint FrameCount; float gain; } params;
            #pragma parameter gain "Gain" 1.0 0.0 1.0 0.1
            layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;

            """ + Vertex;

        private string Write(string name, string text)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllText(path, text);
            return path;
        }

        private string Identity() => Preset("identity.slangp", "shaders = 1\nshader0 = " + Path.GetFileName(Write("identity.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord) * params.gain; }\n")) + "\nfilter_linear0 = false\n");

        private string Preset(string name, string text) => Write(name, text);

        private void Image(string name, int width, int height, Func<int, int, SKColor> pixel)
        {
            using var bitmap = new SKBitmap(width, height);
            for (int y = 0; y < height; y++)
                for (int x = 0; x < width; x++) bitmap.SetPixel(x, y, pixel(x, y));
            using SKData data = bitmap.Encode(SKEncodedImageFormat.Png, 100);
            File.WriteAllBytes(Path.Combine(_root, name), data.ToArray());
        }

        // Three passes that between them read every name a pass may: the source through each wrap and filter, a mip chain, the history, feedback by number and by alias, an earlier pass by number, by the preset's alias and by the source's own name, two lookup images, every size, every count and two parameters.
        private string Everything()
        {
            Image("lut.png", 8, 4, (x, y) => new SKColor((byte)(x * 30 + 10), (byte)(y * 60 + 5), (byte)(200 - x * 20), (byte)(255 - y * 40)));
            Image("tile.png", 3, 3, (x, y) => new SKColor((byte)(x * 100), (byte)(y * 100), (byte)((x + y) * 50)));
            Write("zero.slang", """
                #version 450
                layout(push_constant) uniform Push
                {
                    vec4 SourceSize; vec4 OriginalSize; vec4 OutputSize;
                    uint FrameCount; int FrameDirection; uint Rotation; uint TotalSubFrames; uint CurrentSubFrame;
                    float gain; float OriginalAspect; float OriginalFPS;
                } params;
                layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; vec4 FinalViewportSize; vec4 LutSize; float bias; float unknown; } global;
                #pragma name Zero
                #pragma format R16G16B16A16_SFLOAT
                #pragma parameter gain "Gain" 0.5 0.0 2.0 0.05
                #pragma parameter bias "Bias" 0.1 0.0 1.0 0.01

                """ + Vertex + """
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D Lut;
                layout(set = 0, binding = 3) uniform sampler2D OriginalHistory2;
                layout(set = 0, binding = 4) uniform sampler2D Original;
                layout(set = 0, binding = 5) uniform sampler2D PassOutput2;
                layout(set = 0, binding = 6) uniform sampler2D Mid;
                void main()
                {
                    vec3 c = texture(Source, vTexCoord * 1.5 - 0.25).rgb * params.gain + global.bias + global.unknown;
                    c += texture(Lut, vTexCoord * 2.0).rgb * 0.25 + textureLod(Lut, vTexCoord, 0.5).rgb * 0.125 + textureLod(Lut, vTexCoord, 20.0).rgb * 0.125;
                    c += texture(OriginalHistory2, vTexCoord).rgb * 0.125 + texture(Original, vTexCoord).rgb * 0.0625;
                    c += texture(PassOutput2, vTexCoord).rgb + texture(Mid, vTexCoord).rgb;
                    float counts = float(params.FrameCount) / 7.0 + float(params.FrameDirection) * 0.25 + float(params.Rotation) * 0.5 + float(params.TotalSubFrames) * 0.125 + float(params.CurrentSubFrame) * 0.0625;
                    float sizes = params.OutputSize.x * params.SourceSize.z * 0.07 + params.OriginalSize.y * global.FinalViewportSize.w + params.OutputSize.w * 3.0 + global.LutSize.x * global.LutSize.w * 0.11 + global.FinalViewportSize.x * params.OriginalSize.z * 0.013;
                    FragColor = vec4(fract(c + vec3(counts, sizes, params.OriginalAspect * 0.3 + params.OriginalFPS / 60.0 * 0.37)), 1.0);
                }
                """);
            Write("one.slang", """
                #version 450
                layout(push_constant) uniform Push { vec4 SourceSize; vec4 ZeroSize; vec4 PassOutput0Size; vec4 PassFeedback1Size; vec4 OriginalHistory2Size; vec4 TileSize; uint FrameCount; float gain; } params;
                layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; vec4 MidFeedbackSize; vec4 NothingSize; vec4 OutputSize; } global;
                #pragma parameter gain "Gain again" 0.25 0.0 2.0 0.05
                #pragma parameter depth "Depth" 2.0 0.0 4.0 1.0

                """ + Vertex + """
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D Zero;
                layout(set = 0, binding = 3) uniform sampler2D PassFeedback1;
                layout(set = 0, binding = 4) uniform sampler2D PassOutput0;
                layout(set = 0, binding = 5) uniform sampler2D Tile;
                layout(set = 0, binding = 6) uniform sampler2D Nothing;
                layout(set = 0, binding = 7) uniform sampler2D OriginalHistory1;
                void main()
                {
                    vec3 c = textureLod(Source, vTexCoord, 2.0).rgb * 0.4 + textureLod(Source, vTexCoord, 20.0).rgb * 0.1 + texture(Zero, vTexCoord * 3.0).rgb * 0.2 + texture(PassFeedback1, vTexCoord).rgb * 0.3 + texture(PassOutput0, vTexCoord).rgb * 0.05;
                    c += texture(Tile, vTexCoord * 4.0 - 1.0).rgb * 0.05 + texture(Nothing, vTexCoord).rgb + textureLod(OriginalHistory1, vTexCoord, 3.0).rgb * 0.05;
                    float sizes = params.ZeroSize.x * params.ZeroSize.w * 0.1 + params.PassOutput0Size.y * params.PassFeedback1Size.z * 0.3 + params.OriginalHistory2Size.x * params.TileSize.w * 0.01 + global.MidFeedbackSize.y * global.MidFeedbackSize.z * 0.2 + global.NothingSize.x + global.OutputSize.x * params.SourceSize.w * 0.05 + params.TileSize.x * 0.07;
                    FragColor = vec4(fract(c + vec3(sizes, float(params.FrameCount) / 11.0, params.SourceSize.z * 4.0)), 1.0);
                }
                """);
            Write("two.slang", """
                #version 450
                layout(push_constant) uniform Push { vec4 SourceSize; vec4 OutputSize; vec4 FinalViewportSize; vec4 MidSize; uint FrameCount; float gain; float depth; } params;
                layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
                #pragma parameter MidSize "A parameter with a size's name, which the size takes" 0.5 0.0 1.0 0.1
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D Mid;
                layout(set = 0, binding = 3) uniform sampler2D MidFeedback;
                layout(set = 0, binding = 4) uniform sampler2D ZeroFeedback;
                layout(set = 0, binding = 5) uniform sampler2D Original;
                #pragma stage vertex
                layout(location = 0) in vec4 Position;
                layout(location = 1) in vec2 TexCoord;
                layout(location = 0) out vec2 vTexCoord;
                layout(location = 1) out vec3 Tint;
                void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; Tint = textureLod(Mid, vec2(0.5), 0.0).rgb; }
                #pragma stage fragment
                layout(location = 0) in vec2 vTexCoord;
                layout(location = 1) in vec3 Tint;
                layout(location = 7) in vec2 Unread;
                layout(location = 0) out vec4 FragColor;
                void main()
                {
                    vec4 s = texture(Source, vTexCoord * 1.25 - 0.125);
                    vec3 c = s.rgb * 0.5 + s.a * 0.1 + texture(Mid, vTexCoord).rgb * 0.2 + texture(MidFeedback, vTexCoord).rgb * 0.2 + texture(ZeroFeedback, vTexCoord).rgb * 0.05 + texture(Original, vTexCoord).rgb * 0.05 + Tint * 0.05;
                    float sizes = params.OutputSize.x * params.FinalViewportSize.z * 0.1 + params.MidSize.y * params.SourceSize.w * 0.1 + params.SourceSize.x * 0.003;
                    FragColor = vec4(fract(c * params.gain + vec3(sizes, params.depth * 0.05, float(params.FrameCount) / 255.0)), 1.0);
                }
                """);
            return Preset("everything.slangp", """
                shaders = 3
                shader0 = zero.slang
                filter_linear0 = true
                wrap_mode0 = repeat
                scale_type0 = source
                scale0 = 2.5
                frame_count_mod0 = 5
                shader1 = one.slang
                alias1 = Mid
                mipmap_input1 = true
                filter_linear1 = true
                wrap_mode1 = mirrored_repeat
                scale_type_x1 = absolute
                scale_x1 = 37
                scale_type_y1 = viewport
                scale_y1 = 0.3
                srgb_framebuffer1 = true
                shader2 = two.slang
                filter_linear2 = false
                wrap_mode2 = clamp_to_border
                textures = "Lut;Tile"
                Lut = lut.png
                Lut_linear = true
                Lut_mipmap = true
                Lut_wrap_mode = repeat
                Tile = tile.png
                Tile_linear = false
                Tile_wrap_mode = clamp_to_edge
                gain = 0.8
                """);
        }

        // What each pass is drawn into: a float pass by the preset's word, an 8-bit one after it, and a last pass that asks for sRGB and for float and is given neither.
        private string Formats()
        {
            Write("triple.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = vec4(texture(Source, vTexCoord).rgb * 3.0, 1.0); }\n");
            Write("third.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = vec4(texture(Source, vTexCoord).rgb / 3.0, 1.0); }\n");
            Write("asks.slang", Plain + "#pragma format R8G8B8A8_SRGB\nlayout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            return Preset("formats.slangp", "shaders = 3\nshader0 = triple.slang\nfloat_framebuffer0 = true\nshader1 = third.slang\nshader2 = asks.slang\nfloat_framebuffer2 = true\n");
        }

        // One pass over a frame with a mip chain: the frame read far down the chain, the newest frame at a level, and an older one that is read without its chain whatever level is asked.
        private string Mips()
        {
            Write("levels.slang", Plain + """
                layout(set = 0, binding = 2) uniform sampler2D Source;
                layout(set = 0, binding = 3) uniform sampler2D Original;
                layout(set = 0, binding = 4) uniform sampler2D OriginalHistory1;
                void main() { FragColor = vec4(textureLod(Source, vTexCoord, 20.0).rgb * 0.3 + textureLod(Original, vTexCoord, 2.0).rgb * 0.4 + textureLod(OriginalHistory1, vTexCoord, 2.0).rgb * 0.3, 1.0); }
                """);
            return Preset("mips.slangp", "shaders = 1\nshader0 = levels.slang\nmipmap_input0 = true\nfilter_linear0 = true\n");
        }

        private static List<byte[]> Frames(SlangChain chain, int width, int height, int repeat, int viewWidth, int viewHeight, int count, int first = 0)
        {
            var frames = new List<byte[]>();
            for (int k = first; k < first + count; k++)
            {
                chain.Advance(ShaderBench.Pattern(width, height, k), width, height, repeat);
                frames.Add(chain.Render(viewWidth, viewHeight));
            }
            return frames;
        }

        private static void AssertSame(IReadOnlyList<byte[]> expected, IReadOnlyList<byte[]> pictures, string what)
        {
            Assert.Equal(expected.Count, pictures.Count);
            for (int k = 0; k < expected.Count; k++)
            {
                if (expected[k].AsSpan().SequenceEqual(pictures[k])) continue;
                int different = expected[k].Length == pictures[k].Length ? expected[k].Where((b, i) => b != pictures[k][i]).Count() : -1;
                Assert.Fail($"{what}: picture {k} differs in {different} of {expected[k].Length} bytes (the other has {pictures[k].Length})");
            }
        }

        // Pictures held to the digests recorded of the C#'s: each picture's size and bytes.
        private static void Hold(ParityGolden golden, string what, IReadOnlyList<byte[]> pictures)
        {
            for (int k = 0; k < pictures.Count; k++) golden.Add($"{what} picture {k}", $"{pictures[k].Length} bytes {Digest(pictures[k])}");
        }

        // A picture that is one colour all over would agree with another such whatever the chain did.
        private static void AssertDrawn(byte[] picture) => Assert.True(picture.Distinct().Count() > 16, "the picture is nearly one value");

        private void Compare(ParityGolden golden, string preset, int width, int height, int repeat, int viewWidth, int viewHeight, int count, IReadOnlyDictionary<string, float>? values = null, int builders = 1, bool drawn = true)
        {
            using var chain = new SlangChain(_gpu!, SlangPreset.Load(preset), null, builders);
            golden.Add($"{Path.GetFileName(preset)} parameters", string.Join(", ", chain.Parameters.Select(p => $"{p.Id} {p.Description} {BitConverter.SingleToUInt32Bits(p.Initial):X8} {BitConverter.SingleToUInt32Bits(p.Minimum):X8} {BitConverter.SingleToUInt32Bits(p.Maximum):X8} {BitConverter.SingleToUInt32Bits(p.Step):X8}")));
            if (values is not null) chain.SetParameters(values);
            List<byte[]> pictures = Frames(chain, width, height, repeat, viewWidth, viewHeight, count);
            Hold(golden, $"{Path.GetFileName(preset)} {width}x{height}x{repeat} to {viewWidth}x{viewHeight}, {builders} builders", pictures);
            if (drawn) AssertDrawn(pictures[^1]);
        }

        [Fact]
        public void A_device_is_chosen_by_its_name_in_any_case_and_one_that_is_not_there_is_said_so()
        {
            if (!Gpu) return;
            
            foreach (string? name in new[] { "no device is named like this", "6800 and then some", "\u017F", "\"quoted\"", "caf\u00E9" })
            {
                Assert.Null(SlangVulkan.TryCreate(out string report, name, null));
                Assert.Equal($"no Vulkan device is named like \"{name}\"", report);
            }

            // Whatever its case, and any device for no name at all.
            foreach (string? name in new[] { _gpu!.Name.ToUpperInvariant(), _gpu.Name.ToLowerInvariant(), _gpu.Name[1..^1] })
            {
                using SlangVulkan? found = SlangVulkan.TryCreate(out string report, name, null);
                Assert.Equal(_gpu.Name, found?.Name);
                Assert.Contains(_gpu.Name, report);
            }
            foreach (string? name in new[] { "", null })
            {
                using SlangVulkan? any = SlangVulkan.TryCreate(out string report, name, null);
                Assert.NotNull(any);
            }
        }

        private static readonly string[] FormatPieces =
        {
            "R8G8B8A8", "UNORM", "Unorm", "unorm", "SRGB", "SFLOAT", "R16G16B16A16", "A2B10G10R10", "PACK32", "_", "__", ",", " ", "\t", "\n", "\0", "\u00A0", "\u2003", "\u0085", "\u017F", "\u212A", "\u0130", "\u0131",
            "0", "1", "37", "43", "97", "-", "+", "2147483647", "2147483648", "99999999999", "x", "e", ".", "R", "8", "Undefined", "BLOCK", "ASTC", "4x4", "KHR", "Khr", "IMG", "S", "s", "k", "K", "RGB", "rgb",
        };

        // The C#'s rule, kept as the oracle since it is .NET's own enumeration parse over the binding's names: words split at underscores and capitalised as the binding spells them.
        private static Format DotNetFormat(string? name, Format fallback)
        {
            if (string.IsNullOrEmpty(name)) return fallback;
            string[] parts = name.Split('_');
            string spelled = parts[0] + string.Concat(parts.Skip(1).Select(p => p.Length == 0 ? p : char.ToUpperInvariant(p[0]) + p[1..].ToLowerInvariant()));
            return Enum.TryParse(spelled, out Format format) ? format : fallback;
        }

        [Fact]
        public void A_pragma_s_format_is_the_one_dotnet_reads()
        {
            var words = new List<string>(FormatPieces) { "R8G8B8A8_UNORM", "R8G8B8A8_SRGB", "R16G16B16A16_SFLOAT", "R32G32B32A32_SFLOAT", "A2B10G10R10_UNORM_PACK32", "R8_UNORM", "R16G16_SFLOAT", "B8G8R8A8_UNORM", "ASTC_4x4_UNORM_BLOCK", "G8B8G8R8_422_UNORM", "BC1_RGB_UNORM_BLOC\u212A", "R8G8B8A8_\u017FRGB", "R8G8B8A8_\u017Frgb", "BC1_RGB_UNORM_\u212ALOCK" };
            // Every name the binding has, as it has it, with a comma between two, and spelled as a pragma would spell it.
            string[] names = Enum.GetNames<Format>();
            words.AddRange(names);
            words.AddRange(names.Select((name, i) => name + "," + names[(i * 7 + 3) % names.Length]));
            words.AddRange(names.Select(name => string.Concat(name.Select((c, i) => i > 0 && char.IsUpper(c) ? "_" + c : c.ToString())).ToUpperInvariant()));
            var random = new Random(53);
            for (int i = 0; i < 60_000; i++) words.Add(string.Concat(Enumerable.Range(0, random.Next(1, 6)).Select(_ => FormatPieces[random.Next(FormatPieces.Length)])));

            var different = new List<string>();
            int formats = 0;
            foreach (string word in words)
            {
                Format managed = DotNetFormat(word, (Format)(-7)), library = SlangChain.ParseFormatNative(word, (Format)(-7));
                if (managed != library) different.Add($"[{string.Concat(word.Select(c => c < 32 || c > 126 ? $"<{(int)c:X4}>" : c.ToString()))}] C# {(int)managed}, library {(int)library}");
                if ((int)managed != -7) formats++;
            }
            _output.WriteLine($"{words.Count} words, {formats} of them formats");
            Assert.True(different.Count == 0, $"{different.Count} words differ:\n{string.Join("\n", different.Take(10))}");
            Assert.True(formats > names.Length);
            Assert.Equal(Format.R16G16B16A16Sfloat, SlangChain.ParseFormatNative("R16G16B16A16_SFLOAT", Format.Undefined));
            Assert.Equal(Format.R8Unorm, SlangChain.ParseFormat(null, Format.R8Unorm));
            Assert.Equal(Format.R8Unorm, SlangChain.ParseFormatNative("", Format.R8Unorm));
        }

        [Fact]
        public void A_pass_gives_the_picture_it_gave()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Passes");
            Compare(golden, Identity(), 16, 12, 1, 64, 48, 3);
            Compare(golden, Identity(), 16, 12, 1, 64, 48, 2, new Dictionary<string, float> { ["gain"] = 0.5f, ["not a parameter"] = 3f });
            // A size that is nothing, or less, is one by one.
            Compare(golden, Identity(), 5, 3, 1, 0, -4, 2, drawn: false);
            Compare(golden, Formats(), 16, 12, 1, 48, 36, 3);
            Compare(golden, Mips(), 16, 12, 1, 48, 36, 4);
            Compare(golden, Mips(), 16, 6, 2, 48, 36, 4);
            golden.Check(6 + 3 + 2 + 2 + 3 + 4 + 4);
        }

        // A pass that adds a tenth of white to its own last output: the phosphor tails' shape, read through feedback by its alias.
        private string Counting() => Preset("counting.slangp", "shaders = 1\nshader0 = " + Path.GetFileName(Write("counting.slang", Plain
            + "layout(set = 0, binding = 2) uniform sampler2D CountFeedback;\nvoid main() { FragColor = vec4(texture(CountFeedback, vTexCoord).r + 0.1 * params.gain, 0.0, 0.0, 1.0); }\n"))
            + "\nalias0 = Count\nfilter_linear0 = false\n");

        // A frame rendered again, for a new size, a new value or a present before a readback, leaves feedback where the frame left it; the next frame moves it on - see EmuSen_DevicePicture.md §22.
        [Fact]
        public void Feedback_advances_once_a_game_frame_however_often_the_frame_is_rendered()
        {
            if (!Gpu) return;
            var frame = new byte[8 * 8 * 4];
            foreach (var (name, gpu) in new[] { ("library", _gpu!) })
            {
                using var chain = new SlangChain(gpu, SlangPreset.Load(Counting()), null, 1);
                chain.Advance(frame, 8, 8, 1);
                byte[] once = chain.Render(8, 8), again = chain.Render(8, 8);
                Assert.True(once.AsSpan().SequenceEqual(again), $"{name}: a second render of one frame moved its feedback from {once[0]} to {again[0]}");
                chain.SetParameters(new Dictionary<string, float> { ["gain"] = 1f });
                Assert.Equal(once[0], chain.Render(8, 8)[0]);
                chain.Advance(frame, 8, 8, 1);
                Assert.InRange(chain.Render(8, 8)[0] - once[0], 24, 27);
            }

            // The device path's present and a readback after it, one frame: both read the frame before's output.
            using var presented = new SlangChain(_gpu!, SlangPreset.Load(Counting()), null, 1);
            using var shown = SlangShown.TryCreate(_gpu!, 8, 8, linear: true, out string? problem) ?? throw new InvalidOperationException(problem);
            presented.Advance(frame, 8, 8, 1);
            presented.Present(shown, 8, 8, waitFree: false);
            using var rendered = new SlangChain(_gpu!, SlangPreset.Load(Counting()), null, 1);
            rendered.Advance(frame, 8, 8, 1);
            Assert.Equal(rendered.Render(8, 8)[0], presented.Render(8, 8)[0]);
        }

        [Fact]
        public void Three_passes_that_read_every_name_give_the_pictures_they_gave()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Everything");
            string preset = Everything();
            // 13 by 7 at 2.5 is 32.5 by 17.5: a half goes to the even number, 32 and 18.
            Compare(golden, preset, 13, 7, 1, 97, 61, 7);
            Compare(golden, preset, 13, 7, 3, 64, 50, 4, new Dictionary<string, float> { ["gain"] = 1.25f, ["depth"] = 4f, ["bias"] = 0f });
            Compare(golden, preset, 32, 24, 2, 320, 240, 6, builders: 4);
            golden.Check(3 + 7 + 4 + 6);
        }

        [Fact]
        public void A_chain_fed_frames_of_changing_size_and_drawn_at_changing_sizes_gives_the_pictures_it_gave()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("ChangingSizes");
            string preset = Everything();
            using var chain = new SlangChain(_gpu!, SlangPreset.Load(preset));
            var steps = new (int W, int H, int Repeat, int ViewW, int ViewH)[] { (16, 8, 1, 40, 30), (16, 8, 1, 41, 30), (16, 8, 2, 41, 30), (8, 16, 2, 41, 30), (8, 16, 1, 200, 100), (8, 16, 1, 200, 100), (1, 1, 1, 3, 3), (64, 48, 4, 128, 96) };
            var pictures = new List<byte[]>();
            for (int k = 0; k < steps.Length; k++)
            {
                var (w, h, repeat, viewW, viewH) = steps[k];
                pictures.AddRange(Frames(chain, w, h, repeat, viewW, viewH, 2, k * 2));
                // A redraw with no new frame, as a paused game gives, still moves the feedback.
                pictures.Add(chain.Render(viewW, viewH));
            }
            Hold(golden, "changing sizes", pictures);
            golden.Check(steps.Length * 3);
        }

        [Fact]
        public void Parameters_are_declared_valued_and_set_as_they_were()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Parameters");
            string preset = Everything();
            using var library = new SlangChain(_gpu!, SlangPreset.Load(preset));
            Assert.Equal(new[] { "gain", "bias", "depth", "MidSize" }, library.Parameters.Select(p => p.Id));
            golden.Add("declared", string.Join(", ", library.Parameters.Select(p => $"{p.Id} {p.Description} {BitConverter.SingleToUInt32Bits(p.Initial):X8} {BitConverter.SingleToUInt32Bits(p.Minimum):X8} {BitConverter.SingleToUInt32Bits(p.Maximum):X8} {BitConverter.SingleToUInt32Bits(p.Step):X8}")));
            Assert.Equal(0.8f, library.Parameters[0].Initial);
            int set = 0;

            void Same(params string[] ids)
            {
                golden.Add($"set {set++}", string.Join(", ", ids.Select(id => $"{id}={BitConverter.SingleToUInt32Bits(library.ValueOf(id)):X8}")));
            }
            Same("gain", "bias", "depth", "MidSize", "nothing", "", "GAIN", "caf\u00E9");
            foreach (var values in new IReadOnlyDictionary<string, float>?[]
            {
                new Dictionary<string, float> { ["gain"] = 1.5f },
                new Dictionary<string, float> { ["depth"] = float.NaN, ["bias"] = float.NegativeInfinity, ["other"] = 1f },
                new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase) { ["GAIN"] = 0.125f },
                new Dictionary<string, float>(),
                null,
            })
            {
                library.SetParameters(values);
                Same("gain", "bias", "depth", "nothing", "other", "GAIN");
            }
            Assert.Equal(0.8f, library.ValueOf("gain"));
            Assert.True(float.IsNaN(library.ValueOf("nothing")));
            Assert.Throws<ArgumentNullException>(() => library.ValueOf(null!));
            golden.Check(1 + 1 + 5);
        }

        // What a chain's making or a call on it came to: nothing, or the exception with the words the C# chooses.
        private static string Outcome(Action call)
        {
            try { call(); return "nothing"; }
            catch (FileNotFoundException e) { return $"FileNotFoundException: {e.Message} [{e.FileName}]"; }
            catch (ArgumentOutOfRangeException) { return "ArgumentOutOfRangeException"; }
            catch (SlangCompileException e) { return $"SlangCompileException: {e.Message}"; }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void A_chain_that_cannot_be_built_or_fed_fails_as_it_did()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Failures");
            Write("good.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            Write("bad.slang", Plain + "void main() { FragColor = nothing_declared; }\n");
            Write("worse.slang", Plain + "void main() { this is not GLSL }\n");
            Write("deep.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D OriginalHistory2147483647;\nvoid main() { FragColor = texture(OriginalHistory2147483647, vTexCoord); }\n");
            File.WriteAllText(Path.Combine(_root, "not.png"), "this is not a picture");

            void Alike(string name, string text, int builders = 1)
            {
                string preset = Preset(name, text);
                string library = Plainly(Outcome(() => { using var chain = new SlangChain(_gpu!, SlangPreset.Load(preset), null, builders); }));
                _output.WriteLine($"{name}: {library.Split('\n')[0]}");
                golden.Add($"{name} with {builders} builders", library);
                Assert.NotEqual("nothing", library);
            }

            Alike("missing.slangp", "shaders = 2\nshader0 = good.slang\nshader1 = not-there.slang\n");
            Alike("bad.slangp", "shaders = 1\nshader0 = bad.slang\n");
            Alike("deep.slangp", "shaders = 1\nshader0 = deep.slang\n");
            Alike("undecoded.slangp", "shaders = 1\nshader0 = good.slang\ntextures = \"Lut\"\nLut = not.png\n");
            Alike("unfound.slangp", "shaders = 1\nshader0 = good.slang\ntextures = \"Lut\"\nLut = not-there.png\n");
            // Two passes fail: built in order the first is said; built side by side the lowest-numbered is, which is the same one.
            foreach (int builders in new[] { 1, 4 })
            {
                Alike("two-bad.slangp", "shaders = 4\nshader0 = good.slang\nshader1 = worse.slang\nshader2 = bad.slang\nshader3 = good.slang\n", builders);
                Alike("bad-then-missing.slangp", "shaders = 3\nshader0 = bad.slang\nshader1 = not-there.slang\nshader2 = good.slang\n", builders);
                Alike("missing-then-bad.slangp", "shaders = 3\nshader0 = not-there.slang\nshader1 = bad.slang\nshader2 = good.slang\n", builders);
            }

            string identity = Identity();
            using var library = new SlangChain(_gpu!, SlangPreset.Load(identity));
            Assert.Equal("InvalidOperationException: No frame has been given to the chain yet.", Outcome(() => library.Render(8, 8)));
            Assert.Equal("InvalidOperationException: No frame has been given to the chain yet.", Outcome(() => library.RenderImage(8, 8).Dispose()));
            // Asked again, the library says the same again.
            Assert.Equal("InvalidOperationException: No frame has been given to the chain yet.", Outcome(() => library.RenderImage(8, 8).Dispose()));
            Assert.Equal("ArgumentOutOfRangeException", Outcome(() => library.Advance(new byte[4 * 3 * 4 - 1], 4, 3, 1)));
            Hold(golden, "after a frame too short", Frames(library, 4, 3, 1, 8, 6, 1));
            // More bytes than the frame's size says are not read; the frame is taken.
            byte[] longer = ShaderBench.Pattern(8, 8, 5);
            library.Advance(longer, 4, 3, 1);
            byte[] spare = library.Render(8, 6);
            library.Advance(longer[..(4 * 3 * 4)], 4, 3, 1);
            AssertSame(new[] { library.Render(8, 6) }, new[] { spare }, "a frame with bytes to spare");
            golden.Check(5 + 2 * 3 + 1);
        }

        [Fact]
        public void A_cache_is_asked_and_filled_as_it_was_and_serves_the_same_pictures()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Cache");
            string preset = Everything();
            using var cache = new SpirvCache(Path.Combine(_root, "cache.db"), SpirvCache.DefaultLimit, null);
            long clock = 1_800_000_000;
            cache.Clock = () => clock;
            (int, int, int) Counts(SpirvCache c) => (c.Hits, c.Misses, c.Stored);

            var pictures = new List<List<byte[]>>();
            foreach (int builders in new[] { 1, 4, 1 })
            {
                using (var chain = new SlangChain(_gpu!, SlangPreset.Load(preset), cache, builders)) pictures.Add(Frames(chain, 13, 7, 1, 60, 40, 2));
                golden.Add($"{builders} builders", $"{Counts(cache)}");
                clock += 100_000;
            }
            Assert.Equal((12, 6, 6), Counts(cache));
            Hold(golden, "through a cache", pictures[0]);
            foreach (List<byte[]> other in pictures.Skip(1)) AssertSame(pictures[0], other, "through a cache");

            // A build that fails part way has asked the cache for as much either way: in order up to the failure, side by side for every pass.
            Write("bad.slang", Plain + "void main() { FragColor = nothing_declared; }\n");
            string failing = Preset("failing.slangp", "shaders = 4\nshader0 = zero.slang\nshader1 = bad.slang\nshader2 = one.slang\nalias2 = Mid\nshader3 = two.slang\ntextures = \"Lut;Tile\"\nLut = lut.png\nTile = tile.png\n");
            foreach (int builders in new[] { 1, 4 })
            {
                Assert.Throws<SlangCompileException>(() => new SlangChain(_gpu!, SlangPreset.Load(failing), cache, builders));
                golden.Add($"failing with {builders} builders", $"{Counts(cache)}");
            }

            // A second cache reads the stages the first kept, given the one compiler's name for both.
            var shared = new SpirvCache(Path.Combine(_root, "shared.db"), SpirvCache.DefaultLimit, "one compiler");
            using (var chain = new SlangChain(_gpu!, SlangPreset.Load(preset), shared)) { }
            shared.Dispose();
            SqliteConnection.ClearAllPools();
            using var again = new SpirvCache(Path.Combine(_root, "shared.db"), SpirvCache.DefaultLimit, "one compiler");
            using (var chain = new SlangChain(_gpu!, SlangPreset.Load(preset), again)) AssertSame(pictures[0], Frames(chain, 13, 7, 1, 60, 40, 2), "from another cache's rows");
            Assert.Equal((6, 0, 0), Counts(again));
            golden.Check(3 + 2 + 2);
        }

        private static byte[] Pixels(SKImage image)
        {
            using SKPixmap pixmap = image.PeekPixels();
            return pixmap.GetPixelSpan().ToArray();
        }

        [Fact]
        public void Readbacks_are_lent_held_copied_and_given_back_as_they_were()
        {
            if (!Gpu) return;
            ParityGolden golden = Golden("Readbacks");
            string identity = Identity();
            var library = new SlangChain(_gpu!, SlangPreset.Load(identity));
            var held = new List<(SKImage Library, byte[] Pixels)>();
            var places = new List<nint>();

            void Draw(int k, int viewWidth, int viewHeight, bool keep)
            {
                byte[] frame = ShaderBench.Pattern(16, 12, k);
                library.Advance(frame, 16, 12, 1);
                SKImage b = library.RenderImage(viewWidth, viewHeight);
                byte[] pixels = Pixels(b);
                golden.Add($"picture {k}", $"{b.Width}x{b.Height} {b.AlphaType} {b.ColorType} {Digest(pixels)} readbacks {library.ReadbackCount} copies {library.CopiedImages}");
                using (SKPixmap pb = b.PeekPixels()) places.Add(pb.GetPixels());
                if (keep) held.Add((b, pixels));
                else b.Dispose();
            }

            // One let go each frame: one readback serves them all, the same memory every frame and not one made anew.
            for (int k = 0; k < 4; k++) Draw(k, 32, 24, keep: false);
            Assert.Equal((1, 0), (library.ReadbackCount, library.CopiedImages));
            Assert.Single(places.Distinct());

            // Held: a readback each up to the bound, then copies, and no held picture is written over.
            for (int k = 4; k < 4 + SlangChain.MaxLent + 2; k++) Draw(k, 32, 24, keep: true);
            Assert.Equal((SlangChain.MaxLent, 2), (library.ReadbackCount, library.CopiedImages));
            foreach (var (b, pixels) in held) Assert.True(pixels.AsSpan().SequenceEqual(Pixels(b)));

            // Given back on another thread, as Skia may: the readback is lent again, and none is made.
            var other = new Thread(() => held[1].Library.Dispose());
            other.Start();
            other.Join();
            held.RemoveAt(1);
            Draw(20, 32, 24, keep: true);
            Assert.Equal((SlangChain.MaxLent, 2), (library.ReadbackCount, library.CopiedImages));

            // A larger picture: a readback too small and not held is let go of, and a held one stays out until given back.
            held[0].Library.Dispose();
            held.RemoveAt(0);
            Draw(21, 64, 48, keep: true);
            Draw(22, 64, 48, keep: false);
            Draw(23, 8, 6, keep: false);
            foreach (var (b, pixels) in held) Assert.True(pixels.AsSpan().SequenceEqual(Pixels(b)));

            // The chain goes while pictures are held: each picture keeps its pixels until it lets go, and the device with them.
            library.Dispose();
            foreach (var (b, pixels) in held)
            {
                Assert.True(pixels.AsSpan().SequenceEqual(Pixels(b)));
                b.Dispose();
            }
            golden.Check(4 + SlangChain.MaxLent + 2 + 1 + 3);
        }

        // What a frame costs the collector on the thread that draws it: the frame taken and a picture made and let go, after a few to settle.
        private static long AllocatedAFrame(SlangChain chain, int frames = 200)
        {
            byte[] frame = ShaderBench.Pattern(64, 48, 1);
            for (int k = 0; k < 20; k++)
            {
                chain.Advance(frame, 64, 48, 1);
                chain.RenderImage(160, 120).Dispose();
            }
            long before = GC.GetAllocatedBytesForCurrentThread();
            for (int k = 0; k < frames; k++)
            {
                chain.Advance(frame, 64, 48, 1);
                chain.RenderImage(160, 120).Dispose();
            }
            return (GC.GetAllocatedBytesForCurrentThread() - before) / frames;
        }

        [Fact]
        public void A_frame_through_the_library_s_chain_allocates_the_picture_s_wrapper_and_nothing_for_its_passes()
        {
            if (!Gpu) return;
            var library = new Dictionary<string, long>();
            var presets = new List<(string Name, string Path)> { ("one pass", Identity()), ("three passes", Everything()) };
            if (Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable) is { Length: > 0 } pack)
                foreach (string relative in new[] { "crt/crt-royale.slangp", "bezel/Mega_Bezel/Presets/MBZ__5__POTATO.slangp" })
                    if (File.Exists(Path.Combine(pack, relative))) presets.Add((relative, Path.Combine(pack, relative)));
            foreach (var (name, path) in presets)
            {
                long libraryBytes;
                using (var chain = new SlangChain(_gpu!, SlangPreset.Load(path))) libraryBytes = AllocatedAFrame(chain);
                _output.WriteLine($"{name}: the library's chain {libraryBytes} bytes a frame");
                library[name] = libraryBytes;
            }
            // The same whatever the preset: what is allocated is the picture handed to Skia, not anything a pass needs.
            Assert.True(library.Values.Max() - library.Values.Min() <= 64, string.Join(", ", library.Select(pair => $"{pair.Key} {pair.Value}")));
            Assert.InRange(library.Values.Max(), 0, 1024);
        }

        [Fact]
        public void A_picture_held_past_its_chain_and_its_device_keeps_its_pixels()
        {
            SlangVulkan? device = SlangVulkan.TryCreate(out _, _wanted, null);
            if (device is null) return;
            var chain = new SlangChain(device, SlangPreset.Load(Identity()));
            chain.Advance(ShaderBench.Pattern(16, 12, 3), 16, 12, 1);
            byte[] copy = chain.Render(32, 24);
            SKImage picture = chain.RenderImage(32, 24);
            chain.Dispose();
            device.Dispose();
            GC.Collect();
            GC.WaitForPendingFinalizers();
            Assert.True(copy.AsSpan().SequenceEqual(Pixels(picture)));
            picture.Dispose();
        }

        // Each finding the layer hands the sink, for a device of either kind, or no device where the layer is not installed.
        private (SlangVulkan? Gpu, ConcurrentQueue<string> Findings) Validated()
        {
            var findings = new ConcurrentQueue<string>();
            return (SlangVulkan.TryCreate(out _, _wanted, findings.Enqueue), findings);
        }

        [Fact]
        public void The_library_s_chain_runs_clean_under_synchronisation_validation_and_the_layer_is_listening()
        {
            var (gpu, findings) = Validated();
            if (gpu is null) return;
            string preset = Everything(), identity = Identity();
            using (gpu)
            {
                using (var chain = new SlangChain(gpu, SlangPreset.Load(preset), null, 4))
                {
                    Frames(chain, 13, 7, 1, 97, 61, 4);
                    Frames(chain, 16, 6, 2, 64, 48, 3);
                    using (chain.RenderImage(64, 48)) { }
                    SKImage kept = chain.RenderImage(64, 48);
                    using (chain.RenderImage(32, 24)) { }
                    kept.Dispose();
                }
                using (var chain = new SlangChain(gpu, SlangPreset.Load(identity))) Frames(chain, 4, 4, 1, 8, 8, 2);
                gpu.Heard();
                foreach (string finding in findings.Take(20)) _output.WriteLine(finding);
                Assert.Empty(findings);

                // Its silence proves something only if it speaks: a buffer of no size is a finding it must report, on this device.
                var info = new BufferCreateInfo { SType = StructureType.BufferCreateInfo, Size = 0, Usage = BufferUsageFlags.TransferSrcBit, SharingMode = SharingMode.Exclusive };
                if (gpu.Vk.CreateBuffer(gpu.Device, &info, null, out Silk.NET.Vulkan.Buffer buffer) == Result.Success) gpu.Vk.DestroyBuffer(gpu.Device, buffer, null);
                Assert.Empty(findings);
                gpu.Heard();
                Assert.Contains(findings, f => f.Contains("VUID-VkBufferCreateInfo-size-00912", StringComparison.Ordinal));
            }
        }

        [Fact]
        public void The_layer_is_silent_for_the_work_and_speaks_of_the_faults_it_spoke_of()
        {
            var (library, libraryFindings) = Validated();
            using (library)
            {
                if (library is null) return;
                string preset = Everything();
                foreach (var (gpu, findings) in new[] { (library, libraryFindings) })
                {
                    using (var chain = new SlangChain(gpu, SlangPreset.Load(preset))) Frames(chain, 13, 7, 2, 40, 30, 3);
                    gpu.Heard();
                    Assert.Empty(findings);
                    var info = new BufferCreateInfo { SType = StructureType.BufferCreateInfo, Size = 0, Usage = BufferUsageFlags.TransferSrcBit, SharingMode = SharingMode.Exclusive };
                    if (gpu.Vk.CreateBuffer(gpu.Device, &info, null, out Silk.NET.Vulkan.Buffer buffer) == Result.Success) gpu.Vk.DestroyBuffer(gpu.Device, buffer, null);
                    ProvokeAHazard(gpu);
                    gpu.Heard();
                }
                static string[] Rules(IEnumerable<string> findings) => findings.SelectMany(f => System.Text.RegularExpressions.Regex.Matches(f, @"VUID-[\w-]+|[A-Z_]+ hazard").Select(rule => rule.Value)).Distinct().OrderBy(rule => rule, StringComparer.Ordinal).ToArray();
                _output.WriteLine(string.Join(", ", Rules(libraryFindings)));
                Assert.Contains("VUID-VkBufferCreateInfo-size-00912", Rules(libraryFindings));
                // Silence under synchronisation validation proves something only if that validation is on: a hazard it alone reports, whose words a listener is given without its name.
                Assert.Contains("WRITE_AFTER_WRITE hazard", Rules(libraryFindings));
            }
        }

        // Two fills of one buffer with no barrier between them, recorded and never submitted: a hazard synchronisation validation reports as it is recorded.
        internal static void ProvokeAHazard(SlangVulkan gpu)
        {
            Vk vk = gpu.Vk;
            Device device = gpu.Device;
            uint count = 0;
            vk.EnumeratePhysicalDevices(gpu.Instance, &count, null);
            var physicals = new PhysicalDevice[count];
            fixed (PhysicalDevice* p = physicals) vk.EnumeratePhysicalDevices(gpu.Instance, &count, p);
            PhysicalDevice physical = physicals.First(candidate =>
            {
                vk.GetPhysicalDeviceProperties(candidate, out PhysicalDeviceProperties properties);
                return System.Runtime.InteropServices.Marshal.PtrToStringAnsi((nint)properties.DeviceName) == gpu.Name;
            });
            uint families = 0;
            vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, null);
            var queues = new QueueFamilyProperties[families];
            fixed (QueueFamilyProperties* q = queues) vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, q);

            var bufferInfo = new BufferCreateInfo { SType = StructureType.BufferCreateInfo, Size = 256, Usage = BufferUsageFlags.TransferDstBit, SharingMode = SharingMode.Exclusive };
            Assert.Equal(Result.Success, vk.CreateBuffer(device, &bufferInfo, null, out Silk.NET.Vulkan.Buffer buffer));
            vk.GetBufferMemoryRequirements(device, buffer, out MemoryRequirements needs);
            var allocate = new MemoryAllocateInfo { SType = StructureType.MemoryAllocateInfo, AllocationSize = needs.Size, MemoryTypeIndex = (uint)System.Numerics.BitOperations.TrailingZeroCount(needs.MemoryTypeBits) };
            Assert.Equal(Result.Success, vk.AllocateMemory(device, &allocate, null, out DeviceMemory memory));
            Assert.Equal(Result.Success, vk.BindBufferMemory(device, buffer, memory, 0));
            var poolInfo = new CommandPoolCreateInfo { SType = StructureType.CommandPoolCreateInfo, QueueFamilyIndex = (uint)Array.FindIndex(queues, q => (q.QueueFlags & QueueFlags.GraphicsBit) != 0) };
            Assert.Equal(Result.Success, vk.CreateCommandPool(device, &poolInfo, null, out CommandPool pool));
            var commandsInfo = new CommandBufferAllocateInfo { SType = StructureType.CommandBufferAllocateInfo, CommandPool = pool, Level = CommandBufferLevel.Primary, CommandBufferCount = 1 };
            Assert.Equal(Result.Success, vk.AllocateCommandBuffers(device, &commandsInfo, out CommandBuffer commands));
            var begin = new CommandBufferBeginInfo { SType = StructureType.CommandBufferBeginInfo, Flags = CommandBufferUsageFlags.OneTimeSubmitBit };
            vk.BeginCommandBuffer(commands, &begin);
            vk.CmdFillBuffer(commands, buffer, 0, 256, 1);
            vk.CmdFillBuffer(commands, buffer, 0, 256, 2);
            vk.EndCommandBuffer(commands);
            vk.DestroyCommandPool(device, pool, null);
            vk.DestroyBuffer(device, buffer, null);
            vk.FreeMemory(device, memory, null);
        }

        [Fact]
        public void A_chain_that_fails_part_way_leaves_nothing_on_the_library_s_device()
        {
            Write("good.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            Write("bad.slang", Plain + "void main() { FragColor = nothing_declared; }\n");
            Write("deep.slang", Plain + "layout(set = 0, binding = 2) uniform sampler2D OriginalHistory2147483647;\nvoid main() { FragColor = texture(OriginalHistory2147483647, vTexCoord); }\n");
            File.WriteAllText(Path.Combine(_root, "not.png"), "this is not a picture");
            string[] presets =
            {
                Preset("undecoded.slangp", "shaders = 2\nshader0 = good.slang\nshader1 = good.slang\ntextures = \"Lut\"\nLut = not.png\n"),
                Preset("deep.slangp", "shaders = 2\nshader0 = good.slang\nshader1 = deep.slang\n"),
                Preset("last-bad.slangp", "shaders = 3\nshader0 = good.slang\nshader1 = good.slang\nshader2 = bad.slang\n"),
            };
            {
                var (gpu, findings) = Validated();
                if (gpu is null) return;
                using (gpu)
                    foreach (string preset in presets)
                        foreach (int builders in new[] { 1, 4 })
                            Assert.NotEqual("nothing", Outcome(() => { using var chain = new SlangChain(gpu, SlangPreset.Load(preset), null, builders); }));
                // The device is gone, and what the layer said of its going has been handed on.
                _output.WriteLine($"the library's device: {findings.Count} findings at its end");
                Assert.Empty(findings);
            }
        }

        // What the library's device tells a probe; what a C# chain of another test tells it meanwhile is not this case's.
        private sealed class Heard : ISlangProbe
        {
            public readonly List<(string Kind, int Point, double Value)> Marks = new();
            public void Phase(int point) { }
            public void Phase(int point, long at) { lock (Marks) Marks.Add(("phase", point, at)); }
            public void Gpu(int point, double milliseconds) { lock (Marks) Marks.Add(("gpu", point, milliseconds)); }
        }

        [Fact]
        public void A_probe_hears_the_library_s_device_time_each_phase_and_each_pass()
        {
            SlangVulkan? device = SlangVulkan.TryCreate(out _, _wanted, null);
            if (device is null) return;
            var heard = new Heard();
            using (device)
            using (var chain = new SlangChain(device, SlangPreset.Load(Everything())))
            {
                chain.Advance(ShaderBench.Pattern(13, 7, 0), 13, 7, 1);
                chain.Render(40, 30);
                long before = System.Diagnostics.Stopwatch.GetTimestamp();
                device.SetProbe(true);
                SlangProbe.Current = heard;
                try
                {
                    chain.Advance(ShaderBench.Pattern(13, 7, 1), 13, 7, 1);
                    // The frame's upload waits for the render's submission, so Advance submits nothing - see EmuSen_DevicePicture.md §13.1.
                    Assert.Equal(new[] { SlangProbe.AdvanceBegin, SlangProbe.AdvanceEnd }, heard.Marks.Where(m => m.Kind == "phase").Select(m => m.Point));
                    Assert.DoesNotContain(heard.Marks, m => m.Kind == "gpu");
                    heard.Marks.Clear();
                    chain.Render(40, 30);
                }
                finally
                {
                    SlangProbe.Current = null;
                    device.SetProbe(false);
                }
                long after = System.Diagnostics.Stopwatch.GetTimestamp();

                Assert.Equal(new[] { SlangProbe.RenderBegin, SlangProbe.Bound, SlangProbe.Submitted, SlangProbe.Waited, SlangProbe.Ran, SlangProbe.Copied }, heard.Marks.Where(m => m.Kind == "phase").Select(m => m.Point));
                Assert.Equal(new[] { SlangProbe.SubmissionBegin, 0, 1, 2, SlangProbe.SubmissionEnd }, heard.Marks.Where(m => m.Kind == "gpu").Select(m => m.Point));
                // The device's marks come before the wait is said to be over, as the bench counts submissions by it.
                Assert.True(heard.Marks.FindLastIndex(m => m.Kind == "gpu") < heard.Marks.FindIndex(m => m.Kind == "phase" && m.Point == SlangProbe.Waited));
                double[] phases = heard.Marks.Where(m => m.Kind == "phase").Select(m => m.Value).ToArray(), marks = heard.Marks.Where(m => m.Kind == "gpu").Select(m => m.Value).ToArray();
                Assert.Equal(phases.OrderBy(v => v), phases);
                Assert.Equal(marks.OrderBy(v => v), marks);
                Assert.All(phases, at => Assert.InRange(at, before, after));
                Assert.Equal(0.0, marks[0]);
                Assert.True(marks[1] > 0, "the first pass ended when the submission began");
                Assert.InRange(marks[^1], double.Epsilon, 1000);

                // Off again, nothing is kept and nothing told.
                heard.Marks.Clear();
                SlangProbe.Current = heard;
                chain.Advance(ShaderBench.Pattern(13, 7, 2), 13, 7, 1);
                chain.Render(40, 30);
                SlangProbe.Current = null;
                Assert.Empty(heard.Marks);
            }
        }

        [Theory]
        [InlineData("crt/crt-lottes.slangp", 256, 224, 1, 640, 480)]
        [InlineData("crt/crt-royale.slangp", 256, 224, 1, 640, 480)]
        [InlineData("crt/crt-royale.slangp", 320, 240, 2, 800, 600)]
        [InlineData("crt/crt-guest-advanced.slangp", 256, 224, 1, 640, 480)]
        [InlineData("crt/crt-geom.slangp", 256, 240, 1, 512, 448)]
        [InlineData("handheld/lcd-grid-v2.slangp", 160, 144, 1, 480, 432)]
        [InlineData("ntsc/ntsc-adaptive.slangp", 256, 224, 1, 640, 480)]
        [InlineData("bezel/Mega_Bezel/Presets/MBZ__5__POTATO.slangp", 256, 224, 1, 640, 360)]
        public void A_pack_preset_draws_a_picture_and_is_retuned(string relative, int width, int height, int repeat, int viewWidth, int viewHeight)
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (!Gpu || string.IsNullOrEmpty(pack)) return;
            string preset = Path.Combine(pack, relative);
            if (!File.Exists(preset)) return;
            using var library = new SlangChain(_gpu!, SlangPreset.Load(preset));
            _output.WriteLine($"{relative}: {library.Preset.Passes.Count} passes, {library.Preset.Textures.Count} lookup images, {library.Parameters.Count} parameters");
            AssertDrawn(Frames(library, width, height, repeat, viewWidth, viewHeight, 4)[^1]);

            // Retuned, every parameter half way to its largest, and drawn at another size.
            library.SetParameters(library.Parameters.ToDictionary(p => p.Id, p => (p.Initial + p.Maximum) / 2));
            Assert.Equal(2, Frames(library, width, height, repeat, viewWidth / 2 + 1, viewHeight / 2 + 1, 2, 4).Count);
        }

        [Fact]
        public void The_library_has_every_export_the_device_and_the_chain_call()
        {
            foreach (string export in new[] { "emusen_serenity_device_new", "emusen_serenity_chain_new", "emusen_serenity_chain_render_lent", "emusen_serenity_lent_release", "emusen_serenity_parse_format" })
                Assert.Contains(export, SerenityNative.Exports);
            Assert.Equal(SlangChain.MaxLent, (int)SerenityNative.ChainGet(0, SerenityNative.ChainMaxLent));
            Assert.Equal(48, sizeof(SerenityNative.Lent));
            Assert.Equal(16, sizeof(SerenityNative.Mark));
        }
    }
}
