using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // A pack preset used and its parameters page opened in Settings, timed to the page drawn; gated on EMUSEN_SLANG_PACK - see EmuSen_Settings_Reference.md §4.48.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderBrowseBenchTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderBrowseBenchTests).GetTypeInfo().Assembly);

        // Where each measurement is appended as one line, for runs of two builds interleaved.
        private const string OutVariable = "EMUSEN_SHADER_BENCH_OUT";

        private static readonly string[] Targets =
        {
            "crt/crt-lottes.slangp", "crt/crt-royale.slangp", "crt/crt-guest-advanced.slangp", "bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV.slangp",
        };

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderBrowseBench", Guid.NewGuid().ToString("N"));

        public ShaderBrowseBenchTests(ITestOutputHelper output)
        {
            _out = output;
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void Record(string line)
        {
            _out.WriteLine(line);
            if (Environment.GetEnvironmentVariable(OutVariable) is { Length: > 0 } file) File.AppendAllText(file, line + Environment.NewLine);
        }

        // A preset used on the SNES's page and its parameters page opened, timed to the page laid out and drawn.
        [Fact]
        public Task Using_a_preset_and_opening_its_parameters_are_timed_to_a_drawn_page() => Session.Dispatch(() =>
        {
            string? pack = Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            MainWindow window = SettingsDesk.Open(SettingsScreen.Display);
            typeof(MainWindow).GetField("_shaderShelf", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(window, new ShaderShelf(pack));
            foreach (string relative in Targets)
            {
                var clock = Stopwatch.StartNew();
                SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES");
                window.SettingsShown!.Host.Graphics.SetValue("SNES", GraphicsKeys.ScreenFilter, ShaderCatalog.SlangPrefix + relative);
                SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES", "params");
                long built = clock.ElapsedMilliseconds;
                UiTest.Capture(SettingsDesk.Of(window));
                Record($"{relative}\t{SettingsDesk.Keys(window).Count(k => k.StartsWith("p:", StringComparison.Ordinal))} parameters\tread and built {built} ms\tdrawn {clock.ElapsedMilliseconds} ms");
                Assert.Contains(SettingsDesk.Keys(window), k => k.StartsWith("p:", StringComparison.Ordinal));
            }
            window.Close();
        }, default);
    }
}
