using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;
using System.Collections.Generic;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MarsRT;

namespace EmuSen.WiseMan.Cores
{
    // Rewind is declared by the engine rather than found by its type; since Mars (C#) was retired every engine keeps it - see EmuSen_Settings_Reference.md §4.85.4.
    public class EngineFeaturesTests
    {
        public static IEnumerable<object[]> EveryEngine()
        {
            yield return new object[] { "MarsRT" };
            yield return new object[] { "MoonRT" };
            yield return new object[] { "MercuryRT" };
            yield return new object[] { "VenusRT" };
        }

        [Theory]
        [MemberData(nameof(EveryEngine))]
        public void Every_engine_keeps_rewind(string engine)
        {
            ICore core = engine switch
            {
                "MarsRT" => new MarsRtCore(),
                "MoonRT" => Fixtures.MoonEngines.Create(),
                "MercuryRT" => MercuryEngines.Create(),
                _ => CoreFactory.Create("game.sfc"),
            };
            Assert.Same(EngineFeatures.All, EngineFeatures.Of(core));
            Assert.True(EngineFeatures.Of(core).RewindCapture);
            (core as System.IDisposable)?.Dispose();
        }

        [Fact]
        public void No_core_at_all_has_every_feature() => Assert.True(EngineFeatures.Of(null).RewindCapture);
    }
}
