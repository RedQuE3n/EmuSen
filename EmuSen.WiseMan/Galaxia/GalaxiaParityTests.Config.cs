using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Galaxia.Text;

namespace EmuSen.WiseMan.Galaxia
{
    // The config half: what System.Text.Json writes and reads against what the platform library does - see EmuSen_RustPlatform.md §11.5.
    public partial class GalaxiaParityTests
    {
        private static string Json<T>(T value) => JsonSerializer.Serialize(value, ConfigJson.Options);

        private static T? Read<T>(string text) => JsonSerializer.Deserialize<T>(text, ConfigJson.Options);

        private static void AssertNone(List<string> different, string what) =>
            Assert.True(different.Count == 0, $"{different.Count} {what} differ:\n{string.Join("\n", different.Take(25))}");

        private static string Shown(string text) => text.Length <= 160 ? text : text[..160] + $"… ({text.Length})";

        [Fact]
        public void Every_double_and_float_is_written_as_System_Text_Json_writes_it()
        {
            var different = new List<string>();
            void Double(double value)
            {
                string? native = GalaxiaNative.NumberFormat(single: false, BitConverter.DoubleToUInt64Bits(value));
                string? managed = double.IsFinite(value) ? JsonSerializer.Serialize(value) : null;
                if (native != managed) different.Add($"double {BitConverter.DoubleToUInt64Bits(value):X16}: {managed} and {native}");
            }
            void Single(float value)
            {
                string? native = GalaxiaNative.NumberFormat(single: true, BitConverter.SingleToUInt32Bits(value));
                string? managed = float.IsFinite(value) ? JsonSerializer.Serialize(value) : null;
                if (native != managed) different.Add($"float {BitConverter.SingleToUInt32Bits(value):X8}: {managed} and {native}");
            }

            var random = new Random(20261008);
            for (int i = 0; i < 300_000; i++)
            {
                Double(BitConverter.Int64BitsToDouble(random.NextInt64(long.MinValue, long.MaxValue)));
                Single(BitConverter.Int32BitsToSingle(random.Next(int.MinValue, int.MaxValue)));
                // What a person types: a few digits, a point somewhere, a power of ten.
                double typed = random.Next(1, 1_000_000) * Math.Pow(10, random.Next(-25, 25));
                Double(typed);
                Double(-typed);
                Single((float)typed);
                Double(random.Next(-1000, 1000) / 8.0);
            }
            foreach (double edge in new[] { 0.0, -0.0, 1, -1, 0.1, 0.5, 1e15, 1e16, 1e17, 1e-4, 1e-5, 9.999e16, 1.0000000000000002e17, double.Epsilon, double.MaxValue, double.MinValue, double.NaN, double.PositiveInfinity, double.NegativeInfinity, 0.005, 0.3, 123456789012345678 })
                Double(edge);
            foreach (float edge in new[] { 0f, -0f, 1f, 0.8f, 1e8f, 1e9f, 9.99e8f, 1e-4f, 1e-5f, float.Epsilon, float.MaxValue, float.MinValue, float.NaN, float.PositiveInfinity, 16777216f, 0.1f })
                Single(edge);
            for (int exponent = -330; exponent <= 310; exponent++)
            {
                Double(double.Parse($"1e{exponent}"));
                Double(double.Parse($"9.5e{exponent}"));
                if (exponent is > -50 and < 40) Single(float.Parse($"1e{exponent}"));
            }
            AssertNone(different, "numbers");
        }

        [Fact]
        public void A_files_bytes_are_decoded_as_File_ReadAllText_decodes_them()
        {
            byte[][] marks = { Array.Empty<byte>(), new byte[] { 0xEF, 0xBB, 0xBF }, new byte[] { 0xFF, 0xFE }, new byte[] { 0xFE, 0xFF }, new byte[] { 0xFF, 0xFE, 0, 0 }, new byte[] { 0, 0, 0xFE, 0xFF }, new byte[] { 0xEF, 0xBB }, new byte[] { 0xFF } };
            byte[] alphabet = { (byte)'{', (byte)'}', (byte)'"', (byte)'a', 0, 0, 0xC3, 0xA9, 0xE2, 0x82, 0xAC, 0xF0, 0x9F, 0x98, 0x80, 0xFF, 0xFE, 0xC0, 0x80, 0xED, 0xA0, 0xD8, 0xDC, 0x10, 0x0A, 0xEF, 0xBB, 0xBF, 0xF4, 0x90 };
            var random = new Random(8);
            var different = new List<string>();
            string file = At("decode.bin");
            Directory.CreateDirectory(_temp);
            for (int i = 0; i < 4000; i++)
            {
                byte[] body = Enumerable.Range(0, random.Next(0, 24)).Select(_ => alphabet[random.Next(alphabet.Length)]).ToArray();
                byte[] bytes = marks[random.Next(marks.Length)].Concat(body).ToArray();
                File.WriteAllBytes(file, bytes);
                if (File.ReadAllText(file) != GalaxiaNative.TextDecode(bytes)) different.Add(Convert.ToHexString(bytes));
            }
            AssertNone(different, "files");
        }

        // The classes whose schemas the library owns, each with the type C# binds.
        private static readonly (GalaxiaModel Model, Type Type)[] Owned =
        {
            (GalaxiaModel.AppSettings, typeof(AppSettings)), (GalaxiaModel.AudioConfig, typeof(AudioConfig)), (GalaxiaModel.GraphicsConfig, typeof(GraphicsConfig)), (GalaxiaModel.CheatFile, typeof(CheatFile)),
        };

        private static string TypeName(Type type)
        {
            if (type == typeof(string)) return "string";
            if (type == typeof(bool)) return "bool";
            if (type == typeof(bool?)) return "bool?";
            if (type == typeof(int)) return "int";
            if (type == typeof(int?)) return "int?";
            if (type == typeof(long)) return "long";
            if (type == typeof(double)) return "double";
            if (type == typeof(float)) return "float";
            if (type.IsGenericType && type.GetGenericArguments() is { } arguments)
            {
                if (type.GetGenericTypeDefinition() == typeof(Dictionary<,>) && arguments[0] == typeof(string)) return $"map<{TypeName(arguments[1])}>";
                if (type.GetGenericTypeDefinition() == typeof(List<>) || type.GetGenericTypeDefinition() == typeof(IReadOnlyList<>)) return $"list<{TypeName(arguments[0])}>";
            }
            return type.Name;
        }

        private static IEnumerable<PropertyInfo> Serialized(Type type) =>
            type.GetProperties(BindingFlags.Public | BindingFlags.Instance).Where(p => p.GetMethod is not null).OrderBy(p => p.MetadataToken);

        private static Type? ClassIn(Type type) =>
            type.IsGenericType ? ClassIn(type.GetGenericArguments()[^1]) : type.IsClass && type != typeof(string) ? type : null;

        // The schema as reflection sees the C# class, in the form the library lists its own.
        private static string Reflected(Type root)
        {
            var seen = new List<Type>();
            var lines = new StringBuilder();
            void Walk(Type type)
            {
                if (seen.Contains(type)) return;
                seen.Add(type);
                foreach (PropertyInfo property in Serialized(type))
                {
                    bool skipped = property.GetCustomAttribute<System.Text.Json.Serialization.JsonIgnoreAttribute>()?.Condition == System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull;
                    string kind = property.SetMethod is null ? "\tderived" : skipped ? "\tskipped when null" : "";
                    lines.Append($"{type.Name}.{property.Name}\t{TypeName(property.PropertyType)}{kind}\n");
                }
                foreach (PropertyInfo property in Serialized(type))
                    if (ClassIn(property.PropertyType) is { } inner) Walk(inner);
            }
            Walk(root);
            return lines.ToString();
        }

