using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Xml.Linq;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.DianaOS
{
    // DianaOSPublishLayout.targets run by a real dotnet publish of a scratch project, for osx-arm64 and for linux-x64 - see EmuSen_Settings_Reference.md §4.82.
    public class MacAppBundlePublishTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMacBundleTests", Guid.NewGuid().ToString("N"));
        private readonly string _project;
        private static readonly string Repo = ConfigRoot.Directory;
        private static readonly string Icon = Path.Combine(Repo, "EmuSen.Mistress", "Assets", "Icon", "emusen.icns");

        public MacAppBundlePublishTests()
        {
            _project = Path.Combine(_root, "proj");
            Directory.CreateDirectory(_project);
            string targets = Path.Combine(Repo, "EmuSen.DianaOS", "Publish", "DianaOSPublishLayout.targets");
            File.WriteAllText(Path.Combine(_project, "Probe.csproj"), $"""
                <Project Sdk="Microsoft.NET.Sdk">
                  <PropertyGroup>
                    <OutputType>Exe</OutputType>
                    <TargetFramework>net10.0</TargetFramework>
                    <Version>1.2.3</Version>
                    <DianaOSMacAppName>EmuSen</DianaOSMacAppName>
                    <DianaOSMacBundleIdentifier>org.example.probe</DianaOSMacBundleIdentifier>
                    <DianaOSMacAppIcon>{Icon}</DianaOSMacAppIcon>
                    <DianaOSMacCategory>public.app-category.games</DianaOSMacCategory>
                  </PropertyGroup>
                  <ItemGroup>
                    <None Include="Manual.md" Link="DianaOSRoot\home\Documents\Manual\Manual.md" CopyToOutputDirectory="Never" CopyToPublishDirectory="PreserveNewest" />
                  </ItemGroup>
                  <Import Project="{targets}" />
                </Project>
                """);
            File.WriteAllText(Path.Combine(_project, "Program.cs"), "System.Console.WriteLine(\"probe\");");
            File.WriteAllText(Path.Combine(_project, "Manual.md"), "# manual");
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private (int Exit, string Output, string Dir) Publish(string rid, params string[] extra)
        {
            string output = Path.Combine(_root, rid);
            var start = new ProcessStartInfo("dotnet") { WorkingDirectory = _project, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (string a in new[] { "publish", "-c", "Release", "-r", rid, "--self-contained", "false", "-o", output, "-nologo", "-nodeReuse:false", "-m:1" }.Concat(extra)) start.ArgumentList.Add(a);
            start.Environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1";
            start.Environment["MSBUILDDISABLENODEREUSE"] = "1";
            using Process child = Process.Start(start)!;
            var stdout = child.StandardOutput.ReadToEndAsync();
            var stderr = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(300_000), "dotnet did not finish");
            return (child.ExitCode, stdout.Result + stderr.Result, output);
        }

        private static Dictionary<string, string> PlistKeys(string path)
        {
            XElement dict = XDocument.Load(path).Root!.Element("dict")!;
            var keys = new Dictionary<string, string>();
            XElement[] children = dict.Elements().ToArray();
            for (int i = 0; i + 1 < children.Length; i += 2)
            {
                Assert.Equal("key", children[i].Name.LocalName);
                XElement value = children[i + 1];
                keys[children[i].Value] = value.Name.LocalName is "true" or "false" ? value.Name.LocalName : value.Value;
            }
            return keys;
        }

        [Fact]
        public void An_osx_publish_is_an_app_bundle_with_its_data_outside_it()
        {
            (int exit, string log, string dir) = Publish("osx-arm64");
            Assert.True(exit == 0, log);

            Assert.Equal(new[] { "EmuSen.app" }, Directory.EnumerateFileSystemEntries(dir).Select(Path.GetFileName));
            string contents = Path.Combine(dir, "EmuSen.app", "Contents");
            Assert.Equal(new[] { "Info.plist", "MacOS", "Resources" }, Directory.EnumerateFileSystemEntries(contents).Select(Path.GetFileName).Order());

            string apphost = Path.Combine(contents, "MacOS", "Probe");
            Assert.True(File.Exists(apphost));
            Assert.True(File.GetUnixFileMode(apphost).HasFlag(UnixFileMode.UserExecute));
            Assert.True(File.Exists(Path.Combine(contents, "MacOS", "Probe.dll")));
            Assert.True(File.Exists(Path.Combine(contents, "MacOS", "Probe.runtimeconfig.json")));

            string resources = Path.Combine(contents, "Resources");
            Assert.Equal(File.ReadAllBytes(Icon), File.ReadAllBytes(Path.Combine(resources, "emusen.icns")));
            Assert.Equal("# manual", File.ReadAllText(Path.Combine(resources, "home", "Documents", "Manual", "Manual.md")));
            Assert.True(File.Exists(Path.Combine(resources, "LICENSE")));
            Assert.True(File.Exists(Path.Combine(resources, "THIRD_PARTY_NOTICES.md")));
            Assert.NotEmpty(Directory.EnumerateFiles(Path.Combine(resources, "licenses")));

            Assert.Empty(Directory.EnumerateFiles(dir, ".dianaosroot", SearchOption.AllDirectories));
            Assert.False(Directory.Exists(Path.Combine(contents, "MacOS", "DianaOSRoot")));

            Dictionary<string, string> plist = PlistKeys(Path.Combine(contents, "Info.plist"));
            Assert.Equal("EmuSen", plist["CFBundleName"]);
            Assert.Equal("EmuSen", plist["CFBundleDisplayName"]);
            Assert.Equal("org.example.probe", plist["CFBundleIdentifier"]);
            Assert.Equal("Probe", plist["CFBundleExecutable"]);
            Assert.Equal("emusen", plist["CFBundleIconFile"]);
            Assert.Equal("1.2.3", plist["CFBundleShortVersionString"]);
            Assert.Equal("1.2.3", plist["CFBundleVersion"]);
            Assert.Equal("APPL", plist["CFBundlePackageType"]);
            Assert.Equal("12.0", plist["LSMinimumSystemVersion"]);
            Assert.Equal("true", plist["NSHighResolutionCapable"]);
            Assert.Equal("public.app-category.games", plist["LSApplicationCategoryType"]);
            Assert.DoesNotContain(plist.Values, v => v.Contains('@'));

            if (!OperatingSystem.IsMacOS()) Assert.Contains("EmuSen.app is not signed", log);
        }

        // The Linux tree is what it was: a launcher, the marker, the app in lib/EmuSen and the docs in home.
        [Fact]
        public void A_linux_publish_keeps_the_self_contained_tree()
        {
            (int exit, string log, string dir) = Publish("linux-x64");
            Assert.True(exit == 0, log);

            Assert.True(File.Exists(Path.Combine(dir, ".dianaosroot")));
            Assert.True(File.Exists(Path.Combine(dir, "bin", "Probe")));
            Assert.True(File.Exists(Path.Combine(dir, "lib", "EmuSen", "Probe.dll")));
            Assert.True(File.Exists(Path.Combine(dir, "home", "Documents", "Manual", "Manual.md")));
            Assert.True(File.Exists(Path.Combine(dir, "LICENSE")));
            Assert.True(File.Exists(Path.Combine(dir, "licenses", "MIT.txt")));
            Assert.Empty(Directory.EnumerateDirectories(dir, "*.app", SearchOption.AllDirectories));
            Assert.DoesNotContain("not signed", log);
        }

        // Mistress is the project that asks for the bundle, with the icon settings §4.55 made.
        [Fact]
        public void Mistress_names_its_bundle()
        {
            XDocument project = XDocument.Load(Path.Combine(Repo, "EmuSen.Mistress", "EmuSen.Mistress.csproj"));
            string? Property(string name) => project.Descendants(name).SingleOrDefault()?.Value;

            Assert.Equal("EmuSen", Property("DianaOSMacAppName"));
            Assert.Equal("io.github.redque3n.emusen", Property("DianaOSMacBundleIdentifier"));
            Assert.Equal("public.app-category.games", Property("DianaOSMacCategory"));
            Assert.Equal("$(MSBuildThisFileDirectory)Assets/Icon/emusen.icns", Property("DianaOSMacAppIcon"));
            Assert.True(File.Exists(Icon));
        }
    }
}
