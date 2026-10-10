using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Native;
using Silk.NET.Core.Contexts;

namespace EmuSen.Serenity.Native
{
    // Serenity's half of emusen_platform, the switch that chooses it over the C#, and the lending of Shaderc and SQLite - see EmuSen_RustPlatform.md §3.9, §16 and §17.
    internal static unsafe class SerenityNative
    {
        public const string Variable = "EMUSEN_SERENITY_NATIVE";

        internal const int Absent = -1030, Parse = -1025, BadArgument = -1033;
        internal const uint RawInstance = 0, RawDevice = 1, MarkPhase = 0, MarkGpu = 1, ChainCopiedImages = 0, ChainReadbacks = 1, ChainMaxLent = 2;
        internal const uint CacheWorking = 0, CacheHits = 1, CacheMisses = 2, CacheStored = 3, CacheDamaged = 4, CacheSkipped = 5, CacheEvicted = 6, CacheTouchAfter = 7;
        internal const uint ClassSpace = 0, ClassWord = 1, NumberFloat = 0, NumberInteger = 1;

        private static readonly Lazy<string?> Attached = new(Attach);
        private static readonly PlatformSwitch Chosen = SwitchFor(() => Environment.GetEnvironmentVariable(Variable), () => (Attached.Value is null, Attached.Value ?? ""));

        // State 2: the library unless the variable is 0; one that is absent, refused or not lent Shaderc and SQLite leaves the C#, said once - see EmuSen_RustPlatform.md §19.
        public static bool Active => Chosen.Active;

        // Serenity's switch as the process makes it, for a test that makes one of its own over another load.
        internal static PlatformSwitch SwitchFor(Func<string?> variable, Func<(bool Available, string Report)> load) => new(variable, load, PlatformSwitch.Announce, "Serenity's slang half");

        // The library itself, whichever way the switch stands: what a parity test calls.
        public static bool Ready => Attached.Value is null;

        public static string Report => Attached.Value ?? PlatformLibrary.Report;

        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, int> ShadercLend;
        internal static delegate* unmanaged[Cdecl]<nint, int> SqliteLend;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, long> Take;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, long> CompilerIdentity;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> PresetLoad;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> SourceLoad;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> ParametersMerge;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, uint, byte*, nuint, byte*, nuint, long> Compile;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> Reflect;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> ReflectMerge;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> WithoutUnreadInputs;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, long, byte*, nuint, nint> CacheNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> CacheFree;
        internal static delegate* unmanaged[Cdecl]<nint, int> CacheClose;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, uint, byte*, nuint, long, byte*, nuint, long> CacheCompile;
        internal static delegate* unmanaged[Cdecl]<nint, uint, long> CacheGet;
        internal static delegate* unmanaged[Cdecl]<nint, long, int> CacheSetTouchAfter;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> CacheProblem;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, uint, byte*, nuint, byte*, int> CacheKey;
        internal static delegate* unmanaged[Cdecl]<uint, uint, int> TextClass;
        internal static delegate* unmanaged[Cdecl]<uint, byte*, nuint, uint*, int> ParseNumber;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, int, nint> DeviceNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> DeviceClose;
        internal static delegate* unmanaged[Cdecl]<nint, int> DeviceFree;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> DeviceName;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> DeviceFindings;
        internal static delegate* unmanaged[Cdecl]<nint, uint, ulong> DeviceRaw;
        internal static delegate* unmanaged[Cdecl]<nint, int, int> DeviceProbe;
        internal static delegate* unmanaged[Cdecl]<nint, Mark*, nuint, long> DeviceMarks;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, nint, long, int, nint> ChainNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> ChainFree;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> ChainParameters;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int, int, byte*, nuint, int> ChainLut;
        internal static delegate* unmanaged[Cdecl]<nint, float*, nuint, int> ChainSetParameters;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, float*, int> ChainValue;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, int, int, int, int> ChainAdvance;
        internal static delegate* unmanaged[Cdecl]<nint, int, int, byte*, nuint, long> ChainRender;
        internal static delegate* unmanaged[Cdecl]<nint, int, int, Lent*, int> ChainRenderLent;
        internal static delegate* unmanaged[Cdecl]<ulong, int> LentRelease;
        internal static delegate* unmanaged[Cdecl]<nint, uint, long> ChainGet;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, int, int*, int> ParseFormat;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, long> LastError;

