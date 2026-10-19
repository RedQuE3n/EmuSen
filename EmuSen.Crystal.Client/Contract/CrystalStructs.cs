using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Client.Contract
{
    // emusen_crystal_config, field for field; CrystalHeaderTests holds it to the header.
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct CrystalConfig
    {
        public uint size;
        public uint flags;
        public void* sqlite;
        public byte* cores;
        public nuint cores_len;
        public byte* player_cores;
        public nuint player_cores_len;
    }

    // emusen_crystal_open_params, field for field.
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct CrystalOpenParams
    {
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

    // emusen_crystal_frame_info, field for field.
    [StructLayout(LayoutKind.Sequential)]
    public struct CrystalFrameInfo
    {
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
    public struct CrystalRawInfo
    {
        public uint size;
        public uint element;
        public int width;
        public int height;
        public int stride;
        public uint reserved;
    }
}
