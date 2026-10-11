using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Session;

namespace EmuSen.WiseMan.Crystal
{
    // A caller of emusen_session_v1 as a C module calls it, through the table's pointers, recording which functions it reached - see EmuSen_DianaSh.md §13.6.
    public sealed unsafe class SessionClient
    {
        public delegate long Fill(byte* o, nuint n);

        private readonly EmuSenSessionV1* _table;

        public SessionClient(EmuSenSessionV1* table) => _table = table;

        public EmuSenSessionV1* T => _table;
        public void* Ctx => _table->ctx;
        public HashSet<string> Called { get; } = new();

        // Every function field of the table, in the header's order.
        public static IReadOnlyList<string> Functions { get; } = typeof(EmuSenSessionV1).GetFields()
            .Where(f => f.Name is not ("size" or "version" or "ctx")).Select(f => f.Name).ToList();

        // The length query, then the copy, as a C caller makes them; a status comes back with no bytes.
        public (long Status, byte[] Bytes) Out(string function, Fill fill)
        {
            Called.Add(function);
            long total = fill(null, 0);
            if (total < 0) return (total, Array.Empty<byte>());
            var bytes = new byte[total];
            if (total > 0)
            {
                var shortBuffer = new byte[Math.Max(0, (int)total - 1)];
                fixed (byte* s = shortBuffer) Assert.Equal(total, fill(s, (nuint)shortBuffer.Length));
                fixed (byte* p = bytes) Assert.Equal(total, fill(p, (nuint)bytes.Length));
            }
            return (total, bytes);
        }

        public long Call(string function, Func<long> call)
        {
            Called.Add(function);
            return call();
        }

        public JsonElement Json(string function, Fill fill)
        {
            var (status, bytes) = Out(function, fill);
            Assert.True(status >= 0, $"{function} returned {status}: {LastError()}");
            return JsonDocument.Parse(bytes).RootElement.Clone();
        }

        public string Text(string function, Fill fill)
        {
            var (status, bytes) = Out(function, fill);
            Assert.True(status >= 0, $"{function} returned {status}: {LastError()}");
            return Encoding.UTF8.GetString(bytes);
        }

        public string LastError()
        {
            long n = _table->last_error(_table->ctx, null, 0);
            var bytes = new byte[n];
            fixed (byte* p = bytes) _table->last_error(_table->ctx, p, (nuint)bytes.Length);
            Called.Add("last_error");
            return Encoding.UTF8.GetString(bytes);
        }

        // A UTF-8 string pinned for one call; null passes the absent string.
        public static long With(string? text, Func<nint, nuint, long> call)
        {
            if (text is null) return call(0, 0);
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            fixed (byte* p = bytes) return call((nint)p, (nuint)bytes.Length);
        }

        public static long With(byte[] bytes, Func<nint, nuint, long> call)
        {
            fixed (byte* p = bytes) return call((nint)p, (nuint)bytes.Length);
        }
    }
}
