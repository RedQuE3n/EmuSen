using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using Microsoft.Data.Sqlite;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Serenity's C# readers, compiler, reflection and cache and the platform library's, given the same files and bytes side by side and compared - see EmuSen_RustPlatform.md §16.5.
    public sealed unsafe class SerenityParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSerenityParity", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public SerenityParityTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            SqliteConnection.ClearAllPools();
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void AssertNone(List<string> different, string what) =>
            Assert.True(different.Count == 0, $"{different.Count} {what} differ:\n{string.Join("\n", different.Take(10))}");

        private static string Bits(float value) => BitConverter.SingleToUInt32Bits(value).ToString("X8");

        // What a call came to: its answer, or the exception it threw, with the words the C# chooses and the file or parameter it names.
        private static string Outcome(Func<string> call)
        {
            try { return call(); }
            catch (FileNotFoundException e) { return $"FileNotFoundException: {e.Message} [{e.FileName}]"; }
            catch (ArgumentOutOfRangeException) { return "ArgumentOutOfRangeException"; }
            catch (ArgumentException e) { return $"ArgumentException: {e.Message}"; }
            catch (IndexOutOfRangeException) { return "IndexOutOfRangeException"; }
            catch (UnauthorizedAccessException) { return "UnauthorizedAccessException"; }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void The_switch_is_what_the_variable_says()
        {
            Assert.Equal(Environment.GetEnvironmentVariable(SerenityNative.Variable) == "1", SerenityNative.Active);
        }

        [Fact]
        public void White_space_and_word_characters_are_dotnets_for_every_code_unit()
        {
            Regex space = new(@"\s"), word = new(@"\w");
            var different = new List<string>();
            for (int unit = 0; unit <= 0xFFFF; unit++)
            {
                string one = ((char)unit).ToString();
                bool isSpace = SerenityNative.TextClass(SerenityNative.ClassSpace, (uint)unit) != 0, isWord = SerenityNative.TextClass(SerenityNative.ClassWord, (uint)unit) != 0;
                if (space.IsMatch(one) != isSpace || char.IsWhiteSpace((char)unit) != isSpace) different.Add($"space U+{unit:X4}");
                if (word.IsMatch(one) != isWord) different.Add($"word U+{unit:X4}");
            }
            AssertNone(different, "code units");
            // Past the first plane a character is two code units, neither of which .NET takes for a word character or a space.
            Assert.Equal((0, 0), (SerenityNative.TextClass(SerenityNative.ClassWord, 0x1D400), SerenityNative.TextClass(SerenityNative.ClassSpace, 0x1F600)));
        }

        private static readonly string[] NumberPieces =
        {
            "0", "1", "9", "12", "007", "2147483647", "2147483648", "4294967296", "99999999999999999999", ".", "..", "e", "E", "e5", "E-3", "e+", "+", "-", "++", " ", "\t", "\n", "\v", "\f", "\r", "\0",
            "\u00A0", "\u2003", "\u0085", "Infinity", "infinity", "INFINITY", "NaN", "nan", "inf", "\u221E", ",", "_", "x", "f", "\u0661", "\u0131", "1e38", "3.4028235e38", "3.4028236e38", "1e-45", "7e-46", "1.17549435e-38",
            "0.1", "16777217", "8388608.5", "8388609.5", "1e-400", "1e400", "123456789012345678901234567890", "0.000000000000000000000000000000000000000000001",
        };

        [Fact]
        public void A_number_is_parsed_as_dotnet_parses_it()
        {
            var random = new Random(41);
            var different = new List<string>();
            var texts = new List<string>(NumberPieces);
            for (int i = 0; i < 150_000; i++) texts.Add(string.Concat(Enumerable.Range(0, random.Next(1, 6)).Select(_ => NumberPieces[random.Next(NumberPieces.Length)])));
            for (int i = 0; i < 50_000; i++) texts.Add((random.NextDouble() * Math.Pow(10, random.Next(-50, 50)) * (random.Next(2) == 0 ? 1 : -1)).ToString(random.Next(2) == 0 ? "R" : "E12", CultureInfo.InvariantCulture));
            foreach (string text in texts)
            {
                byte[] bytes = Encoding.UTF8.GetBytes(text);
                uint value;
                bool managedFloat = float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float f), managedInt = int.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out int n);
                int nativeFloat, nativeInt;
                uint floatBits, intBits = 0;
                fixed (byte* b = bytes)
                {
                    nativeFloat = SerenityNative.ParseNumber(SerenityNative.NumberFloat, SerenityNative.Pin(b), (nuint)bytes.Length, &value);
                    floatBits = value;
                    nativeInt = SerenityNative.ParseNumber(SerenityNative.NumberInteger, SerenityNative.Pin(b), (nuint)bytes.Length, &value);
                    if (nativeInt == 1) intBits = value;
                }
                string shown = string.Concat(text.Select(c => c is > ' ' and < (char)127 ? c.ToString() : $"\\u{(int)c:X4}"));
                if (managedFloat != (nativeFloat == 1) || (managedFloat && BitConverter.SingleToUInt32Bits(f) != floatBits)) different.Add($"float [{shown}]: {managedFloat} {Bits(f)} and {nativeFloat} {floatBits:X8}");
                if (managedInt != (nativeInt == 1) || (managedInt && (uint)n != intBits)) different.Add($"int [{shown}]: {managedInt} {n} and {nativeInt} {(int)intBits}");
            }
            AssertNone(different, "numbers");
        }

        private static string Dump(SlangPreset preset) =>
            $"{preset.Path}\n" + string.Join("\n", preset.Passes.Select(p => $"pass {p.ShaderPath}|{p.Alias ?? "<null>"}|{p.FilterLinear}|{p.Wrap}|{p.ScaleTypeX}|{p.ScaleTypeY}|{Bits(p.ScaleX)}|{Bits(p.ScaleY)}|{p.FloatFramebuffer}|{p.SrgbFramebuffer}|{p.MipmapInput}|{p.FrameCountMod}"))
            + "\n" + string.Join("\n", preset.Textures.Select(t => $"texture {t.Name}|{t.Path}|{t.Linear}|{t.Wrap}|{t.Mipmap}"))
            + "\n" + string.Join("\n", preset.Parameters.Select(p => $"parameter {p.Key}={Bits(p.Value)}"));

        private static string Dump(IEnumerable<SlangParameter> parameters) =>
            string.Join("\n", parameters.Select(p => $"{p.Id}|{p.Description}|{Bits(p.Initial)}|{Bits(p.Minimum)}|{Bits(p.Maximum)}|{Bits(p.Step)}|{(p.Choices is null ? "" : string.Join(",", p.Choices))}"));

        private static string Dump(SlangSource source) =>
            $"{source.Path}\nname {source.PassName ?? "<null>"} format {source.FramebufferFormat ?? "<null>"}\n{Dump(source.Parameters)}\n--vertex--\n{source.Vertex}--fragment--\n{source.Fragment}";

        // The awkward ways a line can be spaced, quoted, commented and ended, drawn from what RetroArch's presets and people's hands write.
        private static readonly string[] Spaces = { "", " ", "  ", "\t", " \t ", "\u00A0", "\u2003", "\u3000", "\u2028", "\u0085", "\u001F" };
        private static readonly string[] Values =
        {
            "true", "TRUE", "True", "1", "0", "false", "yes", "2", "2.0", "0.5", ".5", "1e2", "NaN", "Infinity", "-infinity", "source", "SOURCE", "Viewport", "absolute", "absolut", "v\u0130ewport", "\u212Aelvin",
            "clamp_to_edge", "CLAMP_TO_EDGE", "repeat", "mirrored_repeat", "clamp_to_border", "nonsense", "", "a b", "a#b", "a;b", "shader.slang", "sub/shader.slang", "../up.slang", "./shader.slang",
            "/rooted/shader.slang", "missing.slang", "x\0y", "\u65E5\u672C.slang", "LUT;Mask", "LUT; Mask ;;", "lut.png", "-3", "99999999999", " 3 ", "3 # three", "0x10", "+2", "1\0",
        };

        private static readonly string[] Booleans = { "true", "TRUE", "True", "tRuE", "1", "false", "0", "yes", "T\u0280ue", "true1" };
        private static readonly string[] Wraps = { "repeat", "REPEAT", "Repeat", "mirrored_repeat", "Mirrored_Repeat", "clamp_to_edge", "CLAMP_TO_EDGE", "clamp_to_border", "Clamp_To_Border", "m\u0130rrored_repeat" };
        private static readonly string[] ScaleTypes = { "source", "Source", "SOURCE", "viewport", "VIEWPORT", "ViewPort", "absolute", "Absolute", "ABSOLUTE", "relative" };
        private static readonly string[] Numbers = { "2", "2.5", "0.5", "-1.5", "3.5", "1e1", "x", "NaN", "7.5", "-0.5", "1.49", "2147483648.5", "4", "a\u00A0b", "3\u2003#c", "2\u3000" };

        private static string Line(Random random, int passes)
        {
            string[] keys =
            {
                "shaders", "textures", "parameters", "shader", "alias", "filter_linear", "wrap_mode", "scale_type", "scale_type_x", "scale_type_y", "scale", "scale_x", "scale_y", "float_framebuffer",
                "srgb_framebuffer", "mipmap_input", "frame_count_mod", "rgb10_framebuffer", "feedback_pass", "LUT", "LUT_linear", "LUT_wrap_mode", "LUT_mipmap", "Mask", "Mask_linear", "GAIN", "gamma.value", "a-b",
                "shader_extra", "scale_type_z", "k\u00E9y", "",
            };
            string key = keys[random.Next(keys.Length)];
            if (random.Next(3) != 0 && Array.IndexOf(keys, key) is >= 3 and <= 18) key += random.Next(4) == 0 ? "" : random.Next(-1, passes + 1).ToString(CultureInfo.InvariantCulture).Replace("-1", "x");
            string value = Values[random.Next(Values.Length)];
            // More often than not a key is given a value of its own kind, in a case and a form of its own, so that the rule for that kind is the one asked.
            if (random.Next(5) < 3)
            {
                string[]? kind = key.Contains("linear") || key.Contains("framebuffer") || key.Contains("mipmap") ? Booleans
                    : key.Contains("wrap_mode") ? Wraps : key.StartsWith("scale_type", StringComparison.Ordinal) ? ScaleTypes
                    : key.StartsWith("scale", StringComparison.Ordinal) || key.StartsWith("frame_count_mod", StringComparison.Ordinal) ? Numbers : null;
                if (kind is not null) value = kind[random.Next(kind.Length)];
            }
            if (key == "shaders" && random.Next(3) != 0) value = passes.ToString(CultureInfo.InvariantCulture);
            if (key.StartsWith("shader", StringComparison.Ordinal) && key.Length > 6 && random.Next(3) != 0) value = "shader.slang";
            string quoted = random.Next(5) switch { 0 => $"\"{value}\"", 1 => $"\"{value}", 2 => $"\"{value}\" # note", 3 => value + " # note", _ => value };
            string Space() => Spaces[random.Next(Spaces.Length)];
            return random.Next(12) switch
            {
                0 => $"{Space()}#{Space()}comment {key} = {value}",
                1 => $"{Space()}{key}{Space()}{quoted}",
                2 => $"{Space()}={Space()}{quoted}",
                3 => "",
                _ => $"{Space()}{key}{Space()}={Space()}{quoted}{Space()}",
            };
        }

        private static string ReferenceLine(Random random, string target)
        {
            string Space() => Spaces[random.Next(Spaces.Length)];
            return random.Next(9) switch
            {
                0 => $"#reference {target}",
                1 => $"{Space()}#reference{Space()}\"{target}\"{Space()}",
                2 => $"#reference \"{target}",
                3 => $"#reference {target}\" trailing",
                4 => $"#reference{Space()}{Space()}",
                5 => $"#reference{Space()}\"\"{target}",
                6 => $"# reference {target}",
                7 => $"#reference  \"",
                _ => $"#reference{Space()}{target}{Space()}",
            };
        }

        [Fact]
        public void A_preset_is_read_the_same_through_every_spacing_quoting_reference_and_mistake()
        {
            var random = new Random(42);
            var different = new List<string>();
            string[] endings = { "\n", "\r\n", "\r" };
            int read = 0, refused = 0;
            for (int run = 0; run < 2500; run++)
            {
                string folder = Path.Combine(_root, "preset", run.ToString(CultureInfo.InvariantCulture));
                Directory.CreateDirectory(Path.Combine(folder, "sub"));
                File.WriteAllText(Path.Combine(folder, "shader.slang"), "#pragma stage vertex\nvoid main() {}\n");
                int files = random.Next(1, 4);
                for (int file = files - 1; file >= 0; file--)
                {
                    int passes = random.Next(0, 4);
                    var lines = new List<string>();
                    if (random.Next(6) != 0) lines.Add($"shaders = {passes}");
                    for (int p = 0; p < passes; p++) if (random.Next(8) != 0) lines.Add($"shader{p} = {(random.Next(6) == 0 ? "sub/../shader.slang" : "shader.slang")}");
                    for (int extra = random.Next(0, 14); extra > 0; extra--) lines.Insert(random.Next(lines.Count + 1), Line(random, passes));
                    string[] targets = { $"p{file + 1}.slangp", $"sub/p{file + 1}.slangp", "missing.slangp", $"p{file}.slangp", "p0.slangp", "", "sub", "x\0y" };
                    if (file < files - 1 || random.Next(4) == 0)
                        for (int references = random.Next(0, 3); references > 0; references--) lines.Insert(random.Next(lines.Count + 1), ReferenceLine(random, targets[random.Next(file < files - 1 ? 2 : targets.Length)]));
                    if (random.Next(10) == 0) lines.Insert(random.Next(lines.Count + 1), ReferenceLine(random, targets[random.Next(targets.Length)]));
                    string ending = endings[random.Next(endings.Length)];
                    byte[] text = Encoding.UTF8.GetBytes(string.Join(ending, lines) + (random.Next(3) == 0 ? "" : ending));
                    if (random.Next(12) == 0) text = Encoding.UTF8.GetPreamble().Concat(text).ToArray();
                    if (random.Next(40) == 0) text = Encoding.Unicode.GetPreamble().Concat(Encoding.Unicode.GetBytes(Encoding.UTF8.GetString(text))).ToArray();
                    if (random.Next(40) == 0) text = text.Concat(new byte[] { 0xFF, 0xC3, 0x28 }).ToArray();
                    foreach (string where in new[] { folder, Path.Combine(folder, "sub") }) File.WriteAllBytes(Path.Combine(where, $"p{file}.slangp"), text);
                }
                string path = random.Next(8) switch { 0 => Path.Combine(folder, "sub", "..", "p0.slangp"), 1 => Path.Combine(folder, "missing.slangp"), 2 => folder, _ => Path.Combine(folder, "p0.slangp") };
                string managed = Outcome(() => Dump(SlangPreset.Managed.Load(path))), native = Outcome(() => Dump(SlangPreset.LoadNative(path)));
                if (managed != native) different.Add($"run {run} {path}:\n{managed}\n--\n{native}");
                if (managed.Contains("Exception")) refused++; else read++;
                if (!managed.Contains("Exception"))
                {
                    string managedRead = Outcome(() => Dump(SlangParameters.Managed.Merge(SlangPreset.Managed.Load(path).Passes.Select(p => SlangSource.Managed.Load(p.ShaderPath)), SlangPreset.Managed.Load(path).Parameters)));
                    string nativeRead = Outcome(() => Dump(SlangParameters.MergeNative(SlangPreset.LoadNative(path).Passes.Select(p => SlangSource.LoadNative(p.ShaderPath)), SlangPreset.LoadNative(path).Parameters)));
                    if (managedRead != nativeRead) different.Add($"run {run} {path} parameters:\n{managedRead}\n--\n{nativeRead}");
                }
            }
            _output.WriteLine($"{read} presets read and {refused} refused, alike");
            Assert.True(read > 300 && refused > 300, $"the corpus is lopsided: {read} read, {refused} refused");
            AssertNone(different, "presets");
            foreach (string path in new[] { "", "x\0y", " ", "relative/missing.slangp" })
                Assert.Equal(Outcome(() => Dump(SlangPreset.Managed.Load(path))), Outcome(() => Dump(SlangPreset.LoadNative(path))));

            // The rules a random preset seldom asks all at once, asked in one: a scale that is no number does not fall back to the pass's, a count's fraction is cut off, and words are taken in any case.
            string rules = Path.Combine(_root, "rules.slangp");
            File.WriteAllText(rules, string.Join("\n", "shaders = 2", "shader0 = shader.slang", "shader1 = shader.slang", "scale_x0 = x", "scale0 = 3", "scale_y0 = 2.5", "frame_count_mod0 = 2.5",
                "frame_count_mod1 = 3.5", "wrap_mode0 = REPEAT", "wrap_mode1 = Mirrored_Repeat", "filter_linear0 = TRUE", "float_framebuffer1 = True", "scale_type0 = ABSOLUTE", "scale_type_x1 = Source",
                "textures = \"LUT; Mask\"", "LUT = lut.png", "LUT_linear = TRUE", "LUT_wrap_mode = Clamp_To_Edge", "Mask = sub/mask.png", "Mask_mipmap = 1", "GAIN = 1.5", "odd = 2\u00A0# kept whole, so no number"));
            SlangPreset ruled = SlangPreset.Managed.Load(rules);
            Assert.Equal((1f, 2.5f, 2, 3), (ruled.Passes[0].ScaleX, ruled.Passes[0].ScaleY, ruled.Passes[0].FrameCountMod, ruled.Passes[1].FrameCountMod));
            Assert.Equal((SlangWrap.Repeat, SlangWrap.MirroredRepeat, true, true), (ruled.Passes[0].Wrap, ruled.Passes[1].Wrap, ruled.Passes[0].FilterLinear, ruled.Passes[1].FloatFramebuffer));
            Assert.Equal((SlangScaleType.Absolute, SlangScaleType.Source, SlangScaleType.Viewport), (ruled.Passes[0].ScaleTypeX, ruled.Passes[1].ScaleTypeX, ruled.Passes[1].ScaleTypeY));
            Assert.Equal((true, SlangWrap.ClampToEdge, true), (ruled.Textures[0].Linear, ruled.Textures[0].Wrap, ruled.Textures[1].Mipmap));
            Assert.Equal(new[] { "GAIN" }, ruled.Parameters.Keys);
            Assert.Equal(Dump(ruled), Dump(SlangPreset.LoadNative(rules)));

            // A chain of references sixteen deep is read and one of seventeen is taken to be a loop; of includes, thirty-two and thirty-three.
            string deep = Path.Combine(_root, "deep");
            Directory.CreateDirectory(deep);
            File.WriteAllText(Path.Combine(deep, "shader.slang"), "#pragma stage vertex\n");
            for (int n = 0; n <= 17; n++) File.WriteAllText(Path.Combine(deep, $"r{n}.slangp"), n == 17 ? "shaders = 1\nshader0 = shader.slang\n" : $"#reference r{n + 1}.slangp\n");
            for (int n = 0; n <= 33; n++) File.WriteAllText(Path.Combine(deep, $"i{n}.inc"), n == 33 ? "#pragma stage vertex\n" : $"#include \"i{n + 1}.inc\"\n");
            foreach ((string file, bool reads) in new[] { ("r1.slangp", true), ("r0.slangp", false) })
            {
                string managed = Outcome(() => Dump(SlangPreset.Managed.Load(Path.Combine(deep, file))));
                Assert.Equal(reads, !managed.Contains("taken to be a loop"));
                Assert.Equal(managed, Outcome(() => Dump(SlangPreset.LoadNative(Path.Combine(deep, file)))));
            }
            foreach ((string file, bool reads) in new[] { ("i1.inc", true), ("i0.inc", false) })
            {
                string managed = Outcome(() => Dump(SlangSource.Managed.Load(Path.Combine(deep, file))));
                Assert.Equal(reads, !managed.Contains("taken to be a loop"));
                Assert.Equal(managed, Outcome(() => Dump(SlangSource.LoadNative(Path.Combine(deep, file)))));
            }

            // A file that is there and may not be read, as a preset and as a shader's source.
            if (!OperatingSystem.IsWindows() && Environment.UserName != "root")
            {
                string closed = Path.Combine(_root, "closed.slangp");
                File.WriteAllText(closed, "shaders = 1\nshader0 = shader.slang\n");
                File.SetUnixFileMode(closed, UnixFileMode.None);
                Assert.Equal("UnauthorizedAccessException", Outcome(() => Dump(SlangPreset.Managed.Load(closed))));
                Assert.Equal("UnauthorizedAccessException", Outcome(() => Dump(SlangPreset.LoadNative(closed))));
                Assert.Equal("UnauthorizedAccessException", Outcome(() => Dump(SlangSource.Managed.Load(closed))));
                Assert.Equal("UnauthorizedAccessException", Outcome(() => Dump(SlangSource.LoadNative(closed))));
                File.SetUnixFileMode(closed, UnixFileMode.UserRead | UnixFileMode.UserWrite);
            }
        }

        private static string SourceLine(Random random, int depth)
        {
            string Space() => Spaces[random.Next(8)];
            string Gap() => Spaces[random.Next(1, 8)];
            string[] numbers = { "0", "1", "0.5", "-1", "1e2", "NaN", "x", "Infinity", "2.5f", "", "+3", ".25" };
            string Number() => numbers[random.Next(numbers.Length)];
            return random.Next(30) switch
            {
                0 => $"{Space()}#{Space()}pragma{Gap()}stage{Gap()}vertex",
                1 => $"{Space()}#{Space()}pragma{Gap()}stage{Gap()}fragment",
                2 => $"#pragma stage vertex{new[] { "es", "_x", "\u00E9", "-x", " x", "\u0301", "\u0663", "\U0001D400" }[random.Next(8)]}",
                3 => "#pragma stage Vertex",
                4 => $"#pragma{Gap()}name{Gap()}Pass{random.Next(3)} trailing",
                5 => $"#pragma name{Space()}",
                6 => $"#pragma{Gap()}format{Gap()}R8G8B8A8_{new[] { "UNORM", "SRGB", "\u65E5" }[random.Next(3)]}",
                7 or 8 or 9 => $"#pragma{Gap()}parameter{Gap()}P{random.Next(6)}{Gap()}\"Label {random.Next(9)}\"{Gap()}{Number()}{Gap()}{Number()}{Gap()}{Number()}{(random.Next(2) == 0 ? Gap() + Number() : "")}{(random.Next(4) == 0 ? " extra" : "")}",
                10 => $"#pragma parameter P{random.Next(6)} \"unterminated {Number()} {Number()} {Number()}",
                11 => $"#pragma parameter \"quoted\"{Gap()}\"\"{Gap()}1 2 3",
                12 => $"#pragma parameter P{random.Next(6)}\"glued\" 1 2 3",
                13 => $"#pragma parameter P{random.Next(6)} \"few\" 1 2",
                14 when depth < 3 => $"{Space()}#{Space()}include{Gap()}\"inc{depth + 1}_{random.Next(2)}.inc\"",
                15 when depth < 3 => $"#include \"sub/../inc{depth + 1}_{random.Next(2)}.inc\" // again",
                16 => "#include \"missing.inc\"",
                17 => $"#pragma{Gap()}include_optional{Gap()}\"{(random.Next(2) == 0 ? "missing.inc" : $"inc{depth + 1}_0.inc")}\"",
                18 => "#include <system.inc>",
                19 => "#include \"\"",
                20 => "#include \"x\0y\"",
                21 => "#pragmastage vertex",
                22 => "// #pragma stage fragment",
                23 => "",
                _ => $"    float v{random.Next(99)} = {random.Next(9)}.0; // {Space()}\u65E5\u672C",
            };
        }

        [Fact]
        public void A_source_is_expanded_and_split_the_same_through_every_include_and_pragma()
        {
            var random = new Random(43);
            var different = new List<string>();
            string[] endings = { "\n", "\r\n", "\r" };
            int read = 0, refused = 0;
            for (int run = 0; run < 2500; run++)
            {
                string folder = Path.Combine(_root, "source", run.ToString(CultureInfo.InvariantCulture));
                Directory.CreateDirectory(Path.Combine(folder, "sub"));
                void Write(string name, int depth)
                {
                    var lines = Enumerable.Range(0, random.Next(0, 16)).Select(_ => SourceLine(random, depth)).ToList();
                    string ending = endings[random.Next(endings.Length)];
                    byte[] text = Encoding.UTF8.GetBytes(string.Join(ending, lines) + (random.Next(3) == 0 ? "" : ending));
                    if (random.Next(15) == 0) text = Encoding.UTF8.GetPreamble().Concat(text).ToArray();
                    if (random.Next(40) == 0) text = text.Concat(new byte[] { 0xE2, 0x82 }).ToArray();
                    File.WriteAllBytes(Path.Combine(folder, name), text);
                }
                Write("top.slang", 0);
                for (int depth = 1; depth <= 3; depth++)
                    for (int n = 0; n < 2; n++)
                        if (random.Next(5) != 0) Write($"inc{depth}_{n}.inc", depth);
                if (random.Next(30) == 0) File.WriteAllText(Path.Combine(folder, "inc1_0.inc"), "#include \"inc1_0.inc\"\n");
                string path = random.Next(10) switch { 0 => Path.Combine(folder, "missing.slang"), 1 => folder, _ => Path.Combine(folder, "top.slang") };
                string managed = Outcome(() => Dump(SlangSource.Managed.Load(path))), native = Outcome(() => Dump(SlangSource.LoadNative(path)));
                if (managed != native) different.Add($"run {run} {path}:\n{managed}\n--\n{native}");
                if (managed.Contains("Exception")) refused++; else read++;
            }
            _output.WriteLine($"{read} sources read and {refused} refused, alike");
            Assert.True(read > 300 && refused > 300, $"the corpus is lopsided: {read} read, {refused} refused");
            AssertNone(different, "sources");
        }

        [Fact]
        public void Parameters_merge_the_same_and_keep_what_the_csharp_objects_carry()
        {
            var random = new Random(44);
            var different = new List<string>();
            float[] numbers = { 0, 1, -1, 0.5f, float.NaN, float.PositiveInfinity, float.NegativeInfinity, -0f, float.Epsilon, float.MaxValue };
            string folder = Path.Combine(_root, "merge");
            Directory.CreateDirectory(folder);
            File.WriteAllText(Path.Combine(folder, "plain.slang"), "#pragma stage vertex\n#pragma parameter A \"a\" 1 0 2 0.5\n#pragma parameter B \"b\" 0 0 0\n");
            SlangSource plain = SlangSource.Managed.Load(Path.Combine(folder, "plain.slang"));
            for (int run = 0; run < 3000; run++)
            {
                var sources = Enumerable.Range(0, random.Next(0, 5)).Select(_ => (IReadOnlyList<SlangParameter>)Enumerable.Range(0, random.Next(0, 6)).Select(_ =>
                    new SlangParameter($"P{random.Next(7)}", new[] { "Label", "", "\u65E5\u672C \"q\"", "tab\there" }[random.Next(4)], numbers[random.Next(numbers.Length)], numbers[random.Next(numbers.Length)],
                        numbers[random.Next(numbers.Length)], numbers[random.Next(numbers.Length)]) { Choices = random.Next(4) == 0 ? new[] { "Off", "On" } : null }).ToList()).ToList();
                var overrides = Enumerable.Range(0, random.Next(0, 5)).Select(_ => random.Next(9)).Distinct().ToDictionary(n => $"P{n}", _ => numbers[random.Next(numbers.Length)]);
                string managed = Dump(SlangParameters.Managed.Merge(sources, overrides));
                string native = Dump(SlangParameters.MergeNativeLists(sources, overrides));
                if (managed != native) different.Add($"run {run}:\n{managed}\n--\n{native}");
            }
            AssertNone(different, "merges");
            Assert.Equal(Dump(SlangParameters.Managed.Merge(new[] { plain, plain }, new Dictionary<string, float> { ["B"] = 3 })), Dump(SlangParameters.MergeNative(new[] { plain, plain }, new Dictionary<string, float> { ["B"] = 3 })));
        }

        private const string Head = "#version 450\nlayout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; vec4 OutputSize; float GAIN; } global;\nlayout(push_constant) uniform Push { vec4 SourceSize; uint FrameCount; mat3 Twist; float Table[4]; } params;\n";

        // Stages that compile and stages that do not, with names and text outside ASCII.
        private static IEnumerable<(string Text, SlangStage Stage, string Name)> Stages()
        {
            string vertex = Head + "layout(location = 0) in vec4 Position;\nlayout(location = 1) in vec2 TexCoord;\nlayout(location = 0) out vec2 vTexCoord;\nvoid main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }\n";
            string fragment = Head + "layout(location = 0) in vec2 vTexCoord;\nlayout(location = 1) in vec2 unread;\nlayout(location = 0) out vec4 FragColor;\nlayout(set = 0, binding = 2) uniform sampler2D Source;\nlayout(set = 0, binding = 3) uniform sampler2D Original;\n";
            yield return (vertex, SlangStage.Vertex, "plain.slang");
            yield return (vertex, SlangStage.Fragment, "plain.slang");
            yield return (fragment + "void main() { FragColor = texture(Source, vTexCoord) * global.GAIN + texture(Original, vTexCoord) * params.SourceSize.x; }\n", SlangStage.Fragment, "na\u00EFve \u65E5\u672C.slang");
            yield return (fragment + "void main() { FragColor = vec4(params.Table[int(params.FrameCount) % 4]) * params.Twist[1].xyzz; }\n", SlangStage.Fragment, "/abs/path with spaces/f.slang");
            yield return (fragment + "void main() { FragColor = oops; }\n", SlangStage.Fragment, "broken \u00E9.slang");
            yield return ("#version 450\nvoid main() { /* \u65E5\u672C\u8A9E */ }\n", SlangStage.Vertex, "comment.slang");
            yield return ("", SlangStage.Vertex, "empty.slang");
            yield return ("#version 450\nvoid main() {}\n", SlangStage.Vertex, "");
            yield return ("#version 450\n#error stop here\nvoid main() {}\n", SlangStage.Fragment, "cut\0off.slang");
            yield return ("not glsl at all {{{", SlangStage.Fragment, "nonsense.slang");
            // Blocks at another binding and of another size than the rest declare, so that merging two stages has a choice to make.
            string other = "#version 450\nlayout(std140, set = 0, binding = 1) uniform UBO { mat4 MVP; vec4 OutputSize; float GAIN; vec4 Extra[3]; } global;\nlayout(push_constant) uniform Push { vec4 SourceSize; } params;\n";
            yield return (other + "layout(location = 0) out vec4 FragColor;\nlayout(set = 0, binding = 4) uniform sampler2D Source;\nvoid main() { FragColor = global.Extra[2] * params.SourceSize + texture(Source, vec2(0.5)); }\n", SlangStage.Fragment, "other.slang");
            yield return (other + "layout(location = 0) in vec4 Position;\nvoid main() { gl_Position = global.MVP * Position * params.SourceSize.x; }\n", SlangStage.Vertex, "other.slang");
            for (int n = 0; n < 12; n++)
                yield return (fragment + $"const float K = {n}.5;\nvoid main() {{ FragColor = vec4(K) + texture(Source, vTexCoord{(n % 3 == 0 ? " + unread" : "")}); }}\n", SlangStage.Fragment, $"k{n}.slang");
        }

        [Fact]
        public void Each_stage_compiles_to_the_same_bytes_or_fails_with_the_same_words()
        {
            var different = new List<string>();
            int compiled = 0;
            foreach ((string text, SlangStage stage, string name) in Stages())
            {
                byte[]? managedBytes = null, nativeBytes = null;
                string managed = Outcome(() => Convert.ToHexString(managedBytes = SlangCompiler.Managed.Compile(text, stage, name)));
                string native = Outcome(() => Convert.ToHexString(nativeBytes = SlangCompiler.CompileNative(text, stage, name)));
                if (managed != native) different.Add($"{name} {stage}:\n{managed[..Math.Min(300, managed.Length)]}\n--\n{native[..Math.Min(300, native.Length)]}");
                if (managedBytes is not null) compiled++;
            }
            Assert.True(compiled >= 16, $"only {compiled} stages compiled");
            AssertNone(different, "stages");
            Assert.Contains("SlangCompileException: broken \u00E9.slang (Fragment) does not compile:\nbroken \u00E9.slang:", Outcome(() => Convert.ToHexString(SlangCompiler.CompileNative(Stages().ElementAt(4).Text, SlangStage.Fragment, "broken \u00E9.slang"))));
        }

        private static string Dump(SpirvReflection reflection)
        {
            static string Block(SlangBlock? b) => b is null ? "<null>" : $"{b.Binding}/{b.Size}[{string.Join(",", b.Members.Select(m => $"{m.Name}@{m.Offset}+{m.Size}"))}]";
            return $"uniforms {Block(reflection.Uniforms)} push {Block(reflection.PushConstants)} samplers {string.Join(",", reflection.Samplers.Select(s => $"{s.Name}={s.Binding}"))}";
        }

        private static byte[] Module(params uint[][] instructions)
        {
            var words = new List<uint> { 0x07230203, 0x00010000, 0, 100, 0 };
            foreach (uint[] instruction in instructions)
            {
                words.Add((uint)instruction.Length << 16 | instruction[0]);
                words.AddRange(instruction.Skip(1));
            }
            return words.SelectMany(BitConverter.GetBytes).ToArray();
        }

        [Fact]
        public void A_module_is_reflected_pruned_and_merged_the_same_and_one_that_is_not_well_formed_fails_the_same()
        {
            var random = new Random(45);
            var different = new List<string>();
            var modules = Stages().Select(s => { try { return SlangCompiler.Managed.Compile(s.Text, s.Stage, s.Name); } catch (SlangCompileException) { return null; } }).Where(m => m is not null).Select(m => m!).ToList();
            var cases = new List<byte[]>(modules)
            {
                Array.Empty<byte>(), new byte[16], new byte[20], new byte[21], Module(), Module(new uint[] { 5 }), Module(new uint[] { 6, 1 }), Module(new uint[] { 71, 1 }), Module(new uint[] { 72, 1, 2 }),
                Module(new uint[] { 59, 1, 2 }), Module(new uint[] { 21 }), Module(new uint[] { 43, 1, 2 }), Module(new uint[] { 32, 9, 2 }, new uint[] { 59, 9, 5, 2 }),
                Module(new uint[] { 22, 1 }, new uint[] { 30, 2, 1 }, new uint[] { 59, 2, 7, 2 }), Module(new uint[] { 23, 3, 1 }, new uint[] { 22, 1, 32 }, new uint[] { 30, 2, 3 }, new uint[] { 59, 2, 7, 9 }),
                Module(new uint[] { 28, 4, 1 }, new uint[] { 30, 2, 4 }, new uint[] { 59, 2, 7, 2 }), Module(new uint[] { 24, 5, 1 }, new uint[] { 30, 2, 5 }, new uint[] { 59, 2, 7, 2 }),
                Module(new uint[] { 15, 4, 50 }, new uint[] { 59, 7, 60, 1 }), Module(new uint[] { 15, 4, 50, 0x6E69616D }, new uint[] { 59, 7, 60, 1 }), Module(new uint[] { 15, 4, 50, 0x6E69616D, 0, 60 }, new uint[] { 59, 7, 60, 1 }),
                Module(new uint[] { 15, 4, 50, 0, 60, 61 }, new uint[] { 59, 7, 60, 1 }, new uint[] { 59, 7, 61, 1 }, new uint[] { 54, 1, 50, 0, 2 }, new uint[] { 61, 8, 61 }),
            };
            // Damage that cannot make a type hold itself, for which the C# would not return: cuts, a changed count, and changed bytes in the names and decorations ahead of the first type.
            foreach (byte[] module in modules)
            {
                uint[] words = Enumerable.Range(0, module.Length / 4).Select(i => BitConverter.ToUInt32(module, i * 4)).ToArray();
                int firstType = 5;
                while (firstType < words.Length && (words[firstType] & 0xFFFF) is not (>= 19 and <= 33)) firstType += (int)Math.Max(1, words[firstType] >> 16);
                for (int n = 0; n < 150; n++)
                {
                    byte[] damaged = (byte[])module.Clone();
                    switch (random.Next(4))
                    {
                        case 0: damaged = damaged[..random.Next(damaged.Length)]; break;
                        case 1: damaged[random.Next(20, firstType * 4)] ^= (byte)(1 << random.Next(8)); break;
                        case 2: damaged[random.Next(4)] ^= 1; break;
                        default:
                            int at = 5;
                            for (int hops = random.Next(40); hops > 0 && at + (int)(words[at] >> 16) < firstType; hops--) at += (int)(words[at] >> 16);
                            BitConverter.GetBytes((ushort)random.Next(0, 9)).CopyTo(damaged, at * 4 + 2);
                            break;
                    }
                    cases.Add(damaged);
                }
            }
            foreach (byte[] spirv in cases)
            {
                string managed = Outcome(() => Dump(SpirvReflection.Managed.Read(spirv))), native = Outcome(() => Dump(SpirvReflection.ReadNative(spirv)));
                if (managed != native) different.Add($"read {spirv.Length} bytes:\n{managed}\n--\n{native}");
                byte[]? keptManaged = null, keptNative = null;
                managed = Outcome(() => Convert.ToHexString(keptManaged = SpirvReflection.Managed.WithoutUnreadInputs(spirv)));
                native = Outcome(() => Convert.ToHexString(keptNative = SpirvReflection.WithoutUnreadInputsNative(spirv)));
                if (managed != native) different.Add($"prune {spirv.Length} bytes: {managed[..Math.Min(120, managed.Length)]} and {native[..Math.Min(120, native.Length)]}");
                if (keptManaged is not null && ReferenceEquals(keptManaged, spirv) != ReferenceEquals(keptNative, spirv)) different.Add($"prune {spirv.Length} bytes: one kept the caller's array and the other did not");
            }
            var reflections = modules.Select(SpirvReflection.Managed.Read).ToList();
            foreach (SpirvReflection a in reflections)
                foreach (SpirvReflection b in reflections)
                    if (Dump(SpirvReflection.Managed.Merge(a, b)) != Dump(SpirvReflection.MergeNative(a, b))) different.Add($"merge:\n{Dump(SpirvReflection.Managed.Merge(a, b))}\n--\n{Dump(SpirvReflection.MergeNative(a, b))}");
            Assert.Contains(reflections, r => r.Uniforms is not null && r.PushConstants is not null && r.Samplers.Count == 2);
            Assert.Contains(modules, m => !ReferenceEquals(SpirvReflection.Managed.WithoutUnreadInputs(m), m));
            Assert.Contains(reflections, a => reflections.Any(b => a.Uniforms is not null && b.Uniforms is not null && a.Uniforms.Binding != b.Uniforms.Binding && a.Uniforms.Size != b.Uniforms.Size));
            AssertNone(different, "modules");
            // A type that holds itself is the one module the C# does not return from; the library refuses it.
            Assert.Equal("ArgumentException: Not SPIR-V: a type holds itself.", Outcome(() => Dump(SpirvReflection.ReadNative(Module(new uint[] { 23, 2, 2, 4 }, new uint[] { 30, 3, 2 }, new uint[] { 59, 3, 9, 2 })))));
        }

        // Every row of a cache's file, read through a connection of its own once the cache has let go.
        private static string Rows(string path)
        {
            using var raw = new SqliteConnection($"Data Source={path};Pooling=False");
            raw.Open();
            using SqliteCommand read = raw.CreateCommand();
            read.CommandText = "SELECT hex(key), bytes, last_used, hex(digest), hex(spirv), (SELECT user_version FROM pragma_user_version), (SELECT journal_mode FROM pragma_journal_mode) FROM spirv ORDER BY key";
            using SqliteDataReader row = read.ExecuteReader();
            var rows = new List<string>();
            while (row.Read()) rows.Add($"{row.GetString(0)} {row.GetInt64(1)} {row.GetInt64(2)} {row.GetString(3)} {row.GetString(4).Length} v{row.GetInt64(5)} {row.GetString(6)}");
            return string.Join("\n", rows);
        }

        private static string Counts(SpirvCache cache) => $"working {cache.Working} problem {(cache.Problem is null ? "none" : "some")} hits {cache.Hits} misses {cache.Misses} stored {cache.Stored} damaged {cache.Damaged} skipped {cache.Skipped} evicted {cache.Evicted}";

        private static string Shader(int n) => $"#version 450\nlayout(location = 0) out vec4 c;\nvoid main() {{ c = vec4({n}.0); }}\n";

        [Fact]
        public void A_cache_keeps_serves_bounds_and_repairs_the_same_and_leaves_the_same_rows()
        {
            var random = new Random(46);
            var different = new List<string>();
            for (int run = 0; run < 12; run++)
            {
                string managedPath = Path.Combine(_root, $"managed{run}.db"), nativePath = Path.Combine(_root, $"native{run}.db");
                int one = SlangCompiler.Managed.Compile(Shader(0), SlangStage.Fragment, "s.slang").Length;
                long limit = random.Next(3) == 0 ? SpirvCache.DefaultLimit : one * random.Next(2, 6) + one / 2;
                long now = 1000;
                long touch = random.Next(2) == 0 ? 0 : 5;
                SpirvCache managed = new(managedPath, limit, "one identity for both", native: false) { Clock = () => now, TouchAfter = touch }, native = new(nativePath, limit, "one identity for both", native: true) { Clock = () => now, TouchAfter = touch };
                for (int step = 0; step < 40; step++)
                {
                    string what;
                    switch (random.Next(10))
                    {
                        case 0:
                            managed.Dispose(); native.Dispose();
                            string rows = Rows(managedPath), nativeRows = Rows(nativePath);
                            if (rows != nativeRows) different.Add($"run {run} step {step} rows:\n{rows}\n--\n{nativeRows}");
                            // One row's bytes changed, or its digest, in both files alike.
                            foreach (string path in new[] { managedPath, nativePath })
                            {
                                using var raw = new SqliteConnection($"Data Source={path};Pooling=False");
                                raw.Open();
                                using SqliteCommand damage = raw.CreateCommand();
                                damage.CommandText = step % 2 == 0
                                    ? "UPDATE spirv SET digest = zeroblob(32) WHERE key = (SELECT min(key) FROM spirv)"
                                    : "UPDATE spirv SET spirv = zeroblob(length(spirv)) WHERE key = (SELECT max(key) FROM spirv)";
                                damage.ExecuteNonQuery();
                            }
                            managed = new(managedPath, limit, "one identity for both", native: false) { Clock = () => now, TouchAfter = touch };
                            native = new(nativePath, limit, "one identity for both", native: true) { Clock = () => now, TouchAfter = touch };
                            what = "reopened with a damaged row";
                            break;
                        case 1: now += random.Next(0, 20); what = $"clock {now}"; break;
                        case 2:
                            string broken = "#version 450\nvoid main() { nope }\n";
                            string failedManaged = Outcome(() => Convert.ToHexString(managed.Compile(broken, SlangStage.Vertex, "b.slang"))), failedNative = Outcome(() => Convert.ToHexString(native.Compile(broken, SlangStage.Vertex, "b.slang")));
                            what = "a stage that does not compile";
                            if (failedManaged != failedNative) different.Add($"run {run} step {step} {what}: {failedManaged} and {failedNative}");
                            break;
                        default:
                            int n = random.Next(0, 9);
                            SlangStage stage = random.Next(5) == 0 ? SlangStage.Vertex : SlangStage.Fragment;
                            string name = random.Next(6) == 0 ? "other \u00E9.slang" : "s.slang";
                            now++;
                            byte[] a = managed.Compile(Shader(n), stage, name), b = native.Compile(Shader(n), stage, name);
                            what = $"compile {n} {stage} {name}";
                            if (!a.AsSpan().SequenceEqual(b)) different.Add($"run {run} step {step} {what}: the bytes differ");
                            break;
                    }
                    if (Counts(managed) != Counts(native)) different.Add($"run {run} step {step} {what}: {Counts(managed)} and {Counts(native)}");
                }
                managed.Dispose(); native.Dispose();
                if (Rows(managedPath) != Rows(nativePath)) different.Add($"run {run} at the end:\n{Rows(managedPath)}\n--\n{Rows(nativePath)}");
                Assert.Equal(Counts(managed), Counts(native));
            }
            AssertNone(different, "caches");
        }

        [Fact]
        public void A_cache_one_wrote_is_read_by_the_other_and_a_file_neither_can_use_is_handled_alike()
        {
            // The C# writes, the library reads every row as a hit and adds one, and the C# reads those back, on one file with one identity.
            foreach (bool nativeFirst in new[] { false, true })
            {
                string path = Path.Combine(_root, $"shared{nativeFirst}.db");
                using (var first = new SpirvCache(path, SpirvCache.DefaultLimit, "shared", nativeFirst))
                {
                    for (int n = 0; n < 6; n++) first.Compile(Shader(n), SlangStage.Fragment, "s.slang");
                    Assert.Equal((0, 6, 6), (first.Hits, first.Misses, first.Stored));
                }
                using (var second = new SpirvCache(path, SpirvCache.DefaultLimit, "shared", !nativeFirst))
                {
                    for (int n = 0; n < 7; n++) Assert.Equal(SlangCompiler.Managed.Compile(Shader(n), SlangStage.Fragment, "s.slang"), second.Compile(Shader(n), SlangStage.Fragment, "s.slang"));
                    Assert.Equal((6, 1, 1, null), (second.Hits, second.Misses, second.Stored, second.Problem));
                }
                using var third = new SpirvCache(path, SpirvCache.DefaultLimit, "shared", nativeFirst);
                for (int n = 0; n < 7; n++) third.Compile(Shader(n), SlangStage.Fragment, "s.slang");
                Assert.Equal((7, 0, 0), (third.Hits, third.Misses, third.Stored));
            }

            // By default each names its own binding, so neither serves the other's rows; what else the two identities say is the same.
            string managedIdentity = SlangCompiler.Managed.Identity, nativeIdentity = SerenityNative.Identity();
            Assert.NotEqual(managedIdentity, nativeIdentity);
            Regex binding = new("; binding [^;]+;");
            Assert.Equal(binding.Replace(managedIdentity, "; binding X;"), binding.Replace(nativeIdentity, "; binding X;"));
            Assert.DoesNotContain("unfound", nativeIdentity);
            Assert.DoesNotContain("unread", nativeIdentity);

            foreach (bool native in new[] { false, true })
            {
                // Not a database: set aside, and a new one made.
                string junk = Path.Combine(_root, $"junk{native}.db");
                File.WriteAllBytes(junk, Enumerable.Range(0, 8192).Select(i => (byte)(i * 31)).ToArray());
                using (var cache = new SpirvCache(junk, SpirvCache.DefaultLimit, "x", native))
                {
                    Assert.True(cache.Working, cache.Problem);
                    Assert.True(File.Exists(junk + ".damaged"));
                    cache.Compile(Shader(1), SlangStage.Fragment, "s.slang");
                    Assert.Equal(1, cache.Stored);
                }
                // Another schema version: emptied, not migrated.
                string old = Path.Combine(_root, $"old{native}.db");
                using (var cache = new SpirvCache(old, SpirvCache.DefaultLimit, "x", native)) cache.Compile(Shader(1), SlangStage.Fragment, "s.slang");
                using (var raw = new SqliteConnection($"Data Source={old};Pooling=False"))
                {
                    raw.Open();
                    using SqliteCommand version = raw.CreateCommand();
                    version.CommandText = "PRAGMA user_version = 7";
                    version.ExecuteNonQuery();
                }
                using (var cache = new SpirvCache(old, SpirvCache.DefaultLimit, "x", native))
                {
                    cache.Compile(Shader(1), SlangStage.Fragment, "s.slang");
                    Assert.Equal((0, 1, 1), (cache.Hits, cache.Misses, cache.Stored));
                }
                // A row whose bytes are not a blob: the cache turns itself off and goes on compiling.
                using (var raw = new SqliteConnection($"Data Source={old};Pooling=False"))
                {
                    raw.Open();
                    using SqliteCommand text = raw.CreateCommand();
                    text.CommandText = "PRAGMA ignore_check_constraints = 1; UPDATE spirv SET spirv = 'text', bytes = 4";
                    text.ExecuteNonQuery();
                }
                using (var cache = new SpirvCache(old, SpirvCache.DefaultLimit, "x", native))
                {
                    Assert.Equal(SlangCompiler.Managed.Compile(Shader(1), SlangStage.Fragment, "s.slang"), cache.Compile(Shader(1), SlangStage.Fragment, "s.slang"));
                    Assert.Equal((false, 0, 0, 0), (cache.Working, cache.Hits, cache.Misses, cache.Stored));
                    Assert.StartsWith("spirv-cache.db: ", cache.Problem);
                }
                // A path that cannot hold a file, and one that is no path.
                foreach (string bad in new[] { Path.Combine(junk, "under-a-file", "cache.db"), "" })
                {
                    using var cache = new SpirvCache(bad, SpirvCache.DefaultLimit, "x", native);
                    Assert.False(cache.Working);
                    Assert.StartsWith("spirv-cache.db: ", cache.Problem);
                    Assert.Equal(SlangCompiler.Managed.Compile(Shader(2), SlangStage.Fragment, "s.slang"), cache.Compile(Shader(2), SlangStage.Fragment, "s.slang"));
                }
            }
            // A database another connection is writing: two stages are compiled around it, the third failure turns the cache off, and both say why in SQLite's words.
            var problems = new List<string?>();
            foreach (bool native in new[] { false, true })
            {
                string held = Path.Combine(_root, $"held{native}.db");
                using (var fill = new SpirvCache(held, SpirvCache.DefaultLimit, "x", native)) fill.Compile(Shader(0), SlangStage.Fragment, "s.slang");
                using var cache = new SpirvCache(held, SpirvCache.DefaultLimit, "x", native);
                using var writer = new SqliteConnection($"Data Source={held};Pooling=False");
                writer.Open();
                using (SqliteCommand hold = writer.CreateCommand())
                {
                    hold.CommandText = "BEGIN IMMEDIATE; DELETE FROM spirv WHERE bytes < 0;";
                    hold.ExecuteNonQuery();
                }
                Assert.Equal(1, cache.Compile(Shader(0), SlangStage.Fragment, "s.slang").Length > 0 ? cache.Hits : -1);
                // A statement the writer blocks is tried again for the connection's second before it is let go, by either.
                var waited = System.Diagnostics.Stopwatch.StartNew();
                cache.Compile(Shader(1), SlangStage.Fragment, "s.slang");
                Assert.InRange(waited.ElapsedMilliseconds, 900, 5000);
                for (int n = 2; n <= 4; n++) cache.Compile(Shader(n), SlangStage.Fragment, "s.slang");
                Assert.Equal((false, 0, 2, 1), (cache.Working, cache.Stored, cache.Skipped, cache.Hits));
                problems.Add(cache.Problem);
            }
            Assert.Equal("spirv-cache.db: SQLite Error 5: 'database is locked'.", problems[0]);
            Assert.Equal(problems[0], problems[1]);

            using var emptyManaged = new SpirvCache("", SpirvCache.DefaultLimit, "x", native: false);
            using var emptyNative = new SpirvCache("", SpirvCache.DefaultLimit, "x", native: true);
            Assert.Equal(emptyManaged.Problem, emptyNative.Problem);
        }

        [Fact]
        public void A_key_is_the_same_hash_of_the_same_parts()
        {
            var random = new Random(47);
            string[] parts = { "", "a", "shaderc x", "void main() {}", "\u65E5\u672C", "a\0b", "na\u00EFve.slang", new string('x', 5000), "\U0001F600" };
            for (int i = 0; i < 2000; i++)
            {
                string identity = parts[random.Next(parts.Length)], text = parts[random.Next(parts.Length)], name = parts[random.Next(parts.Length)];
                SlangStage stage = (SlangStage)random.Next(2);
                Assert.Equal(Convert.ToHexString(SpirvCache.Managed.Key(identity, text, stage, name)), Convert.ToHexString(SpirvCache.KeyNative(identity, text, stage, name)));
            }
        }

        // Every preset of libretro's pack, when a run names where it is: read, split and compiled through both, which is P6 of EmuSen_RustPlatform.md §8.
        [Fact]
        public void Every_preset_source_and_stage_of_the_pack_comes_out_the_same()
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            var different = new List<string>();
            var sources = new HashSet<string>(StringComparer.Ordinal);
            int presets = 0, stages = 0, failed = 0;
            foreach (string path in Directory.EnumerateFiles(pack, "*.slangp", SearchOption.AllDirectories).OrderBy(p => p, StringComparer.Ordinal))
            {
                presets++;
                string managed = Outcome(() => Dump(SlangPreset.Managed.Load(path))), native = Outcome(() => Dump(SlangPreset.LoadNative(path)));
                if (managed != native) different.Add($"preset {path}:\n{managed}\n--\n{native}");
                if (managed.Contains("Exception: ")) continue;
                foreach (SlangPassSpec pass in SlangPreset.Managed.Load(path).Passes) sources.Add(pass.ShaderPath);
            }
            foreach (string path in sources.OrderBy(p => p, StringComparer.Ordinal))
            {
                string managed = Outcome(() => Dump(SlangSource.Managed.Load(path))), native = Outcome(() => Dump(SlangSource.LoadNative(path)));
                if (managed != native) { different.Add($"source {path}"); continue; }
                if (managed.Contains("Exception: ")) continue;
                SlangSource source = SlangSource.Managed.Load(path);
                foreach ((string text, SlangStage stage) in new[] { (source.Vertex, SlangStage.Vertex), (source.Fragment, SlangStage.Fragment) })
                {
                    stages++;
                    byte[]? spirv = null;
                    string a = Outcome(() => Convert.ToHexString(spirv = SlangCompiler.Managed.Compile(text, stage, path))), b = Outcome(() => Convert.ToHexString(SlangCompiler.CompileNative(text, stage, path)));
                    if (a != b) different.Add($"stage {path} {stage}");
                    if (spirv is null) { failed++; continue; }
                    if (Outcome(() => Dump(SpirvReflection.Managed.Read(spirv))) != Outcome(() => Dump(SpirvReflection.ReadNative(spirv)))) different.Add($"reflection {path} {stage}");
                    if (!SpirvReflection.Managed.WithoutUnreadInputs(spirv).AsSpan().SequenceEqual(SpirvReflection.WithoutUnreadInputsNative(spirv))) different.Add($"pruning {path} {stage}");
                }
            }
            _output.WriteLine($"{presets} presets, {sources.Count} sources, {stages} stages of which {failed} do not compile in either");
            AssertNone(different, "of the pack's files");
        }

        [Fact]
        public void The_library_has_every_export_this_half_calls_and_no_other()
        {
            string header = File.ReadAllText(Path.Combine(EmuSen.Galaxia.ConfigRoot.Managed.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in SerenityNative.Exports) Assert.Contains(export + "(", header);
            Assert.Equal(SerenityNative.Exports.Count(e => e.StartsWith("emusen_serenity_", StringComparison.Ordinal)), Regex.Matches(header, @"^\w+ \*?emusen_serenity_\w+\(", RegexOptions.Multiline).Count);
        }
    }
}