        [Fact]
        public void Every_model_has_the_fields_types_and_new_values_its_class_has()
        {
            foreach ((GalaxiaModel model, Type type) in Owned)
            {
                Assert.Equal(Reflected(type), GalaxiaNative.ModelSchema(model));
                string fresh = JsonSerializer.Serialize(Activator.CreateInstance(type), type, ConfigJson.Options);
                Assert.Equal(fresh, GalaxiaNative.ModelFormat(model, GalaxiaNative.ModelNew(model, upgrade: false)));
                Assert.Equal(fresh, GalaxiaNative.ModelFormat(model, fresh));
                Assert.Equal(GalaxiaNative.ModelNew(model, upgrade: false), GalaxiaNative.ModelBind(model, "{}", upgrade: false));
            }
            Assert.Equal(GalaxiaModel.CheatFile, GalaxiaNative.ModelOf(typeof(CheatFile)));
            Assert.Null(GalaxiaNative.ModelOf(typeof(Dictionary<string, string>)));
            Assert.Equal(Enum.GetValues<GalaxiaModel>().Length, Owned.Length);
        }

        private static readonly string[] Words =
        {
            "", " ", "x", "All consoles", "SNES (Venus)", "é", "日本語", "😀", "a\"b", "a\\b", "a/b", "<&'+`>", "\t\r\n", "\u0000\u001f\u007f\u0080", "\u2028\u2029", "\ufffd", "C:\\Roms\\é", "/home/red/Roms",
            "{\"not\":\"json\"}", "line one\nline two", new string('x', 300), "\ud83d\ude00\ud801\udc37", "RomPatch", "rompatch", "RamPoke", "7E0DBF", "0x1F", "$FF",
        };

        private static readonly double[] Doubles = { 0, -0.0, 1, 0.5, 0.1, 0.005, 1.5, -2.25, 1e15, 1e16, 1e17, 1e-5, 1e-4, 123456.789, double.MaxValue, double.Epsilon, 0.30000000000000004, -1e300, 70, 1.0 / 3 };
        private static readonly float[] Singles = { 0, -0f, 1, 0.8f, 0.1f, 0.5f, 1e8f, 1e9f, 1e-5f, float.MaxValue, float.Epsilon, 1f / 3, -2.5f, 16777216f };
        private static readonly int[] Ints = { 0, 1, -1, 8, 60, 32000, int.MaxValue, int.MinValue, 300000 };

