using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Fixtures
{
    // The conformance kit's corpus form over a core's synthetic images, each image's cases at once beside the core's - see EmuSen_CoreAPI.md §28.21.
    public static class ConformKit
    {
        public static string Runner => Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");

        // Every core case and every image's cases must pass, and each image must report exactly the cases named.
        public static void Passes(string library, IReadOnlyList<(string Name, byte[] Image)> images, string[] coreCases, string[] imageCases, ITestOutputHelper output, int frames = 120)
        {
            Assert.True(File.Exists(Runner), $"no kit runner at {Runner}");
            string root = Path.Combine(Path.GetTempPath(), "EmuSenKit_" + Guid.NewGuid().ToString("N"));
            string folder = Directory.CreateDirectory(Path.Combine(root, "images")).FullName, reports = Path.Combine(root, "reports");
            try
            {
                foreach (var (name, image) in images) File.WriteAllBytes(Path.Combine(folder, name), image);
                var start = new ProcessStartInfo(Runner) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in new[] { "--core", library, "--images", folder, "--jobs", images.Count.ToString(), "--frames", frames.ToString(), "--report-dir", reports }) start.ArgumentList.Add(a);
                using var process = Process.Start(start)!;
                var stderr = process.StandardError.ReadToEndAsync();
                string text = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                output.WriteLine(text + stderr.Result);
                Assert.True(process.ExitCode == 0, text + stderr.Result);
                Assert.Contains($"the core's cases pass; {images.Count} of {images.Count} images pass", text);
                string[] core = text.Split('\n').SkipWhile(l => !l.StartsWith("the core's cases")).Where(l => l.StartsWith("pass C")).Select(l => l.Split(' ', StringSplitOptions.RemoveEmptyEntries)[1]).ToArray();
                Assert.Equal(coreCases, core);
                var each = Directory.EnumerateFiles(reports, "*.json").Where(f => Path.GetFileName(f) != "summary.json").ToList();
                Assert.Equal(images.Count, each.Count);
                foreach (string file in each)
                {
                    using var report = JsonDocument.Parse(File.ReadAllText(file));
                    var cases = report.RootElement.GetProperty("cases").EnumerateArray().Select(c => (c.GetProperty("id").GetString()!, c.GetProperty("passed").GetBoolean())).ToArray();
                    Assert.Equal(imageCases.Select(id => (id, true)).ToArray(), cases);
                }
            }
            finally
            {
                try { Directory.Delete(root, recursive: true); } catch (IOException) { }
            }
        }
    }
}
