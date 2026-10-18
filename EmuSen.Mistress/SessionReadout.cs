using System;
using EmuSen.Crystal.Client;
using EmuSen.Serenity.Dashboards;

namespace EmuSen.Mistress
{
    // The window's current session read through the binding for the coretop window; no session, or a call refused, reads as nothing - see EmuSen_Serenity.md §2.11.
    public sealed class SessionReadout(Func<CrystalSession?> session) : ISessionReadout
    {
        public string? Telemetry(uint kind) => Read(s => s.Telemetry(kind));
        public string? Info() => Read(s => s.ServiceInfo());
        public string? Summary() => Read(s => s.Summary());
        public byte[]? TileSheet() => Read(s => s.TileSheet());
        public byte[]? PaletteSwatch() => Read(s => s.PaletteSwatch());
        public byte[]? AudioPeek() => Read(s => s.AudioPeek());

        private T? Read<T>(Func<CrystalSession, T> call) where T : class
        {
            if (session() is not { Closed: false } s) return null;
            try { return call(s); }
            catch (CrystalException) { return null; }
        }
    }
}
