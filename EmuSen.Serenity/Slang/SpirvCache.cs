using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity.Native;
using Microsoft.Data.Sqlite;

namespace EmuSen.Serenity.Slang
{
    // Compiled SPIR-V kept in SQLite by a hash of everything a compile reads; a miss, a damaged row or an unusable database compiles instead - see EmuSen_Serenity.md §9.4.
    public sealed class SpirvCache : IDisposable
    {
        public const string FileName = "spirv-cache.db";
        public const long DefaultLimit = 64L << 20;
        public const int SchemaVersion = 1;

        // Beside the packs it serves, in Galaxia's data home - see EmuSen_Serenity.md §9.4.
        public static string DefaultPath => System.IO.Path.Combine(DataStore.Shaders, FileName);

        // The library's cache and the clock a test may give it, which the library is told the time by - see EmuSen_RustPlatform.md §16.2.
        private readonly SerenityHandle _native;
        private Func<long> _clock = () => DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        internal SerenityHandle NativeValue => _native;

        public string Path { get; }

        // Why the cache is off, or null while it works; builds go on compiling either way.
        public unsafe string? Problem => Kept(SerenityNative.Text((buffer, capacity) => SerenityNative.CacheProblem(_native.Value, buffer, capacity)));

        public bool Working => Get(SerenityNative.CacheWorking) != 0;

        internal int Hits => (int)Get(SerenityNative.CacheHits);
        internal int Misses => (int)Get(SerenityNative.CacheMisses);
        internal int Stored => (int)Get(SerenityNative.CacheStored);
        internal int Damaged => (int)Get(SerenityNative.CacheDamaged);
        internal int Skipped => (int)Get(SerenityNative.CacheSkipped);
        internal int Evicted => (int)Get(SerenityNative.CacheEvicted);

        // Unix seconds, and how old a row's use must be before a hit writes it again; a test sets both.
        internal Func<long> Clock
        {
            get => _clock;
            set => _clock = value;
        }

        internal unsafe long TouchAfter
        {
            get => Get(SerenityNative.CacheTouchAfter);
            set
            {
                SerenityNative.CacheSetTouchAfter(_native.Value, value);
                GC.KeepAlive(_native);
            }
        }

        // The library's cache; half a surrogate pair in the path or the identity crosses as U+FFFD - see EmuSen_Backend.md §40.6.
        public unsafe SpirvCache(string path, long limit = DefaultLimit, string? compilerIdentity = null)
        {
            SerenityNative.Up();
            Path = path;
            byte[] file = SerenityNative.Utf8(path)!;
            byte[]? identity = SerenityNative.Utf8(compilerIdentity);
            nint made;
            fixed (byte* f = file, i = identity)
                made = SerenityNative.CacheNew(SerenityNative.Pin(f), (nuint)file.Length, limit, identity is null ? null : SerenityNative.Pin(i), (nuint)(identity?.Length ?? 0));
            _native = new SerenityHandle(made, SerenityNative.CacheFree);
        }

        // The stage's SPIR-V, from the cache or compiled and kept; a stage that does not compile throws as SlangCompiler does, and is not kept.
        public unsafe byte[] Compile(string text, SlangStage stage, string name)
        {
            byte[] source = Encoding.UTF8.GetBytes(text), file = Encoding.UTF8.GetBytes(name);
            long now = _clock();
            byte[]? spirv = SerenityNative.Bytes((buffer, capacity) =>
            {
                fixed (byte* s = source, f = file)
                    return SerenityNative.CacheCompile(_native.Value, SerenityNative.Pin(s), (nuint)source.Length, (uint)stage, SerenityNative.Pin(f), (nuint)file.Length, now, buffer, capacity);
            }, out long status);
            GC.KeepAlive(_native);
            if (spirv is not null) return spirv;
            if (status == SerenityNative.Parse) throw new SlangCompileException(SerenityNative.Words());
            throw new InvalidOperationException(SerenityNative.Words());
        }

        // SHA-256 over each part, length-prefixed, so no two different inputs run together into the same bytes.
        internal static byte[] Key(string identity, string text, SlangStage stage, string name) => KeyNative(identity, text, stage, name);

        internal static unsafe byte[] KeyNative(string identity, string text, SlangStage stage, string name)
        {
            byte[] who = Encoding.UTF8.GetBytes(identity), source = Encoding.UTF8.GetBytes(text), file = Encoding.UTF8.GetBytes(name), key = new byte[32];
            fixed (byte* w = who, s = source, f = file, k = key)
                if (SerenityNative.CacheKey(SerenityNative.Pin(w), (nuint)who.Length, SerenityNative.Pin(s), (nuint)source.Length, (uint)stage, SerenityNative.Pin(f), (nuint)file.Length, k) != 0)
                    throw new InvalidOperationException(SerenityNative.Words());
            return key;
        }

        public unsafe void Dispose()
        {
            SerenityNative.CacheClose(_native.Value);
            GC.KeepAlive(_native);
        }

        private unsafe long Get(uint which)
        {
            long value = SerenityNative.CacheGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        private string? Kept(string? text)
        {
            GC.KeepAlive(_native);
            return text;
        }
    }
}