        // emusen_serenity_mark: one thing a device's probe saw.
        [StructLayout(LayoutKind.Sequential)]
        internal struct Mark
        {
            public uint Kind;
            public int Point;
            public double Value;
        }

        // emusen_serenity_lent: a picture over a readback, and the number that gives the readback back.
        [StructLayout(LayoutKind.Sequential)]
        internal struct Lent
        {
            public uint Size;
            public byte* Pixels;
            public nuint Bytes;
            public int Width, Height, RowBytes;
            public ulong Slot;
        }

        // Every export this half calls, in the order Attach takes them; a name the library lacks refuses it.
        internal static readonly string[] Exports =
        {
            "emusen_serenity_shaderc_lend", "emusen_serenity_sqlite_lend", "emusen_serenity_take", "emusen_serenity_compiler_identity", "emusen_serenity_preset_load",
            "emusen_serenity_source_load", "emusen_serenity_parameters_merge", "emusen_serenity_compile", "emusen_serenity_reflect", "emusen_serenity_reflect_merge",
            "emusen_serenity_without_unread_inputs", "emusen_serenity_cache_new", "emusen_serenity_cache_free", "emusen_serenity_cache_close", "emusen_serenity_cache_compile",
            "emusen_serenity_cache_get", "emusen_serenity_cache_set_touch_after", "emusen_serenity_cache_problem", "emusen_serenity_cache_key", "emusen_serenity_text_class",
            "emusen_serenity_parse_number", "emusen_serenity_device_new", "emusen_serenity_device_close", "emusen_serenity_device_free", "emusen_serenity_device_name",
            "emusen_serenity_device_findings", "emusen_serenity_device_raw", "emusen_serenity_device_probe", "emusen_serenity_device_marks", "emusen_serenity_chain_new",
            "emusen_serenity_chain_free", "emusen_serenity_chain_parameters", "emusen_serenity_chain_lut", "emusen_serenity_chain_set_parameters", "emusen_serenity_chain_value",
            "emusen_serenity_chain_advance", "emusen_serenity_chain_render", "emusen_serenity_chain_render_lent", "emusen_serenity_lent_release", "emusen_serenity_chain_get",
            "emusen_serenity_parse_format", "emusen_platform_last_error",
        };

