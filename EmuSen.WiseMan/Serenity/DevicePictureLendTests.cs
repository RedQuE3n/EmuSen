using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
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
    // The slang device lent to Diamond through the frontend API, as a window lends it: the picture taken on the device and read by a chain there, the frame it is drawn - see EmuSen_DevicePicture.md §17.
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
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static byte[] Digest(byte[] pixels) => System.Security.Cryptography.SHA256.HashData(pixels);

        private static byte[] Copy(CrystalSession session)
        {
            byte[] picture = new byte[session.PictureBytes];
            session.CopyPicture(picture);
            return picture;
        }

        // Lent the device, each picture the chain reads there is the machine's own frame_copy of that frame, and a machine lent nothing hands the same pictures a frame later; with the layer listening, nothing is said.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_lent_machines_picture_reaches_the_chain_on_the_device_the_frame_it_is_drawn(bool validate)
        {
            var findings = new List<string>();
            SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, validate ? f => { lock (findings) findings.Add(f); } : null);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, DiamondEnhancementsTests.Triangle());
            using (device)
            {
                using (SlangLend? probe = SlangLend.TryCreate(device, out string? why))
                    if (probe is null) { _output.WriteLine($"not run: {why}"); return; }
                using var runtime = new CrystalRuntime();
                using CrystalSession plain = runtime.Open(new CrystalOpen { Path = path, NoBattery = true });
                using CrystalSession lent = runtime.Open(new CrystalOpen { Path = path, NoBattery = true, Device = device.DeviceHandle });
                Assert.Equal(0, plain.PictureLend());
                using var chain = new SlangChain(device, SlangPreset.Load(EmuSen.Serenity.Shaders.BuiltInPreset.Write(EmuSen.Serenity.Shaders.BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
                var (lentCopies, plainCopies) = (new List<byte[]>(), new List<byte[]>());
                const int On = 3, Frames = 14;
                var (plainInfo, lentInfo) = (new CrystalFrameInfo(), new CrystalFrameInfo());
                for (int f = 0; f < Frames; f++)
                {
                    if (f == On) lent.SetSetting("resolution", "2", game: false);
                    plain.Frame(ref plainInfo);
                    lent.Frame(ref lentInfo);
                    Assert.Equal(f >= On, lentInfo.picture_on_device != 0);
                    Assert.Equal(0u, plainInfo.picture_on_device);
                    byte[] copy = Copy(lent);
                    lentCopies.Add(Digest(copy));
                    plainCopies.Add(Digest(Copy(plain)));
                    if (lentInfo.picture_on_device == 0) continue;
                    nint lend = lent.PictureLend();
                    Assert.NotEqual(0, lend);
                    SlangLend borrowed = SlangLend.Borrow(device, lend);
                    Assert.True(chain.AdvanceLent(borrowed), $"frame {f}: no picture was taken");
                    Assert.False(chain.AdvanceLent(borrowed), $"frame {f}: a picture taken twice");
                    byte[] read = chain.Render(lentInfo.width, lentInfo.height);
                    Assert.Equal(lentInfo.width * lentInfo.height * 4, copy.Length);
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
