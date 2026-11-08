namespace EmuSen.Crystal.Client.Contract
{
    // emusen_crystal.h's exports in its order, each field a function of the implementation the switch chose; CrystalHeaderTests holds it to the header.
    public unsafe struct CrystalExports
    {
        public delegate* unmanaged[Cdecl]<uint> abi_version;
        public delegate* unmanaged[Cdecl]<byte*, nuint, long> version_text;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> last_error;
        public delegate* unmanaged[Cdecl]<CrystalConfig*, nint> @new;
        public delegate* unmanaged[Cdecl]<nint, int> free;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> systems;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> sessions;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> shelves;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> engines;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> cores;
        public delegate* unmanaged[Cdecl]<nint, int> rescan;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, int> core_approve;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, int> core_revoke;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> route;
        public delegate* unmanaged[Cdecl]<nint, CrystalOpenParams*, long> open;
        public delegate* unmanaged[Cdecl]<nint, uint, int> close;
        public delegate* unmanaged[Cdecl]<nint, uint, int> reset;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> session_info;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> disc_insert;
        public delegate* unmanaged[Cdecl]<nint, uint, int> disc_eject;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> disc_change;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> disc_lid;
        public delegate* unmanaged[Cdecl]<nint, uint, CrystalFrameInfo*, int> frame;
        public delegate* unmanaged[Cdecl]<nint, uint, int> frame_advance;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> picture_copy;
        public delegate* unmanaged[Cdecl]<nint, uint, nint> picture_lend;
        public delegate* unmanaged[Cdecl]<nint, uint, nint> picture_ring;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int> raw_select;
        public delegate* unmanaged[Cdecl]<nint, uint, CrystalRawInfo*, byte*, nuint, long> raw_copy;
        public delegate* unmanaged[Cdecl]<nint, uint, short*, nuint, uint*, long> audio_drain;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> wait;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> phase_names;
        public delegate* unmanaged[Cdecl]<nint, uint, double*, nuint, long> phases;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, int> input_button;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, uint, double, int> input_axis;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, uint, int> input_connected;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, ushort*, ushort*, int> input_rumble;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> input_rumble_switch;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, uint, int> speed_set;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> speed_mode;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, uint, ulong, uint, int> rewind_set;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, CrystalFrameInfo*, long> rewind_step;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> rewind_reel;
        public delegate* unmanaged[Cdecl]<nint, uint, ulong, byte*, nuint, long> rewind_picture;
        public delegate* unmanaged[Cdecl]<nint, uint, ulong, int> rewind_to;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> state_slots;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> state_slot_save;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, int> state_slot_load;
        public delegate* unmanaged[Cdecl]<nint, uint, int> resume_save;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> state_record;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, int> state_forget;
        public delegate* unmanaged[Cdecl]<nint, uint, int> resume_load;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, byte*, nuint, byte*, nuint, long> settings_schema;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, byte*, nuint, byte*, nuint, byte*, nuint, long> settings_get;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, byte*, nuint, byte*, nuint, byte*, nuint, int> settings_set;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, byte*, nuint, long> game_setting_get;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, byte*, nuint, int> game_setting_set;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, byte*, nuint, long> firmware_needed;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, int> firmware_install;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> firmware_overview;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> firmware_status;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> save_media;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, byte*, nuint, long> cards_op;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, nint> subscribe;
        public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> subscription_drain;
        public delegate* unmanaged[Cdecl]<nint, uint, int> subscription_wait;
        public delegate* unmanaged[Cdecl]<nint, int> unsubscribe;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, uint, byte*, nuint, long> record_start;
        public delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long> record_stop;
        public delegate* unmanaged[Cdecl]<nint, uint, uint, nint> session_service;
    }
}
