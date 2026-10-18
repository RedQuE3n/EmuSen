using System;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.Hotaru
{
    // The session events a game window acts on between frames: a state loaded by anyone resets the audio rate control (C7), the session closed ends the window - see EmuSen_Backend.md §14.11.
    public sealed class HotaruEvents : IDisposable
    {
        private readonly CrystalSubscription _events;

        // Every session's, so a swap needs no new subscription; each drain keeps the current session's.
        public HotaruEvents(CrystalRuntime runtime) => _events = runtime.Subscribe(0, CrystalAbi.EventState | CrystalAbi.EventSession);

        // True when the current session was closed under the window; stateLoaded once for each state it loaded.
        public bool Drain(uint current, Action stateLoaded)
        {
            if (!_events.Wait(0)) return false;
            using JsonDocument events = JsonDocument.Parse(_events.Drain());
            bool closed = false;
            foreach (JsonElement e in events.RootElement.EnumerateArray())
            {
                if (!e.TryGetProperty("session", out JsonElement id) || id.GetUInt32() != current) continue;
                string? kind = e.GetProperty("kind").GetString(), op = e.TryGetProperty("op", out JsonElement o) ? o.GetString() : null;
                if (kind == "state" && op == "loaded") stateLoaded();
                if (kind == "session" && op == "closed") closed = true;
            }
            return closed;
        }

        public void Dispose() => _events.Dispose();
    }
}
