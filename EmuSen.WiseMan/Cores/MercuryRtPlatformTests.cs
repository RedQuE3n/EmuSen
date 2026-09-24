using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Both Game Boy engines' sound, picture and state, digested and held to the digests linux-x64 recorded - see Mercury_Native.md §8.6.
    public class MercuryRtPlatformTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtPlatformTests(ITestOutputHelper output) => _output = output;

        // Keys all four channels panned apart and fills tile data with the LCD on; then, each time DIV wraps, re-keys the noise and the sweep and scrolls.
        private static readonly byte[] FourChannels =
        {
            0x3E, 0x80, 0xE0, 0x26, 0x3E, 0x77, 0xE0, 0x24, 0x3E, 0xE7, 0xE0, 0x25,
            0x3E, 0x15, 0xE0, 0x10, 0x3E, 0x80, 0xE0, 0x11, 0x3E, 0xF3, 0xE0, 0x12, 0x3E, 0x00, 0xE0, 0x13, 0x3E, 0x87, 0xE0, 0x14,
            0x3E, 0x40, 0xE0, 0x16, 0x3E, 0xA5, 0xE0, 0x17, 0x3E, 0x30, 0xE0, 0x18, 0x3E, 0x86, 0xE0, 0x19,
            0x3E, 0x80, 0xE0, 0x1A, 0x3E, 0x00, 0xE0, 0x1B, 0x3E, 0x20, 0xE0, 0x1C, 0x3E, 0x50, 0xE0, 0x1D, 0x3E, 0x85, 0xE0, 0x1E,
            0x21, 0x00, 0x80, 0x7D, 0x22, 0x7C, 0xFE, 0x98, 0x20, 0xF9,
            0xF0, 0x04, 0xB7, 0x20, 0xFB,
            0x21, 0x00, 0xC0, 0x34, 0x7E, 0xE0, 0x22, 0xE0, 0x43, 0x3E, 0xF1, 0xE0, 0x21, 0x3E, 0x80, 0xE0, 0x23, 0x3E, 0x87, 0xE0, 0x14,
            0xF0, 0x04, 0xB7, 0x28, 0xFB,
            0x18, 0xDF,
        };

        private static byte[] Program(string name) => name switch
        {
            "four channels" => SyntheticGbRom.Build(patches: (0, FourChannels)),
            "busy" => SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x1B, ramSizeCode: 0x03, cgbFlag: 0x80, patches: (0, MercuryRtMachineTests.Busy)),
            "interrupts" => MercuryRtMachineTests.InterruptsRom(0x13, 0x03, 0x00),
            "colour" => MercuryRtMachineTests.ColourRom(),
            _ => throw new ArgumentException(name),
        };

        // Math.Pow's two uses in the APU, as linux-x64's libm answers them; the mixer's one transcendental (§3.3).
        [Fact]
        public void The_high_pass_charge_factors_are_linuxs()
        {
            long integral = BitConverter.DoubleToInt64Bits(Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, MercuryCore.CpuClockHz / 44100));
            long fractional = BitConverter.DoubleToInt64Bits(Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, MercuryCore.CpuClockHz / 44100.0));
            _output.WriteLine($"{RuntimeInformation.RuntimeIdentifier}: pow(seed, 95) {integral:X16}, pow(seed, 95.108) {fractional:X16}");
            Assert.Equal(0x3FEFDF60DC188482L, integral);
            Assert.Equal(0x3FEFDF574D84E421L, fractional);
        }

        // The C# mixer's high-pass capacitors, bit for bit after every frame: a difference below what a sample's truncation hides.
        [Theory]
        [InlineData("four channels", "9597B2BA68D371334789B33C7B1DA8FBAD50342B72C1CF73836E92639C0CC0D4")]
        [InlineData("busy", "510D50CA0AC43A645CDACE5CE85567745872CCAA1B0C00C5771AFE94DA2DEEFB")]
        [InlineData("interrupts", "15CA5785B5DAA90D6AA85ACAE4CA9C6F6B1F23BF7F040B42297D219558630D19")]
        public void The_csharp_mixers_doubles_match_linuxs(string program, string capacitors)
        {
            CoreOptions.BatteryRamDisabled = true;
            var csharp = new MercuryCore();
            string path = SyntheticGbRom.WriteTemp(Program(program));
            try { csharp.LoadRom(path); }
            finally { File.Delete(path); }
            var flags = System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic;
            var left = typeof(EmuSen.Cores.Nintendo.Mercury.Audio.Apu).GetField("_leftCapacitor", flags)!;
            var right = typeof(EmuSen.Cores.Nintendo.Mercury.Audio.Apu).GetField("_rightCapacitor", flags)!;
            using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            int moving = 0;
            for (int f = 0; f < 600; f++)
            {
                csharp.RunFrame();
                csharp.DequeueAudioSamples(int.MaxValue);
                double l = (double)left.GetValue(csharp.Bus!.Apu)!, r = (double)right.GetValue(csharp.Bus!.Apu)!;
                if (l != 0 || r != 0) moving++;
                digest.AppendData(BitConverter.GetBytes(BitConverter.DoubleToInt64Bits(l)));
                digest.AppendData(BitConverter.GetBytes(BitConverter.DoubleToInt64Bits(r)));
            }
            string got = Convert.ToHexString(digest.GetHashAndReset());
            _output.WriteLine($"{RuntimeInformation.RuntimeIdentifier} {program}: {moving} frames with a charged capacitor; capacitors {got}");
            Assert.True(moving > 500, $"{moving} frames with a charged capacitor");
            Assert.True(got == capacitors, "the C# mixer's doubles differ from linux-x64's");
        }

        [Theory]
        [InlineData("four channels", "80D5E574D625A8D1FD5B072A444FB89BA450B9C1B0C5F433D8AA167ECE868EAA", "C0B17C1556732C6B10B6529BAA908977C7699AE27497BE3F70788D2899F222BA", "9E73C418C4944B687EF3670A3713C30B9AC7344F0C7681916643588D4E9808E8")]
        [InlineData("busy", "C5414EFAFB44DB231B3FA950E498F7C70EBB23BC9A0827066CA2F4B072CBD259", "8A0803654A95C3659704CC9C2BC839A75C04626BAA5F4926B09ADE2EE56FD705", "768FAE5853FB905300AB6474B6379599F99F3FB0F5222A493CE80A41CFEC67AC")]
        [InlineData("interrupts", "8199C9D14DF528E164A967480C410D19E6229D1284E29F9CD261A6799E86BB2E", "8A0803654A95C3659704CC9C2BC839A75C04626BAA5F4926B09ADE2EE56FD705", "838CF3A819F9E796D21D2D9304A9296A920AE0CE0A6BD8C2572056F2593D2FEF")]
        [InlineData("colour", "28854D0CC9A258D300408ED623739791BE57146D33695A513F0BA11014D70F51", "E293A91DAADF31E4B368DD0D0C2BDE7190D8D5312A4E924370CA4A8F4F180C39", "D527246653AEC4AD2A6E1B6548436CA245B251F53DF5A5361945DBCB0F3D34C6")]
        public void Sound_picture_and_state_match_linuxs_on_both_engines(string program, string sound, string picture, string state)
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            CoreOptions.BatteryRamDisabled = true;
            byte[] rom = Program(program);
            var csharp = new MercuryCore();
            string path = SyntheticGbRom.WriteTemp(rom);
            try { csharp.LoadRom(path); }
            finally { File.Delete(path); }
            using var rust = new MercuryMachine(rom, null);

            var digests = new[] { new Digests(), new Digests() };
            var frame = new byte[MercuryMachine.FrameBytes];
            for (int f = 0; f < 600; f++)
            {
                csharp.RunFrame();
                rust.RunFrame();
                digests[0].Frame(csharp.DequeueAudioSamples(int.MaxValue), csharp.GetFrameBufferRgba());
                rust.CopyFrame(frame);
                digests[1].Frame(rust.DrainAudio(int.MaxValue), frame);
            }
            using (var stream = new MemoryStream())
            {
                csharp.SaveState(stream);
                digests[0].State = Convert.ToHexString(SHA256.HashData(stream.ToArray()));
            }
            digests[1].State = Convert.ToHexString(SHA256.HashData(rust.Save()));

            string[] engines = { "Mercury (C#)", "MercuryRT" };
            for (int e = 0; e < 2; e++)
                _output.WriteLine($"{RuntimeInformation.RuntimeIdentifier} {engines[e]}: {digests[e].Samples} samples; sound {digests[e].Sound}, picture {digests[e].Picture}, state {digests[e].State}");
            Assert.True(digests[0].Samples > 600 * 700, $"{digests[0].Samples} samples");
            Assert.True(digests[0].Sound == digests[1].Sound, "the engines' sound differs on this platform");
            Assert.True(digests[0].Picture == digests[1].Picture, "the engines' pictures differ on this platform");
            Assert.True(digests[0].State == digests[1].State, "the engines' states differ on this platform");
            for (int e = 0; e < 2; e++)
            {
                Assert.True(digests[e].Sound == sound, $"{engines[e]}'s sound differs from linux-x64's");
                Assert.True(digests[e].Picture == picture, $"{engines[e]}'s picture differs from linux-x64's");
                Assert.True(digests[e].State == state, $"{engines[e]}'s state differs from linux-x64's");
            }
        }

        private sealed class Digests
        {
            private readonly IncrementalHash _sound = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            private readonly IncrementalHash _picture = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            private string? _soundHex, _pictureHex;
            public long Samples;
            public string State = "";

            public void Frame(short[] samples, byte[] rgba)
            {
                _sound.AppendData(MemoryMarshal.AsBytes(samples.AsSpan()));
                _picture.AppendData(rgba);
                Samples += samples.Length;
            }

            public string Sound => _soundHex ??= Convert.ToHexString(_sound.GetHashAndReset());
            public string Picture => _pictureHex ??= Convert.ToHexString(_picture.GetHashAndReset());
        }
    }
}
