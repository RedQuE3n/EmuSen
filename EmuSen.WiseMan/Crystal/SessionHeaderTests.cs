using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // emusen_session.h is the service's machine-checked half; these hold the C# mirror and the provider to it - see EmuSen_DianaSh.md §13.6.
    public class SessionHeaderTests
    {
        private static string Repo => ConfigRoot.Managed.Directory;
        private static string HeaderPath => Path.Combine(Repo, "EmuSen.Crystal", "include", "emusen_session.h");
        private static string Header => File.ReadAllText(HeaderPath);

        private static readonly Dictionary<string, Type> CTypes = new()
        {
            ["void *"] = typeof(void*), ["uint32_t"] = typeof(uint), ["int32_t"] = typeof(int), ["int64_t"] = typeof(long),
            ["uint64_t"] = typeof(ulong), ["size_t"] = typeof(nuint), ["uint8_t *"] = typeof(byte*), ["const uint8_t *"] = typeof(byte*),
        };

        // Each function of the table as the header declares it: its name, return type and parameter types.
        private static List<(string Name, Type Return, Type[] Parameters)> HeaderFunctions()
        {
            string h = Header;
            string body = h[h.IndexOf("typedef struct emusen_session_v1 {", StringComparison.Ordinal)..h.IndexOf("} emusen_session_v1;", StringComparison.Ordinal)];
            body = Regex.Replace(body, @"/\*.*?\*/", "", RegexOptions.Singleline);
            var functions = new List<(string, Type, Type[])>();
            foreach (Match m in Regex.Matches(body, @"(int64_t|int32_t)\s*\(\*(\w+)\)\(([^;]*?)\);", RegexOptions.Singleline))
            {
                Type[] parameters = m.Groups[3].Value.Split(',').Select(a =>
                {
                    Match p = Regex.Match(Regex.Replace(a.Trim(), @"\s+", " "), @"^(const uint8_t \*|uint8_t \*|void \*|uint32_t|int32_t|int64_t|uint64_t|size_t)\s*\w+$");
                    Assert.True(p.Success, $"{m.Groups[2].Value}: a parameter the test does not know: '{a.Trim()}'");
                    return CTypes[p.Groups[1].Value];
                }).ToArray();
                functions.Add((m.Groups[2].Value, CTypes[m.Groups[1].Value], parameters));
            }
            return functions;
        }

        [Fact]
        public void The_managed_table_is_the_header_s_field_for_field()
        {
            var header = HeaderFunctions();
            FieldInfo[] fields = typeof(EmuSenSessionV1).GetFields();
            Assert.Equal(new[] { "size", "version", "ctx" }, fields.Take(3).Select(f => f.Name));
            Assert.Equal(header.Select(f => f.Name), fields.Skip(3).Select(f => f.Name));

            foreach (var (field, declared) in fields.Skip(3).Zip(header))
            {
                Type pointer = field.GetModifiedFieldType();
                Assert.True(pointer.IsFunctionPointer, field.Name);
                Assert.True(pointer.IsUnmanagedFunctionPointer, field.Name);
                Assert.Equal(declared.Return, pointer.GetFunctionPointerReturnType().UnderlyingSystemType);
                Assert.Equal(declared.Parameters, pointer.GetFunctionPointerParameterTypes().Select(p => p.UnderlyingSystemType));
            }
        }

        [Fact]
        public void Every_number_the_header_defines_is_SessionAbi_s()
        {
            var defines = Regex.Matches(Header, @"^#define (EMUSEN_(?:SVC|SESSION)_\w+)\s+\(?(-?\d+)u?\)?", RegexOptions.Multiline)
                .ToDictionary(m => m.Groups[1].Value, m => long.Parse(m.Groups[2].Value));
            var constants = typeof(SessionAbi).GetFields(BindingFlags.Public | BindingFlags.Static).Where(f => f.IsLiteral && f.Name != "Version")
                .ToDictionary(f => f.Name, f => Convert.ToInt64(f.GetRawConstantValue()));

            static string Pascal(string name) => string.Concat(Regex.Replace(name, "^EMUSEN_(SVC|SESSION)_", "").ToLowerInvariant().Split('_')
                .Select(p => char.ToUpperInvariant(p[0]) + p[1..]));

            Assert.Equal(defines.Keys.Select(Pascal).OrderBy(n => n), constants.Keys.OrderBy(n => n));
            foreach (var (name, value) in defines) Assert.Equal(value, constants[Pascal(name)]);
            Assert.Equal((SessionAbi.Major << 16) | SessionAbi.Minor, SessionAbi.Version);
            Assert.Contains("#define EMUSEN_SESSION_SERVICE \"emusen.session\"", Header);
        }

        // The statuses sit in the platform's band, apart from the two the project shares, and the core ABI's codes stay its own - see EmuSen_DianaSh.md §9.7.
        [Fact]
        public void The_statuses_keep_to_their_band()
        {
            foreach (FieldInfo f in typeof(SessionAbi).GetFields().Where(f => f.FieldType == typeof(int)))
            {
                int status = (int)f.GetRawConstantValue()!;
                Assert.True(status is -1 or -5 or (<= -1024 and >= -1279), $"{f.Name} = {status}");
            }
        }

        [Fact]
        public unsafe void The_provider_fills_every_function_and_states_its_size_and_version()
        {
            using var service = new SessionService(null);
            foreach (EmuSenSessionV1* table in new[] { service.Table, service.ReadOnlyTable })
            {
                Assert.Equal((uint)sizeof(EmuSenSessionV1), table->size);
                Assert.Equal(SessionAbi.Version, table->version);
                Assert.True(table->ctx != null);
                byte* raw = (byte*)table;
                foreach (FieldInfo f in typeof(EmuSenSessionV1).GetFields().Skip(3))
                    Assert.NotEqual(0, *(nint*)(raw + Marshal.OffsetOf<EmuSenSessionV1>(f.Name)));
            }
            Assert.True(service.Table->ctx != service.ReadOnlyTable->ctx);
        }

        // A C compiler reads the header beside the other two, and lays the table out where the mirror does.
        [Fact]
        public void A_C_compiler_lays_the_table_out_as_the_mirror_does()
        {
            string? cc = new[] { "cc", "clang", "gcc" }.FirstOrDefault(OnPath);
            if (cc is null) return;
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenSessionHeader", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                var fields = typeof(EmuSenSessionV1).GetFields().Select(f => f.Name).ToList();
                string source = "#include <stdio.h>\n#include <string.h>\n#include \"emusen_core.h\"\n#include \"emusen_platform.h\"\n#include \"emusen_session.h\"\n"
                    + "int main(void) {\n    emusen_session_v1 t;\n    memset(&t, 0, sizeof t);\n    t.size = sizeof t;\n"
                    + "    printf(\"sizeof %zu\\n\", sizeof(emusen_session_v1));\n    printf(\"has %d\\n\", EMUSEN_SESSION_HAS(&t, events_drain) ? 1 : 0);\n"
                    + string.Concat(fields.Select(f => $"    printf(\"{f} %zu\\n\", offsetof(emusen_session_v1, {f}));\n"))
                    + "    return 0;\n}\n";
                File.WriteAllText(Path.Combine(dir, "layout.c"), source);
                string includes = $"-I\"{Path.Combine(Repo, "EmuSen.Crystal", "include")}\" -I\"{Path.Combine(Repo, "EmuSen", "Cores", "Shared", "emusen-native", "include")}\" -I\"{Path.Combine(Repo, "Platform", "include")}\"";
                string exe = Path.Combine(dir, "layout");
                Run(cc, $"-std=c99 -Wall -Wextra -Werror -pedantic {includes} -o \"{exe}\" \"{Path.Combine(dir, "layout.c")}\"");
                var printed = Run(exe, "").Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(l => l.Split(' ')).ToDictionary(p => p[0], p => int.Parse(p[1]));

                Assert.Equal(Marshal.SizeOf<EmuSenSessionV1>(), printed["sizeof"]);
                Assert.Equal(0, printed["has"]);
                foreach (string f in fields) Assert.Equal((int)Marshal.OffsetOf<EmuSenSessionV1>(f), printed[f]);
            }
            finally
            {
                try { Directory.Delete(dir, recursive: true); } catch (IOException) { }
            }
        }

        private static bool OnPath(string program) =>
            (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator).Any(d => File.Exists(Path.Combine(d, program)));

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
