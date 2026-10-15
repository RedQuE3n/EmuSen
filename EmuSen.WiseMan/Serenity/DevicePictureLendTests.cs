using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The slang device lent to Diamond through the core ABI: the picture taken on the device and read by a chain there, the frame it is drawn - see EmuSen_DevicePicture.md §17.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class DevicePictureLendTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDevicePictureLend_" + Guid.NewGuid().ToString("N"));
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public DevicePictureLendTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = _root;
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static byte[] Digest(byte[] pixels) => System.Security.Cryptography.SHA256.HashData(pixels);

        // Lent the device, each picture the chain reads there is the machine's own frame_copy of that frame, and a machine lent nothing hands the same pictures a frame later; with the layer listening, nothing is said.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_lent_machines_picture_reaches_the_chain_on_the_device_the_frame_it_is_drawn(bool validate)
        {
            var findings = new List<string>();
            SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, validate ? f => { lock (findings) findings.Add(f); } : null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, DiamondEnhancementsTests.Triangle());
            using (device)
            {
                using (SlangLend? probe = SlangLend.TryCreate(device, out string? why))
                    if (probe is null) { _output.WriteLine($"not run: {why}"); return; }
                int lends = 0;
                using var plain = (CoreEngine)CoreFactory.Loaded(path);
                using var lent = (CoreEngine)CoreFactory.Loaded(path, lender: () => { lends++; return SlangLend.TryCreate(device, out _); });
                Assert.Equal(1, lends);
                Assert.NotNull(lent.Lend);
                using var chain = new SlangChain(device, SlangPreset.Load(EmuSen.Serenity.Shaders.BuiltInPreset.Write(EmuSen.Serenity.Shaders.BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
                var (lentCopies, plainCopies) = (new List<byte[]>(), new List<byte[]>());
                const int On = 3, Frames = 14;
                for (int f = 0; f < Frames; f++)
                {
                    if (f == On) { plain.Set("resolution", "2"); lent.Set("resolution", "2"); }
                    plain.RunFrame();
                    lent.RunFrame();
                    Assert.Equal(f >= On, lent.PictureOnDevice);
                    Assert.False(plain.PictureOnDevice);
                    byte[] copy = lent.GetFrameBufferRgba();
                    lentCopies.Add(Digest(copy));
                    plainCopies.Add(Digest(plain.GetFrameBufferRgba()));
                    if (!lent.PictureOnDevice) continue;
                    Assert.True(chain.AdvanceLent(lent.Lend!), $"frame {f}: no picture was taken");
                    Assert.False(chain.AdvanceLent(lent.Lend!), $"frame {f}: a picture taken twice");
                    byte[] read = chain.Render(lent.ScreenWidth, lent.ScreenHeight);
                    Assert.Equal(lent.ScreenWidth * lent.ScreenHeight * 4, copy.Length);
                    Assert.True(read.AsSpan().SequenceEqual(copy), $"frame {f}: the picture read on the device differs from frame_copy in {read.Zip(copy).Count(p => p.First != p.Second)} bytes");
                }
                for (int f = 0; f < On; f++) Assert.Equal(plainCopies[f], lentCopies[f]);
                Assert.NotEqual(plainCopies[On], lentCopies[On]);
                for (int f = On; f < Frames - 1; f++) Assert.True(lentCopies[f].SequenceEqual(plainCopies[f + 1]), $"frame {f}: the lent machine's picture is not the other's a frame later");
                _output.WriteLine($"{Frames - On} pictures read on the device, each the frame it was drawn; the machine lent nothing a frame later");
            }
            _output.WriteLine($"{findings.Count} findings");
            foreach (string finding in findings.Take(10)) _output.WriteLine(finding);
            Assert.Empty(findings);
        }
    }
}
