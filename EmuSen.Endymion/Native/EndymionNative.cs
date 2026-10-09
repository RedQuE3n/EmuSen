using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Native;

namespace EmuSen.Endymion.Native
{
    // A change the router asks the host to make, as emusen_platform.h's emusen_endymion_change.
    [StructLayout(LayoutKind.Sequential)]
    internal struct RouterChange
    {
        public uint Kind;
        public int Port;
        public uint Which;
        public uint On;
        public double Value;
    }

    // A rate control's whole state, as emusen_endymion_rate_state.
    [StructLayout(LayoutKind.Sequential)]
    internal struct RateState
    {
        public uint Size;
        public int TargetQueuedFrames;
        public int SheddingEvents;
        public uint IsShedding;
        public long TotalInputFrames;
        public long TotalOutputFrames;
        public double MaxDeviation;
        public double SheddingEntryFactor;
        public double SheddingExitFactor;
        public double NominalRatio;
        public double LastRatio;
    }

    // A seat as the seats' rules take it, as emusen_endymion_seat.
    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct SeatIn
    {
        public uint State;
        public byte* Guid;
        public nuint GuidLength;
        public byte* Path;
        public nuint PathLength;
    }

    // Endymion's half of emusen_platform, and the switch that chooses it over the C# - see EmuSen_RustPlatform.md §3.9 and §12.
    internal static unsafe class EndymionNative
    {
        public const string Variable = "EMUSEN_ENDYMION_NATIVE";

        internal const int Null = -1, BadArgument = -1033;
        internal const uint ChangeConnected = 0, ChangeButton = 1, ChangeAxis = 2;
        internal const uint RouterKeyboardPlayer = 0, RouterMirror = 1, RouterPorts = 2;
        internal const uint RateTarget = 0, RateMaxDeviation = 1, RateEntry = 2, RateExit = 3, RateNominal = 4;

        private static readonly Lazy<bool> Chosen = new(Choose);
        private static readonly Lazy<string?> Attached = new(Attach);

        // State 1: the C# unless the variable asks for the library and the library loads; unset, Endymion asks nothing of the library.
        public static bool Active => Chosen.Value;

        private static bool Choose()
        {
            if (Environment.GetEnvironmentVariable(Variable) != "1") return false;
            if (Attached.Value is null) return true;
            ConfigDiagnostics.Report($"{Variable}=1 was asked for and {Attached.Value}; Endymion runs on its C# implementation.");
            return false;
        }

        // The library itself, whichever the switch chose: what a parity test calls.
        public static bool Ready => Attached.Value is null;

        public static string Report => Attached.Value ?? PlatformLibrary.Report;

