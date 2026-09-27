using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using EmuSen.Galaxia;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Q40: ScreenScraperDeveloper.targets run by a real dotnet publish of a scratch project, with a fake developer file - see EmuSen_Settings_Reference.md §4.65.
    public class ScreenScraperEmbedTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenEmbedTests", Guid.NewGuid().ToString("N"));
        private readonly string _project, _config;
        private readonly string _id = "FAKEID" + Guid.NewGuid().ToString("N")[..10];
        private readonly string _password = "FAKEPW\"&<" + Guid.NewGuid().ToString("N")[..12];

        public ScreenScraperEmbedTests()
        {
            _project = Path.Combine(_root, "proj");
            _config = Path.Combine(_root, "config");
            Directory.CreateDirectory(_project);
            Directory.CreateDirectory(_config);
            string targets = Path.Combine(ConfigRoot.Directory, "EmuSen.Mistress", "Scraping", "ScreenScraperDeveloper.targets");
            File.WriteAllText(Path.Combine(_project, "Probe.csproj"),
                $"<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup><Import Project=\"{targets}\" /></Project>");
            File.WriteAllText(Path.Combine(_project, "Program.cs"), "System.Console.WriteLine(\"probe\");");
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string WriteFake(string directory)
        {
            Directory.CreateDirectory(directory);
            string path = Path.Combine(directory, DeveloperCredentials.FileName);
            File.WriteAllText(path, JsonSerializer.Serialize(new { devid = _id, devpassword = _password, softname = "EmuSen-Test" }));
            File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite);
            return path;
        }

        // The player's config directory is moved into scratch for every child, so a default path can never reach the real file.
        private (int Exit, string Output) Dotnet(params string[] arguments)
        {
            var start = new ProcessStartInfo("dotnet") { WorkingDirectory = _project, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (string a in arguments.Concat(["-nologo", "-nodeReuse:false"])) start.ArgumentList.Add(a);
            start.Environment["XDG_CONFIG_HOME"] = _config;
            start.Environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1";
            start.Environment["MSBUILDDISABLENODEREUSE"] = "1";
            using Process child = Process.Start(start)!;
            var output = child.StandardOutput.ReadToEndAsync();
            var errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(300_000), "dotnet did not finish");
            return (child.ExitCode, output.Result + errors.Result);
        }

        private (int Exit, string Output) Publish(string output, params string[] extra) =>
            Dotnet(["publish", "-c", "Release", "-o", Path.Combine(_root, output), "-v:n", "-clp:NoSummary", .. extra]);

        internal static byte[]? Resource(string assembly, string name)
        {
            using var pe = new PEReader(File.OpenRead(assembly));
            MetadataReader md = pe.GetMetadataReader();
            foreach (ManifestResourceHandle handle in md.ManifestResources)
            {
                ManifestResource resource = md.GetManifestResource(handle);
                if (md.GetString(resource.Name) != name) continue;
                PEMemoryBlock block = pe.GetSectionData(pe.PEHeaders.CorHeader!.ResourcesDirectory.RelativeVirtualAddress);
                BlobReader reader = block.GetReader((int)resource.Offset, block.Length - (int)resource.Offset);
                return reader.ReadBytes(reader.ReadInt32());
            }
            return null;
        }

        private static bool Holds(byte[] haystack, string text) =>
            new[] { Encoding.UTF8.GetBytes(text), Encoding.Unicode.GetBytes(text) }.Any(needle => haystack.AsSpan().IndexOf(needle) >= 0);

        // Every spelling a value could be written in: itself, JSON-escaped, URL-escaped.
        private IEnumerable<string> Spellings() =>
            new[] { _id, _password, JsonSerializer.Serialize(_password).Trim('"'), Uri.EscapeDataString(_password) }.Distinct();

        [Fact]
        public void A_publish_embeds_the_named_file_scrambled_and_it_decodes_to_the_file_s_values()
        {
            string fake = WriteFake(Path.Combine(_root, "fake"));
            (int exit, string log) = Publish("out", $"-p:EmuSenScreenScraperDeveloper={fake}");
            Assert.True(exit == 0, log);
            Assert.DoesNotContain("EMUSEN0040", log);

            string dll = Path.Combine(_root, "out", "Probe.dll");
            byte[] blob = Resource(dll, DeveloperCredentials.ResourceName)!;
            Assert.NotNull(blob);
            DeveloperCredentials decoded = DeveloperCredentials.Decode(blob)!;
            Assert.Equal((_id, _password, "EmuSen-Test"), (decoded.DevId, decoded.DevPassword, decoded.SoftName));
            Assert.Equal(DeveloperOrigin.Embedded, decoded.Origin);

            // No plain value in the assembly, in the log, or in anything the build left in obj or bin.
            byte[] image = File.ReadAllBytes(dll);
            foreach (string spelling in Spellings())
            {
                Assert.False(Holds(image, spelling), "a value in plain text in the assembly");
                Assert.DoesNotContain(spelling, log);
            }
            Assert.False(Holds(image, "devpassword"));
            Assert.Empty(Directory.EnumerateFiles(_project, "screenscraper-developer.bin", SearchOption.AllDirectories));
            foreach (string file in Directory.EnumerateFiles(_root, "*", SearchOption.AllDirectories).Where(f => f != fake))
                Assert.False(Spellings().Any(s => Holds(File.ReadAllBytes(file), s)), $"a value in plain text in {Path.GetRelativePath(_root, file)}");

            // A second publish scrambles with a key of its own.
            (exit, log) = Publish("out2", $"-p:EmuSenScreenScraperDeveloper={fake}");
            Assert.True(exit == 0, log);
            byte[] again = Resource(Path.Combine(_root, "out2", "Probe.dll"), DeveloperCredentials.ResourceName)!;
            Assert.Equal(_password, DeveloperCredentials.Decode(again)!.DevPassword);
            Assert.NotEqual(blob[6..38], again[6..38]);
        }

        [Fact]
        public void A_plain_build_never_embeds_and_a_publish_without_the_file_succeeds_warning_once_and_carries_none()
        {
            string fake = WriteFake(Path.Combine(_root, "fake"));
            (int exit, string log) = Dotnet("build", "-c", "Release", $"-p:EmuSenScreenScraperDeveloper={fake}");
            Assert.True(exit == 0, log);
            Assert.Null(Resource(Path.Combine(_project, "bin", "Release", "net10.0", "Probe.dll"), DeveloperCredentials.ResourceName));

            // The same obj after a publish that embedded: the next plain build must compile again, without it.
            (exit, log) = Publish("out", $"-p:EmuSenScreenScraperDeveloper={fake}");
            Assert.True(exit == 0, log);
            (exit, log) = Dotnet("build", "-c", "Release", $"-p:EmuSenScreenScraperDeveloper={fake}");
            Assert.True(exit == 0, log);
            Assert.Null(Resource(Path.Combine(_project, "bin", "Release", "net10.0", "Probe.dll"), DeveloperCredentials.ResourceName));

            string missing = Path.Combine(_root, "nowhere", DeveloperCredentials.FileName);
            (exit, log) = Publish("none", $"-p:EmuSenScreenScraperDeveloper={missing}");
            Assert.True(exit == 0, log);
            Assert.Single(Regex.Matches(log, "warning EMUSEN0040"));
            Assert.Contains(missing, log);
            Assert.Null(Resource(Path.Combine(_root, "none", "Probe.dll"), DeveloperCredentials.ResourceName));
        }

        [Fact]
        public void With_no_property_the_publish_reads_the_user_s_config_directory()
        {
            (int exit, string where) = Dotnet("msbuild", "Probe.csproj", "-getProperty:EmuSenScreenScraperDeveloper");
            Assert.True(exit == 0, where);
            string expected = Path.Combine(_config, "EmuSen", DeveloperCredentials.FileName);
            // The guard that keeps the real file out: nothing is published unless the default is inside this test's scratch.
            Assert.Equal(expected, where.Trim());

            (exit, string log) = Publish("none");
            Assert.True(exit == 0, log);
            Assert.Single(Regex.Matches(log, "warning EMUSEN0040"));
            Assert.Contains("No ScreenScraper developer file at " + expected, log);
            Assert.Null(Resource(Path.Combine(_root, "none", "Probe.dll"), DeveloperCredentials.ResourceName));

            WriteFake(Path.Combine(_config, "EmuSen"));
            (exit, log) = Publish("out");
            Assert.True(exit == 0, log);
            Assert.DoesNotContain("EMUSEN0040", log);
            Assert.Equal(_id, DeveloperCredentials.Decode(Resource(Path.Combine(_root, "out", "Probe.dll"), DeveloperCredentials.ResourceName)!)!.DevId);
        }

        [Fact]
        public void A_file_missing_a_field_is_not_embedded_and_the_warning_names_the_field_not_a_value()
        {
            string path = Path.Combine(_root, "half", DeveloperCredentials.FileName);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, JsonSerializer.Serialize(new { devid = _id, softname = "EmuSen-Test" }));
            (int exit, string log) = Publish("half-out", $"-p:EmuSenScreenScraperDeveloper={path}");
            Assert.True(exit == 0, log);
            Assert.Contains("it has no devpassword", log);
            Assert.DoesNotContain(_id, log);
            Assert.Null(Resource(Path.Combine(_root, "half-out", "Probe.dll"), DeveloperCredentials.ResourceName));
        }

        [Fact]
        public void The_scrambled_file_s_place_in_obj_is_ignored_by_git()
        {
            var check = new ProcessStartInfo("git") { WorkingDirectory = ConfigRoot.Directory, UseShellExecute = false, RedirectStandardOutput = true };
            foreach (string a in (string[])["check-ignore", "-q", "EmuSen.Mistress/obj/Release/net10.0/linux-x64/screenscraper-developer.bin"]) check.ArgumentList.Add(a);
            using Process git = Process.Start(check)!;
            git.WaitForExit();
            Assert.Equal(0, git.ExitCode);
        }
    }
}
