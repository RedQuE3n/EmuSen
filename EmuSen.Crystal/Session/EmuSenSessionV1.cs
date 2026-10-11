using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Session
{
    // The managed mirror of emusen_session.h's emusen_session_v1, field for field and in its order; SessionHeaderTests holds the two equal.
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct EmuSenSessionV1
    {
        public uint size;
        public uint version;
        public void* ctx;
        public delegate* unmanaged[Cdecl]<void*, byte*, nuint, long> last_error;
        public delegate* unmanaged[Cdecl]<void*, byte*, nuint, long> sessions;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> info;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> summary;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, long, long> run_control;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> halt_state;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> spaces;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, byte*, nuint, long> space_read;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, byte*, nuint, int> space_write;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> resolve_physical;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> processors;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> registers;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, ulong, int> set_register;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, uint, uint, byte*, nuint, byte*, nuint, long> disassemble;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> bp_add;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> bp_add_data;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, uint, int> bp_set_enabled;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, int> bp_remove;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> bp_list;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, long> bp_log;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> bp_log_clear;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, uint, int> bp_arm_condition;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> bp_conditions;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int, int> bp_depth_guard;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, uint, long> bp_forbid_range;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, uint, int> bp_uninitialized;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, uint, uint, long> watch_add;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> watch_remove;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> watch_list;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, long> watch_events;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> watch_summary;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> watch_clear;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, uint, long> framelog_add;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> framelog_remove;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> framelog_list;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, long> framelog_samples;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> framelog_clear;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, byte*, nuint, long> coverage_op;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, byte*, nuint, long> profile_op;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> counters_op;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, byte*, nuint, long> call_stack;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, uint, long> freeze_add;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> freeze_remove;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> freeze_list;
        public delegate* unmanaged[Cdecl]<void*, uint, int> freeze_clear;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, int> label_add;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, int> label_remove;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> label_list;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> label_at;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, byte*, nuint, long> label_load;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> label_save;
        public delegate* unmanaged[Cdecl]<void*, uint, int> label_clear;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> dma_channels;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> dma_log_op;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> register_flow_op;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> coprocessor_history;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> vectors;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, byte*, nuint, long> tilemap_entry;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, uint, uint, byte*, nuint, long> tile_pixels;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> sprites;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> palettes;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> audio_channels;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, int> set_channel_muted;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> evaluate;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> symbols;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, byte*, nuint, long> cheat_op;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> cheat_codecs;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, uint, int> trace;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, int> trace_off;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> telemetry;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> tile_sheet;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> palette_swatch;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> audio_peek;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, int> state_save;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, int> state_load;
        public delegate* unmanaged[Cdecl]<void*, uint, byte*, nuint, long> state_default_path;
        public delegate* unmanaged[Cdecl]<void*, uint, int> log_switch;
        public delegate* unmanaged[Cdecl]<void*, int> log_status;
        public delegate* unmanaged[Cdecl]<void*, uint, uint, byte*, nuint, long> events_drain;
    }
}