        // A value of any type a model holds, from a seed, with the unusual values among the usual.
        private static object? Random_(Type type, Random random, int depth, bool nullable)
        {
            if (type == typeof(string)) return nullable && random.Next(6) == 0 ? null : Words[random.Next(Words.Length)];
            if (type == typeof(bool)) return random.Next(2) == 0;
            if (type == typeof(bool?)) return random.Next(3) switch { 0 => null, 1 => true, _ => false };
            if (type == typeof(int)) return random.Next(3) == 0 ? Ints[random.Next(Ints.Length)] : random.Next(-5000, 5000);
            if (type == typeof(int?)) return random.Next(3) == 0 ? null : random.Next(0, 32);
            if (type == typeof(long)) return random.Next(4) switch { 0 => long.MaxValue, 1 => long.MinValue, _ => random.NextInt64(-1_000_000, 1_000_000) };
            if (type == typeof(double)) return random.Next(2) == 0 ? Doubles[random.Next(Doubles.Length)] : Math.Round(random.NextDouble() * 4, random.Next(0, 6));
            if (type == typeof(float)) return random.Next(2) == 0 ? Singles[random.Next(Singles.Length)] : (float)Math.Round(random.NextDouble(), 2);
            if (nullable && random.Next(12) == 0) return null;
            if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(List<>))
            {
                var list = (IList)Activator.CreateInstance(type)!;
                Type item = type.GetGenericArguments()[0];
                for (int i = random.Next(0, depth > 2 ? 2 : 4); i > 0; i--) list.Add(Random_(item, random, depth + 1, nullable: !item.IsValueType));
                return list;
            }
            if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Dictionary<,>))
            {
                var map = (IDictionary)Activator.CreateInstance(type)!;
                Type item = type.GetGenericArguments()[1];
                for (int i = random.Next(0, depth > 2 ? 2 : 4); i > 0; i--) map[Words[random.Next(Words.Length)] + (random.Next(3) == 0 ? "" : i.ToString())] = Random_(item, random, depth + 1, nullable: !item.IsValueType);
                return map;
            }
            object made = Activator.CreateInstance(type)!;
            foreach (PropertyInfo property in Serialized(type).Where(p => p.SetMethod is not null))
            {
                // A cheat with no list of writes cannot be serialized at all; that case is asked about as text.
                bool mayBeNull = !property.PropertyType.IsValueType && !(type == typeof(CheatFileEntry) && property.Name == nameof(CheatFileEntry.Writes));
                if (random.Next(5) != 0) property.SetValue(made, Random_(property.PropertyType, random, depth + 1, mayBeNull));
            }
            return made;
        }

        [Fact]
        public void Every_model_is_written_as_the_bytes_System_Text_Json_writes_and_reads_back_the_same()
        {
            var different = new List<string>();
            foreach ((GalaxiaModel model, Type type) in Owned)
            {
                var random = new Random(1000 + (int)model);
                for (int i = 0; i < 400; i++)
                {
                    object instance = Random_(type, random, 0, nullable: false)!;
                    string managed = JsonSerializer.Serialize(instance, type, ConfigJson.Options);
                    string native = GalaxiaNative.ModelFormat(model, managed);
                    if (native != managed) { different.Add($"{type.Name} written #{i}: {FirstDifference(managed, native)}"); continue; }

                    // What the library read of those bytes, bound by C# and written again, is those bytes.
                    string bound = GalaxiaNative.ModelBind(model, native, upgrade: false);
                    string back = JsonSerializer.Serialize(JsonSerializer.Deserialize(bound, type, ConfigJson.Options), type, ConfigJson.Options);
                    if (back != managed) different.Add($"{type.Name} read back #{i}: {FirstDifference(managed, back)}");
                }
            }
            AssertNone(different, "documents");
        }

        private static string FirstDifference(string expected, string actual)
        {
            int at = 0;
            while (at < expected.Length && at < actual.Length && expected[at] == actual[at]) at++;
            string Around(string text) => text[Math.Max(0, at - 40)..Math.Min(text.Length, at + 40)].Replace("\n", "\\n");
            return $"at {at}: [{Around(expected)}] and [{Around(actual)}]";
        }

        [Fact]
        public void Every_character_is_escaped_as_System_Text_Json_escapes_it_in_a_value_and_in_a_key()
        {
            var different = new List<string>();
            for (int start = 0; start < 0x10000; start += 256)
            {
                // Lone surrogates included: C# writes each as U+FFFD, and so the text that reaches the library holds that.
                string chunk = new(Enumerable.Range(start, 256).Select(c => (char)c).ToArray());
                var settings = new AppSettings { LibrarySearch = chunk };
                settings.BigPicture[chunk] = new BigPictureChoices { Variant = chunk };
                string managed = Json(settings);
                if (GalaxiaNative.ModelFormat(GalaxiaModel.AppSettings, managed) != managed) different.Add($"U+{start:X4}…");
            }
            var beyond = new AppSettings { LibrarySearch = string.Concat(new[] { 0x10000, 0x1F600, 0x10437, 0x2F800, 0xE0001, 0x10FFFF }.Select(char.ConvertFromUtf32)) };
            Assert.Equal(Json(beyond), GalaxiaNative.ModelFormat(GalaxiaModel.AppSettings, Json(beyond)));
            AssertNone(different, "blocks of characters");
        }

        // What reading a text comes to: nothing, a refusal, or the document it makes, written out; with what cannot be written said so.
        private static string Outcome(Type type, Func<string?> text)
        {
            object? value;
            try
            {
                if (text() is not { } read) return "absent";
                value = JsonSerializer.Deserialize(read, type, ConfigJson.Options);
            }
            catch (Exception)
            {
                return "refused";
            }
            if (value is null) return "null";
            try { return JsonSerializer.Serialize(value, type, ConfigJson.Options); }
            catch (Exception) { return "read, and cannot be written"; }
        }

        // One text read both ways; the difference, or null. Where the document can be written, the library's bytes for it are held to C#'s too.
        private static string? ReadBothWays(GalaxiaModel model, Type type, string text)
        {
            string managed = Outcome(type, () => text);
            // The library's refusal must be its own: what it binds, C# binds again on the way in, and would refuse for it.
            string native;
            try
            {
                string bound = GalaxiaNative.ModelBind(model, text, upgrade: false);
                native = bound == "null" ? "null" : Outcome(type, () => bound);
                if (native == "refused") native = $"bound by the library as what C# refuses: {bound}";
            }
            catch (FormatException)
            {
                native = "refused";
            }
            if (managed != native) return $"{type.Name} {Shown(text)}\n    C#: {Shown(managed)}\n    library: {Shown(native)}";
            if (managed is "refused" or "null") return null;

            string written;
            try { written = GalaxiaNative.ModelFormat(model, GalaxiaNative.ModelBind(model, text, upgrade: false)); }
            catch (FormatException) { written = "read, and cannot be written"; }
            return written == managed ? null : $"{type.Name} {Shown(text)}\n    written by C#: {Shown(managed)}\n    by the library: {Shown(written)}";
        }

        private static readonly string[] Tokens =
        {
            "null", "true", "false", "0", "-0", "1", "-1", "1.0", "1.5", "1e2", "1E+2", "2147483647", "2147483648", "-2147483648", "-2147483649", "9223372036854775807", "9223372036854775808",
            "-9223372036854775808", "-9223372036854775809", "1e308", "1e309", "-1e999", "1e-999", "5e-324", "0.1", "0.800000011920929", "16777217", "3.4028235e38", "3.4028236e38", "1e39", "\"\"", "\"x\"",
            "\"\\u00e9\\ud83d\\ude00é\"", "\"\\ud800\"", "\"\\udc00\\ud800\"", "\"1\"", "\"true\"", "\"RomPatch\"", "\"ROMPATCH\"", "[]", "[1]", "[null]", "[\"a\",null]", "[\"a\",1]", "[[\"a\"]]", "[{}]", "[{},null]",
            "[{\"Address\":\"1\"}]", "{}", "{\"a\":null}", "{\"a\":\"b\",\"a\":\"c\",\"A\":\"d\"}", "{\"a\":1}", "{\"a\":{}}", "{\"a\":[]}", "{\"a\":[\"x\",null]}", "{\"a\":{\"b\":{\"c\":\"d\"}}}", "{\"a\":{\"b\":null}}",
            "{\"\\ud800\":null}", "{\"Variant\":\"v\",\"variant\":null,\"Unknown\":[1,{\"x\":\"\\ud800\"}]}",
        };

        // A value a type takes, in one of a few spellings, for a map's keys to be given more than once.
        private static string Sample(Type type, int which)
        {
            if (type == typeof(string)) return which % 3 == 0 ? "null" : $"\"s{which}\"";
            if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Dictionary<,>)) return $"{{\"k{which}\":{Sample(type.GetGenericArguments()[1], which + 1)}}}";
            if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(List<>)) return $"[{Sample(type.GetGenericArguments()[0], which + 1)}]";
            return which % 3 == 0 ? "null" : type == typeof(BigPictureChoices) ? $"{{\"Variant\":\"v{which}\"}}" : "{}";
        }

        // For every property a model's classes have, a document setting it to each token: at the top, and wherever a class is held.
        private static IEnumerable<string> PropertyDocuments(Type type, int depth = 0)
        {
            int n = 0;
            foreach (PropertyInfo property in Serialized(type))
            {
                foreach (string token in Tokens)
                {
                    string name = (n++ % 7) switch { 0 => property.Name.ToUpperInvariant(), 1 => property.Name.ToLowerInvariant(), _ => property.Name };
                    yield return $"{{\"{name}\":{token}}}";
                }
                yield return $"{{\"{property.Name}\":1,\"{property.Name.ToLowerInvariant()}\":null,\"{property.Name}\":\"x\"}}";
                if (property.PropertyType.IsGenericType && property.PropertyType.GetGenericTypeDefinition() == typeof(Dictionary<,>))
                {
                    // A key given twice, three times, and in another case: where it ends up and what it holds.
                    Type item = property.PropertyType.GetGenericArguments()[1];
                    for (int i = 0; i < 4; i++)
                    {
                        yield return $"{{\"{property.Name}\":{{\"a\":{Sample(item, i)},\"b\":{Sample(item, i + 1)},\"a\":{Sample(item, i + 2)},\"A\":{Sample(item, i + 3)}}}}}";
                        yield return $"{{\"{property.Name}\":{{\"z\":{Sample(item, i)},\"a\":{Sample(item, i + 1)},\"z\":{Sample(item, i + 2)},\"m\":{Sample(item, i + 3)},\"z\":{Sample(item, i + 4)}}}}}";
                    }
                    if (item.IsGenericType && item.GetGenericTypeDefinition() == typeof(Dictionary<,>))
                        yield return $"{{\"{property.Name}\":{{\"x\":{{\"a\":{Sample(item.GetGenericArguments()[1], 1)},\"b\":{Sample(item.GetGenericArguments()[1], 2)},\"a\":{Sample(item.GetGenericArguments()[1], 4)}}}}}}}";
                }
                if (depth > 2 || ClassIn(property.PropertyType) is not { } inner) continue;
                Type holder = property.PropertyType;
                foreach (string document in PropertyDocuments(inner, depth + 1))
                {
                    yield return holder == inner ? $"{{\"{property.Name}\":{document}}}"
                        : holder.GetGenericTypeDefinition() == typeof(Dictionary<,>) ? $"{{\"{property.Name}\":{{\"key\":{document},\"other\":null}}}}"
                        : $"{{\"{property.Name}\":[{document}]}}";
                }
            }
        }

        [Fact]
        public void Every_property_takes_and_refuses_the_same_values_both_ways()
        {
            var different = new List<string>();
            int asked = 0;
            foreach ((GalaxiaModel model, Type type) in Owned)
                foreach (string document in PropertyDocuments(type))
                {
                    asked++;
                    if (ReadBothWays(model, type, document) is { } difference) different.Add(difference);
                }
            Assert.True(asked > 8000, $"{asked} documents");
            AssertNone(different, $"of {asked} documents");
        }

        private static readonly string[] Malformed =
        {
            "", " ", "\n", "null", " null ", "nul", "nullx", "[]", "5", "\"s\"", "true", "{}", "{ }", "{,}", "{} x", "{} // c", "{} /* c */", "/* c */ {}", "// c\n{}", "{ /* c", "{ // c", "{}//", "{}/", "//", "/**/",
            "/**/null", "null/**/", "{/**/}", "{\"Unknown\"/*c*/:1}", "{\"Unknown\":/*c*/1}", "{\"Unknown\":1/*c*/}", "{\"Unknown\":1,/*c*/}", "{\"Unknown\":1/*c*/,}", "{\"Unknown\"://c\n1}", "{\"Unknown\"//c\n:1}",
            "{\"Unknown\":1//c\u2028\n}", "{\"Unknown\":1//c\u2029\n}", "{\"Unknown\":1/*\u2028*/}", "{\"Unknown\":1//\r}", "{\"Unknown\":1/}", "{\"Unknown\":1/*/}", "{\"Unknown\":[/*c*/1/*c*/,/*c*/2,/*c*/]}",
            "{\"Unknown\":1,}", "{\"Unknown\":1,,}", "{\"Unknown\":[1,2,]}", "{\"Unknown\":[,]}", "{\"Unknown\":[1,,2]}", "{\"Unknown\":[1 2]}", "{\"Unknown\":1 \"b\":2}", "{\"Unknown\" 1}", "{Unknown:1}", "{'Unknown':1}",
            "{\"Unknown\":1", "{\"Unknown\":\"abc", "{\"Unknown\":01}", "{\"Unknown\":-01}", "{\"Unknown\":1.}", "{\"Unknown\":.5}", "{\"Unknown\":-.5}", "{\"Unknown\":+1}", "{\"Unknown\":-}", "{\"Unknown\":1e}",
            "{\"Unknown\":1e+}", "{\"Unknown\":1.e2}", "{\"Unknown\":00}", "{\"Unknown\":0e0}", "{\"Unknown\":1e999}", "{\"Unknown\":1x}", "{\"Unknown\":tru}", "{\"Unknown\":truex}", "{\"Unknown\":true false}",
            "{\"Unknown\":nullx}", "{\"Unknown\":\"a\tb\"}", "{\"Unknown\":\"a\nb\"}", "{\"Unknown\":\"\\x\"}", "{\"Unknown\":\"\\/\"}", "{\"Unknown\":\"\\u00e9\\U00e9\"}", "{\"Unknown\":\"\\u00E\"}", "{\"Unknown\":\"\\uZZZZ\"}",
            "{\"Unknown\":\"\\ud800\"}", "{\"Unknown\":\"\\ud83d\\n\"}", "{\"Unknown\":\"\u007f\u0080\"}", "{\"\\ud800\":1}", "{\"\\u0055nknown\":1}", "{\"Unknown\":{\"a\":\\}}", "{\"Unknown\":[]{}}", "{\"Unknown\":1}\u00a0",
            "{\"Unknown\":1}\f", "\ufeff{}", "{\"Unknown\":1\u000b}", "{\"Unknown\":1}\n\n", "\n\t {\"Unknown\":1}", "{\"Unknown\" :\t1 , \"Other\"\n:\r2}", "{\"a\":1,\"A\":2,\"a\":3}", "{\"\":1}", "{\"Unknown\":{\"\":{\"\":[]}}}",
        };

        // A text damaged at random: a piece dropped, doubled, or replaced by something JSON gives a meaning.
        private static string Damaged(string text, Random random)
        {
            string[] pieces = { "\"", "\\", "{", "}", "[", "]", ",", ":", "null", "true", "false", "1", "-", ".", "e", " ", "\n", "/*", "*/", "//", "\\u00e9", "\\ud800", "é", "\u2028", "0", "1e999", "\"x\"", "\t", "/" };
            var damaged = new StringBuilder(text);
            for (int edits = random.Next(1, 4); edits > 0 && damaged.Length > 0; edits--)
            {
                int at = random.Next(damaged.Length), length = Math.Min(random.Next(0, 6), damaged.Length - at);
                // Whole characters only: half of a pair would not be text.
                if (char.IsLowSurrogate(damaged[at]) || (at + length < damaged.Length && char.IsLowSurrogate(damaged[at + length]))) continue;
                switch (random.Next(4))
                {
                    case 0: damaged.Remove(at, length); break;
                    case 1: damaged.Insert(at, pieces[random.Next(pieces.Length)]); break;
                    case 2: damaged.Remove(at, length).Insert(at, pieces[random.Next(pieces.Length)]); break;
                    default: damaged.Insert(at, damaged.ToString(at, length)); break;
                }
            }
            return damaged.ToString();
        }

        [Fact]
        public void A_malformed_file_is_refused_or_read_the_same_both_ways()
        {
            var different = new List<string>();
            int asked = 0;
            void Ask(GalaxiaModel model, Type type, string text)
            {
                if (!GalaxiaNative.Crosses(text)) return;
                asked++;
                if (ReadBothWays(model, type, text) is { } difference) different.Add(difference);
            }

            foreach ((GalaxiaModel model, Type type) in Owned)
            {
                foreach (string text in Malformed) Ask(model, type, text);
                for (int depth = 60; depth <= 68; depth++)
                {
                    Ask(model, type, $"{{\"Unknown\":{new string('[', depth)}{new string(']', depth)}}}");
                    Ask(model, type, $"{new string('[', depth)}{new string(']', depth)}");
                    Ask(model, type, string.Concat(Enumerable.Repeat("{\"Unknown\":", depth)) + "1" + new string('}', depth));
                }

                // Every way a file can end early, and every one-character loss.
                var random = new Random(2000 + (int)model);
                string whole = JsonSerializer.Serialize(Random_(type, random, 0, nullable: false), type, ConfigJson.Options);
                string fresh = JsonSerializer.Serialize(Activator.CreateInstance(type), type, ConfigJson.Options);
                foreach (string text in new[] { whole, fresh })
                    for (int cut = 0; cut <= text.Length; cut += text.Length > 3000 ? 3 : 1)
                    {
                        if (cut < text.Length && char.IsLowSurrogate(text[cut])) continue;
                        Ask(model, type, text[..cut]);
                        if (cut < text.Length && !char.IsSurrogate(text[cut])) Ask(model, type, text.Remove(cut, 1));
                    }

                for (int i = 0; i < 4000; i++)
                {
                    string source = i % 4 == 0 ? fresh : JsonSerializer.Serialize(Random_(type, random, 0, nullable: false), type, ConfigJson.Options);
                    Ask(model, type, Damaged(source, random));
                }
            }
            Assert.True(asked > 25000, $"{asked} texts");
            AssertNone(different, $"of {asked} texts");
        }

        // The config directory each side of a file comparison uses, with the old place config sat beside it.
        private void Side(string side)
        {
            ConfigStore.OverrideDirectory = At(side, "etc");
            ConfigStore.OverrideLegacyDirectory = At(side, "legacy");
        }

        private List<string> Framed(string side) => Reported().Select(m => m.Replace(At(side), "<side>")).ToList();

        // One scenario run on each side with its own folders: what the file call answered, what it left on disk, and what it said.
        private void FileBothWays<T>(string name, Action<string> arrange, Func<ConfigFile<T>, bool, object?> act, string? category = null, string file = "settings.json") where T : class
        {
            var answers = new List<string>();
            var said = new List<List<string>>();
            foreach (string side in new[] { "managed", "native" })
            {
                string root = At(side + "-" + name);
                Directory.CreateDirectory(root);
                ConfigStore.OverrideDirectory = Path.Combine(root, "etc");
                ConfigStore.OverrideLegacyDirectory = Path.Combine(root, "legacy");
                arrange(root);
                Stamp(root);
                var config = category is null ? new ConfigFile<T>(file) : new ConfigFile<T>(category, file);
                object? answer = act(config, side == "native");
                string shown = answer switch { null => "null", bool flag => flag.ToString(), string text => text, _ => JsonSerializer.Serialize(answer, answer.GetType(), ConfigJson.Options) };
                string folders = string.Join(",", Directory.EnumerateDirectories(root, "*", SearchOption.AllDirectories).Select(d => Path.GetRelativePath(root, d)).OrderBy(d => d, StringComparer.Ordinal));
                answers.Add($"{shown} | error {(config.LastLoadError is null ? "none" : "said")} | {string.Join(" ; ", Listing(root).Select(JustWritten))} | folders {folders}");
                said.Add(Reported().Select(m => m.Replace(root, "<root>")).ToList());
            }
            Assert.True(answers[0] == answers[1], $"{name}:\n  C#: {answers[0]}\n  library: {answers[1]}");
            Assert.True(said[0].Count == said[1].Count, $"{name}: C# said {said[0].Count} things and the library {said[1].Count}");
            foreach ((string managed, string native) in said[0].Zip(said[1]))
            {
                // The frame is Galaxia's and exact; the words between are each implementation's own.
                Assert.StartsWith("<root>" + Path.DirectorySeparatorChar, managed);
                Assert.Equal(managed[..managed.IndexOf(": ", StringComparison.Ordinal)], native[..native.IndexOf(": ", StringComparison.Ordinal)]);
                Assert.EndsWith(" Falling back to defaults.", managed);
                Assert.EndsWith(" Falling back to defaults.", native);
            }
        }

        // A listing's line with the time of a file written during the test replaced, since the two sides write a moment apart; a copy keeps its source's time and is compared.
        private static string JustWritten(string line)
        {
            int at = line.LastIndexOf(" | ", StringComparison.Ordinal);
            return at > 0 && long.TryParse(line[(at + 3)..], out long ticks) && ticks > DateTime.UtcNow.AddMinutes(-10).Ticks ? line[..at] + " | just written" : line;
        }

        private static T? LoadSide<T>(ConfigFile<T> file, bool native) where T : class => native ? file.LoadNative(upgrade: false) : file.LoadManaged();

        private static bool SaveSide<T>(ConfigFile<T> file, bool native, T value) where T : class => native ? file.SaveNative(value) : file.SaveManaged(value);

        private void FileScenarios<T>(Func<Random, T> make, string good, string bad, T? unwritable = null) where T : class
        {
            var random = new Random(typeof(T).Name.Length);
            T value = make(random);
            string etc = "etc", legacy = "legacy";
            void Put_(string root, string folder, string name, string text) => Put(Path.Combine(root, folder, name), text);

            FileBothWays<T>("missing", _ => { }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("good", root => Put_(root, etc, "settings.json", good), (file, native) => LoadSide(file, native));
            FileBothWays<T>("bad", root => Put_(root, etc, "settings.json", bad), (file, native) => LoadSide(file, native));
            FileBothWays<T>("not json", root => Put_(root, etc, "settings.json", "{ not json at all"), (file, native) => LoadSide(file, native));
            FileBothWays<T>("null", root => Put_(root, etc, "settings.json", " null "), (file, native) => LoadSide(file, native));
            FileBothWays<T>("empty", root => Put_(root, etc, "settings.json", ""), (file, native) => LoadSide(file, native));
            FileBothWays<T>("a folder in its place", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => LoadSide(file, native));
            FileBothWays<T>("an error cleared by a good load", root => Put_(root, etc, "settings.json", bad), (file, native) =>
            {
                LoadSide(file, native);
                File.WriteAllText(file.Path, good);
                return LoadSide(file, native);
            });
            FileBothWays<T>("an error kept by a missing file", root => Put_(root, etc, "settings.json", bad), (file, native) =>
            {
                LoadSide(file, native);
                File.Delete(file.Path);
                return LoadSide(file, native);
            });
            FileBothWays<T>("from before Galaxia", root => Put_(root, legacy, "settings.json", good), (file, native) => LoadSide(file, native));
            FileBothWays<T>("from before Galaxia, twice", root => Put_(root, legacy, "settings.json", good), (file, native) => Json(LoadSide(file, native)) + Json(LoadSide(file, native)));
            FileBothWays<T>("both places", root => { Put_(root, legacy, "settings.json", bad); Put_(root, etc, "settings.json", good); }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("the old place, no copy possible", root => { Put_(root, legacy, "settings.json", good); Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")); }, (file, native) => LoadSide(file, native));
            FileBothWays<T>("a category never sat in the old place", root => Put_(root, Path.Combine(legacy, "group"), "settings.json", good), (file, native) => LoadSide(file, native), category: "group");
            FileBothWays<T>("a category, with a file of its name in the old place", root => Put_(root, legacy, "settings.json", good), (file, native) => LoadSide(file, native), category: "group");
            FileBothWays<T>("a category", root => Put_(root, Path.Combine(etc, "group"), "settings.json", good), (file, native) => LoadSide(file, native), category: "group");

            FileBothWays<T>("saved", _ => { }, (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved over", root => Put_(root, etc, "settings.json", bad), (file, native) => SaveSide(file, native, value) && SaveSide(file, native, value));
            FileBothWays<T>("saved in a category", _ => { }, (file, native) => SaveSide(file, native, value), category: "group");
            FileBothWays<T>("saved where a file blocks the folder", root => Put(Path.Combine(root, etc), "a file where the folder should be"), (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved over a folder", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => SaveSide(file, native, value));
            FileBothWays<T>("saved, then read by the other", _ => { }, (file, native) => SaveSide(file, native, value) ? LoadSide(file, !native) : null);
            FileBothWays<T>("saved null", _ => { }, (file, native) => SaveSide(file, native, null!));
            // What cannot be serialized is not saved, and its folder is made all the same.
            if (unwritable is not null) FileBothWays<T>("saved what cannot be written", _ => { }, (file, native) => SaveSide(file, native, unwritable), category: "group");

            FileBothWays<T>("deleted", root => Put_(root, etc, "settings.json", good), (file, native) => native ? file.DeleteNative() : file.DeleteManaged());
            FileBothWays<T>("deleted when absent", root => Directory.CreateDirectory(Path.Combine(root, etc)), (file, native) => native ? file.DeleteNative() : file.DeleteManaged());
            FileBothWays<T>("deleted with no folder", _ => { }, (file, native) => native ? file.DeleteNative() : file.DeleteManaged());
            FileBothWays<T>("deleted when a folder", root => Directory.CreateDirectory(Path.Combine(root, etc, "settings.json")), (file, native) => native ? file.DeleteNative() : file.DeleteManaged());
        }

        [Fact]
        public void A_models_file_is_loaded_saved_migrated_and_deleted_the_same_both_ways()
        {
            FileScenarios(random => (AudioConfig)Random_(typeof(AudioConfig), random, 0, false)!, "{ \"SampleRate\": 44100, // by hand\n \"mastervolume\": 0.8, }", "{\"SampleRate\":\"fast\"}",
                new AudioConfig { RateControlMaxDeviation = double.NaN });
            FileScenarios(random => (AppSettings)Random_(typeof(AppSettings), random, 0, false)!, "{\"SelectedCore\":\"SNES (Venus)\",\"OnlineCovers\":false,\"BigPicture\":{\"/themes/é\":{\"Variant\":\"dark\"}}}", "{\"Volume\":null}", new AppSettings { StickDeadzone = double.PositiveInfinity });
            FileScenarios(random => (GraphicsConfig)Random_(typeof(GraphicsConfig), random, 0, false)!, "{\"Consoles\":{\"N64\":{\"RenderScale\":\"2\"}},\"RecentShaders\":{\"SNES\":[\"crt\"]}}", "{\"Consoles\":[]}");
            FileScenarios(random => (CheatFile)Random_(typeof(CheatFile), random, 0, false)!, "{\"Cheats\":[{\"Space\":\"CpuBus\",\"Address\":\"7E0019\",\"Value\":\"01\"},{\"Kind\":\"RomPatch\",\"Writes\":[{\"Address\":\"8000\"}],\"Compare\":\"-\"}]}", "{\"Cheats\":{}}", new CheatFile { Cheats = { new CheatFileEntry { Writes = null! } } });
        }

        [Fact]
        public void A_file_whose_type_is_a_frontends_goes_through_the_same_file_rules()
        {
            FileScenarios(random => new Dictionary<string, Dictionary<string, int>> { ["NES"] = new() { ["Up"] = random.Next(100), ["é<>"] = -1 }, ["SNES Player 2"] = new() },
                "{\"NES\":{\"Up\":38,/* a key */\"B\":90,},}", "{\"NES\":{\"Up\":\"DPadUpp\"}}");

            // The text reaches C# decoded as .NET decodes a file, whatever mark it carries.
            foreach (Encoding encoding in new Encoding[] { new UTF8Encoding(true), Encoding.Unicode, Encoding.BigEndianUnicode, new UTF32Encoding(false, true), new UTF32Encoding(true, true) })
                FileBothWays<Dictionary<string, string>>("marked " + encoding.WebName + encoding.GetPreamble().Length,
                    root => { Directory.CreateDirectory(Path.Combine(root, "etc")); File.WriteAllBytes(Path.Combine(root, "etc", "settings.json"), encoding.GetPreamble().Concat(encoding.GetBytes("{\"é\":\"日本 😀\"}")).ToArray()); },
                    (file, native) => LoadSide(file, native));
            FileBothWays<Dictionary<string, string>>("not UTF-8",
                root => { Directory.CreateDirectory(Path.Combine(root, "etc")); File.WriteAllBytes(Path.Combine(root, "etc", "settings.json"), new byte[] { (byte)'{', (byte)'"', 0xFF, (byte)'"', (byte)':', (byte)'"', 0xC3, (byte)'"', (byte)'}' }); },
                (file, native) => LoadSide(file, native));
        }

        [Fact]
        public void Settings_are_upgraded_the_same_way_whether_read_or_made()
        {
            (string Name, string? File)[] cases =
            {
                ("no file", null), ("the legacy default", "{\"SelectedCore\":\"SNES (Venus)\",\"OnlineCovers\":false}"), ("the legacy default, chosen since", "{\"SelectedCore\":\"SNES (Venus)\",\"SelectedCoreUpgraded\":true}"),
                ("a choice", "{\"SelectedCore\":\"NES (Moon)\",\"OnlineCovers\":true,\"LogDirectory\":\"/logs/é\"}"), ("no core at all", "{\"SelectedCore\":null}"), ("a null document", "null"),
                ("a file that will not load", "{\"Volume\":\"loud\"}"), ("a different case", "{\"selectedcore\":\"SNES (Venus)\",\"SELECTEDCOREUPGRADED\":false}"),
            };
            foreach ((string name, string? file) in cases)
            {
                var loaded = new List<string>();
                var said = new List<int>();
                foreach (string side in new[] { "managed", "native" })
                {
                    Side(side + "-upgrade-" + name);
                    if (file is not null) Put(Path.Combine(ConfigStore.OverrideDirectory!, "appsettings.json"), file);
                    loaded.Add(Json(side == "native" ? AppSettings.LoadNative() : AppSettings.LoadManaged()));
                    said.Add(Reported().Count);
                }
                Assert.True(loaded[0] == loaded[1], $"{name}: {FirstDifference(loaded[0], loaded[1])}");
                Assert.True(said[0] == said[1], $"{name}: {said[0]} and {said[1]} things said");
                Assert.Contains("\"SelectedCoreUpgraded\": true", loaded[1]);
                Assert.DoesNotContain("OnlineCovers", loaded[1]);
            }
            Assert.Equal(Json(new AudioConfig()), Json(ConfigFile<AudioConfig>.NewNative(upgrade: false)));
            Assert.Equal(Json(new GraphicsConfig()), Json(ConfigFile<GraphicsConfig>.NewNative(upgrade: false)));
        }

        [Fact]
        public void A_cheat_list_goes_to_any_path_and_the_lists_are_named_the_same_both_ways()
        {
            var random = new Random(31);
            var lists = Enumerable.Range(0, 12).Select(_ => (CheatFile)Random_(typeof(CheatFile), random, 0, false)!).ToList();
            lists.Add(new CheatFile { Cheats = { new CheatFileEntry { Writes = null! } } });
            lists.Add(null!);
            for (int i = 0; i < lists.Count; i++)
            {
                string Path_(string side) => At(side, "exports", $"list {i} é", "deep", "mine.json");
                string OnDisk(string side) => File.Exists(Path_(side)) ? File.ReadAllText(Path_(side)) : Directory.Exists(Path.GetDirectoryName(Path_(side))) ? "no file, its folder made" : "nothing";
                bool savedManaged = CheatFile.Managed.SaveTo(Path_("managed"), lists[i]), savedNative = CheatFile.SaveToNative(Path_("native"), lists[i]);
                Assert.True(savedManaged == savedNative, $"list {i}: saved {savedManaged} and {savedNative}");
                Assert.True(OnDisk("managed") == OnDisk("native"), $"list {i}: {FirstDifference(OnDisk("managed"), OnDisk("native"))}");
                // Each reads what the other wrote, and what it wrote itself.
                foreach (string side in new[] { "managed", "native" })
                    Assert.True(Json(CheatFile.Managed.LoadFrom(Path_(side))) == Json(CheatFile.LoadFromNative(Path_(side))), $"list {i} read from {side}");
            }
            Assert.Equal("no file, its folder made", File.Exists(At("native", "exports", $"list {lists.Count - 2} é", "deep", "mine.json")) ? "a file" : Directory.Exists(At("native", "exports", $"list {lists.Count - 2} é", "deep")) ? "no file, its folder made" : "nothing");

            foreach (string unreadable in new[] { At("absent.json"), _temp, "" })
            {
                Assert.Null(CheatFile.Managed.LoadFrom(unreadable));
                Assert.Null(CheatFile.LoadFromNative(unreadable));
            }
            Assert.Equal(CheatFile.Managed.SaveTo("", lists[0]), CheatFile.SaveToNative("", lists[0]));
            // A path that is a folder: written straight, so nothing is left beside it, where a temp file and a rename would strand one.
            foreach (string side in new[] { "managed", "native" })
            {
                Directory.CreateDirectory(At(side, "taken", "list.json"));
                Assert.False(side == "native" ? CheatFile.SaveToNative(At(side, "taken", "list.json"), lists[0]) : CheatFile.Managed.SaveTo(At(side, "taken", "list.json"), lists[0]));
                Assert.Equal(new[] { At(side, "taken", "list.json") }, Directory.GetFileSystemEntries(At(side, "taken")));
            }

            string[] names = { "zelda", "Mario", "alttp", "ALTTP2", "_x", "é", "É2", "a.b", "", "Z", "[bracket", "9lives", "日本", "😀", "~tilde", "a b", "A", "a" };
            foreach (string side in new[] { "managed", "native" })
            {
                Side(side + "-names");
                Assert.Empty(side == "native" ? CheatFile.ListNamesNative() : CheatFile.Managed.ListNames());
                string folder = Path.Combine(ConfigStore.OverrideDirectory!, CheatFile.CategoryDirName);
                foreach (string name in names) Put(Path.Combine(folder, name + ".json"), "{}");
                Put(Path.Combine(folder, "notes.txt"), "x");
                Put(Path.Combine(folder, "upper.JSON"), "x");
                Directory.CreateDirectory(Path.Combine(folder, "a folder.json"));
                if (!OperatingSystem.IsWindows()) File.CreateSymbolicLink(Path.Combine(folder, "dangling.json"), At("nowhere"));
            }
            Side("managed-names");
            IReadOnlyList<string> managed = CheatFile.Managed.ListNames();
            Side("native-names");
            IReadOnlyList<string> native = CheatFile.ListNamesNative();
            Assert.Equal(managed, native);
            Assert.Contains("dangling", native);
            Assert.DoesNotContain("a folder", native);
            Assert.True(native.Count >= names.Length - 2, string.Join(",", native));

            var different = new List<string>();
            for (int c = 0; c < 0x3000; c++)
            {
                if (char.IsSurrogate((char)c)) continue;
                foreach (string name in new[] { ((char)c).ToString(), "a" + (char)c + "b" })
                    if (CheatFile.Managed.IsValidName(name) != GalaxiaNative.CheatNameValid(name)) different.Add($"U+{c:X4}");
            }
            foreach (string? name in new[] { null, "", " ", ".", "..", "...", "a/b", "Zelda (U) [!]", "\t", "con", "a\\b" })
                if (CheatFile.Managed.IsValidName(name!) != GalaxiaNative.CheatNameValid(name)) different.Add($"'{name}'");
            AssertNone(different, "names");
        }

        private static string? WriteSide(bool native, DateTime now, string level, string area, string message, Exception? fault, string? context) =>
            native ? ErrorLog.WriteNative(now, level, area, message, fault, context) : ErrorLog.Managed.Write(now, level, area, message, fault, context);

        // One sequence of log calls run on each side in its own folder; the answers and the folder are compared.
        private void LogBothWays(string name, Action<string> arrange, Func<bool, string, object?> act)
        {
            var results = new List<string>();
            foreach (string side in new[] { "managed", "native" })
            {
                string folder = At(side + "-log-" + name, "Logs");
                ErrorLog.DirectoryOverride = folder;
                ErrorLog.ResetForTests();
                arrange(folder);
                object? answer = act(side == "native", folder);
                string listing = !Directory.Exists(folder) ? "no folder" : string.Join(" ; ", Directory.EnumerateFiles(folder).Select(f => $"{Path.GetFileName(f)}={Held(f)}").OrderBy(l => l, StringComparer.Ordinal));
                results.Add($"{(answer as string)?.Replace(folder, "<logs>") ?? answer ?? "null"} | {listing}");
            }
            Assert.True(results[0] == results[1], $"{name}:\n  C#: {results[0]}\n  library: {results[1]}");
        }

        // A log file by what it holds, briefly; a link that names nothing holds nothing.
        private static string Held(string file)
        {
            try { byte[] bytes = File.ReadAllBytes(file); return $"{Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))[..16]}/{bytes.Length}"; }
            catch (FileNotFoundException) { return "a dangling link"; }
        }

        [Fact]
        public void An_error_is_logged_as_the_same_bytes_in_the_same_file_both_ways()
        {
            var now = new DateTime(2026, 10, 8, 1, 2, 3, 4, DateTimeKind.Local);
            Exception fault;
            try { throw new InvalidDataException("<size> is \"w 0.02\"\r\nnot a pair", new IOException("inner cause é")); }
            catch (Exception caught) { fault = caught; }

            LogBothWays("plain", _ => { }, (native, _) => WriteSide(native, now, "ERROR", "themes", "A theme download failed", null, null));
            LogBothWays("whole", _ => { }, (native, _) => WriteSide(native, now, "ERROR", "themes", "failed\non two lines", fault, "Artflix (Revisited)\r\n(https://x)"));
            LogBothWays("every line ending", _ => { }, (native, _) => WriteSide(native, now, "WARN", "a\nb", "a\r\nb\nc\rd\fe\u0085f\u2028g\u2029h\vi\r\r\n\n", null, "\u2028\u0085"));
            LogBothWays("blank contexts", _ => { }, (native, _) => string.Concat(new[] { "", " ", "\t\n", "\u00a0" }.Select(context => WriteSide(native, now, "WARN", "config", "m", null, context))));
            LogBothWays("no area", _ => { }, (native, _) => WriteSide(native, now, "WARN", null!, "m", null, null));
            LogBothWays("no message", _ => { }, (native, _) => WriteSide(native, now, "WARN", "config", null!, null, null));
            LogBothWays("text that is not valid", _ => { }, (native, _) => WriteSide(native, now, "WARN", "config", "half \ud83d a pair", new Exception("\udc00"), "\ud800\ud800"));
            LogBothWays("appended", _ => { }, (native, _) => WriteSide(native, now, "WARN", "config", "first", null, null) + WriteSide(native, now.AddSeconds(1), "ERROR", "launch", "second é 😀", fault, null));
            LogBothWays("another day", _ => { }, (native, _) => WriteSide(native, now, "WARN", "config", "first", null, null) + WriteSide(native, now.AddDays(1), "ERROR", "launch", "second", null, null));

            ErrorLog.Redactor = text => text.Replace("FAKEERRORLOGPW", "***");
            LogBothWays("redacted", _ => { }, (native, _) => WriteSide(native, now, "ERROR", "scraping", "GET ...&devpassword=FAKEERRORLOGPW", new InvalidOperationException("devpassword=FAKEERRORLOGPW"), "ctx FAKEERRORLOGPW"));
            ErrorLog.Redactor = _redactor;

            void Old(string folder)
            {
                Directory.CreateDirectory(folder);
                (string Name, int Days)[] files = { ("emusen_20000101.log", 15), ("emusen_recent.log", 3), ("emusen_.log", 20), ("emusen_edge.log", 14), ("emusen_old.txt", 30), ("crash_20000101.txt", 400), ("EMUSEN_upper.log", 30), ("emusen_x.LOG", 30), ("xemusen_.log", 30) };
                foreach ((string file, int days) in files)
                {
                    File.WriteAllText(Path.Combine(folder, file), "x");
                    File.SetLastWriteTime(Path.Combine(folder, file), now.AddDays(-days).AddMinutes(days == 14 ? 30 : 0));
                }
                Directory.CreateDirectory(Path.Combine(folder, "emusen_folder.log"));
                if (OperatingSystem.IsWindows()) return;
                File.CreateSymbolicLink(Path.Combine(folder, "emusen_dangling.log"), At("nowhere"));
                // A new link to an old file, and an old file's link: which of the two times decides.
                string target = Path.Combine(Path.GetDirectoryName(folder)!, "target.log");
                File.WriteAllText(target, "x");
                File.SetLastWriteTime(target, now.AddDays(-30));
                File.CreateSymbolicLink(Path.Combine(folder, "emusen_linked.log"), target);
            }
            LogBothWays("pruned", Old, (native, _) => WriteSide(native, now, "WARN", "test", "prune", null, null));
            LogBothWays("pruned once", Old, (native, folder) =>
            {
                WriteSide(native, now, "WARN", "test", "first", null, null);
                File.WriteAllText(Path.Combine(folder, "emusen_later.log"), "x");
                File.SetLastWriteTime(Path.Combine(folder, "emusen_later.log"), now.AddDays(-30));
                return WriteSide(native, now, "WARN", "test", "second", null, null);
            });

            foreach (int room in new[] { 0, 1, 30, 49, 50, 51, 52, 53, 54, 60, 200 })
                LogBothWays($"with {room} bytes left", folder => { Directory.CreateDirectory(folder); File.WriteAllText(Path.Combine(folder, $"emusen_{now:yyyyMMdd}.log"), new string('x', (int)ErrorLog.MaxBytes - room)); },
                    (native, _) => WriteSide(native, now, "ERROR", "test", "tips it é 😀", null, null) + "/" + WriteSide(native, now, "ERROR", "test", "refused?", null, null));

            LogBothWays("a file where the folder should be", folder => Put(folder, "a file"), (native, _) => WriteSide(native, now, "ERROR", "test", "nowhere to go", null, null));
            LogBothWays("the day's file is a folder", folder => Directory.CreateDirectory(Path.Combine(folder, $"emusen_{now:yyyyMMdd}.log")), (native, _) => WriteSide(native, now, "ERROR", "test", "m", null, null));
            LogBothWays("the path for a day", _ => { }, (native, folder) => native ? GalaxiaNative.LogPath(GalaxiaNative.LogRoot(folder), $"{now:yyyyMMdd}") : ErrorLog.Managed.PathFor(now));
        }

        [Fact]
        public void The_log_folder_is_chosen_the_same_way_both_ways()
        {
            ErrorLog.DirectoryOverride = null;
            Put(At("blocker"), "a file");
            (string Name, string? Settings)[] cases =
            {
                ("no settings", null), ("no folder named", "{}"), ("a folder that can be made", $"{{\"LogDirectory\":{JsonSerializer.Serialize(At("made", "Logs é"))}}}"),
                ("a folder that cannot", $"{{\"LogDirectory\":{JsonSerializer.Serialize(At("blocker", "Logs"))}}}"), ("a blank folder", "{\"LogDirectory\":\"  \"}"), ("settings that will not load", "{\"LogDirectory\":5}"),
            };
            foreach ((string name, string? settings) in cases)
            {
                var roots = new List<string>();
                var said = new List<int>();
                foreach (string side in new[] { "managed", "native" })
                {
                    Side(side + "-root-" + name);
                    if (settings is not null) Put(Path.Combine(ConfigStore.OverrideDirectory!, "appsettings.json"), settings);
                    roots.Add(side == "native" ? GalaxiaNative.LogRoot(null) : ErrorLog.Managed.Root());
                    said.Add(Reported().Count);
                }
                Assert.True(roots[0] == roots[1], $"{name}: {roots[0]} and {roots[1]}");
                Assert.True(said[0] == said[1], $"{name}: {said[0]} and {said[1]} things said");
            }
            Assert.True(Directory.Exists(At("made", "Logs é")));
            Assert.Equal(ErrorLog.Managed.DefaultRoot, GalaxiaNative.Directory(GalaxiaDirectory.LogDefault));
            Assert.Equal(At("given"), GalaxiaNative.LogRoot(At("given")));

            foreach (string? directory in new[] { null, "", " ", "\t", At("usable", "a", "b"), At("blocker", "under"), At("blocker") })
            {
                bool managed = ErrorLog.Managed.Usable(directory);
                if (directory is not null && Directory.Exists(At("usable"))) Directory.Delete(At("usable"), recursive: true);
                Assert.True(managed == GalaxiaNative.LogUsable(directory), $"Usable({directory})");
            }
        }

        [Fact]
        public void A_suggestion_is_the_same_sentence_both_ways()
        {
            string[] alphabet = { "a", "b", "c", "A", "B", "e", "t", "h", "é", "É", "ß", "İ", "ı", "K", "\u212a", "σ", "ς", "Σ", "😀", "\ud801\udc00", "\ud801\udc28", "1", "_", "-", " ", "日", "ǅ", "ǆ" };
            var random = new Random(77);
            string Word(int most) => string.Concat(Enumerable.Range(0, random.Next(0, most)).Select(_ => alphabet[random.Next(random.Next(3) == 0 ? alphabet.Length : 8)]));
            var different = new List<string>();
            for (int i = 0; i < 6000; i++)
            {
                string typed = Word(9);
                string[] candidates = Enumerable.Range(0, random.Next(0, 9)).Select(_ => random.Next(4) == 0 ? Damaged(typed.Length == 0 ? "x" : typed, random) : Word(9)).Where(c => GalaxiaNative.Crosses(c, listed: true)).ToArray();
                int max = random.Next(-1, 5);
                string a = Word(9), b = random.Next(3) == 0 ? a.ToUpperInvariant() : Word(9);
                if (Suggestion.Managed.Distance(a, b) != GalaxiaNative.SuggestDistance(a, b)) different.Add($"distance '{a}' '{b}'");
                string managed = Suggestion.Managed.Hint(typed, candidates, max), native = GalaxiaNative.SuggestHint(typed, candidates, max);
                if (managed != native) different.Add($"hint '{typed}' among [{string.Join("|", candidates)}] max {max}: [{managed}] and [{native}]");
                if (!Suggestion.Managed.Nearest(typed, candidates, max).SequenceEqual(GalaxiaNative.SuggestNearest(typed, candidates, max))) different.Add($"nearest '{typed}' among [{string.Join("|", candidates)}] max {max}");
            }
            for (int c = 1; c < 0x10000; c++)
            {
                if (char.IsSurrogate((char)c)) continue;
                string text = ((char)c).ToString();
                foreach (string other in new[] { text.ToUpperInvariant(), text.ToLowerInvariant(), "a", "i", "I", "k", "K", "s", "S" })
                    if (Suggestion.Managed.Distance(text, other) != GalaxiaNative.SuggestDistance(text, other)) different.Add($"distance U+{c:X4} '{other}'");
            }
            AssertNone(different, "suggestions");

            string[] commands = { "cheat", "cat", "clear", "cls" };
            Assert.Equal(" Did you mean 'cheat'?", GalaxiaNative.SuggestHint("chaet", commands, 2));
            Assert.Equal(Suggestion.Managed.Hint("cl", commands, 2), GalaxiaNative.SuggestHint("cl", commands, 2));
        }

        [Fact]
        public void Names_are_ordered_as_OrdinalIgnoreCase_orders_them()
        {
            string[] alphabet = { "a", "A", "b", "B", "z", "Z", "_", "[", "@", "`", "0", "9", " ", "é", "É", "ß", "ı", "I", "i", "日", "😀", "\ud801\udc00", "\ud801\udc28", "\ue000", "\uffff", "~" };
            var random = new Random(5);
            var different = new List<string>();
            foreach (string side in new[] { "managed", "native" }) Directory.CreateDirectory(At(side + "-order", "etc", CheatFile.CategoryDirName));
            for (int round = 0; round < 60; round++)
            {
                var names = Enumerable.Range(0, random.Next(2, 14)).Select(_ => string.Concat(Enumerable.Range(0, random.Next(1, 5)).Select(_ => alphabet[random.Next(alphabet.Length)]))).Distinct().ToList();
                var listed = new List<IReadOnlyList<string>>();
                foreach (string side in new[] { "managed", "native" })
                {
                    string folder = At(side + "-order", "etc", CheatFile.CategoryDirName);
                    foreach (string file in Directory.GetFiles(folder)) File.Delete(file);
                    foreach (string name in names) File.WriteAllText(Path.Combine(folder, name + ".json"), "{}");
                    ConfigStore.OverrideDirectory = At(side + "-order", "etc");
                    listed.Add(side == "native" ? CheatFile.ListNamesNative() : CheatFile.Managed.ListNames());
                }
                // Names that differ only by case sort by the folder's own order, which two folders need not share.
                if (!listed[0].SequenceEqual(listed[1], StringComparer.OrdinalIgnoreCase)) different.Add($"[{string.Join("|", listed[0])}] and [{string.Join("|", listed[1])}]");
            }
            AssertNone(different, "orders");
        }
    }
}
