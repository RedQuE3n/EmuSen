using System;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Galaxia;
using SDL3;

namespace EmuSen.Endymion.Native
{
    // A pad opened or closed, as emusen_platform.h's emusen_endymion_pad_event.
    [StructLayout(LayoutKind.Sequential)]
    internal struct PadEvent
    {
        public uint Kind;
        public uint Announce;
        public ulong Key;
    }

    // The device half of Endymion in emusen_platform: SDL lent to it, simulated pads, the pads' bookkeeping and the two audio players - see EmuSen_RustPlatform.md §14.
    internal static unsafe class DeviceNative
    {
        internal const int Absent = -1030;
        internal const uint PadName = 0, PadGuid = 1, PadPath = 2, PadPlayerIndex = 0, PadKind = 1;
        internal const uint SetFirst = 0, SetOpenHandles = 1, SetOpens = 2, SetCloses = 3, SetInitialized = 4;
        internal const uint DeviceInit = 0, DeviceQuit = 1, DeviceOpen = 2, DeviceClose = 3, DeviceIsAttached = 4, DeviceChanged = 5, DeviceUpdate = 6,
            DeviceButton = 7, DeviceAxis = 8, DeviceKind = 9, DeviceLabel = 10, DeviceSetPlayerIndex = 11;
        internal const uint DeviceName = 0, DeviceGuid = 1, DevicePath = 2;
        internal const uint DevicesSdl = 0, DevicesSimulated = 1;
        internal const uint PadsId = 0, PadsIsOpen = 1, PadsKind = 2, PadsRawPressed = 3, PadsAxisValue = 4, PadsLabel = 5, PadsFrontendCount = 6, PadsStarted = 7,
            PadsAnyPressed = 8, PadsLastRescan = 9;
        internal const uint PadsGuid = 0, PadsPath = 1, PadsName = 2, AxisRaw = 0, AxisPad = 1;
        internal const uint SettingFirstOnly = 0, SettingStickAsDpad = 1, SettingStickDeadzone = 2, SettingLeftStickAnalog = 3, SettingAnalogDeadzone = 4;
        internal const uint AudioQueuedFrames = 0, AudioAvailable = 1, AudioSampleRate = 2, UiPlay = 0, UiPreload = 1, UiIsOpen = 0, UiQueued = 1;

