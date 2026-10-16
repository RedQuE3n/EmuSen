using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT's sound, picture and state, digested and held to the digests linux-x64 recorded - see Mercury_Native.md §8.6.
    public class MercuryRtPlatformTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtPlatformTests(ITestOutputHelper output) => _output = output;

        private static byte[] Program(string name) => name switch
        {
            "four channels" => SyntheticGbRom.Build(patches: (0, MercuryPrograms.FourChannels)),
            "busy" => SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x1B, ramSizeCode: 0x03, cgbFlag: 0x80, patches: (0, MercuryPrograms.Busy)),
            "interrupts" => MercuryPrograms.InterruptsRom(0x13, 0x03, 0x00),
            "colour" => MercuryPrograms.ColourRom(),
            _ => throw new ArgumentException(name),
        };

        // Math.Pow's two uses for the mixer, as linux-x64's libm answers them; since D1 the shim sends the second alone (§3.3, §9.4).
        [Fact]
        public void The_high_pass_charge_factors_are_linuxs()
        {
            long integral = BitConverter.DoubleToInt64Bits(Math.Pow(MercuryRtCore.HighPassSeed, MercuryRtCore.CpuClockHz / 44100));
            long fractional = BitConverter.DoubleToInt64Bits(Math.Pow(MercuryRtCore.HighPassSeed, MercuryRtCore.CpuClockHz / 44100.0));
            _output.WriteLine($"{RuntimeInformation.RuntimeIdentifier}: pow(seed, 95) {integral:X16}, pow(seed, 95.108) {fractional:X16}");
            Assert.Equal(0x3FEFDF60DC188482L, integral);
            Assert.Equal(0x3FEFDF574D84E421L, fractional);
        }

        [Theory]
        [InlineData("four channels", "420DFAA8EBA40746C2D711BEFF1F0DB7AAF122785EF8A9654F2F72D00F745E3A", "C0B17C1556732C6B10B6529BAA908977C7699AE27497BE3F70788D2899F222BA", "45789F271262BD0DCA53611A743F0451DC3146DE6974BA0374A88D5207D9862D", "77C2E5E688DADF3C29F3D9B59042B906F16516215980777F42CBA7BEB0659D1C")]
        [InlineData("busy", "B327BA1B261D9A7859CB8DE670F6C84ED355B853EC6B0C2F360F1EE82B399BF5", "8A0803654A95C3659704CC9C2BC839A75C04626BAA5F4926B09ADE2EE56FD705", "AE29E11F1B96005D34020DA3C8CC2F9FFC2C5C449343846124592FD2E493E038", "03A4D5FD14721517124E10FF600EBD5B416C549F4009206B28BA27975407A2D2")]
        [InlineData("interrupts", "2802C96FCC6175A4C59CCAD791DB5C69860133AEFCF42611F41A9A7808BB87AB", "8A0803654A95C3659704CC9C2BC839A75C04626BAA5F4926B09ADE2EE56FD705", "CEC354F94F5B6C23C19ED25364236AADEFDC9D7BDD48CE7FAF67B4BD5EE0C38D", "F740F8515E3809F250D80630A8FD86A718F319C030C04D8508846DAF5BE3A6CD")]
        [InlineData("colour", "D3A2D8F2637085A1BAFD9F981A948D3170B2738889A7F70109ADDA16964B698B", "E293A91DAADF31E4B368DD0D0C2BDE7190D8D5312A4E924370CA4A8F4F180C39", "0DAE338EA80B1AC6D5914FF8035831667C52F57111DF6F38F912A3A1E77FD6B7", "320BA9EE56401C683645F0C0042C8CD2E3A1FDA9637CB7867A3CDAC02192B776")]
        public void Sound_picture_and_state_match_linuxs(string program, string sound, string picture, string state, string stateVersion7)
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            CoreOptions.BatteryRamDisabled = true;
            using var rust = new MercuryMachine(Program(program));

            var digests = new Digests();
            var frame = new byte[MercuryMachine.FrameBytes];
            for (int f = 0; f < 600; f++)
            {
                rust.RunFrame();
                rust.CopyFrame(frame);
                digests.Frame(rust.DrainAudio(int.MaxValue), frame);
            }
            byte[] saved = rust.Save();
            digests.State = Convert.ToHexString(SHA256.HashData(saved));

            // Version 11 is version 7 with the mixer's 24 bytes, OAM DMA's 4, the sweep's 1 and the frozen LY=LYC bit's 1 after the walks: undone, the state hashes as version 7 would - see Mercury_Native.md §10, §13.
            byte[] asVersion7 = saved[..^30];
            BitConverter.GetBytes(7).CopyTo(asVersion7, 4);
            _output.WriteLine($"{program}: as version 7 {Convert.ToHexString(SHA256.HashData(asVersion7))}");
            Assert.True(Convert.ToHexString(SHA256.HashData(asVersion7)) == stateVersion7, "the state differs from version 7's by more than the version and the mixer");

            _output.WriteLine($"{RuntimeInformation.RuntimeIdentifier} MercuryRT: {digests.Samples} samples; sound {digests.Sound}, picture {digests.Picture}, state {digests.State}");
            Assert.True(digests.Samples > 600 * 700, $"{digests.Samples} samples");
            Assert.True(digests.Sound == sound, "MercuryRT's sound differs from linux-x64's");
            Assert.True(digests.Picture == picture, "MercuryRT's picture differs from linux-x64's");
            Assert.True(digests.State == state, "MercuryRT's state differs from linux-x64's");
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
