using System.Collections.Generic;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Venus;

namespace EmuSen.WiseMan.Cores
{
    // Rewind is withheld by Mars (C#) alone, declared by the engine rather than found by its type - see EmuSen_Settings_Reference.md §4.85.4.
    public class EngineFeaturesTests
    {
        [Fact]
        public void Mars_withholds_rewind_and_says_why()
        {
            EngineFeatures features = EngineFeatures.Of(new MarsCore());
            Assert.False(features.RewindCapture);
            Assert.Equal("not kept for Mars (C#)", features.RewindWithheld);
        }

        public static IEnumerable<object[]> EveryOtherEngine()
        {
            yield return new object[] { "MarsRT" };
            yield return new object[] { "Moon" };
            yield return new object[] { "MoonRT" };
            yield return new object[] { "Mercury" };
            yield return new object[] { "MercuryRT" };
            yield return new object[] { "Venus" };
        }

        [Theory]
        [MemberData(nameof(EveryOtherEngine))]
        public void Every_other_engine_keeps_rewind(string engine)
        {
            ICore core = engine switch
            {
                "MarsRT" => new MarsRtCore(),
                "Moon" => new MoonCore(),
                "MoonRT" => new MoonRtCore(),
                "Mercury" => new MercuryCore(),
                "MercuryRT" => new MercuryRtCore(),
                _ => new VenusCore(headless: true),
            };
            Assert.Same(EngineFeatures.All, EngineFeatures.Of(core));
            Assert.True(EngineFeatures.Of(core).RewindCapture);
            (core as System.IDisposable)?.Dispose();
        }

        [Fact]
        public void No_core_at_all_has_every_feature() => Assert.True(EngineFeatures.Of(null).RewindCapture);
    }
}
