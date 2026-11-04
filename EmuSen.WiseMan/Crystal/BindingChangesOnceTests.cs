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

        // The build's own engines name the card's medium, which the runtime reads from core info (EmuSen_Backend.md §29.14).
        [Fact]
        public void A_card_made_through_the_binding_is_made_once_and_said_done()
        {
            using var runtime = new CrystalRuntime();
            string card = Path.Combine(_f.Root, "Cards", "New.mcr");
            Directory.CreateDirectory(Path.GetDirectoryName(card)!);
            JsonElement answer = JsonDocument.Parse(runtime.CardsOp(CrystalAbi.CardsNew, JsonSerializer.Serialize(new { path = card }))).RootElement;
            Assert.True(answer.GetProperty("done").GetBoolean(), answer.GetProperty("words").GetString());
            Assert.True(File.Exists(card));
            JsonElement again = JsonDocument.Parse(runtime.CardsOp(CrystalAbi.CardsNew, JsonSerializer.Serialize(new { path = card }))).RootElement;
            Assert.False(again.GetProperty("done").GetBoolean(), "a second new over the card made it again");
        }

    }
}
