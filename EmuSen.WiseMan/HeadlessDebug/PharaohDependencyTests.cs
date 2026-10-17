using System;
using System.IO;
using System.Linq;
using System.Text.Json;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // The headless runner takes no window toolkit, and still carries the imaging its recorder writes PNGs with, on each platform it ships on - see EmuSen_Surfaces.md §7.1.
    public class PharaohDependencyTests
    {
        private static JsonDocument Dependencies() =>
            JsonDocument.Parse(File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "EmuSen.Pharaoh.deps.json")));

        [Fact]
        public void Pharaoh_takes_no_Avalonia_library()
        {
            using JsonDocument deps = Dependencies();
            string[] libraries = deps.RootElement.GetProperty("libraries").EnumerateObject().Select(p => p.Name).ToArray();

            Assert.Contains(libraries, l => l.StartsWith("SkiaSharp/", StringComparison.Ordinal));
            Assert.DoesNotContain(libraries, l => l.StartsWith("Avalonia", StringComparison.OrdinalIgnoreCase) || l.StartsWith("EmuSen.Serenity", StringComparison.Ordinal));
        }

        [Theory]
        [InlineData("linux-x64", "libSkiaSharp.so")]
        [InlineData("linux-arm64", "libSkiaSharp.so")]
        [InlineData("win-x64", "libSkiaSharp.dll")]
        [InlineData("osx", "libSkiaSharp.dylib")]
        public void Pharaoh_carries_SkiaSharps_native_library(string rid, string file)
        {
            using JsonDocument deps = Dependencies();
            var natives = deps.RootElement.GetProperty("targets").EnumerateObject().First().Value.EnumerateObject()
                .Where(p => p.Value.TryGetProperty("runtimeTargets", out _))
                .SelectMany(p => p.Value.GetProperty("runtimeTargets").EnumerateObject())
                .Select(t => (Path: t.Name, Rid: t.Value.GetProperty("rid").GetString()));

            Assert.Contains(natives, n => n.Rid == rid && Path.GetFileName(n.Path) == file);
        }
    }
}
