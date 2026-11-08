namespace EmuSen.Crystal.Client.Contract
{
    // One function per export of emusen_crystal.h, named for it, forwarding to the implementation the switch chose - see EmuSen_Backend.md §14.
    public static unsafe partial class Crystal
    {
        public static uint AbiVersion() => Table->abi_version();

        public static long VersionText(byte* out_, nuint len) => Table->version_text(out_, len);

        public static long LastError(nint rt, byte* out_, nuint len) => Table->last_error(rt, out_, len);

        public static nint New(CrystalConfig* config) => Table->@new(config);

        public static int Free(nint rt) => Table->free(rt);

        public static long Systems(nint rt, byte* out_, nuint len) => Table->systems(rt, out_, len);

        public static long Sessions(nint rt, byte* out_, nuint len) => Table->sessions(rt, out_, len);

        public static long Shelves(nint rt, byte* out_, nuint len) => Table->shelves(rt, out_, len);

        public static long Engines(nint rt, byte* out_, nuint len) => Table->engines(rt, out_, len);

        public static long Cores(nint rt, byte* out_, nuint len) => Table->cores(rt, out_, len);

        public static int Rescan(nint rt) => Table->rescan(rt);

        public static int CoreApprove(nint rt, byte* sha256, nuint sha256_len) => Table->core_approve(rt, sha256, sha256_len);

        public static int CoreRevoke(nint rt, byte* sha256, nuint sha256_len) => Table->core_revoke(rt, sha256, sha256_len);

        public static long Route(nint rt, byte* path, nuint path_len, byte* out_, nuint len) => Table->route(rt, path, path_len, out_, len);

        public static long Open(nint rt, CrystalOpenParams* params_) => Table->open(rt, params_);

        public static int Close(nint rt, uint session) => Table->close(rt, session);

        public static int Reset(nint rt, uint session) => Table->reset(rt, session);

        public static long SessionInfo(nint rt, uint session, byte* out_, nuint len) => Table->session_info(rt, session, out_, len);

        public static int DiscInsert(nint rt, uint session, byte* path, nuint path_len) => Table->disc_insert(rt, session, path, path_len);

        public static int DiscEject(nint rt, uint session) => Table->disc_eject(rt, session);

        public static int DiscChange(nint rt, uint session, uint index) => Table->disc_change(rt, session, index);

        public static int DiscLid(nint rt, uint session, uint open) => Table->disc_lid(rt, session, open);

        public static int Frame(nint rt, uint session, CrystalFrameInfo* out_) => Table->frame(rt, session, out_);

        public static long PictureCopy(nint rt, uint session, byte* out_, nuint len) => Table->picture_copy(rt, session, out_, len);

        public static nint PictureLend(nint rt, uint session) => Table->picture_lend(rt, session);

        public static nint PictureRing(nint rt, uint session) => Table->picture_ring(rt, session);

        public static int FrameAdvance(nint rt, uint session) => Table->frame_advance(rt, session);

        public static int RawSelect(nint rt, uint session, byte* encoding, nuint len) => Table->raw_select(rt, session, encoding, len);

        public static long RawCopy(nint rt, uint session, CrystalRawInfo* info, byte* out_, nuint len) => Table->raw_copy(rt, session, info, out_, len);

        public static long AudioDrain(nint rt, uint session, short* out_, nuint frames, uint* rate) => Table->audio_drain(rt, session, out_, frames, rate);

        public static int Wait(nint rt, uint session, uint timeout_ms) => Table->wait(rt, session, timeout_ms);

        public static long PhaseNames(nint rt, uint session, byte* out_, nuint len) => Table->phase_names(rt, session, out_, len);

        public static long Phases(nint rt, uint session, double* out_, nuint count) => Table->phases(rt, session, out_, count);

        public static int InputButton(nint rt, uint session, uint port, uint control, uint pressed) => Table->input_button(rt, session, port, control, pressed);

        public static int InputAxis(nint rt, uint session, uint port, uint axis, double value) => Table->input_axis(rt, session, port, axis, value);

        public static int InputConnected(nint rt, uint session, uint port, uint connected) => Table->input_connected(rt, session, port, connected);

        public static int InputRumble(nint rt, uint session, uint port, ushort* strong, ushort* weak) => Table->input_rumble(rt, session, port, strong, weak);
        public static int InputRumbleSwitch(nint rt, uint session, uint on) => Table->input_rumble_switch(rt, session, on);

        public static int SpeedSet(nint rt, uint session, uint percent, uint turbo_percent, uint slow_percent, uint flags) => Table->speed_set(rt, session, percent, turbo_percent, slow_percent, flags);

        public static int SpeedMode(nint rt, uint session, uint mode) => Table->speed_mode(rt, session, mode);

        public static int RewindSet(nint rt, uint session, uint enabled, uint interval_frames, ulong budget_bytes, uint thumbnail_width) => Table->rewind_set(rt, session, enabled, interval_frames, budget_bytes, thumbnail_width);

        public static long RewindStep(nint rt, uint session, uint count, CrystalFrameInfo* out_) => Table->rewind_step(rt, session, count, out_);

        public static long RewindReel(nint rt, uint session, byte* out_, nuint len) => Table->rewind_reel(rt, session, out_, len);

        public static long RewindPicture(nint rt, uint session, ulong frame, byte* out_, nuint len) => Table->rewind_picture(rt, session, frame, out_, len);

        public static int RewindTo(nint rt, uint session, ulong frame) => Table->rewind_to(rt, session, frame);

        public static long StateSlots(nint rt, uint session, byte* out_, nuint len) => Table->state_slots(rt, session, out_, len);

        public static int StateSlotSave(nint rt, uint session, uint slot) => Table->state_slot_save(rt, session, slot);

        public static int StateSlotLoad(nint rt, uint session, uint slot) => Table->state_slot_load(rt, session, slot);

        public static int ResumeSave(nint rt, uint session) => Table->resume_save(rt, session);

        public static long StateRecord(nint rt, byte* path, nuint path_len, byte* out_, nuint len) => Table->state_record(rt, path, path_len, out_, len);

        public static int StateForget(nint rt, byte* path, nuint path_len) => Table->state_forget(rt, path, path_len);

        public static int ResumeLoad(nint rt, uint session) => Table->resume_load(rt, session);

        public static long SettingsSchema(nint rt, uint session, byte* console, nuint console_len, byte* engine, nuint engine_len, byte* out_, nuint len) => Table->settings_schema(rt, session, console, console_len, engine, engine_len, out_, len);

        public static long SettingsGet(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* out_, nuint len) => Table->settings_get(rt, session, scope, console, console_len, key, key_len, out_, len);

        public static int SettingsSet(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* value, nuint value_len) => Table->settings_set(rt, session, scope, console, console_len, key, key_len, value, value_len);

        public static long GameSettingGet(nint rt, byte* path, nuint path_len, byte* key, nuint key_len, byte* out_, nuint len) => Table->game_setting_get(rt, path, path_len, key, key_len, out_, len);

        public static int GameSettingSet(nint rt, byte* path, nuint path_len, byte* key, nuint key_len, byte* value, nuint value_len) => Table->game_setting_set(rt, path, path_len, key, key_len, value, value_len);

        public static long FirmwareNeeded(nint rt, byte* path, nuint path_len, byte* engine, nuint engine_len, byte* out_, nuint len) => Table->firmware_needed(rt, path, path_len, engine, engine_len, out_, len);

        public static int FirmwareInstall(nint rt, byte* request, nuint request_len, byte* source, nuint source_len) => Table->firmware_install(rt, request, request_len, source, source_len);

        public static long FirmwareOverview(nint rt, byte* out_, nuint len) => Table->firmware_overview(rt, out_, len);
        public static long FirmwareStatus(nint rt, byte* system, nuint system_len, byte* out_, nuint len) => Table->firmware_status(rt, system, system_len, out_, len);

        public static long SaveMedia(nint rt, uint session, byte* out_, nuint len) => Table->save_media(rt, session, out_, len);

        public static long CardsOp(nint rt, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->cards_op(rt, op, args, args_len, out_, len);

        public static nint Subscribe(nint rt, uint session, uint kinds) => Table->subscribe(rt, session, kinds);

        public static long SubscriptionDrain(nint sub, byte* out_, nuint len) => Table->subscription_drain(sub, out_, len);

        public static int SubscriptionWait(nint sub, uint timeout_ms) => Table->subscription_wait(sub, timeout_ms);

        public static int Unsubscribe(nint sub) => Table->unsubscribe(sub);

        public static long RecordStart(nint rt, uint session, byte* base_, nuint base_len, uint stride, byte* out_, nuint len) => Table->record_start(rt, session, base_, base_len, stride, out_, len);

        public static long RecordStop(nint rt, uint session, byte* out_, nuint len) => Table->record_stop(rt, session, out_, len);

        public static nint SessionService(nint rt, uint session, uint flags) => Table->session_service(rt, session, flags);
    }
}
