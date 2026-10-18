using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Threading;

namespace EmuSen.Crystal.Managed
{
    // One subscriber's bounded queue of events: drained by the client, the oldest dropped and counted when it is full - see EmuSen_Backend.md §3.5.
    internal sealed class ManagedSubscription : IDisposable
    {
        public const int Capacity = 256;

        private readonly Queue<string> _queue = new();
        private readonly object _gate = new();
        private readonly ManagedEvents _owner;
        private long _dropped;

        public ManagedSubscription(ManagedEvents owner, uint session, uint kinds)
        {
            _owner = owner;
            Session = session;
            Kinds = kinds;
            Handle = GCHandle.ToIntPtr(GCHandle.Alloc(this));
        }

        public uint Session { get; }
        public uint Kinds { get; }
        public nint Handle { get; private set; }

        public bool Wants(uint session, uint kind) => (Session == 0 || Session == session) && (Kinds == 0 || (Kinds & kind) != 0);

        public void Add(string json)
        {
            lock (_gate)
            {
                if (_queue.Count == Capacity)
                {
                    _queue.Dequeue();
                    _dropped++;
                }
                _queue.Enqueue(json);
                Monitor.PulseAll(_gate);
            }
        }

        // The waiting events as one array, removed only when the buffer takes them all; the bytes asked for are its length.
        public unsafe long Drain(byte* o, nuint n)
        {
            lock (_gate)
            {
                var text = new StringBuilder("[");
                bool first = true;
                if (_dropped > 0)
                {
                    text.Append($"{{\"kind\":\"dropped\",\"count\":{_dropped}}}");
                    first = false;
                }
                foreach (string e in _queue)
                {
                    if (!first) text.Append(',');
                    text.Append(e);
                    first = false;
                }
                text.Append(']');
                byte[] bytes = Encoding.UTF8.GetBytes(text.ToString());
                if (o == null || n < (nuint)bytes.Length) return bytes.Length;
                bytes.CopyTo(new Span<byte>(o, bytes.Length));
                _queue.Clear();
                _dropped = 0;
                return bytes.Length;
            }
        }

        public bool Wait(int timeoutMs)
        {
            lock (_gate)
            {
                if (_queue.Count > 0 || _dropped > 0) return true;
                Monitor.Wait(_gate, timeoutMs);
                return _queue.Count > 0 || _dropped > 0;
            }
        }

        public void Dispose()
        {
            if (Handle == 0) return;
            _owner.Remove(this);
            GCHandle.FromIntPtr(Handle).Free();
            Handle = 0;
        }
    }

    // The runtime's subscriptions, and the one place an event is made: once, and only when someone wants it.
    internal sealed class ManagedEvents
    {
        private readonly List<ManagedSubscription> _subscriptions = new();
        private readonly object _gate = new();

        public ManagedSubscription Subscribe(uint session, uint kinds)
        {
            var subscription = new ManagedSubscription(this, session, kinds);
            lock (_gate) _subscriptions.Add(subscription);
            return subscription;
        }

        public void Remove(ManagedSubscription subscription)
        {
            lock (_gate) _subscriptions.Remove(subscription);
        }

        public bool Wanted(uint session, uint kind)
        {
            lock (_gate)
                foreach (ManagedSubscription s in _subscriptions)
                    if (s.Wants(session, kind)) return true;
            return false;
        }

        public void Raise(uint session, uint kind, string name, Action<Utf8JsonWriter>? fields = null)
        {
            if (!Wanted(session, kind)) return;
            string json = Encoding.UTF8.GetString(CrystalJson.Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("kind", name);
                w.WriteNumber("session", session);
                fields?.Invoke(w);
                w.WriteEndObject();
            }));
            lock (_gate)
                foreach (ManagedSubscription s in _subscriptions)
                    if (s.Wants(session, kind)) s.Add(json);
        }

        public void Clear()
        {
            ManagedSubscription[] all;
            lock (_gate) all = _subscriptions.ToArray();
            foreach (ManagedSubscription s in all) s.Dispose();
        }
    }
}
