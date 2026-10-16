using System;
using System.IO;
using EmuSen.Cores.Nintendo.MarsRT;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's state read and written back byte for byte, and refused whole where it is not one - see Mars_Native.md §5.1, §7.
    [Collection("MarsStatics")]
    public class MarsNativeStateTests
    {
        // Where copies of real game states are looked for; absent, those cases pass without running.
        public const string StatesVariable = "EMUSEN_MARSRT_STATES";

        // A stock console's 4 MB of RDRAM.
        private const int Rdram = 4 * 1024 * 1024;

        private readonly ITestOutputHelper _output;

        public MarsNativeStateTests(ITestOutputHelper output) => _output = output;

        [Theory]
        [InlineData("sm64.state")]
        [InlineData("oot.state")]
        [InlineData("ge-dam.state")]
        public void A_real_games_state_comes_back_from_rust_byte_for_byte(string name)
        {
            string? folder = Environment.GetEnvironmentVariable(StatesVariable);
            string path = Path.Combine(folder ?? "", name);
            if (folder is null || !File.Exists(path))
            {
                _output.WriteLine($"{name}: absent, not run");
                return;
            }

            Assert.True(MarsMachine.Available, MarsNative.Report);
            byte[] state = File.ReadAllBytes(path);
            using var machine = new MarsMachine(Rdram);
            machine.Load(state);
            bool snapshot = machine.LoadedKind == MarsMachine.SnapshotKind;
            AssertSameBytes(state, machine.Save(snapshot));
            _output.WriteLine($"{name}: {state.Length} bytes, version {machine.LoadedKind}, {machine.RdramBytes / (1024 * 1024)} MB, identical");
        }

        [Fact]
        public void A_state_that_is_not_marss_or_is_cut_short_is_refused_and_changes_nothing()
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            byte[] state;
            using (var source = new MarsMachine(Rdram)) state = source.Save(snapshot: false);

            using var machine = new MarsMachine(Rdram);
            machine.Load(state);
            Assert.Throws<InvalidDataException>(() => machine.Load(state.AsSpan(0, state.Length - 1)));
            Assert.Throws<InvalidDataException>(() => machine.Load(new byte[] { 0x4D, 0x41, 0x52, 0x54, 1, 0, 0, 0 }));
            byte[] wrongSize = (byte[])state.Clone();
            BitConverter.GetBytes(3 * 1024 * 1024).CopyTo(wrongSize, 8);
            Assert.Throws<InvalidDataException>(() => machine.Load(wrongSize));
            AssertSameBytes(state, machine.Save(snapshot: false));
            Assert.Throws<ArgumentOutOfRangeException>(() => new MarsMachine(1234));
        }

        private static void AssertSameBytes(byte[] expected, byte[] actual)
        {
            int length = Math.Min(expected.Length, actual.Length);
            int first = expected.AsSpan(0, length).CommonPrefixLength(actual.AsSpan(0, length));
            Assert.True(first == length && expected.Length == actual.Length,
                $"MarsRT wrote {actual.Length} bytes against the state's {expected.Length}; the first difference is at {first}.");
        }
    }
}