        // Null when every export resolved and both libraries were lent; otherwise why the library is not in use.
        private static string? Attach()
        {
            if (!PlatformLibrary.Available) return PlatformLibrary.Report;
            var found = new nint[Exports.Length];
            for (int i = 0; i < Exports.Length; i++)
                if ((found[i] = PlatformLibrary.Export(Exports[i])) == 0) return $"{PlatformLibrary.FileName} has no {Exports[i]}";

            int next = 0;
            ShadercLend = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, int>)found[next++];
            SqliteLend = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            Take = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            CompilerIdentity = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            PresetLoad = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            SourceLoad = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            ParametersMerge = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            Compile = (delegate* unmanaged[Cdecl]<byte*, nuint, uint, byte*, nuint, byte*, nuint, long>)found[next++];
            Reflect = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            ReflectMerge = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            WithoutUnreadInputs = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            CacheNew = (delegate* unmanaged[Cdecl]<byte*, nuint, long, byte*, nuint, nint>)found[next++];
            CacheFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            CacheClose = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            CacheCompile = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, uint, byte*, nuint, long, byte*, nuint, long>)found[next++];
            CacheGet = (delegate* unmanaged[Cdecl]<nint, uint, long>)found[next++];
            CacheSetTouchAfter = (delegate* unmanaged[Cdecl]<nint, long, int>)found[next++];
            CacheProblem = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            CacheKey = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, uint, byte*, nuint, byte*, int>)found[next++];
            TextClass = (delegate* unmanaged[Cdecl]<uint, uint, int>)found[next++];
            ParseNumber = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, uint*, int>)found[next++];
            DeviceNew = (delegate* unmanaged[Cdecl]<byte*, nuint, int, nint>)found[next++];
            DeviceClose = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            DeviceFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            DeviceName = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            DeviceFindings = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            DeviceRaw = (delegate* unmanaged[Cdecl]<nint, uint, ulong>)found[next++];
            DeviceProbe = (delegate* unmanaged[Cdecl]<nint, int, int>)found[next++];
            DeviceMarks = (delegate* unmanaged[Cdecl]<nint, Mark*, nuint, long>)found[next++];
            ChainNew = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, nint, long, int, nint>)found[next++];
            ChainFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            ChainParameters = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            ChainLut = (delegate* unmanaged[Cdecl]<nint, uint, int, int, byte*, nuint, int>)found[next++];
            ChainSetParameters = (delegate* unmanaged[Cdecl]<nint, float*, nuint, int>)found[next++];
            ChainValue = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, float*, int>)found[next++];
            ChainAdvance = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, int, int, int, int>)found[next++];
            ChainRender = (delegate* unmanaged[Cdecl]<nint, int, int, byte*, nuint, long>)found[next++];
            ChainRenderLent = (delegate* unmanaged[Cdecl]<nint, int, int, Lent*, int>)found[next++];
            LentRelease = (delegate* unmanaged[Cdecl]<ulong, int>)found[next++];
            ChainGet = (delegate* unmanaged[Cdecl]<nint, uint, long>)found[next++];
            ParseFormat = (delegate* unmanaged[Cdecl]<byte*, nuint, int, int*, int>)found[next++];
            LastError = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            return Lend();
        }

        // The two libraries the C# has loaded, by the handles its own bindings hold, so the SPIR-V is one compiler's and the process has one SQLite - see EmuSen_RustPlatform.md §16.2.
        private static string? Lend()
        {
            try
            {
                nint shaderc = ((DefaultNativeContext)Slang.SlangCompiler.Managed.Api.Context).Library.Handle;
                byte[]? file = Utf8(Slang.SlangCompiler.Managed.NativePath());
                int lent;
                fixed (byte* f = file) lent = ShadercLend(shaderc, file is null ? null : Pin(f), (nuint)(file?.Length ?? 0));
                if (lent != 0) return Words();

                SQLitePCL.Batteries_V2.Init();
                if (!NativeLibrary.TryLoad(SQLitePCL.raw.GetNativeLibraryName(), typeof(SQLitePCL.SQLite3Provider_e_sqlite3).Assembly, null, out nint sqlite)) return "SQLite could not be found as the C# loads it";
                return SqliteLend(sqlite) == 0 ? null : Words();
            }
            catch (Exception e) when (e is InvalidCastException or DllNotFoundException or FileNotFoundException or TypeInitializationException or InvalidOperationException)
            {
                return $"Shaderc or SQLite could not be lent ({e.GetType().Name}: {e.Message.Split('\n')[0]})";
            }
        }

        // The words for the last failing call on this thread.
        internal static string Words()
        {
            byte* small = stackalloc byte[512];
            long length = LastError(small, 512);
            if (length <= 512) return Encoding.UTF8.GetString(small, (int)Math.Max(0, length));
            byte[] large = new byte[length];
            fixed (byte* p = large) LastError(p, (nuint)large.Length);
            return Encoding.UTF8.GetString(large);
        }

        // Whether a string has UTF-8 of its own: UTF-16 with half a surrogate pair has none, and null is not text.
        internal static bool Crosses(string? text)
        {
            if (text is null) return false;
            for (int i = 0; i < text.Length; i++)
            {
                if (char.IsHighSurrogate(text[i]) && i + 1 < text.Length && char.IsLowSurrogate(text[i + 1])) i++;
                else if (char.IsSurrogate(text[i])) return false;
            }
            return true;
        }

        internal static byte[]? Utf8(string? text) => text is null ? null : Encoding.UTF8.GetBytes(text);

        // A pinned array's pointer, never null for an empty one, which fixed makes null.
        internal static byte* Pin(byte* pinned) => pinned is null ? (byte*)1 : pinned;

        internal delegate long Call(byte* buffer, nuint capacity);

        // A call's bytes by the waiting idiom: the room given when they fitted, else taken from where they wait; null for ABSENT, and the status for any other failure.
        internal static byte[]? Bytes(Call call, out long status, int room = 16 * 1024)
        {
            byte[] buffer = new byte[room];
            fixed (byte* b = buffer) status = call(b, (nuint)buffer.Length);
            if (status < 0) return null;
            if (status <= buffer.Length) return buffer.AsSpan(0, (int)status).ToArray();
            byte[] whole = new byte[status];
            fixed (byte* w = whole) Take(w, (nuint)whole.Length);
            return whole;
        }

        // A call's JSON document; a status below zero is the library's own failure, which no input of the C#'s should cause.
        internal static JsonDocument Document(Call call)
        {
            byte[]? bytes = Bytes(call, out long status);
            if (bytes is null) throw new InvalidOperationException(Words());
            return JsonDocument.Parse(bytes);
        }

        // A text the library answers by the length-query idiom, asked again with room when the first is too small; null for ABSENT.
        internal static string? Text(Call call)
        {
            byte* small = stackalloc byte[512];
            long length = call(small, 512);
            if (length == Absent) return null;
            if (length < 0) throw new InvalidOperationException(Words());
            if (length <= 512) return Encoding.UTF8.GetString(small, (int)length);
            byte[] large = new byte[length];
            fixed (byte* p = large) call(p, (nuint)large.Length);
            return Encoding.UTF8.GetString(large);
        }

        // The library's identity for the compiler, which names its own binding - see EmuSen_RustPlatform.md §16.2.
        internal static string Identity() => Text((buffer, capacity) => CompilerIdentity(buffer, capacity))!;

        // The exception a reader's failure document stands for, or null for a document that is not one.
        internal static Exception? Failure(JsonElement root)
        {
            if (!root.TryGetProperty("fail", out JsonElement kind)) return null;
            string message = root.GetProperty("message").GetString()!;
            return kind.GetString() switch
            {
                "not_found" => new FileNotFoundException(message, root.GetProperty("file").GetString()),
                "invalid_data" => new InvalidDataException(message),
                "argument" => new ArgumentException(message, root.GetProperty("parameter").GetString()),
                "denied" => new UnauthorizedAccessException(message),
                "not_spirv" => new ArgumentException(message),
                "index" => new IndexOutOfRangeException(),
                "range" => new ArgumentOutOfRangeException(),
                "endless" => new ArgumentException("Not SPIR-V: a type holds itself."),
                "compile" => new Slang.SlangCompileException(message),
                "device" => new InvalidOperationException(message),
                "overflow" => new OverflowException(),
                _ => new IOException(message),
            };
        }

        internal static float Float(JsonElement bits) => BitConverter.UInt32BitsToSingle(bits.GetUInt32());

        internal static uint Bits(float value) => BitConverter.SingleToUInt32Bits(value);
    }

    // A handle the library made, freed once by whoever is done with it or by the collector.
    internal sealed unsafe class SerenityHandle : IDisposable
    {
        private nint _handle;
        private readonly delegate* unmanaged[Cdecl]<nint, int> _free;

        public SerenityHandle(nint handle, delegate* unmanaged[Cdecl]<nint, int> free)
        {
            if (handle == 0) throw new InvalidOperationException(SerenityNative.Words());
            _handle = handle;
            _free = free;
        }

        public nint Value => _handle;

        public void Dispose()
        {
            Free();
            GC.SuppressFinalize(this);
        }

        ~SerenityHandle() => Free();

        private void Free()
        {
            nint handle = System.Threading.Interlocked.Exchange(ref _handle, 0);
            if (handle != 0) _free(handle);
        }
    }
}
