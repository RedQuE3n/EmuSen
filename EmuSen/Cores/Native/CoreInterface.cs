using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace EmuSen.Cores.Native
{
    // The core ABI v1's exports and structs as emusen_core.h declares them, checked against abi/v1/baseline.txt by WiseMan - see EmuSen_CoreAPI.md §5.3.
    public sealed unsafe class CoreInterface
    {
        public const uint Major = 1;
        public const uint Minor = 13;
        public const uint Version = (Major << 16) | Minor;

        // The capability bits, numbered as emusen_core.h's EMUSEN_CAP_*.
        public const ulong CapReset = 1UL << 0, CapPresent = 1UL << 1, CapSnapshot = 1UL << 2, CapAxes = 1UL << 3, CapAudioPeek = 1UL << 4, CapMutes = 1UL << 5;
        public const ulong CapSettings = 1UL << 6, CapPhases = 1UL << 7, CapFrameSerial = 1UL << 8, CapRowRepeat = 1UL << 9, CapBatteryDirty = 1UL << 10;
        public const ulong CapRomPatches = 1UL << 11, CapDebug = 1UL << 12, CapDebugStack = 1UL << 13, CapCheatPokes = 1UL << 14, CapSettingNotes = 1UL << 15;
        public const ulong CapDebugRegisters = 1UL << 16, CapDebugDisassemble = 1UL << 17, CapMedia = 1UL << 18, CapDebugReads = 1UL << 19;
        public const ulong CapBorrow = 1UL << 20, CapDevicePicture = 1UL << 21, CapRawPicture = 1UL << 22;
        public const ulong CapTelemetry = 1UL << 23, CapPresence = 1UL << 24, CapDebugTranslate = 1UL << 25;

        // Each bit's name, as core info lists it.
        public static readonly IReadOnlyList<(ulong Bit, string Name)> CapabilityNames = new[]
        {
            (CapReset, "RESET"), (CapPresent, "PRESENT"), (CapSnapshot, "SNAPSHOT"), (CapAxes, "AXES"), (CapAudioPeek, "AUDIO_PEEK"), (CapMutes, "MUTES"),
            (CapSettings, "SETTINGS"), (CapPhases, "PHASES"), (CapFrameSerial, "FRAME_SERIAL"), (CapRowRepeat, "ROW_REPEAT"), (CapBatteryDirty, "BATTERY_DIRTY"),
            (CapRomPatches, "ROM_PATCHES"), (CapDebug, "DEBUG"), (CapDebugStack, "DEBUG_STACK"), (CapCheatPokes, "CHEAT_POKES"), (CapSettingNotes, "SETTING_NOTES"),
            (CapDebugRegisters, "DEBUG_REGISTERS"), (CapDebugDisassemble, "DEBUG_DISASSEMBLE"), (CapMedia, "MEDIA"), (CapDebugReads, "DEBUG_READS"),
            (CapBorrow, "BORROW"), (CapDevicePicture, "DEVICE_PICTURE"), (CapRawPicture, "RAW_PICTURE"),
            (CapTelemetry, "TELEMETRY"), (CapPresence, "PRESENCE"), (CapDebugTranslate, "DEBUG_TRANSLATE"),
        };

        // Every export, and the bit that claims it; zero for a required one.
        public static readonly IReadOnlyList<(string Name, ulong Bit)> Exports = new (string, ulong)[]
        {
            ("emusen_core_abi_version", 0), ("emusen_core_capabilities", 0), ("emusen_core_info", 0), ("emusen_core_settings_schema", 0),
            ("emusen_core_firmware_for", 0), ("emusen_core_status_text", 0), ("emusen_core_set_crash_log", 0), ("emusen_core_log_drain", 0),
            ("emusen_core_create", 0), ("emusen_core_free", 0), ("emusen_core_reset", CapReset), ("emusen_core_machine_info", 0), ("emusen_core_last_error", 0),
            ("emusen_core_advance", 0), ("emusen_core_present", CapPresent), ("emusen_core_set_options", 0), ("emusen_core_frame_count", 0),
            ("emusen_core_phases", CapPhases), ("emusen_core_events", 0), ("emusen_core_frame_info", 0), ("emusen_core_frame_copy", 0),
            ("emusen_core_audio_rate", 0), ("emusen_core_audio_buffered", 0), ("emusen_core_audio_drain", 0), ("emusen_core_set_audio_limit", 0),
            ("emusen_core_audio_peek", CapAudioPeek), ("emusen_core_set_mutes", CapMutes), ("emusen_core_set_buttons", 0), ("emusen_core_set_axis", CapAxes),
            ("emusen_core_state_size", 0), ("emusen_core_state_save", 0), ("emusen_core_state_load", 0), ("emusen_core_state_layout", 0),
            ("emusen_core_space_size", 0), ("emusen_core_space_read", 0), ("emusen_core_space_write", 0), ("emusen_core_battery", 0), ("emusen_core_battery_saved", 0),
            ("emusen_core_set_rom_patches", CapRomPatches), ("emusen_core_set_cheat_pokes", CapCheatPokes), ("emusen_core_set_settings", CapSettings),
            ("emusen_core_setting_notes", CapSettingNotes), ("emusen_core_debug_set", CapDebug), ("emusen_core_debug_set_stack", CapDebugStack),
            ("emusen_core_debug_set_breakpoints", CapDebug), ("emusen_core_debug_set_ranges", CapDebug), ("emusen_core_debug_run_frame", CapDebug),
            ("emusen_core_debug_writes", CapDebug), ("emusen_core_debug_calls", CapDebug), ("emusen_core_debug_profile", CapDebug),
            ("emusen_core_debug_coverage", CapDebug), ("emusen_core_debug_counters", CapDebug), ("emusen_core_debug_pc", CapDebug),
            ("emusen_core_debug_registers", CapDebugRegisters), ("emusen_core_debug_disassemble", CapDebugDisassemble),
            ("emusen_core_media_lid", CapMedia), ("emusen_core_media_insert", CapMedia),
            ("emusen_core_debug_reads", CapDebugReads), ("emusen_core_frame_device", CapDevicePicture),
            ("emusen_core_raw_select", CapRawPicture), ("emusen_core_raw_info", CapRawPicture), ("emusen_core_raw_copy", CapRawPicture),
            ("emusen_core_telemetry", CapTelemetry), ("emusen_core_set_present", CapPresence), ("emusen_core_debug_translate", CapDebugTranslate),
        };

        // The host obligations this host meets, for a core's host_requires - see EmuSen_CoreAPI.md §4.6.
        public static readonly IReadOnlyList<string> HostObligations = new[] { "present" };

        // The status codes a host acts on by number.
        public const int StatusNull = -1, StatusBufferTooSmall = -7, StatusNotSupported = -256, StatusUnknownSetting = -259, StatusBadSetting = -260, StatusBadStruct = -263;
        public const int StatusLidClosed = -265, StatusWrongMedia = -266, StatusNoSuchDrive = -267;

        // EMUSEN_FILE_MEDIA, the first companion of an image set, and EMUSEN_MEDIA_RESTORE - see EmuSen_CoreAPI.md §6.23.
        public const uint FileMedia = 65536, MediaRestore = 1;

        // EMUSEN_CREATE_LENT and EMUSEN_MEDIA_LENT: the image and its media files kept in place by the host until the loan ends - see EmuSen_CoreAPI.md §6.24.
        public const uint CreateLent = 1, MediaLent = 2;

        // EMUSEN_CREATE_SHARED_REMOVABLE: the host keeps the removable battery files apart from the game, so a state load leaves them - see EmuSen_CoreAPI.md §6.27.
        public const uint CreateSharedRemovable = 2;

        // EMUSEN_CREATE_SHARED_CONSOLE: the same for the console's own battery files - see EmuSen_CoreAPI.md §6.28.
        public const uint CreateSharedConsole = 4;

        // EMUSEN_FRAME_CHEATS_HELD, since 1.13: frame_info's flag for a frame after which no cheat is applied - see EmuSen_CoreAPI.md §6.37.
        public const uint FrameCheatsHeld = 2;

        // The event kinds of enum emusen_event_kind; 7 is kept for a controller's outputs.
        public const uint EventAudioRate = 1, EventGeometry = 2, EventStateSize = 3, EventBattery = 4, EventLog = 5, EventMachineInfo = 6;

        // EMUSEN_EVENT_MEDIA_SEEK and EMUSEN_FILE_IMAGE: a drive names the file and byte it reads next, a = (drive << 32) | file - see EmuSen_CoreAPI.md §6.25.
        public const uint EventMediaSeek = 8, FileImage = 0xFFFFFFFF;

        [StructLayout(LayoutKind.Sequential)]
        public struct File
        {
            public uint Size;
            public uint Which;
            public byte* Data;
            public nuint Len;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct CreateParams
        {
            public uint Size;
            public uint HostAbiVersion;
            public byte* Image;
            public nuint ImageLen;
            public byte* Settings;
            public nuint SettingsLen;
            public File* Files;
            public nuint FileCount;
            public nuint FileSize;
            public ulong PixelFormats;
            public byte* Error;
            public nuint ErrorLen;
            public uint Flags;
            public DeviceLend* Device;
            public byte* System;
        }

        // emusen_device_lend, since 1.9: the host's Vulkan device lent to a core, with a queue the core alone submits to - see EmuSen_CoreAPI.md §6.29.
        [StructLayout(LayoutKind.Sequential)]
        public struct DeviceLend
        {
            public uint Size;
            public uint ApiVersion;
            public nint GetInstanceProcAddr;
            public ulong Instance;
            public ulong PhysicalDevice;
            public ulong Device;
            public uint QueueFamily;
            public uint QueueIndex;
            public uint HostQueueFamily;
            public uint Reserved;
            public ulong Done;
        }

        // emusen_device_frame, since 1.9: the frame's picture on the lent device, written once Ready reaches Value.
        [StructLayout(LayoutKind.Sequential)]
        public struct DeviceFrame
        {
            public uint Size;
            public uint Format;
            public ulong Buffer;
            public ulong Offset;
            public int Width;
            public int Height;
            public int Stride;
            public uint Reserved;
            public ulong Ready;
            public ulong Value;
        }

        // emusen_raw_frame, since 1.10: the shape of a frame's picture before the core's palette - see EmuSen_CoreAPI.md §6.30.
        [StructLayout(LayoutKind.Sequential)]
        public struct RawFrame
        {
            public uint Size;
            public uint Element;
            public int Width;
            public int Height;
            public int Stride;
            public uint Reserved;
            public long Bytes;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct FrameInfo
        {
            public uint Size;
            public uint Format;
            public int Width;
            public int Height;
            public int Stride;
            public int RowRepeat;
            public uint Flags;
            public uint AspectNum;
            public uint AspectDen;
            public uint Reserved;
            public long Serial;
            public long Bytes;
            public int Lines;
            public uint Reserved2;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct Event
        {
            public uint Size;
            public uint Kind;
            public long A;
            public long B;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct Media
        {
            public uint Size;
            public uint Flags;
            public byte* Image;
            public nuint ImageLen;
            public File* Files;
            public nuint FileCount;
            public nuint FileSize;
        }

        public readonly delegate* unmanaged<uint> AbiVersion;
        public readonly delegate* unmanaged<ulong> Capabilities;
        public readonly delegate* unmanaged<byte*, nuint, long> Info;
        public readonly delegate* unmanaged<byte*, nuint, long> SettingsSchema;
        public readonly delegate* unmanaged<byte*, nuint, byte*, nuint, long> FirmwareFor;
        public readonly delegate* unmanaged<int, byte*, nuint, long> StatusText;
        public readonly delegate* unmanaged<byte*, int> SetCrashLog;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> LogDrain;
        public readonly delegate* unmanaged<CreateParams*, int*, nint> Create;
        public readonly delegate* unmanaged<nint, int> Free;
        public readonly delegate* unmanaged<nint, int> Reset;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> MachineInfo;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> LastError;
        public readonly delegate* unmanaged<nint, ulong*, int> Advance;
        public readonly delegate* unmanaged<nint, int> Present;
        public readonly delegate* unmanaged<nint, uint, int> SetOptions;
        public readonly delegate* unmanaged<nint, long> FrameCount;
        public readonly delegate* unmanaged<nint, long*, nuint, long> Phases;
        public readonly delegate* unmanaged<nint, Event*, nuint, nuint, long> Events;
        public readonly delegate* unmanaged<nint, FrameInfo*, int> FrameInfoOf;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> FrameCopy;
        public readonly delegate* unmanaged<nint, int> AudioRate;
        public readonly delegate* unmanaged<nint, long> AudioBuffered;
        public readonly delegate* unmanaged<nint, short*, nuint, long, int*, long> AudioDrain;
        public readonly delegate* unmanaged<nint, ulong, int> SetAudioLimit;
        public readonly delegate* unmanaged<nint, short*, nuint, long> AudioPeek;
        public readonly delegate* unmanaged<nint, uint, int> SetMutes;
        public readonly delegate* unmanaged<nint, uint, uint, uint, int> SetButtons;
        public readonly delegate* unmanaged<nint, uint, uint, double, int> SetAxis;
        public readonly delegate* unmanaged<nint, uint, long> StateSize;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, long> StateSave;
        public readonly delegate* unmanaged<nint, byte*, nuint, int> StateLoad;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, long> StateLayout;
        public readonly delegate* unmanaged<nint, uint, long> SpaceSize;
        public readonly delegate* unmanaged<nint, uint, uint, byte*, nuint, long> SpaceRead;
        public readonly delegate* unmanaged<nint, uint, uint, byte*, nuint, long> SpaceWrite;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, uint*, long> Battery;
        public readonly delegate* unmanaged<nint, uint, int> BatterySaved;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> SetRomPatches;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> SetCheatPokes;
        public readonly delegate* unmanaged<nint, byte*, nuint, int> SetSettings;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> SettingNotes;
        public readonly delegate* unmanaged<nint, uint, int, int, int> DebugSet;
        public readonly delegate* unmanaged<nint, uint*, nuint, int> DebugSetStack;
        public readonly delegate* unmanaged<nint, uint, int*, nuint, int> DebugSetBreakpoints;
        public readonly delegate* unmanaged<nint, uint, uint*, nuint, int> DebugSetRanges;
        public readonly delegate* unmanaged<nint, uint, uint*, ulong*, ulong*, int> DebugRunFrame;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> DebugWrites;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> DebugCalls;
        public readonly delegate* unmanaged<nint, long*, nuint, long> DebugProfile;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, long*, long> DebugCoverage;
        public readonly delegate* unmanaged<nint, long*, nuint, long> DebugCounters;
        public readonly delegate* unmanaged<nint, uint, ulong*, int> DebugPc;
        public readonly delegate* unmanaged<nint, uint, long*, nuint, long> DebugRegisters;
        public readonly delegate* unmanaged<nint, uint, uint, uint, uint, byte*, nuint, long> DebugDisassemble;
        public readonly delegate* unmanaged<nint, uint, uint, int> MediaLid;
        public readonly delegate* unmanaged<nint, uint, Media*, int> MediaInsert;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> DebugReads;
        public readonly delegate* unmanaged<nint, DeviceFrame*, int> FrameDevice;
        public readonly delegate* unmanaged<nint, ulong, int> RawSelect;
        public readonly delegate* unmanaged<nint, uint, RawFrame*, int> RawInfo;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, long> RawCopy;
        public readonly delegate* unmanaged<nint, long*, nuint, long> Telemetry;
        public readonly delegate* unmanaged<nint, uint, uint, int> SetPresent;
        public readonly delegate* unmanaged<nint, uint, uint, uint*, int> DebugTranslate;

        // Each export resolved from a loaded library by name; zero where the library lacks it.
        public CoreInterface(nint library)
        {
            nint E(string name) => NativeLibrary.TryGetExport(library, "emusen_core_" + name, out nint p) ? p : 0;
            AbiVersion = (delegate* unmanaged<uint>)E("abi_version");
            Capabilities = (delegate* unmanaged<ulong>)E("capabilities");
            Info = (delegate* unmanaged<byte*, nuint, long>)E("info");
            SettingsSchema = (delegate* unmanaged<byte*, nuint, long>)E("settings_schema");
            FirmwareFor = (delegate* unmanaged<byte*, nuint, byte*, nuint, long>)E("firmware_for");
            StatusText = (delegate* unmanaged<int, byte*, nuint, long>)E("status_text");
            SetCrashLog = (delegate* unmanaged<byte*, int>)E("set_crash_log");
            LogDrain = (delegate* unmanaged<nint, byte*, nuint, long>)E("log_drain");
            Create = (delegate* unmanaged<CreateParams*, int*, nint>)E("create");
            Free = (delegate* unmanaged<nint, int>)E("free");
            Reset = (delegate* unmanaged<nint, int>)E("reset");
            MachineInfo = (delegate* unmanaged<nint, byte*, nuint, long>)E("machine_info");
            LastError = (delegate* unmanaged<nint, byte*, nuint, long>)E("last_error");
            Advance = (delegate* unmanaged<nint, ulong*, int>)E("advance");
            Present = (delegate* unmanaged<nint, int>)E("present");
            SetOptions = (delegate* unmanaged<nint, uint, int>)E("set_options");
            FrameCount = (delegate* unmanaged<nint, long>)E("frame_count");
            Phases = (delegate* unmanaged<nint, long*, nuint, long>)E("phases");
            Events = (delegate* unmanaged<nint, Event*, nuint, nuint, long>)E("events");
            FrameInfoOf = (delegate* unmanaged<nint, FrameInfo*, int>)E("frame_info");
            FrameCopy = (delegate* unmanaged<nint, byte*, nuint, long>)E("frame_copy");
            AudioRate = (delegate* unmanaged<nint, int>)E("audio_rate");
            AudioBuffered = (delegate* unmanaged<nint, long>)E("audio_buffered");
            AudioDrain = (delegate* unmanaged<nint, short*, nuint, long, int*, long>)E("audio_drain");
            SetAudioLimit = (delegate* unmanaged<nint, ulong, int>)E("set_audio_limit");
            AudioPeek = (delegate* unmanaged<nint, short*, nuint, long>)E("audio_peek");
            SetMutes = (delegate* unmanaged<nint, uint, int>)E("set_mutes");
            SetButtons = (delegate* unmanaged<nint, uint, uint, uint, int>)E("set_buttons");
            SetAxis = (delegate* unmanaged<nint, uint, uint, double, int>)E("set_axis");
            StateSize = (delegate* unmanaged<nint, uint, long>)E("state_size");
            StateSave = (delegate* unmanaged<nint, uint, byte*, nuint, long>)E("state_save");
            StateLoad = (delegate* unmanaged<nint, byte*, nuint, int>)E("state_load");
            StateLayout = (delegate* unmanaged<nint, uint, byte*, nuint, long>)E("state_layout");
            SpaceSize = (delegate* unmanaged<nint, uint, long>)E("space_size");
            SpaceRead = (delegate* unmanaged<nint, uint, uint, byte*, nuint, long>)E("space_read");
            SpaceWrite = (delegate* unmanaged<nint, uint, uint, byte*, nuint, long>)E("space_write");
            Battery = (delegate* unmanaged<nint, uint, byte*, nuint, uint*, long>)E("battery");
            BatterySaved = (delegate* unmanaged<nint, uint, int>)E("battery_saved");
            SetRomPatches = (delegate* unmanaged<nint, uint*, nuint, long>)E("set_rom_patches");
            SetCheatPokes = (delegate* unmanaged<nint, uint*, nuint, long>)E("set_cheat_pokes");
            SetSettings = (delegate* unmanaged<nint, byte*, nuint, int>)E("set_settings");
            SettingNotes = (delegate* unmanaged<nint, byte*, nuint, long>)E("setting_notes");
            DebugSet = (delegate* unmanaged<nint, uint, int, int, int>)E("debug_set");
            DebugSetStack = (delegate* unmanaged<nint, uint*, nuint, int>)E("debug_set_stack");
            DebugSetBreakpoints = (delegate* unmanaged<nint, uint, int*, nuint, int>)E("debug_set_breakpoints");
            DebugSetRanges = (delegate* unmanaged<nint, uint, uint*, nuint, int>)E("debug_set_ranges");
            DebugRunFrame = (delegate* unmanaged<nint, uint, uint*, ulong*, ulong*, int>)E("debug_run_frame");
            DebugWrites = (delegate* unmanaged<nint, uint*, nuint, long>)E("debug_writes");
            DebugCalls = (delegate* unmanaged<nint, uint*, nuint, long>)E("debug_calls");
            DebugProfile = (delegate* unmanaged<nint, long*, nuint, long>)E("debug_profile");
            DebugCoverage = (delegate* unmanaged<nint, uint, byte*, nuint, long*, long>)E("debug_coverage");
            DebugCounters = (delegate* unmanaged<nint, long*, nuint, long>)E("debug_counters");
            DebugPc = (delegate* unmanaged<nint, uint, ulong*, int>)E("debug_pc");
            DebugRegisters = (delegate* unmanaged<nint, uint, long*, nuint, long>)E("debug_registers");
            DebugDisassemble = (delegate* unmanaged<nint, uint, uint, uint, uint, byte*, nuint, long>)E("debug_disassemble");
            MediaLid = (delegate* unmanaged<nint, uint, uint, int>)E("media_lid");
            MediaInsert = (delegate* unmanaged<nint, uint, Media*, int>)E("media_insert");
            DebugReads = (delegate* unmanaged<nint, uint*, nuint, long>)E("debug_reads");
            FrameDevice = (delegate* unmanaged<nint, DeviceFrame*, int>)E("frame_device");
            RawSelect = (delegate* unmanaged<nint, ulong, int>)E("raw_select");
            RawInfo = (delegate* unmanaged<nint, uint, RawFrame*, int>)E("raw_info");
            RawCopy = (delegate* unmanaged<nint, uint, byte*, nuint, long>)E("raw_copy");
            Telemetry = (delegate* unmanaged<nint, long*, nuint, long>)E("telemetry");
            SetPresent = (delegate* unmanaged<nint, uint, uint, int>)E("set_present");
            DebugTranslate = (delegate* unmanaged<nint, uint, uint, uint*, int>)E("debug_translate");
        }

        // The C# field that holds an export: its suffix in PascalCase, and FrameInfoOf where that names the struct.
        public static string FieldFor(string export)
        {
            string suffix = export.StartsWith("emusen_core_") ? export["emusen_core_".Length..] : export;
            if (suffix == "frame_info") return "FrameInfoOf";
            var parts = suffix.Split('_');
            return string.Concat(System.Linq.Enumerable.Select(parts, p => p.Length == 0 ? p : char.ToUpperInvariant(p[0]) + p[1..]));
        }
    }
}
