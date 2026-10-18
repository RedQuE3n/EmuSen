using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Managed;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // emusen_crystal.h against the binding and the C# implementation: every export, number and struct field, both mirrors - see EmuSen_Backend.md §14.
    public class CrystalHeaderTests
    {
        private static string Repo => ConfigRoot.Managed.Directory;
        private static string Include => Path.Combine(Repo, "EmuSen.Crystal", "include");
        private static string Header => File.ReadAllText(Path.Combine(Include, "emusen_crystal.h"));
        private static string SessionHeader => File.ReadAllText(Path.Combine(Include, "emusen_session.h"));

        private static string Code(string header) =>
            string.Join("\n", Regex.Replace(header, @"/\*.*?\*/", "", RegexOptions.Singleline).Split('\n').Where(l => !l.TrimStart().StartsWith('#')));

        // Each export the header declares, in order: its name, its return type and its parameters' types.
        public static List<(string Name, string Return, string[] Parameters)> Exports()
        {
            var found = new List<(string, string, string[])>();
            foreach (string raw in Code(Header).Replace('\n', ' ').Split(';'))
            {
                string statement = raw.Trim();
                int open = statement.IndexOf('(');
                if (open < 0 || statement.Contains("typedef") || statement.Contains('{')) continue;
                Match head = Regex.Match(statement[..open].Trim(), @"^(.*?)\s*\**\s*(emusen_crystal_\w+)$");
                if (!head.Success) continue;
                string prefix = statement[..open].Trim();
                string returns = Normal(prefix[..^head.Groups[2].Value.Length]);
                string list = statement[(open + 1)..statement.LastIndexOf(')')];
                string[] parameters = list.Trim() == "void" ? Array.Empty<string>() : list.Split(',').Select(p => Normal(Regex.Replace(p.Trim(), @"\w+$", ""))).ToArray();
                found.Add((head.Groups[2].Value, returns, parameters));
            }
            return found;
        }

        private static string Normal(string type) => Regex.Replace(Regex.Replace(type.Trim(), @"\s+", " "), @"\s*\*", " *").Trim();

        private static readonly Dictionary<string, Type> Client = new()
        {
            ["void"] = typeof(void), ["uint32_t"] = typeof(uint), ["int32_t"] = typeof(int), ["int64_t"] = typeof(long), ["uint64_t"] = typeof(ulong),
            ["size_t"] = typeof(nuint), ["double"] = typeof(double), ["uint8_t *"] = typeof(byte*), ["const uint8_t *"] = typeof(byte*),
            ["int16_t *"] = typeof(short*), ["uint32_t *"] = typeof(uint*), ["double *"] = typeof(double*), ["emusen_crystal_runtime *"] = typeof(nint),
            ["emusen_crystal_subscription *"] = typeof(nint), ["const emusen_session_v1 *"] = typeof(nint),
            ["const emusen_crystal_config *"] = typeof(CrystalConfig*), ["const emusen_crystal_open_params *"] = typeof(CrystalOpenParams*),
            ["emusen_crystal_frame_info *"] = typeof(CrystalFrameInfo*),
        };

        private static Type ManagedType(string c) => c switch
        {
            "const emusen_crystal_config *" => typeof(ManagedCrystal).Assembly.GetType("EmuSen.Crystal.Managed.ConfigV1")!.MakePointerType(),
            "const emusen_crystal_open_params *" => typeof(ManagedCrystal).Assembly.GetType("EmuSen.Crystal.Managed.OpenParamsV1")!.MakePointerType(),
            "emusen_crystal_frame_info *" => typeof(ManagedCrystal).Assembly.GetType("EmuSen.Crystal.Managed.FrameInfoV1")!.MakePointerType(),
            _ => Client[c],
        };

        private static string Pascal(string name) => string.Concat(name["emusen_crystal_".Length..].Split('_').Select(w => char.ToUpperInvariant(w[0]) + w[1..]));

        [Fact]
        public void The_header_declares_the_exports_this_test_reads()
        {
            var exports = Exports();
            Assert.Equal(132, exports.Count);
            Assert.Equal(exports.Count, exports.Select(e => e.Name).Distinct().Count());
            Assert.All(exports, e => Assert.All(e.Parameters.Append(e.Return), t => Assert.True(Client.ContainsKey(t), $"{e.Name}: a type the test does not know: '{t}'")));
        }

        // The client's table is the header's exports field for field, in its order, each a function pointer of the header's signature.
        [Fact]
        public void The_binding_s_table_is_the_header_s_exports()
        {
            var exports = Exports();
            FieldInfo[] fields = typeof(CrystalExports).GetFields();
            Assert.Equal(exports.Select(e => e.Name["emusen_crystal_".Length..]), fields.Select(f => f.Name));
            foreach (var (field, declared) in fields.Zip(exports))
            {
                Type pointer = field.GetModifiedFieldType();
                Assert.True(pointer.IsUnmanagedFunctionPointer, field.Name);
                Assert.Equal(Client[declared.Return], pointer.GetFunctionPointerReturnType().UnderlyingSystemType);
                Assert.Equal(declared.Parameters.Select(p => Client[p]), pointer.GetFunctionPointerParameterTypes().Select(p => p.UnderlyingSystemType));
                Assert.Equal(fields.ToList().IndexOf(field) * IntPtr.Size, (int)Marshal.OffsetOf<CrystalExports>(field.Name));
            }
        }

        // One contract function per export, named for it, taking and returning what the header says.
        [Fact]
        public void The_contract_layer_has_one_function_per_export()
        {
            MethodInfo[] functions = typeof(EmuSen.Crystal.Client.Contract.Crystal).GetMethods(BindingFlags.Public | BindingFlags.Static);
            Assert.Equal(Exports().Select(e => Pascal(e.Name)).OrderBy(n => n), functions.Select(m => m.Name).OrderBy(n => n));
            foreach (var (name, returns, parameters) in Exports())
            {
                MethodInfo m = functions.Single(f => f.Name == Pascal(name));
                Assert.Equal(Client[returns], m.ReturnType);
                Assert.Equal(parameters.Select(p => Client[p]), m.GetParameters().Select(p => p.ParameterType));
            }
        }

        // The C# implementation exports every function of the header, each an UnmanagedCallersOnly entry of its signature.
        [Fact]
        public void The_C_sharp_implementation_exports_every_function_with_its_signature()
        {
            foreach (var (name, returns, parameters) in Exports())
            {
                Assert.NotEqual(0, ManagedCrystal.Export(name));
                MethodInfo m = typeof(ManagedCrystal).GetMethod(name, BindingFlags.NonPublic | BindingFlags.Static) ?? throw new Xunit.Sdk.XunitException($"no entry {name}");
                Assert.NotNull(m.GetCustomAttribute<UnmanagedCallersOnlyAttribute>());
                Assert.Equal(ManagedType(returns), m.ReturnType);
                Assert.Equal(parameters.Select(ManagedType), m.GetParameters().Select(p => p.ParameterType));
            }
            Assert.Equal(0, ManagedCrystal.Export("emusen_crystal_nothing"));
        }

        private static Dictionary<string, long> Defines(string header, string pattern) =>
            Regex.Matches(header, @"^#define (" + pattern + @")\s+\(?(-?\d+)u?\)?\s*(?:/\*.*)?$", RegexOptions.Multiline).ToDictionary(m => m.Groups[1].Value, m => long.Parse(m.Groups[2].Value));

        private static string PascalOf(string name, string prefix) => string.Concat(Regex.Replace(name, "^" + prefix, "").ToLowerInvariant().Split('_').Select(p => char.ToUpperInvariant(p[0]) + p[1..]));

        private static Dictionary<string, long> Constants(Type type) =>
            type.GetFields(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static).Where(f => f.IsLiteral && f.Name != "Version").ToDictionary(f => f.Name, f => Convert.ToInt64(f.GetRawConstantValue()));

        // Both sides' numbers are the header's: the binding's two classes and the implementation's.
        [Fact]
        public void Every_number_the_headers_define_is_both_mirrors()
        {
            var crystal = Defines(Header, @"EMUSEN_CRYSTAL_\w+").Where(d => !d.Key.EndsWith("_VERSION")).ToDictionary(d => PascalOf(d.Key, "EMUSEN_CRYSTAL_"), d => d.Value);
            var session = Defines(SessionHeader, @"EMUSEN_(?:SVC|SESSION)_\w+").Where(d => !d.Key.EndsWith("_VERSION")).ToDictionary(d => PascalOf(d.Key, "EMUSEN_(SVC|SESSION)_"), d => d.Value);
            Assert.Equal(crystal.OrderBy(d => d.Key), Constants(typeof(CrystalAbi)).OrderBy(d => d.Key));
            Assert.Equal(session.OrderBy(d => d.Key), Constants(typeof(CrystalSessionAbi)).OrderBy(d => d.Key));
            Assert.Equal(crystal.OrderBy(d => d.Key), Constants(typeof(ManagedCrystal).Assembly.GetType("EmuSen.Crystal.Managed.CrystalNumbers")!).OrderBy(d => d.Key));
            Assert.Equal((CrystalAbi.Major << 16) | CrystalAbi.Minor, CrystalAbi.Version);
        }

        private static readonly (string C, Type Client, string Managed)[] Structs =
        {
            ("emusen_crystal_config", typeof(CrystalConfig), "ConfigV1"),
            ("emusen_crystal_open_params", typeof(CrystalOpenParams), "OpenParamsV1"),
            ("emusen_crystal_frame_info", typeof(CrystalFrameInfo), "FrameInfoV1"),
        };

        // Each struct as a C compiler lays it out from the header, and both mirrors laid out alike, field by field by name.
        [Fact]
        public void A_C_compiler_lays_the_structs_out_as_both_mirrors_do()
        {
            string? cc = new[] { "cc", "clang", "gcc" }.FirstOrDefault(p => (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator).Any(d => File.Exists(Path.Combine(d, p))));
            if (cc is null) return;
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenCrystalHeader", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                string source = "#include <stdio.h>\n#include <stddef.h>\n#include \"emusen_crystal.h\"\nint main(void) {\n"
                    + string.Concat(Structs.Select(s => $"    printf(\"{s.C} %lu\\n\", (unsigned long)sizeof({s.C}));\n"
                        + string.Concat(s.Client.GetFields().Select(f => $"    printf(\"{s.C}.{f.Name} %lu\\n\", (unsigned long)offsetof({s.C}, {f.Name}));\n"))))
                    + "    return 0;\n}\n";
                File.WriteAllText(Path.Combine(dir, "layout.c"), source);
                string exe = Path.Combine(dir, "layout");
                Run(cc, $"-std=c99 -Wall -Wextra -Werror -pedantic -I\"{Include}\" -o \"{exe}\" \"{Path.Combine(dir, "layout.c")}\"");
                var printed = Run(exe, "").Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(l => l.Split(' ')).ToDictionary(p => p[0], p => int.Parse(p[1]));
                foreach (var (c, client, managedName) in Structs)
                {
                    Type managed = typeof(ManagedCrystal).Assembly.GetType("EmuSen.Crystal.Managed." + managedName)!;
                    Assert.Equal(printed[c], Marshal.SizeOf(client));
                    Assert.Equal(printed[c], Marshal.SizeOf(managed));
                    Assert.Equal(client.GetFields().Select(f => f.Name), managed.GetFields().Select(f => f.Name));
                    foreach (FieldInfo f in client.GetFields())
                    {
                        Assert.Equal(printed[$"{c}.{f.Name}"], (int)Marshal.OffsetOf(client, f.Name));
                        Assert.Equal(printed[$"{c}.{f.Name}"], (int)Marshal.OffsetOf(managed, f.Name));
                    }
                }
            }
            finally
            {
                try { Directory.Delete(dir, recursive: true); } catch (IOException) { }
            }
        }

        // C++17 takes the header as cleanly as C99 does.
        [Fact]
        public void The_header_compiles_as_C_plus_plus_17()
        {
            string? cxx = new[] { "c++", "clang++", "g++" }.FirstOrDefault(p => (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator).Any(d => File.Exists(Path.Combine(d, p))));
            if (cxx is null) return;
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenCrystalHeader", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                File.WriteAllText(Path.Combine(dir, "probe.cpp"), "#include \"emusen_crystal.h\"\nint main() { emusen_crystal_frame_info f = emusen_crystal_frame_info(); f.size = sizeof f; return static_cast<int>(f.ran); }\n");
                Run(cxx, $"-std=c++17 -Wall -Wextra -Werror -pedantic -I\"{Include}\" -c \"{Path.Combine(dir, "probe.cpp")}\" -o \"{Path.Combine(dir, "probe.o")}\"");
            }
            finally
            {
                try { Directory.Delete(dir, recursive: true); } catch (IOException) { }
            }
        }

        private static string Run(string program, string arguments)
        {
            using var process = Process.Start(new ProcessStartInfo(program, arguments) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false })!;
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();
            Assert.True(process.ExitCode == 0, $"{program} {arguments}\n{error}");
            return output;
        }
    }
}