        internal static delegate* unmanaged[Cdecl]<nint> ResamplerNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> ResamplerFree;
        internal static delegate* unmanaged[Cdecl]<nint, int> ResamplerReset;
        internal static delegate* unmanaged[Cdecl]<nint, short*, nuint, double, short*, nuint, long> ResamplerRun;
        internal static delegate* unmanaged[Cdecl]<nint, short*, nuint, long> ResamplerTake;
        internal static delegate* unmanaged[Cdecl]<int, nint> RateNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> RateFree;
        internal static delegate* unmanaged[Cdecl]<nint, int> RateReset;
        internal static delegate* unmanaged[Cdecl]<nint, uint, double, int> RateSet;
        internal static delegate* unmanaged[Cdecl]<nint, RateState*, int> RateRead;
        internal static delegate* unmanaged[Cdecl]<nint, int, double*, int> RateCompute;
        internal static delegate* unmanaged[Cdecl]<nint, short*, nuint, int, short*, nuint, long> RateProcess;
        internal static delegate* unmanaged[Cdecl]<nint, short*, nuint, long> RateTake;
        internal static delegate* unmanaged[Cdecl]<nint> RouterNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> RouterFree;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int> RouterGet;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int, int> RouterSet;
        internal static delegate* unmanaged[Cdecl]<nint, int, uint*, nuint, int> RouterReset;
        internal static delegate* unmanaged[Cdecl]<nint, int, RouterChange*, nuint, long> RouterResize;
        internal static delegate* unmanaged[Cdecl]<nint, ushort*, nuint, double*, nuint, uint, uint, RouterChange*, nuint, long> RouterPoll;
        internal static delegate* unmanaged[Cdecl]<nint, uint, uint, RouterChange*, nuint, long> RouterSend;
        internal static delegate* unmanaged[Cdecl]<nint, RouterChange*, nuint, long> RouterTake;
        internal static delegate* unmanaged[Cdecl]<nint, int, uint, int> RouterPadHeld;
        internal static delegate* unmanaged[Cdecl]<nint, int, uint, int> RouterConnected;
        internal static delegate* unmanaged[Cdecl]<SeatIn*, byte*, nuint, byte*, nuint, int> SlotsSeat;
        internal static delegate* unmanaged[Cdecl]<SeatIn*, uint, int, int, int*, int> SlotsMove;
        internal static delegate* unmanaged[Cdecl]<SeatIn*, int, int> SlotsForget;
        internal static delegate* unmanaged[Cdecl]<SeatIn*, int> SlotsHighest;
        internal static delegate* unmanaged[Cdecl]<uint, double, uint, double*, int> PadResolve;
        internal static delegate* unmanaged[Cdecl]<double, uint, uint, double*, int> PadCombine;
        internal static delegate* unmanaged[Cdecl]<uint*, nuint, uint*, nuint, uint*, nuint, long> PadControlsFor;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, long> LastError;

        // The logic half's exports, in the order Attach takes them.
        private static readonly string[] Logic =
        {
            "emusen_endymion_resampler_new", "emusen_endymion_resampler_free", "emusen_endymion_resampler_reset", "emusen_endymion_resampler_run", "emusen_endymion_resampler_take",
            "emusen_endymion_rate_new", "emusen_endymion_rate_free", "emusen_endymion_rate_reset", "emusen_endymion_rate_set", "emusen_endymion_rate_read", "emusen_endymion_rate_compute",
            "emusen_endymion_rate_process", "emusen_endymion_rate_take", "emusen_endymion_router_new", "emusen_endymion_router_free", "emusen_endymion_router_get",
            "emusen_endymion_router_set", "emusen_endymion_router_reset", "emusen_endymion_router_resize", "emusen_endymion_router_poll", "emusen_endymion_router_send",
            "emusen_endymion_router_take", "emusen_endymion_router_pad_held", "emusen_endymion_router_connected", "emusen_endymion_slots_seat", "emusen_endymion_slots_move",
            "emusen_endymion_slots_forget", "emusen_endymion_slots_highest", "emusen_endymion_pad_resolve", "emusen_endymion_pad_combine", "emusen_endymion_pad_controls_for",
            "emusen_platform_last_error",
        };

        // Every export Endymion calls, both halves; a name the library lacks refuses it - see EmuSen_RustPlatform.md §3.1.
        internal static readonly string[] Exports = [.. Logic, .. DeviceNative.Exports];