        internal static delegate* unmanaged[Cdecl]<nint, int> SdlLend;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> SdlHint;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, nint> SimPadNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> SimPadFree;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> SimPadSetText;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> SimPadText;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int, int> SimPadSet;
        internal static delegate* unmanaged[Cdecl]<nint, uint, int*, int> SimPadGet;
        internal static delegate* unmanaged[Cdecl]<nint, int, uint, int> SimPadPress;
        internal static delegate* unmanaged[Cdecl]<nint, int, double, int> SimPadSetAxis;
        internal static delegate* unmanaged[Cdecl]<nint, int, int> SimPadHeld;
        internal static delegate* unmanaged[Cdecl]<nint, int, int> SimPadAxis;
        internal static delegate* unmanaged[Cdecl]<nint> SimSetNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> SimSetFree;
        internal static delegate* unmanaged[Cdecl]<nint, nint, long> SimSetConnect;
        internal static delegate* unmanaged[Cdecl]<nint, nint, int> SimSetDisconnect;
        internal static delegate* unmanaged[Cdecl]<nint, uint, long> SimSetGet;
        internal static delegate* unmanaged[Cdecl]<nint, uint*, nuint, long> SimSetAttached;
        internal static delegate* unmanaged[Cdecl]<nint, uint, ulong, int, long> SimSetCall;
        internal static delegate* unmanaged[Cdecl]<nint, uint, ulong, byte*, nuint, long> SimSetText;
        internal static delegate* unmanaged[Cdecl]<uint, nint, uint*, nuint, nint> PadsNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> PadsFree;
        internal static delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long> PadsStart;
        internal static delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long> PadsPoll;
        internal static delegate* unmanaged[Cdecl]<nint, uint*, nuint, int> PadsShowOnly;
        internal static delegate* unmanaged[Cdecl]<nint, int> PadsUpdate;
        internal static delegate* unmanaged[Cdecl]<nint, uint, nint, uint*, nuint, PadEvent*, nuint, long> PadsUse;
        internal static delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long> PadsTake;
        internal static delegate* unmanaged[Cdecl]<nint, int> PadsCloseAll;
        internal static delegate* unmanaged[Cdecl]<nint, ulong*, nuint, long> PadsOpen;
        internal static delegate* unmanaged[Cdecl]<nint, ulong, uint, int, long> PadsGet;
        internal static delegate* unmanaged[Cdecl]<nint, ulong, uint, byte*, nuint, long> PadsText;
        internal static delegate* unmanaged[Cdecl]<nint, ulong, uint, uint, double*, int> PadsAxis;
        internal static delegate* unmanaged[Cdecl]<nint, ulong, uint, int, int> PadsPressed;
        internal static delegate* unmanaged[Cdecl]<nint, ulong, int, int> PadsSetPlayerIndex;
        internal static delegate* unmanaged[Cdecl]<nint, uint, double, int> PadsSetSetting;
        internal static delegate* unmanaged[Cdecl]<nint, uint, double*, int> PadsSetting;
        internal static delegate* unmanaged[Cdecl]<long, long, int> PadsRescanDue;
        internal static delegate* unmanaged[Cdecl]<nint, int, int, int, nint> AudioNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> AudioFree;
        internal static delegate* unmanaged[Cdecl]<nint, int> AudioDispose;
        internal static delegate* unmanaged[Cdecl]<nint, nint, short*, nuint, int, int> AudioSubmit;
        internal static delegate* unmanaged[Cdecl]<nint, uint, long> AudioGet;
        internal static delegate* unmanaged[Cdecl]<nint, float, int> AudioSetVolume;
        internal static delegate* unmanaged[Cdecl]<nint, float> AudioVolume;
        internal static delegate* unmanaged[Cdecl]<nint> UiNew;
        internal static delegate* unmanaged[Cdecl]<nint, int> UiFree;
        internal static delegate* unmanaged[Cdecl]<nint, int> UiDispose;
        internal static delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> UiSound;
        internal static delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, int> UiRemember;
        internal static delegate* unmanaged[Cdecl]<nint, uint, long> UiGet;
        internal static delegate* unmanaged[Cdecl]<nint, float, int> UiSetVolume;
        internal static delegate* unmanaged[Cdecl]<nint, float> UiVolume;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> UiDecode;
        internal static delegate* unmanaged[Cdecl]<byte*, nuint, long> UiDecodeTake;

        // Every export this half calls, in the order Assign takes them.
        internal static readonly string[] Exports =
        {
            "emusen_endymion_sdl_lend", "emusen_endymion_sdl_hint", "emusen_endymion_sim_pad_new", "emusen_endymion_sim_pad_free", "emusen_endymion_sim_pad_set_text",
            "emusen_endymion_sim_pad_text", "emusen_endymion_sim_pad_set", "emusen_endymion_sim_pad_get", "emusen_endymion_sim_pad_press", "emusen_endymion_sim_pad_set_axis",
            "emusen_endymion_sim_pad_held", "emusen_endymion_sim_pad_axis", "emusen_endymion_sim_set_new", "emusen_endymion_sim_set_free", "emusen_endymion_sim_set_connect",
            "emusen_endymion_sim_set_disconnect", "emusen_endymion_sim_set_get", "emusen_endymion_sim_set_attached", "emusen_endymion_sim_set_call", "emusen_endymion_sim_set_text",
            "emusen_endymion_pads_new", "emusen_endymion_pads_free", "emusen_endymion_pads_start", "emusen_endymion_pads_poll", "emusen_endymion_pads_show_only", "emusen_endymion_pads_update", "emusen_endymion_pads_use",
            "emusen_endymion_pads_take", "emusen_endymion_pads_close_all", "emusen_endymion_pads_open", "emusen_endymion_pads_get", "emusen_endymion_pads_text", "emusen_endymion_pads_axis",
            "emusen_endymion_pads_pressed", "emusen_endymion_pads_set_player_index", "emusen_endymion_pads_set_setting", "emusen_endymion_pads_setting", "emusen_endymion_pads_rescan_due",
            "emusen_endymion_audio_new", "emusen_endymion_audio_free", "emusen_endymion_audio_dispose", "emusen_endymion_audio_submit", "emusen_endymion_audio_get",
            "emusen_endymion_audio_set_volume", "emusen_endymion_audio_volume", "emusen_endymion_ui_new", "emusen_endymion_ui_free", "emusen_endymion_ui_dispose", "emusen_endymion_ui_sound",
            "emusen_endymion_ui_remember", "emusen_endymion_ui_get", "emusen_endymion_ui_set_volume", "emusen_endymion_ui_volume", "emusen_endymion_ui_decode", "emusen_endymion_ui_decode_take",
        };

