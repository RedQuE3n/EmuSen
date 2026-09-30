using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace EmuSen.Cores.Native
{
    // The common native interface's fixed exports, resolved once per library; zero where the library lacks one - see EmuSen_NativeCores.md §3.
    public sealed unsafe class NativeInterface
    {
        public const ushort CommonVersion = 1;
        public const string VersionExport = "emusen_native_interface_version";
        public const string CapabilitiesExport = "emusen_native_capabilities";
        public const string CrashLogExport = "emusen_native_set_crash_log";

        public static uint Version(ushort coreVersion) => ((uint)CommonVersion << 16) | coreVersion;

        // The capability bits, as emusen-native's abi::caps numbers them - see EmuSen_NativeCores.md §3.2.
        public const ulong Reset = 1UL << 0;
        public const ulong Present = 1UL << 1;
        public const ulong Snapshot = 1UL << 2;
        public const ulong Axes = 1UL << 3;
        public const ulong AudioPeek = 1UL << 4;
        public const ulong Mutes = 1UL << 5;
        public const ulong Settings = 1UL << 6;
        public const ulong Phases = 1UL << 7;
        public const ulong FrameSerial = 1UL << 8;
        public const ulong RowRepeat = 1UL << 9;
        public const ulong BatteryDirty = 1UL << 10;
        public const ulong RomPatches = 1UL << 11;
        public const ulong Debug = 1UL << 12;
        public const ulong DebugStack = 1UL << 13;

        // Each bit's name and the optional exports it stands for; the library must have exactly the exports its bits claim.
        public static readonly IReadOnlyList<(ulong Bit, string Name, string[] Exports)> Optional = new (ulong, string, string[])[]
        {
            (Reset, "RESET", new[] { "emusen_native_reset" }),
            (Present, "PRESENT", new[] { "emusen_native_present" }),
            (Snapshot, "SNAPSHOT", Array.Empty<string>()),
            (Axes, "AXES", new[] { "emusen_native_set_axis" }),
            (AudioPeek, "AUDIO_PEEK", new[] { "emusen_native_audio_peek" }),
            (Mutes, "MUTES", new[] { "emusen_native_set_mutes" }),
            (Settings, "SETTINGS", new[] { "emusen_native_set_settings" }),
            (Phases, "PHASES", new[] { "emusen_native_phases" }),
            (FrameSerial, "FRAME_SERIAL", Array.Empty<string>()),
            (RowRepeat, "ROW_REPEAT", Array.Empty<string>()),
            (BatteryDirty, "BATTERY_DIRTY", Array.Empty<string>()),
            (RomPatches, "ROM_PATCHES", new[] { "emusen_native_set_rom_patches" }),
            (Debug, "DEBUG", new[] { "emusen_native_debug_set", "emusen_native_debug_set_breakpoints", "emusen_native_debug_set_ranges", "emusen_native_debug_run_frame", "emusen_native_debug_writes", "emusen_native_debug_calls", "emusen_native_debug_profile", "emusen_native_debug_coverage", "emusen_native_debug_counters", "emusen_native_debug_pc" }),
            (DebugStack, "DEBUG_STACK", new[] { "emusen_native_debug_set_stack" }),
        };

        // The exports every library on the interface has, whatever its capabilities.
        public static readonly string[] Required =
        {
            VersionExport, CapabilitiesExport, CrashLogExport,
            "emusen_native_create", "emusen_native_free",
            "emusen_native_advance", "emusen_native_set_options", "emusen_native_frame_count",
            "emusen_native_frame_info", "emusen_native_frame_copy",
            "emusen_native_audio_rate", "emusen_native_audio_buffered", "emusen_native_audio_drain", "emusen_native_set_audio_limit",
            "emusen_native_set_buttons",
            "emusen_native_state_size", "emusen_native_state_save", "emusen_native_state_load", "emusen_native_state_layout",
            "emusen_native_space_size", "emusen_native_space_read", "emusen_native_space_write",
            "emusen_native_battery", "emusen_native_battery_saved",
        };

        public static string Describe(ulong bits)
        {
            var names = new List<string>();
            foreach (var (bit, name, _) in Optional) if ((bits & bit) != 0) names.Add(name);
            return names.Count == 0 ? "none" : string.Join(", ", names);
        }

        // The interface's own status codes, -256 to -319, and the reproduced C# exceptions from -320 down - see EmuSen_NativeCores.md §3.3.
        public const int NotSupported = -256;
        public const int NoSuchSpace = -257;
        public const int ReadOnly = -258;
        public const int UnknownSetting = -259;
        public const int BadSetting = -260;
        public const int NoSuchPort = -261;
        public const int BadFile = -262;
        public const int FaultBase = -320;

        [StructLayout(LayoutKind.Sequential)]
        public struct NativeFile
        {
            public uint Which;
            public byte* Data;
            public nuint Length;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct FrameInfo
        {
            public int Width;
            public int Height;
            public int RowRepeat;
            public uint Flags;
            public long Serial;
            public long Bytes;
        }

        public readonly NativeCoreLibrary Library;
        public readonly delegate* unmanaged<byte*, nuint, byte*, nuint, NativeFile*, nuint, int*, nint> Create;
        public readonly delegate* unmanaged<nint, int> Free;
        public readonly delegate* unmanaged<nint, int> ResetOf;
        public readonly delegate* unmanaged<nint, ulong*, int> Advance;
        public readonly delegate* unmanaged<nint, uint, int> SetOptions;
        public readonly delegate* unmanaged<nint, long> FrameCount;
        public readonly delegate* unmanaged<nint, FrameInfo*, int> FrameInfoOf;
        public readonly delegate* unmanaged<nint, byte*, nuint, long> FrameCopy;
        public readonly delegate* unmanaged<nint, int> AudioRate;
        public readonly delegate* unmanaged<nint, long> AudioBuffered;
        public readonly delegate* unmanaged<nint, short*, nuint, long, long> AudioDrain;
        public readonly delegate* unmanaged<nint, ulong, int> SetAudioLimit;
        public readonly delegate* unmanaged<nint, uint, int> SetMutes;
        public readonly delegate* unmanaged<nint, uint, uint, uint, int> SetButtons;
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
        public readonly delegate* unmanaged<nint, uint, int, int, int> DebugSet;
        public readonly delegate* unmanaged<nint, uint*, nuint, int> DebugSetStack;
        public readonly delegate* unmanaged<nint, int*, nuint, int> DebugSetBreakpoints;
        public readonly delegate* unmanaged<nint, uint, uint*, nuint, int> DebugSetRanges;
        public readonly delegate* unmanaged<nint, uint, ulong*, ulong*, int> DebugRunFrame;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> DebugWrites;
        public readonly delegate* unmanaged<nint, uint*, nuint, long> DebugCalls;
        public readonly delegate* unmanaged<nint, long*, nuint, long> DebugProfile;
        public readonly delegate* unmanaged<nint, uint, byte*, nuint, long*, long> DebugCoverage;
        public readonly delegate* unmanaged<nint, long*, nuint, long> DebugCounters;
        public readonly delegate* unmanaged<nint, uint, ulong*, int> DebugPc;

        public NativeInterface(NativeCoreLibrary library)
        {
            Library = library;
            nint E(string name) => library.Export("emusen_native_" + name);
            Create = (delegate* unmanaged<byte*, nuint, byte*, nuint, NativeFile*, nuint, int*, nint>)E("create");
            Free = (delegate* unmanaged<nint, int>)E("free");
            ResetOf = (delegate* unmanaged<nint, int>)E("reset");
            Advance = (delegate* unmanaged<nint, ulong*, int>)E("advance");
            SetOptions = (delegate* unmanaged<nint, uint, int>)E("set_options");
            FrameCount = (delegate* unmanaged<nint, long>)E("frame_count");
            FrameInfoOf = (delegate* unmanaged<nint, FrameInfo*, int>)E("frame_info");
            FrameCopy = (delegate* unmanaged<nint, byte*, nuint, long>)E("frame_copy");
            AudioRate = (delegate* unmanaged<nint, int>)E("audio_rate");
            AudioBuffered = (delegate* unmanaged<nint, long>)E("audio_buffered");
            AudioDrain = (delegate* unmanaged<nint, short*, nuint, long, long>)E("audio_drain");
            SetAudioLimit = (delegate* unmanaged<nint, ulong, int>)E("set_audio_limit");
            SetMutes = (delegate* unmanaged<nint, uint, int>)E("set_mutes");
            SetButtons = (delegate* unmanaged<nint, uint, uint, uint, int>)E("set_buttons");
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
            DebugSet = (delegate* unmanaged<nint, uint, int, int, int>)E("debug_set");
            DebugSetStack = (delegate* unmanaged<nint, uint*, nuint, int>)E("debug_set_stack");
            DebugSetBreakpoints = (delegate* unmanaged<nint, int*, nuint, int>)E("debug_set_breakpoints");
            DebugSetRanges = (delegate* unmanaged<nint, uint, uint*, nuint, int>)E("debug_set_ranges");
            DebugRunFrame = (delegate* unmanaged<nint, uint, ulong*, ulong*, int>)E("debug_run_frame");
            DebugWrites = (delegate* unmanaged<nint, uint*, nuint, long>)E("debug_writes");
            DebugCalls = (delegate* unmanaged<nint, uint*, nuint, long>)E("debug_calls");
            DebugProfile = (delegate* unmanaged<nint, long*, nuint, long>)E("debug_profile");
            DebugCoverage = (delegate* unmanaged<nint, uint, byte*, nuint, long*, long>)E("debug_coverage");
            DebugCounters = (delegate* unmanaged<nint, long*, nuint, long>)E("debug_counters");
            DebugPc = (delegate* unmanaged<nint, uint, ulong*, int>)E("debug_pc");
        }

        // Every required export resolved; a library that loads but lacks one is not in use.
        public bool Complete => Create != null && Free != null && Advance != null && SetOptions != null && FrameCount != null && FrameInfoOf != null
            && FrameCopy != null && AudioRate != null && AudioBuffered != null && AudioDrain != null && SetAudioLimit != null && SetButtons != null
            && StateSize != null && StateSave != null && StateLoad != null && StateLayout != null && SpaceSize != null && SpaceRead != null
            && SpaceWrite != null && Battery != null && BatterySaved != null;

        public bool Has(ulong capability) => (Library.Capabilities & capability) == capability;
    }
}