        // Null when every export resolved; otherwise why the library is not in use.
        private static string? Attach()
        {
            if (!PlatformLibrary.Available) return PlatformLibrary.Report;
            var found = new nint[Exports.Length];
            for (int i = 0; i < Exports.Length; i++)
                if ((found[i] = PlatformLibrary.Export(Exports[i])) == 0) return $"{PlatformLibrary.FileName} has no {Exports[i]}";

            int next = 0;
            ResamplerNew = (delegate* unmanaged[Cdecl]<nint>)found[next++];
            ResamplerFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            ResamplerReset = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            ResamplerRun = (delegate* unmanaged[Cdecl]<nint, short*, nuint, double, short*, nuint, long>)found[next++];
            ResamplerTake = (delegate* unmanaged[Cdecl]<nint, short*, nuint, long>)found[next++];
            RateNew = (delegate* unmanaged[Cdecl]<int, nint>)found[next++];
            RateFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            RateReset = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            RateSet = (delegate* unmanaged[Cdecl]<nint, uint, double, int>)found[next++];
            RateRead = (delegate* unmanaged[Cdecl]<nint, RateState*, int>)found[next++];
            RateCompute = (delegate* unmanaged[Cdecl]<nint, int, double*, int>)found[next++];
            RateProcess = (delegate* unmanaged[Cdecl]<nint, short*, nuint, int, short*, nuint, long>)found[next++];
            RateTake = (delegate* unmanaged[Cdecl]<nint, short*, nuint, long>)found[next++];
            RouterNew = (delegate* unmanaged[Cdecl]<nint>)found[next++];
            RouterFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            RouterGet = (delegate* unmanaged[Cdecl]<nint, uint, int>)found[next++];
            RouterSet = (delegate* unmanaged[Cdecl]<nint, uint, int, int>)found[next++];
            RouterReset = (delegate* unmanaged[Cdecl]<nint, int, uint*, nuint, int>)found[next++];
            RouterResize = (delegate* unmanaged[Cdecl]<nint, int, RouterChange*, nuint, long>)found[next++];
            RouterPoll = (delegate* unmanaged[Cdecl]<nint, ushort*, nuint, double*, nuint, uint, uint, RouterChange*, nuint, long>)found[next++];
            RouterSend = (delegate* unmanaged[Cdecl]<nint, uint, uint, RouterChange*, nuint, long>)found[next++];
            RouterTake = (delegate* unmanaged[Cdecl]<nint, RouterChange*, nuint, long>)found[next++];
            RouterPadHeld = (delegate* unmanaged[Cdecl]<nint, int, uint, int>)found[next++];
            RouterConnected = (delegate* unmanaged[Cdecl]<nint, int, uint, int>)found[next++];
            SlotsSeat = (delegate* unmanaged[Cdecl]<SeatIn*, byte*, nuint, byte*, nuint, int>)found[next++];
            SlotsMove = (delegate* unmanaged[Cdecl]<SeatIn*, uint, int, int, int*, int>)found[next++];
            SlotsForget = (delegate* unmanaged[Cdecl]<SeatIn*, int, int>)found[next++];
            SlotsHighest = (delegate* unmanaged[Cdecl]<SeatIn*, int>)found[next++];
            PadResolve = (delegate* unmanaged[Cdecl]<uint, double, uint, double*, int>)found[next++];
            PadCombine = (delegate* unmanaged[Cdecl]<double, uint, uint, double*, int>)found[next++];
            PadControlsFor = (delegate* unmanaged[Cdecl]<uint*, nuint, uint*, nuint, uint*, nuint, long>)found[next++];
            LastError = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            DeviceNative.Assign(found.AsSpan(next));
            return null;
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

        // Whether a string has UTF-8: UTF-16 with half a surrogate pair has none.
        internal static bool Crosses(string? text)
        {
            if (text is null) return true;
            for (int i = 0; i < text.Length; i++)
            {
                if (char.IsHighSurrogate(text[i]) && i + 1 < text.Length && char.IsLowSurrogate(text[i + 1])) i++;
                else if (char.IsSurrogate(text[i])) return false;
            }
            return true;
        }
    }

    // A handle the library made, freed once by whoever is done with it or by the collector.
    internal sealed unsafe class NativeHandle : IDisposable
    {
        private nint _handle;
        private readonly delegate* unmanaged[Cdecl]<nint, int> _free;

        public NativeHandle(nint handle, delegate* unmanaged[Cdecl]<nint, int> free)
        {
            if (handle == 0) throw new OutOfMemoryException("The platform library made no object.");
            _handle = handle;
            _free = free;
        }

        public nint Value => _handle;

        public void Dispose()
        {
            Free();
            GC.SuppressFinalize(this);
        }

        ~NativeHandle() => Free();

        private void Free()
        {
            nint handle = System.Threading.Interlocked.Exchange(ref _handle, 0);
            if (handle != 0) _free(handle);
        }
    }
}
