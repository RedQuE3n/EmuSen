using System;
using System.Collections.Generic;
using System.Text.Json;
using EmuSen.Crystal.Session;

namespace EmuSen.DianaSh.DianaSh.Bin.Native
{
    // A session service table a shell is handed: this process's C# provider, or one the platform library's Crystal made - see EmuSen_Backend.md §29.25.
    internal sealed unsafe class ServiceTable
    {
        private readonly SessionService? _provider;

        public ServiceTable(SessionService provider)
        {
            _provider = provider;
            Table = provider.Table;
        }

        public ServiceTable(nint table) => Table = (EmuSenSessionV1*)table;

        public EmuSenSessionV1* Table { get; }

        // Run control's requests since the last drain, oldest first: the provider's own, or the table's events_drain.
        public IReadOnlyList<string> DrainRunRequests() => _provider?.DrainRunRequests() ?? DrainEvents();

        private IReadOnlyList<string> DrainEvents()
        {
            long n = Table->events_drain(Table->ctx, 0, SessionAbi.EventRun, null, 0);
            if (n <= 0) return Array.Empty<string>();
            var bytes = new byte[n];
            fixed (byte* b = bytes) n = Table->events_drain(Table->ctx, 0, SessionAbi.EventRun, b, (nuint)bytes.Length);
            if (n <= 0) return Array.Empty<string>();
            using JsonDocument events = JsonDocument.Parse(bytes.AsMemory(0, (int)n));
            var ops = new List<string>();
            foreach (JsonElement e in events.RootElement.EnumerateArray()) ops.Add(e.GetProperty("op").GetString() ?? "");
            return ops;
        }
    }
}
