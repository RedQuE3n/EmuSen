using System;
using System.Collections.Generic;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Native;

namespace EmuSen.WiseMan.DianaSh
{
    // The settings service from a host's runtime to a command module, through the C# bridge and the Rust shell's guard: cores default and cores approve - see EmuSen_DianaSh.md §9.9.
    public class SettingsServiceTests
    {
        private sealed class FakeSettings : IShellSettings
        {
            public readonly List<string> Asked = new();
            public readonly Dictionary<(uint, string, string), string?> Stored = new();

            public string? Get(uint scope, string target, string key)
            {
                Asked.Add($"get {scope} {target} {key}");
                return Stored.TryGetValue((scope, target, key), out string? v) ? v : null;
            }

            public void Set(uint scope, string target, string key, string? value)
            {
                Asked.Add($"set {scope} {target} {key} {value ?? "-"}");
                if (value == "NoSuchEngine") throw new InvalidOperationException("NoSuchEngine is not an engine of PlayStation 2");
                Stored[(scope, target, key)] = value;
            }

            public void Approve(string sha256) => Asked.Add($"approve {sha256}");

            public void Revoke(string sha256) => Asked.Add($"revoke {sha256}");
        }

        [Fact]
        public void Cores_default_reaches_the_runtimes_engine_setting_and_its_refusal()
        {
            if (!DianaShNative.Ready) return;
            var settings = new FakeSettings();
            DianaShInterpreter shell = DianaShInterpreter.CreateWithoutSession();
            shell.UseSettings(settings);
            Assert.Equal("PlayStation 2: its first engine, none being chosen\n", shell.Execute("cores default \"PlayStation 2\""));
            Assert.Equal("PlayStation 2 now opens on ExtraPS2.\n", shell.Execute("cores default \"PlayStation 2\" ExtraPS2"));
            Assert.Equal("PlayStation 2: ExtraPS2\n", shell.Execute("cores default \"PlayStation 2\""));
            Assert.Contains("NoSuchEngine is not an engine of PlayStation 2", shell.Execute("cores default \"PlayStation 2\" NoSuchEngine"));
            Assert.Equal(new[] { "get 0 PlayStation 2 Engine", "set 0 PlayStation 2 Engine ExtraPS2", "get 0 PlayStation 2 Engine", "set 0 PlayStation 2 Engine NoSuchEngine" }, settings.Asked);
        }

        [Fact]
        public void Cores_approve_is_refused_from_a_pipe_and_a_shell_without_the_service_says_so()
        {
            if (!DianaShNative.Ready) return;
            var settings = new FakeSettings();
            DianaShInterpreter shell = DianaShInterpreter.CreateWithoutSession();
            shell.UseSettings(settings);
            Assert.Contains("approved at the prompt", shell.Execute("echo y | cores approve pcsx2"));
            Assert.Empty(settings.Asked);
            DianaShInterpreter bare = DianaShInterpreter.CreateWithoutSession();
            Assert.Contains("needs emusen.settings", bare.Execute("cores default \"PlayStation 2\" ExtraPS2"));
        }
    }
}
