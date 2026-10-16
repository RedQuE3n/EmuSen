using System.Collections.Generic;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Venus;

namespace EmuSen.WiseMan.Cores
{
    // Rewind is declared by the engine rather than found by its type; since Mars (C#) was retired every engine keeps it - see EmuSen_Settings_Reference.md §4.85.4.
    public class EngineFeaturesTests
    {
        public static IEnumerable<object[]> EveryEngine()
        {
            yield return new object[] { "MarsRT" };
            yield return new object[] { "Moon" };
            yield return new object[] { "MoonRT" };
            yield return new object[] { "MercuryRT" };
            yield return new object[] { "Venus" };
        }

        [Theory]
        [MemberData(nameof(EveryEngine))]
        public void Every_engine_keeps_rewind(string engine)
        {
            ICore core = engine switch
            {
                "MarsRT" => new MarsRtCore(),
                "Moon" => new MoonCore(),
                "MoonRT" => new MoonRtCore(),
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
