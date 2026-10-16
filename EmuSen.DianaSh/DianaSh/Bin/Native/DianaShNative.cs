using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Galaxia.Native;

namespace EmuSen.DianaSh.DianaSh.Bin.Native
{
    // The structs of dianash_module.h and dianash.h a host fills or reads - see EmuSen_DianaSh.md §13.7.
    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashBytes
    {
        public byte* Data;
        public nuint Len;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashIo
    {
        public uint Size;
        public void* Ctx;
        public delegate* unmanaged[Cdecl]<void*, byte*, nuint, int> Out;
        public delegate* unmanaged[Cdecl]<void*, byte*, nuint, int> Err;
        public void* Resolve;
        public void* Display;
        public delegate* unmanaged[Cdecl]<void*, int> Cancelled;
        public delegate* unmanaged[Cdecl]<void*, void**, nint*, int> Slot;
        public void* Environment;
        public void* Terminal;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashCall
    {
        public uint Size;
        public DianashBytes Command;
        public DianashBytes* Argv;
        public nuint Argc;
        public DianashBytes StdinData;
        public uint StdinPresent;
        public uint Flags;
        public DianashBytes Cwd;
        public DianashBytes User;
        public DianashIo* Io;
        public void* Services;
        public nuint ServiceCount;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct DianashHostResult
    {
        public uint Size;
        public int ExitStatus;
        public ulong Request;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashHostCommands
    {
        public uint Size;
        public DianashBytes Manifest;
        public void* Ctx;
        public delegate* unmanaged[Cdecl]<void*, DianashCall*, DianashHostResult*, int> Run;
        public delegate* unmanaged[Cdecl]<void*, DianashCall*, int> Jobs;
        public delegate* unmanaged[Cdecl]<void*, DianashCall*, byte*, nuint, DianashHostResult*, int> Kill;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct DianashConfig
    {
        public uint Size;
        public uint Flags;
        public DianashBytes Root;
        public uint BudgetMs;
    }

    // DianaSh's host API in emusen_platform, and the switch that chooses the Rust shell - see EmuSen_DianaSh.md §13.3 and §13.7.
    internal static unsafe class DianaShNative
    {
        public const string Variable = "EMUSEN_DIANASH_NATIVE";

        // dianash_abi_version's major, the one this facade speaks - see EmuSen_DianaSh.md §11.
        public const uint AbiMajor = 1;

        public const int Ok = 0, NeedsMore = 1, Exit = 2;
        public const int Busy = -267, Absent = -268, Exists = -269, NoSuchSession = -270, LastSession = -271;
        public const uint ConfigSessions = 1, ConfigInteractive = 2;

        private static readonly Lazy<string?> Attached = new(Attach);
        private static readonly PlatformSwitch Chosen = SwitchFor(() => Environment.GetEnvironmentVariable(Variable), () => (Attached.Value is null, Attached.Value ?? ""));

        // State 2: the Rust shell unless the variable is 0; a library that is absent or refused leaves the C#, said once - see EmuSen_DianaSh.md §13.10.
        public static bool Active => Chosen.Active;

        // DianaSh's switch as the process makes it, for a test that makes one over another load.
        internal static PlatformSwitch SwitchFor(Func<string?> variable, Func<(bool Available, string Report)> load) => new(variable, load, PlatformSwitch.Announce, "DianaSh");

        // The Rust shell usable, whichever the switch chose: what a parity test asks before making one.
        public static bool Ready => Attached.Value is null;

        internal static delegate* unmanaged[Cdecl]<uint> AbiVersion;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, long> LastErrorFn;
        internal static delegate* unmanaged[Cdecl]<DianashConfig*, nint> New;
        internal static delegate* unmanaged[Cdecl]<nint, int> Free;
        internal static delegate* unmanaged[Cdecl]<nint, uint, DianashHostCommands*, int> AddCommands;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> Submit;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> OutputTake;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> RequestsDrain;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int*, int> LastStatus;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> IsReadOnly;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int> Awaiting;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> History;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> Commands;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> User;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> SetUser;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> Sessions;
        internal static delegate* unmanaged[Cdecl]<nint, long> SessionNew;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> SessionAttach;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int> SessionSwitch;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int> SessionKill;
        internal static delegate* unmanaged[Cdecl]<nint, uint, uint, int> SessionReplace;

        // Every export the facade calls, in the order Attach takes them; a missing one refuses the library.
        internal static readonly string[] Exports =
        {
            "dianash_abi_version", "dianash_last_error", "dianash_new", "dianash_free", "dianash_add_commands", "dianash_submit",
            "dianash_output_take", "dianash_requests_drain", "dianash_last_status", "dianash_is_read_only", "dianash_awaiting",
            "dianash_history", "dianash_commands", "dianash_user", "dianash_set_user", "dianash_sessions", "dianash_session_new",
            "dianash_session_attach", "dianash_session_switch", "dianash_session_kill", "dianash_session_replace",
        };

        // Null when every export resolves, the major is this facade's and a shell can be made and freed; otherwise why not - see EmuSen_DianaSh.md §13.10.
        internal static string? Check(Func<string, nint> export, out nint[] found)
        {
            found = new nint[Exports.Length];
            for (int i = 0; i < Exports.Length; i++)
                if ((found[i] = export(Exports[i])) == 0) return $"{PlatformLibrary.FileName} has no {Exports[i]}";
            uint version = ((delegate* unmanaged[Cdecl]<uint>)found[0])();
            if (version >> 16 != AbiMajor) return $"{PlatformLibrary.FileName} speaks DianaSh host API {version >> 16}.{version & 0xFFFF}, this build {AbiMajor}.x";
            var config = new DianashConfig { Size = (uint)sizeof(DianashConfig) };
            nint shell = ((delegate* unmanaged[Cdecl]<DianashConfig*, nint>)found[2])(&config);
            if (shell == 0) return $"{PlatformLibrary.FileName} could not make a DianaSh shell";
            ((delegate* unmanaged[Cdecl]<nint, int>)found[3])(shell);
            return null;
        }

        // Null when the process's library passed Check, its exports then taken; otherwise why the library is not used.
        private static string? Attach()
        {
            if (!PlatformLibrary.Available) return PlatformLibrary.Report;
            if (!PlatformLibrary.GalaxiaActive) return "Galaxia runs on its C#, and the Rust shell reads the tree through the library's";
            if (Check(PlatformLibrary.Export, out nint[] found) is { } refused) return refused;
            int next = 0;
            AbiVersion = (delegate* unmanaged[Cdecl]<uint>)found[next++];
            LastErrorFn = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            New = (delegate* unmanaged[Cdecl]<DianashConfig*, nint>)found[next++];
            Free = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            AddCommands = (delegate* unmanaged[Cdecl]<nint, uint, DianashHostCommands*, int>)found[next++];
            Submit = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            OutputTake = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            RequestsDrain = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            LastStatus = (delegate* unmanaged[Cdecl]<nint, uint, int*, int>)found[next++];
            IsReadOnly = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            Awaiting = (delegate* unmanaged[Cdecl]<nint, uint, int>)found[next++];
            History = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)found[next++];
            Commands = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)found[next++];
            User = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)found[next++];
            SetUser = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            Sessions = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[next++];
            SessionNew = (delegate* unmanaged[Cdecl]<nint, long>)found[next++];
            SessionAttach = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            SessionSwitch = (delegate* unmanaged[Cdecl]<nint, uint, int>)found[next++];
            SessionKill = (delegate* unmanaged[Cdecl]<nint, uint, int>)found[next++];
            SessionReplace = (delegate* unmanaged[Cdecl]<nint, uint, uint, int>)found[next++];
            return null;
        }

        // The words for the last failing call on this thread.
        public static string LastError() => Text((out_, len) => LastErrorFn((byte*)out_, len));

        // A length-query answer as text: a stack buffer first, then one of the size asked for.
        public static string Text(Func<nint, nuint, long> call) => Encoding.UTF8.GetString(Bytes(call));

        public static byte[] Bytes(Func<nint, nuint, long> call)
        {
            byte* stack = stackalloc byte[512];
            long length = call((nint)stack, 512);
            if (length < 0) throw new InvalidOperationException($"DianaSh: {LastError()} (status {length})");
            if (length <= 512) return new ReadOnlySpan<byte>(stack, (int)length).ToArray();
            var heap = new byte[length];
            fixed (byte* p = heap)
            {
                long again = call((nint)p, (nuint)heap.Length);
                if (again != length) throw new InvalidOperationException($"DianaSh: an answer changed between its two reads ({length}, then {again})");
            }
            return heap;
        }

        // Text with a callback that takes it as UTF-8; empty text is a pointer with no length, never the null that means absent.
        public static int WithUtf8(string text, Func<nint, nuint, int> call)
        {
            byte[] bytes = text.Length == 0 ? Empty : Encoding.UTF8.GetBytes(text);
            fixed (byte* p = bytes) return call((nint)p, (nuint)(text.Length == 0 ? 0 : bytes.Length));
        }

        private static readonly byte[] Empty = new byte[1];

        // NUL-ended entries, as dianash_history gives them.
        public static List<string> NulSeparated(byte[] bytes)
        {
            var entries = new List<string>();
            int start = 0;
            for (int i = 0; i < bytes.Length; i++)
            {
                if (bytes[i] != 0) continue;
                entries.Add(Encoding.UTF8.GetString(bytes, start, i - start));
                start = i + 1;
            }
            return entries;
        }
    }
}
