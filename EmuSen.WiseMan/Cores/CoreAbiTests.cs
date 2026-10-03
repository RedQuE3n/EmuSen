using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using EmuSen.Cores.Native;

namespace EmuSen.WiseMan.Cores
{
    // §5.3's check: the C# table and structs of the core ABI v1 against the baseline the header was recorded in - see EmuSen_CoreAPI.md §5.3, §18.
    public unsafe class CoreAbiTests
    {
        private static readonly string BaselinePath = Path.Combine(AppContext.BaseDirectory, "Cores", "CoreAbiBaseline.txt");

        private static IEnumerable<string[]> Lines(string kind) =>
            File.ReadLines(BaselinePath).Where(l => l.StartsWith(kind + " ")).Select(l => l.Split(' '));

        private static readonly Dictionary<string, Type> Structs = new()
        {
            ["emusen_file"] = typeof(CoreInterface.File),
            ["emusen_create_params"] = typeof(CoreInterface.CreateParams),
            ["emusen_frame_info"] = typeof(CoreInterface.FrameInfo),
            ["emusen_event"] = typeof(CoreInterface.Event),
        };

        // A C type as the baseline spells it, as the C# a host must declare it with.
        private static Type CSharp(string c)
        {
            c = c.Replace("const ", "");
            if (c == "emusen_machine*") return typeof(nint);
            if (c.EndsWith('*')) return c == "char*" ? typeof(byte*) : CSharp(c[..^1]).MakePointerType();
            return c switch
            {
                "void" => typeof(void),
                "uint8_t" => typeof(byte),
                "int16_t" => typeof(short),
                "int32_t" => typeof(int),
                "uint32_t" => typeof(uint),
                "int64_t" => typeof(long),
                "uint64_t" => typeof(ulong),
                "size_t" => typeof(nuint),
                "double" => typeof(double),
                _ => Structs.TryGetValue(c, out var t) ? t : throw new ArgumentException($"no C# type for {c}"),
            };
        }

        private static string Name(Type t) => t.UnderlyingSystemType.ToString();

        // Every disagreement between a table's function-pointer fields and the baseline's exports, in words.
        public static List<string> Compare(Type table, IEnumerable<string[]> functions)
        {
            var problems = new List<string>();
            foreach (var w in functions)
            {
                string export = w[1];
                string sig = string.Join(' ', w[2..^1]);
                int arrow = sig.LastIndexOf(" -> ", StringComparison.Ordinal);
                string args = sig[1..sig.LastIndexOf(')', arrow)];
                Type ret = CSharp(sig[(arrow + 4)..]);
                Type[] want = args.Length == 0 ? Type.EmptyTypes : args.Split(", ").Select(CSharp).ToArray();
                FieldInfo? field = table.GetField(CoreInterface.FieldFor(export));
                if (field is null)
                {
                    problems.Add($"{export}: no field {CoreInterface.FieldFor(export)}");
                    continue;
                }
                Type fn = field.GetModifiedFieldType();
                Type[] have = fn.GetFunctionPointerParameterTypes();
                if (Name(fn.GetFunctionPointerReturnType()) != Name(ret)) problems.Add($"{export}: returns {Name(fn.GetFunctionPointerReturnType())}, the header {Name(ret)}");
                if (!have.Select(Name).SequenceEqual(want.Select(Name)))
                    problems.Add($"{export}: takes ({string.Join(", ", have.Select(Name))}), the header ({string.Join(", ", want.Select(Name))})");
            }
            return problems;
        }

        [Fact]
        public void Every_export_of_the_baseline_has_its_field_with_the_headers_signature()
        {
            var functions = Lines("fn").ToList();
            Assert.Equal(55, functions.Count);
            Assert.Empty(Compare(typeof(CoreInterface), functions));
            var fields = typeof(CoreInterface).GetFields().Where(f => !f.IsLiteral && f.FieldType.IsFunctionPointer).Select(f => f.Name).ToHashSet();
            Assert.Equal(functions.Select(w => CoreInterface.FieldFor(w[1])).ToHashSet(), fields);
        }

        [Fact]
        public void Every_struct_has_the_headers_size_and_every_field_its_offset_and_type()
        {
            foreach (var w in Lines("struct"))
                Assert.True(Marshal.SizeOf(Structs[w[1]]) == int.Parse(w[3]), $"{w[1]} is {Marshal.SizeOf(Structs[w[1]])} bytes, the header {w[3]}");
            int checkedFields = 0;
            foreach (var w in Lines("field"))
            {
                var parts = w[1].Split('.');
                Type s = Structs[parts[0]];
                string name = CoreInterface.FieldFor(parts[1]);
                FieldInfo f = s.GetField(name) ?? throw new Xunit.Sdk.XunitException($"{w[1]}: no field {name}");
                Assert.True(Marshal.OffsetOf(s, name).ToInt32() == int.Parse(w[3]), $"{w[1]} at {Marshal.OffsetOf(s, name)}, the header {w[3]}");
                Assert.True(f.FieldType == CSharp(string.Join(' ', w[5..^1])), $"{w[1]} is {f.FieldType}, the header {string.Join(' ', w[5..^1])}");
                checkedFields++;
            }
            Assert.Equal(Structs.Values.Sum(t => t.GetFields().Length), checkedFields);
            Assert.Contains(Lines("const"), w => w[1] == "EMUSEN_CORE_ABI_VERSION" && Convert.ToUInt32(w[2], 16) == CoreInterface.Version);
        }

        private sealed class PreStableCrashLogDeclaration
        {
            public readonly delegate* unmanaged<nint, void> SetCrashLog;
        }

        // The pre-stable host calls the crash-log export as returning void (NativeCoreLibrary.cs), and the check finds it - see EmuSen_CoreAPI.md §1.2 item 8.
        [Fact]
        public void The_check_finds_a_declaration_whose_return_disagrees_with_the_header()
        {
            var problems = Compare(typeof(PreStableCrashLogDeclaration), Lines("fn").Where(w => w[1] == "emusen_core_set_crash_log"));
            Assert.Equal(new[] { "emusen_core_set_crash_log: returns System.Void, the header System.Int32", "emusen_core_set_crash_log: takes (System.IntPtr), the header (System.Byte*)" }, problems);
        }
    }
}