        // The resolved exports, in Exports' order.
        internal static void Assign(ReadOnlySpan<nint> found)
        {
            int next = 0;
            SdlLend = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            SdlHint = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            SimPadNew = (delegate* unmanaged[Cdecl]<byte*, nuint, nint>)found[next++];
            SimPadFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            SimPadSetText = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            SimPadText = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)found[next++];
            SimPadSet = (delegate* unmanaged[Cdecl]<nint, uint, int, int>)found[next++];
            SimPadGet = (delegate* unmanaged[Cdecl]<nint, uint, int*, int>)found[next++];
            SimPadPress = (delegate* unmanaged[Cdecl]<nint, int, uint, int>)found[next++];
            SimPadSetAxis = (delegate* unmanaged[Cdecl]<nint, int, double, int>)found[next++];
            SimPadHeld = (delegate* unmanaged[Cdecl]<nint, int, int>)found[next++];
            SimPadAxis = (delegate* unmanaged[Cdecl]<nint, int, int>)found[next++];
            SimSetNew = (delegate* unmanaged[Cdecl]<nint>)found[next++];
            SimSetFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            SimSetConnect = (delegate* unmanaged[Cdecl]<nint, nint, long>)found[next++];
            SimSetDisconnect = (delegate* unmanaged[Cdecl]<nint, nint, int>)found[next++];
            SimSetGet = (delegate* unmanaged[Cdecl]<nint, uint, long>)found[next++];
            SimSetAttached = (delegate* unmanaged[Cdecl]<nint, uint*, nuint, long>)found[next++];
            SimSetCall = (delegate* unmanaged[Cdecl]<nint, uint, ulong, int, long>)found[next++];
            SimSetText = (delegate* unmanaged[Cdecl]<nint, uint, ulong, byte*, nuint, long>)found[next++];
            PadsNew = (delegate* unmanaged[Cdecl]<uint, nint, uint*, nuint, nint>)found[next++];
            PadsFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            PadsStart = (delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long>)found[next++];
            PadsPoll = (delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long>)found[next++];
            PadsShowOnly = (delegate* unmanaged[Cdecl]<nint, uint*, nuint, int>)found[next++];
            PadsUpdate = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            PadsUse = (delegate* unmanaged[Cdecl]<nint, uint, nint, uint*, nuint, PadEvent*, nuint, long>)found[next++];
            PadsTake = (delegate* unmanaged[Cdecl]<nint, PadEvent*, nuint, long>)found[next++];
            PadsCloseAll = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            PadsOpen = (delegate* unmanaged[Cdecl]<nint, ulong*, nuint, long>)found[next++];
            PadsGet = (delegate* unmanaged[Cdecl]<nint, ulong, uint, int, long>)found[next++];
            PadsText = (delegate* unmanaged[Cdecl]<nint, ulong, uint, byte*, nuint, long>)found[next++];
            PadsAxis = (delegate* unmanaged[Cdecl]<nint, ulong, uint, uint, double*, int>)found[next++];
            PadsPressed = (delegate* unmanaged[Cdecl]<nint, ulong, uint, int, int>)found[next++];
            PadsSetPlayerIndex = (delegate* unmanaged[Cdecl]<nint, ulong, int, int>)found[next++];
            PadsSetSetting = (delegate* unmanaged[Cdecl]<nint, uint, double, int>)found[next++];
            PadsSetting = (delegate* unmanaged[Cdecl]<nint, uint, double*, int>)found[next++];
            PadsRescanDue = (delegate* unmanaged[Cdecl]<long, long, int>)found[next++];
            AudioNew = (delegate* unmanaged[Cdecl]<nint, int, int, int, nint>)found[next++];
            AudioFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            AudioDispose = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            AudioSubmit = (delegate* unmanaged[Cdecl]<nint, nint, short*, nuint, int, int>)found[next++];
            AudioGet = (delegate* unmanaged[Cdecl]<nint, uint, long>)found[next++];
            AudioSetVolume = (delegate* unmanaged[Cdecl]<nint, float, int>)found[next++];
            AudioVolume = (delegate* unmanaged[Cdecl]<nint, float>)found[next++];
            UiNew = (delegate* unmanaged[Cdecl]<nint>)found[next++];
            UiFree = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            UiDispose = (delegate* unmanaged[Cdecl]<nint, int>)found[next++];
            UiSound = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)found[next++];
            UiRemember = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, int>)found[next++];
            UiGet = (delegate* unmanaged[Cdecl]<nint, uint, long>)found[next++];
            UiSetVolume = (delegate* unmanaged[Cdecl]<nint, float, int>)found[next++];
            UiVolume = (delegate* unmanaged[Cdecl]<nint, float>)found[next++];
            UiDecode = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            UiDecodeTake = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
        }

        private static readonly Lazy<string?> Lent = new(Lend);
        private static readonly Lazy<bool> Chosen = new(Choose);

        // The library has the SDL this process loaded: what a parity test needs, whichever way the switch stands.
        public static bool Ready => Lent.Value is null;

        public static string Report => Lent.Value ?? EndymionNative.Report;

        // State 1 for the device half: the switch, and SDL lent; without either the C# runs, and says why when the switch asked.
        public static bool Active => Chosen.Value;

        private static bool Choose()
        {
            if (!EndymionNative.Active) return false;
            if (Ready) return true;
            ConfigDiagnostics.Report($"{EndymionNative.Variable}=1 was asked for and {Lent.Value}; Endymion's devices run on their C# implementation.");
            return false;
        }

        // The handle is the one SDL3-CS's own calls resolve to, so the library and the C# share one SDL - see EmuSen_RustPlatform.md §14.2.
        private static string? Lend()
        {
            if (!EndymionNative.Ready) return EndymionNative.Report;
            if (!NativeLibrary.TryLoad("SDL3", typeof(SDL).Assembly, null, out nint sdl)) return "SDL3 could not be loaded";
            return SdlLend(sdl) == 0 ? null : EndymionNative.Words();
        }

        internal delegate long TextCall(byte* buffer, nuint capacity);

        // A text the library answers by the length-query idiom; null for ABSENT.
        internal static string? Text(TextCall call)
        {
            byte* small = stackalloc byte[256];
            long length = call(small, 256);
            if (length == Absent) return null;
            if (length < 0) throw new InvalidOperationException(EndymionNative.Words());
            if (length <= 256) return Encoding.UTF8.GetString(small, (int)length);
            byte[] large = new byte[length];
            fixed (byte* p = large) call(p, (nuint)large.Length);
            return Encoding.UTF8.GetString(large);
        }

        // UTF-8 as SDL3-CS's own marshaller makes it, half a surrogate pair replaced; null stays null.
        internal static byte[]? Utf8(string? text) => text is null ? null : Encoding.UTF8.GetBytes(text);

        // A hint as the library's SDL has it.
        internal static string? Hint(string name)
        {
            byte[] bytes = Utf8(name)!;
            return Text((buffer, capacity) =>
            {
                fixed (byte* n = bytes) return SdlHint(Pin(n), (nuint)bytes.Length, buffer, capacity);
            });
        }

        // A pinned array's pointer, never null for an empty one, which fixed makes null.
        internal static byte* Pin(byte* pinned) => pinned is null ? (byte*)1 : pinned;
    }
}
