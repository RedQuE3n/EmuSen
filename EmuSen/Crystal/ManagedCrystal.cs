using System;
using System.IO;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Crystal.Session;

namespace EmuSen.Crystal.Managed
{
    // Step B2's C# implementation of emusen_crystal.h: each export catches what it throws, so nothing unwinds into a caller - see EmuSen_Backend.md §14.
    public static unsafe partial class ManagedCrystal
    {
        [ThreadStatic] private static string? t_lastError;

        private static readonly UTF8Encoding StrictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);

        // The words this implementation gives for itself, with the build it is part of.
        public static string VersionText { get; } =
            $"EmuSen's C# implementation of the frontend API {CrystalNumbers.Major}.{CrystalNumbers.Minor}, " +
            (typeof(ManagedCrystal).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? "unknown build");

        internal static long Fail(Exception e)
        {
            (int status, string words) = e switch
            {
                CrystalException c => (c.Status, c.Message),
                SessionException s => (s.Status, s.Message),
                Cores.Native.CoreRefusedException r => (SessionAbi.Fault, r.Message),
                NotSupportedException n => (SessionAbi.NotSupported, n.Message),
                IOException or UnauthorizedAccessException => (CrystalNumbers.Io, e.Message),
                ObjectDisposedException => (SessionAbi.NoSession, "the session was closed"),
                _ => (SessionAbi.Fault, $"{e.GetType().Name}: {e.Message}"),
            };
            t_lastError = words;
            return status;
        }

        // A twin's answer from the session's table, its words taken across when it failed.
        internal static long Twin(long result, ManagedSession s) => TwinOf(result, s.Table);

        internal static long TwinOf(long result, EmuSenSessionV1* table)
        {
            if (result >= 0) return result;
            long n = table->last_error(table->ctx, null, 0);
            var words = new byte[Math.Max(0, n)];
            fixed (byte* w = words) table->last_error(table->ctx, w, (nuint)words.Length);
            t_lastError = Encoding.UTF8.GetString(words);
            return result;
        }

        internal static long Copy(byte[] bytes, byte* o, nuint n)
        {
            if (o != null && n >= (nuint)bytes.Length) bytes.CopyTo(new Span<byte>(o, bytes.Length));
            return bytes.Length;
        }

        internal static string? Text(byte* p, nuint n)
        {
            if (p == null) return n == 0 ? null : throw new CrystalException(SessionAbi.Null, "a null string with a length");
            try { return StrictUtf8.GetString(p, checked((int)n)); }
            catch (DecoderFallbackException) { throw new CrystalException(SessionAbi.BadString, "text that is not UTF-8"); }
        }

        internal static string Required(byte* p, nuint n, string what) =>
            Text(p, n) is { Length: > 0 } text ? text : throw new CrystalException(SessionAbi.BadArgument, $"{what} must be given");

        internal static ManagedRuntime RuntimeOf(nint rt) =>
            rt == 0 ? throw new CrystalException(SessionAbi.Null, "a null runtime")
            : GCHandle.FromIntPtr(rt).Target as ManagedRuntime ?? throw new CrystalException(SessionAbi.Null, "not a runtime");

        internal static ManagedSession SessionOf(nint rt, uint session) => RuntimeOf(rt).Session(session);

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static uint emusen_crystal_abi_version() => CrystalNumbers.Version;

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_version_text(byte* out_, nuint len)
        {
            try { return Copy(Encoding.UTF8.GetBytes(VersionText), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_last_error(nint rt, byte* out_, nuint len)
        {
            try { return Copy(Encoding.UTF8.GetBytes(t_lastError ?? ""), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint emusen_crystal_new(ConfigV1* config)
        {
            try
            {
                if (config != null && config->size < sizeof(ConfigV1)) throw CrystalException.BadStruct("emusen_crystal_config", config->size, sizeof(ConfigV1));
                if (config != null && config->flags != 0) throw new CrystalException(SessionAbi.NotSupported, $"config flags 0x{config->flags:X} are not known to 1.{CrystalNumbers.Minor}");
                return new ManagedRuntime().Handle;
            }
            catch (Exception e)
            {
                Fail(e);
                return 0;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_free(nint rt)
        {
            try
            {
                RuntimeOf(rt).Dispose();
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }
    }

    // A status with its words, as an export returns it.
    internal sealed class CrystalException(int status, string words) : Exception(words)
    {
        public int Status { get; } = status;

        public static CrystalException BadStruct(string name, uint size, int least) =>
            new(CrystalNumbers.BadStruct, $"{name} of {size} bytes; version 1.0's is {least}");
    }
}
