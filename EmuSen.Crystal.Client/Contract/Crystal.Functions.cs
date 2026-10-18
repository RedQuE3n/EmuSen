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

        public static long Engines(nint rt, byte* out_, nuint len) => Table->engines(rt, out_, len);

        public static long Cores(nint rt, byte* out_, nuint len) => Table->cores(rt, out_, len);

        public static int Rescan(nint rt) => Table->rescan(rt);

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

        public static long AudioDrain(nint rt, uint session, short* out_, nuint frames, uint* rate) => Table->audio_drain(rt, session, out_, frames, rate);

        public static int Wait(nint rt, uint session, uint timeout_ms) => Table->wait(rt, session, timeout_ms);

        public static long PhaseNames(nint rt, uint session, byte* out_, nuint len) => Table->phase_names(rt, session, out_, len);

        public static long Phases(nint rt, uint session, double* out_, nuint count) => Table->phases(rt, session, out_, count);

        public static int InputButton(nint rt, uint session, uint port, uint control, uint pressed) => Table->input_button(rt, session, port, control, pressed);

        public static int InputAxis(nint rt, uint session, uint port, uint axis, double value) => Table->input_axis(rt, session, port, axis, value);

        public static int InputConnected(nint rt, uint session, uint port, uint connected) => Table->input_connected(rt, session, port, connected);

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

        public static int ResumeLoad(nint rt, uint session) => Table->resume_load(rt, session);

        public static long SettingsSchema(nint rt, uint session, byte* console, nuint console_len, byte* engine, nuint engine_len, byte* out_, nuint len) => Table->settings_schema(rt, session, console, console_len, engine, engine_len, out_, len);

        public static long SettingsGet(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* out_, nuint len) => Table->settings_get(rt, session, scope, console, console_len, key, key_len, out_, len);

        public static int SettingsSet(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* value, nuint value_len) => Table->settings_set(rt, session, scope, console, console_len, key, key_len, value, value_len);

        public static long FirmwareNeeded(nint rt, byte* path, nuint path_len, byte* engine, nuint engine_len, byte* out_, nuint len) => Table->firmware_needed(rt, path, path_len, engine, engine_len, out_, len);

        public static int FirmwareInstall(nint rt, byte* request, nuint request_len, byte* source, nuint source_len) => Table->firmware_install(rt, request, request_len, source, source_len);

        public static long FirmwareOverview(nint rt, byte* out_, nuint len) => Table->firmware_overview(rt, out_, len);

        public static long SaveMedia(nint rt, uint session, byte* out_, nuint len) => Table->save_media(rt, session, out_, len);

        public static long CardsOp(nint rt, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->cards_op(rt, op, args, args_len, out_, len);

        public static nint Subscribe(nint rt, uint session, uint kinds) => Table->subscribe(rt, session, kinds);

        public static long SubscriptionDrain(nint sub, byte* out_, nuint len) => Table->subscription_drain(sub, out_, len);

        public static int SubscriptionWait(nint sub, uint timeout_ms) => Table->subscription_wait(sub, timeout_ms);

        public static int Unsubscribe(nint sub) => Table->unsubscribe(sub);

        public static long RecordStart(nint rt, uint session, byte* base_, nuint base_len, uint stride, byte* out_, nuint len) => Table->record_start(rt, session, base_, base_len, stride, out_, len);

        public static long RecordStop(nint rt, uint session, byte* out_, nuint len) => Table->record_stop(rt, session, out_, len);

        public static nint SessionService(nint rt, uint session, uint flags) => Table->session_service(rt, session, flags);

        public static long Sessions(nint rt, byte* out_, nuint len) => Table->sessions(rt, out_, len);

        public static long Info(nint rt, uint session, byte* out_, nuint len) => Table->info(rt, session, out_, len);

        public static long Summary(nint rt, uint session, byte* out_, nuint len) => Table->summary(rt, session, out_, len);

        public static long RunControl(nint rt, uint session, uint op, uint processor, long argument) => Table->run_control(rt, session, op, processor, argument);

        public static long HaltState(nint rt, uint session, uint processor, byte* out_, nuint len) => Table->halt_state(rt, session, processor, out_, len);

        public static long Spaces(nint rt, uint session, byte* out_, nuint len) => Table->spaces(rt, session, out_, len);

        public static long SpaceRead(nint rt, uint session, byte* space, nuint space_len, uint address, byte* out_, nuint len) => Table->space_read(rt, session, space, space_len, address, out_, len);

        public static int SpaceWrite(nint rt, uint session, byte* space, nuint space_len, uint address, byte* data, nuint len) => Table->space_write(rt, session, space, space_len, address, data, len);

        public static long ResolvePhysical(nint rt, uint session, uint address, byte* out_, nuint len) => Table->resolve_physical(rt, session, address, out_, len);

        public static long Processors(nint rt, uint session, byte* out_, nuint len) => Table->processors(rt, session, out_, len);

        public static long Registers(nint rt, uint session, uint processor, byte* out_, nuint len) => Table->registers(rt, session, processor, out_, len);

        public static int SetRegister(nint rt, uint session, uint processor, byte* name, nuint name_len, ulong value) => Table->set_register(rt, session, processor, name, name_len, value);

        public static long Disassemble(nint rt, uint session, uint processor, byte* space, nuint space_len, uint address, uint count, byte* hints, nuint hints_len, byte* out_, nuint len) => Table->disassemble(rt, session, processor, space, space_len, address, count, hints, hints_len, out_, len);

        public static long BpAdd(nint rt, uint session, uint processor, byte* spec, nuint spec_len) => Table->bp_add(rt, session, processor, spec, spec_len);

        public static long BpAddData(nint rt, uint session, uint processor, byte* spec, nuint spec_len) => Table->bp_add_data(rt, session, processor, spec, spec_len);

        public static int BpSetEnabled(nint rt, uint session, uint processor, uint id, uint enabled) => Table->bp_set_enabled(rt, session, processor, id, enabled);

        public static int BpRemove(nint rt, uint session, uint processor, uint id) => Table->bp_remove(rt, session, processor, id);

        public static long BpList(nint rt, uint session, uint processor, byte* out_, nuint len) => Table->bp_list(rt, session, processor, out_, len);

        public static long BpLog(nint rt, uint session, uint processor, uint count, byte* out_, nuint len) => Table->bp_log(rt, session, processor, count, out_, len);

        public static int BpLogClear(nint rt, uint session, uint processor) => Table->bp_log_clear(rt, session, processor);

        public static int BpArmCondition(nint rt, uint session, uint processor, byte* name, nuint name_len, uint flags) => Table->bp_arm_condition(rt, session, processor, name, name_len, flags);

        public static long BpConditions(nint rt, uint session, uint processor, byte* out_, nuint len) => Table->bp_conditions(rt, session, processor, out_, len);

        public static int BpDepthGuard(nint rt, uint session, uint processor, int depth) => Table->bp_depth_guard(rt, session, processor, depth);

        public static long BpForbidRange(nint rt, uint session, uint processor, uint address, uint end) => Table->bp_forbid_range(rt, session, processor, address, end);

        public static int BpUninitialized(nint rt, uint session, uint processor, byte* space, nuint space_len, uint size) => Table->bp_uninitialized(rt, session, processor, space, space_len, size);

        public static long WatchAdd(nint rt, uint session, byte* space, nuint space_len, uint address, uint length, uint kind) => Table->watch_add(rt, session, space, space_len, address, length, kind);

        public static int WatchRemove(nint rt, uint session, uint id) => Table->watch_remove(rt, session, id);

        public static long WatchList(nint rt, uint session, byte* out_, nuint len) => Table->watch_list(rt, session, out_, len);

        public static long WatchEvents(nint rt, uint session, uint id, uint count, byte* out_, nuint len) => Table->watch_events(rt, session, id, count, out_, len);

        public static long WatchSummary(nint rt, uint session, uint id, byte* out_, nuint len) => Table->watch_summary(rt, session, id, out_, len);

        public static int WatchClear(nint rt, uint session, uint id) => Table->watch_clear(rt, session, id);

        public static long FramelogAdd(nint rt, uint session, byte* space, nuint space_len, uint address, uint width) => Table->framelog_add(rt, session, space, space_len, address, width);

        public static int FramelogRemove(nint rt, uint session, uint id) => Table->framelog_remove(rt, session, id);

        public static long FramelogList(nint rt, uint session, byte* out_, nuint len) => Table->framelog_list(rt, session, out_, len);

        public static long FramelogSamples(nint rt, uint session, uint id, uint count, byte* out_, nuint len) => Table->framelog_samples(rt, session, id, count, out_, len);

        public static int FramelogClear(nint rt, uint session, uint id) => Table->framelog_clear(rt, session, id);

        public static long CoverageOp(nint rt, uint session, uint processor, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->coverage_op(rt, session, processor, op, args, args_len, out_, len);

        public static long ProfileOp(nint rt, uint session, uint processor, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->profile_op(rt, session, processor, op, args, args_len, out_, len);

        public static long CountersOp(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->counters_op(rt, session, op, args, args_len, out_, len);

        public static long CallStack(nint rt, uint session, uint processor, uint count, byte* out_, nuint len) => Table->call_stack(rt, session, processor, count, out_, len);

        public static long FreezeAdd(nint rt, uint session, byte* space, nuint space_len, uint address, uint value) => Table->freeze_add(rt, session, space, space_len, address, value);

        public static int FreezeRemove(nint rt, uint session, uint id) => Table->freeze_remove(rt, session, id);

        public static long FreezeList(nint rt, uint session, byte* out_, nuint len) => Table->freeze_list(rt, session, out_, len);

        public static int FreezeClear(nint rt, uint session) => Table->freeze_clear(rt, session);

        public static int LabelAdd(nint rt, uint session, uint address, byte* name, nuint name_len, byte* comment, nuint comment_len) => Table->label_add(rt, session, address, name, name_len, comment, comment_len);

        public static int LabelRemove(nint rt, uint session, byte* name, nuint name_len, uint address) => Table->label_remove(rt, session, name, name_len, address);

        public static long LabelList(nint rt, uint session, byte* out_, nuint len) => Table->label_list(rt, session, out_, len);

        public static long LabelAt(nint rt, uint session, uint address, byte* out_, nuint len) => Table->label_at(rt, session, address, out_, len);

        public static long LabelLoad(nint rt, uint session, byte* text, nuint text_len, byte* out_, nuint len) => Table->label_load(rt, session, text, text_len, out_, len);

        public static long LabelSave(nint rt, uint session, byte* out_, nuint len) => Table->label_save(rt, session, out_, len);

        public static int LabelClear(nint rt, uint session) => Table->label_clear(rt, session);

        public static long DmaChannels(nint rt, uint session, byte* out_, nuint len) => Table->dma_channels(rt, session, out_, len);

        public static long DmaLogOp(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->dma_log_op(rt, session, op, args, args_len, out_, len);

        public static long RegisterFlowOp(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->register_flow_op(rt, session, op, args, args_len, out_, len);

        public static long CoprocessorHistory(nint rt, uint session, uint count, byte* out_, nuint len) => Table->coprocessor_history(rt, session, count, out_, len);

        public static long Vectors(nint rt, uint session, byte* out_, nuint len) => Table->vectors(rt, session, out_, len);

        public static long TilemapEntry(nint rt, uint session, byte* space, nuint space_len, uint address, byte* out_, nuint len) => Table->tilemap_entry(rt, session, space, space_len, address, out_, len);

        public static long TilePixels(nint rt, uint session, byte* space, nuint space_len, uint address, uint bpp, byte* out_, nuint len) => Table->tile_pixels(rt, session, space, space_len, address, bpp, out_, len);

        public static long Sprites(nint rt, uint session, byte* out_, nuint len) => Table->sprites(rt, session, out_, len);

        public static long Palettes(nint rt, uint session, byte* out_, nuint len) => Table->palettes(rt, session, out_, len);

        public static long AudioChannels(nint rt, uint session, byte* out_, nuint len) => Table->audio_channels(rt, session, out_, len);

        public static int SetChannelMuted(nint rt, uint session, uint channel, uint muted) => Table->set_channel_muted(rt, session, channel, muted);

        public static long Evaluate(nint rt, uint session, uint processor, byte* text, nuint text_len, byte* out_, nuint len) => Table->evaluate(rt, session, processor, text, text_len, out_, len);

        public static long Symbols(nint rt, uint session, uint processor, byte* filter, nuint filter_len, byte* out_, nuint len) => Table->symbols(rt, session, processor, filter, filter_len, out_, len);

        public static long CheatOp(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len) => Table->cheat_op(rt, session, op, args, args_len, out_, len);

        public static long CheatCodecs(nint rt, uint session, byte* out_, nuint len) => Table->cheat_codecs(rt, session, out_, len);

        public static int Trace(nint rt, uint session, uint processor, uint count) => Table->trace(rt, session, processor, count);

        public static int TraceOff(nint rt, uint session, uint processor) => Table->trace_off(rt, session, processor);

        public static long Telemetry(nint rt, uint session, uint kind, byte* out_, nuint len) => Table->telemetry(rt, session, kind, out_, len);

        public static long TileSheet(nint rt, uint session, byte* out_, nuint len) => Table->tile_sheet(rt, session, out_, len);

        public static long PaletteSwatch(nint rt, uint session, byte* out_, nuint len) => Table->palette_swatch(rt, session, out_, len);

        public static long AudioPeek(nint rt, uint session, byte* out_, nuint len) => Table->audio_peek(rt, session, out_, len);

        public static int StateSave(nint rt, uint session, byte* path, nuint path_len) => Table->state_save(rt, session, path, path_len);

        public static int StateLoad(nint rt, uint session, byte* path, nuint path_len) => Table->state_load(rt, session, path, path_len);

        public static long StateDefaultPath(nint rt, uint session, byte* out_, nuint len) => Table->state_default_path(rt, session, out_, len);

        public static int LogSwitch(nint rt, uint on) => Table->log_switch(rt, on);

        public static int LogStatus(nint rt) => Table->log_status(rt);

        public static long EventsDrain(nint rt, uint session, uint kinds, byte* out_, nuint len) => Table->events_drain(rt, session, kinds, out_, len);
    }
}
