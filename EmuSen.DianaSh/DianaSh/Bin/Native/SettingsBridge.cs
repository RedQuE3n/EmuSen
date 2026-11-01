using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.DianaSh.DianaSh.Bin.Native
{
    // What a host's runtime answers for the settings service, emusen.settings@1: a refusal is an InvalidOperationException with the runtime's words - see EmuSen_DianaSh.md §9.9.
    public interface IShellSettings
    {
        // A console's setting (scope 0, target its name) or a game's (scope 1, target its path); null where none is stored.
        string? Get(uint scope, string target, string key);
        void Set(uint scope, string target, string key, string? value);
        void Approve(string sha256);
        void Revoke(string sha256);
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashSettingsV1
    {
        public uint Size;
        public void* Ctx;
        public delegate* unmanaged[Cdecl]<void*, uint, DianashBytes, DianashBytes, byte*, nuint, long*, int> Get;
        public delegate* unmanaged[Cdecl]<void*, uint, DianashBytes, DianashBytes, DianashBytes, int> Set;
        public delegate* unmanaged[Cdecl]<void*, DianashBytes, int> Approve;
        public delegate* unmanaged[Cdecl]<void*, DianashBytes, int> Revoke;
        public delegate* unmanaged[Cdecl]<void*, byte*, nuint, long*, int> LastError;
    }

    // The settings service's table over a host's IShellSettings, kept in unmanaged memory for the shell's life; the shell guards it per module.
    internal static unsafe class SettingsBridge
    {
        public const string Service = "emusen.settings";
        public const uint Major = 1, Minor = 0;
        private const int Ok = 0, Null = -1, BufferTooSmall = -7, Absent = -268, Refused = -256;

        [ThreadStatic] private static string? _lastError;

        // A table over settings, with the handle that keeps it reachable; the caller frees both with Release.
        public static DianashSettingsV1* Make(IShellSettings settings, out GCHandle handle)
        {
            handle = GCHandle.Alloc(settings);
            var table = (DianashSettingsV1*)NativeMemory.AllocZeroed((nuint)sizeof(DianashSettingsV1));
            table->Size = (uint)sizeof(DianashSettingsV1);
            table->Ctx = (void*)GCHandle.ToIntPtr(handle);
            table->Get = &Get;
            table->Set = &Set;
            table->Approve = &Approve;
            table->Revoke = &Revoke;
            table->LastError = &LastError;
            return table;
        }

        public static void Release(DianashSettingsV1* table, GCHandle handle)
        {
            NativeMemory.Free(table);
            if (handle.IsAllocated) handle.Free();
        }

        private static IShellSettings Of(void* ctx) => (IShellSettings)GCHandle.FromIntPtr((nint)ctx).Target!;

        private static string? Text(DianashBytes b) => b.Data == null ? null : Encoding.UTF8.GetString(b.Data, (int)b.Len);

        private static int Fail(Exception e)
        {
            _lastError = e.Message;
            return Refused;
        }

        private static int Answer(string text, byte* out_, nuint cap, long* needed)
        {
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            if (needed != null) *needed = bytes.Length;
            if (out_ == null || (nuint)bytes.Length > cap) return BufferTooSmall;
            bytes.AsSpan().CopyTo(new Span<byte>(out_, bytes.Length));
            return Ok;
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int Get(void* ctx, uint scope, DianashBytes target, DianashBytes key, byte* out_, nuint cap, long* needed)
        {
            try { return Of(ctx).Get(scope, Text(target) ?? "", Text(key) ?? "") is { } value ? Answer(value, out_, cap, needed) : Absent; }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int Set(void* ctx, uint scope, DianashBytes target, DianashBytes key, DianashBytes value)
        {
            try { Of(ctx).Set(scope, Text(target) ?? "", Text(key) ?? "", Text(value)); return Ok; }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int Approve(void* ctx, DianashBytes sha256)
        {
            try { Of(ctx).Approve(Text(sha256) ?? ""); return Ok; }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int Revoke(void* ctx, DianashBytes sha256)
        {
            try { Of(ctx).Revoke(Text(sha256) ?? ""); return Ok; }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LastError(void* ctx, byte* out_, nuint cap, long* needed)
        {
            try { return ctx == null ? Null : Answer(_lastError ?? "", out_, cap, needed); }
            catch (Exception e) { return Fail(e); }
        }
    }
}
