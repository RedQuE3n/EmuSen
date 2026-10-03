using System.Runtime.InteropServices;

namespace EmuSen.Cores.Native
{
    // The core ABI v1's exports and structs as emusen_core.h declares them, checked against abi/v1/baseline.txt by WiseMan - see EmuSen_CoreAPI.md §5.3.
    public sealed unsafe class CoreInterface
    {
        public const uint Major = 1;
        public const uint Minor = 0;
        public const uint Version = (Major << 16) | Minor;

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
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct Event
        {
            public uint Size;
            public uint Kind;
            public long A;
            public long B;
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
