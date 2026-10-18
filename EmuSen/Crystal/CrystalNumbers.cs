using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Managed
{
    // emusen_crystal.h's own numbers as the implementation answers them; CrystalHeaderTests holds them to the header.
    internal static class CrystalNumbers
    {
        public const uint Major = 1u;
        public const uint Minor = 1u;
        public const uint Version = (Major << 16) | Minor;
        public const int NoEngine = -1109;
        public const int Firmware = -1110;
        public const int Busy = -1111;
        public const int BadStruct = -1112;
        public const int Io = -1113;
        public const int StateRefused = -1114;
        public const uint OpenNoBattery = 1u;
        public const uint OpenSingleRows = 2u;
        public const uint SpeedNormal = 0u;
        public const uint SpeedTurbo = 1u;
        public const uint SpeedSlow = 2u;
        public const uint SpeedDrawNone = 1u;
        public const uint ScopeConsole = 0u;
        public const uint ScopeGame = 2u;
        public const uint ServiceReadOnly = 1u;
        public const uint EventSession = 1u;
        public const uint EventRun = 2u;
        public const uint EventState = 4u;
        public const uint EventRewind = 8u;
        public const uint EventSettings = 16u;
        public const uint EventDisc = 32u;
        public const uint EventLog = 64u;
        public const uint CardsList = 0u;
        public const uint CardsOpen = 1u;
        public const uint CardsCopy = 2u;
        public const uint CardsMove = 3u;
        public const uint CardsDelete = 4u;
        public const uint CardsImport = 5u;
        public const uint CardsNew = 6u;
        public const uint CardsDuplicate = 7u;
        public const uint CardsRename = 8u;
    }

    // emusen_crystal_config as the implementation reads it.
    [StructLayout(LayoutKind.Sequential)]
    internal struct ConfigV1
    {
        public uint size;
        public uint flags;
    }

    // emusen_crystal_open_params as the implementation reads it.
    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct OpenParamsV1
    {
        // 1.0's size, the shortest taken; device is read only from a struct that holds it.
        public const int Size10 = 64;

        public uint size;
        public uint flags;
        public byte* path;
        public nuint path_len;
        public byte* console;
        public nuint console_len;
        public byte* engine;
        public nuint engine_len;
        public uint audio_buffer_max_samples;
        public uint reserved;
        public void* device;
    }

    // emusen_crystal_frame_info as the implementation fills it.
    [StructLayout(LayoutKind.Sequential)]
    internal struct FrameInfoV1
    {
        // 1.0's size, the shortest taken; picture_on_device is written only into a struct that holds it.
        public const int Size10 = 104;

        public uint size;
        public uint ran;
        public uint halted;
        public uint rendered;
        public ulong picture_serial;
        public int width;
        public int height;
        public int row_repeat;
        public int display_lines;
        public double display_aspect;
        public uint audio_frames;
        public uint audio_rate;
        public uint play_audio;
        public uint speed_percent;
        public ulong frame_interval_ns;
        public ulong total_frames;
        public double frame_rate_hz;
        public uint paused;
        public uint halted_processor;
        public ulong halted_address;
        public uint picture_on_device;
        public uint reserved;
    }

    // emusen_crystal_raw_info, field for field, since 1.1.
    [StructLayout(LayoutKind.Sequential)]
    internal struct RawInfoV1
    {
        public uint size;
        public uint element;
        public int width;
        public int height;
        public int stride;
        public uint reserved;
    }
}
