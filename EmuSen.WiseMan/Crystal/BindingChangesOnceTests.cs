using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // A call that changes something and answers by the copy-out rule is made once through the binding: the length query's first call would otherwise make the change and the second make it again - see EmuSen_Backend.md §14.13 and §29.
    [Collection(TestCollections.ProcessGlobals)]
    public class BindingChangesOnceTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        [Fact]
        public void A_recording_through_the_binding_is_started_and_stopped_once()
        {
            using CrystalSession s = _f.Open();
            string dir = s.StartRecording(Path.Combine(_f.Root, "Recordings"), 1);
            Assert.True(Directory.Exists(dir));
            var info = new CrystalFrameInfo();
            s.Frame(ref info);
            Assert.Equal(dir, s.StopRecording());
        }

    }
}
