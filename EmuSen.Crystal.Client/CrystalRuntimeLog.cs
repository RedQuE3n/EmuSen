using System.Text.Json;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.Crystal.Client
{
    // The runtime's own lines, written where the program's console writes go, so each per-game log keeps them - see EmuSen_Backend.md §29.11, Q2.
    public sealed class CrystalRuntimeLog : IDisposable
    {
        private readonly CrystalSubscription _events;

        // Made with the runtime, before any session opens, so the lines an open says are kept.
        public CrystalRuntimeLog(CrystalRuntime runtime) => _events = runtime.Subscribe(0, CrystalAbi.EventLog);

        // Each runtime line waiting, to Console.Out as it stands at the call; with nothing waiting it allocates nothing.
        public void File()
        {
            if (!_events.Wait(0)) return;
            using JsonDocument drained = JsonDocument.Parse(_events.Drain());
            foreach (JsonElement e in drained.RootElement.EnumerateArray())
                if (e.TryGetProperty("source", out JsonElement source) && source.GetString() == "runtime")
                    foreach (JsonElement line in e.GetProperty("lines").EnumerateArray()) Console.Out.WriteLine(line.GetString());
        }

        // Files what is still waiting, such as a close's lines, then ends the subscription.
        public void Dispose()
        {
            File();
            _events.Dispose();
        }
    }
}
