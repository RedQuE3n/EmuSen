using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Session;

namespace EmuSen.Crystal.Managed
{
    // The frontend API's own functions, each a thin entry onto the runtime or a session - see emusen_crystal.h.
    public static unsafe partial class ManagedCrystal
    {
        private static int Done(Action act)
        {
            act();
            return 0;
        }

        private static ReadOnlySpan<byte> Bytes(byte* p, nuint n) =>
            p == null ? (n == 0 ? default : throw new CrystalException(SessionAbi.Null, "a null buffer with a length")) : new ReadOnlySpan<byte>(p, checked((int)n));

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_systems(nint rt, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.SystemsJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_engines(nint rt, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.EnginesJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_cores(nint rt, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.CoresJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_rescan(nint rt)
        {
            try { RuntimeOf(rt); return Done(Cores.Native.CoreDiscovery.Rescan); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_route(nint rt, byte* path, nuint path_len, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.RouteJson(Required(path, path_len, "the path")), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_open(nint rt, OpenParamsV1* params_)
        {
            try { return RuntimeOf(rt).Open(params_).Id; }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_close(nint rt, uint session)
        {
            try { return Done(SessionOf(rt, session).Close); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_reset(nint rt, uint session)
        {
            try { return Done(SessionOf(rt, session).Reset); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_session_info(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(SessionOf(rt, session).Info(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_disc_insert(nint rt, uint session, byte* path, nuint path_len)
        {
            try { ManagedSession s = SessionOf(rt, session); string p = Required(path, path_len, "the disc's path"); return Done(() => s.InsertDisc(p)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_disc_eject(nint rt, uint session)
        {
            try { return Done(SessionOf(rt, session).EjectDisc); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_disc_change(nint rt, uint session, uint index)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.ChangeDisc(index)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_disc_lid(nint rt, uint session, uint open)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.Lid(open != 0)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_frame(nint rt, uint session, FrameInfoV1* out_)
        {
            try
            {
                SessionOf(rt, session).Frame(out_);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_picture_copy(nint rt, uint session, byte* out_, nuint len)
        {
            try { return SessionOf(rt, session).CopyPicture(out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_audio_drain(nint rt, uint session, short* out_, nuint frames, uint* rate)
        {
            try { return SessionOf(rt, session).DrainAudio(out_, frames, rate); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_wait(nint rt, uint session, uint timeout_ms)
        {
            try { return SessionOf(rt, session).Wait((int)Math.Min(timeout_ms, int.MaxValue)) ? 1 : 0; }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_phase_names(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(SessionOf(rt, session).PhaseNames(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_phases(nint rt, uint session, double* out_, nuint count)
        {
            try { return SessionOf(rt, session).Phases(out_, count); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_input_button(nint rt, uint session, uint port, uint control, uint pressed)
        {
            try
            {
                SessionOf(rt, session).Button(port, control, pressed != 0);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_input_axis(nint rt, uint session, uint port, uint axis, double value)
        {
            try
            {
                SessionOf(rt, session).Axis(port, axis, value);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_input_connected(nint rt, uint session, uint port, uint connected)
        {
            try
            {
                SessionOf(rt, session).Connected(port, connected != 0);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_speed_set(nint rt, uint session, uint percent, uint turbo_percent, uint slow_percent, uint flags)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.SetSpeed(percent, turbo_percent, slow_percent, flags)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_speed_mode(nint rt, uint session, uint mode)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.SetMode(mode)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_rewind_set(nint rt, uint session, uint enabled, uint interval_frames, ulong budget_bytes, uint thumbnail_width)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.SetRewind(enabled != 0, interval_frames, budget_bytes, thumbnail_width)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_rewind_step(nint rt, uint session, uint count, FrameInfoV1* out_)
        {
            try { return SessionOf(rt, session).StepBack(count, out_); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_rewind_reel(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(SessionOf(rt, session).ReelJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_rewind_picture(nint rt, uint session, ulong frame, byte* out_, nuint len)
        {
            try { return Copy(SessionOf(rt, session).RewindPicture(frame), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_rewind_to(nint rt, uint session, ulong frame)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.RewindTo(frame)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_state_slots(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(SessionOf(rt, session).SlotsJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_state_slot_save(nint rt, uint session, uint slot)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.SaveSlot(slot)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_state_slot_load(nint rt, uint session, uint slot)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(() => s.LoadSlot(slot)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_resume_save(nint rt, uint session)
        {
            try { return Done(SessionOf(rt, session).SaveResume); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_resume_load(nint rt, uint session)
        {
            try { return Done(SessionOf(rt, session).LoadResume); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_settings_schema(nint rt, uint session, byte* console, nuint console_len, byte* engine, nuint engine_len, byte* out_, nuint len)
        {
            try
            {
                ManagedRuntime r = RuntimeOf(rt);
                ManagedSession? s = session != 0 ? r.Session(session) : null;
                string name = Text(console, console_len) ?? s?.Console ?? throw new CrystalException(SessionAbi.BadArgument, "with no session, the console is named");
                return Copy(ManagedRuntime.SchemaJson(ManagedRuntime.Schema(name, Text(engine, engine_len), s)), out_, len);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_settings_get(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* out_, nuint len)
        {
            try
            {
                string k = Required(key, key_len, "the key");
                string value = RuntimeOf(rt).GetSetting(session, scope, Text(console, console_len), k) ?? throw new CrystalException(SessionAbi.Absent, $"{k} has no value and no default");
                return Copy(Encoding.UTF8.GetBytes(value), out_, len);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_settings_set(nint rt, uint session, uint scope, byte* console, nuint console_len, byte* key, nuint key_len, byte* value, nuint value_len)
        {
            try
            {
                ManagedRuntime r = RuntimeOf(rt);
                string? c = Text(console, console_len), v = Text(value, value_len);
                string k = Required(key, key_len, "the key");
                return Done(() => r.SetSetting(session, scope, c, k, v));
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_firmware_needed(nint rt, byte* path, nuint path_len, byte* engine, nuint engine_len, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.FirmwareNeededJson(Required(path, path_len, "the path"), Text(engine, engine_len)), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_firmware_install(nint rt, byte* request, nuint request_len, byte* source, nuint source_len)
        {
            try
            {
                RuntimeOf(rt);
                ManagedRuntime.InstallFirmware(Bytes(request, request_len), Required(source, source_len, "the source file"));
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_firmware_overview(nint rt, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.FirmwareOverviewJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_save_media(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(ManagedRuntime.SaveMediaJson(SessionOf(rt, session)), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_cards_op(nint rt, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.CardsOp(op, Bytes(args, args_len)), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        private static ManagedSubscription SubscriptionOf(nint sub) =>
            sub == 0 ? throw new CrystalException(SessionAbi.Null, "a null subscription")
            : GCHandle.FromIntPtr(sub).Target as ManagedSubscription ?? throw new CrystalException(SessionAbi.Null, "not a subscription");

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint emusen_crystal_subscribe(nint rt, uint session, uint kinds)
        {
            try
            {
                ManagedRuntime r = RuntimeOf(rt);
                if (session != 0) r.Session(session);
                return r.Events.Subscribe(session, kinds).Handle;
            }
            catch (Exception e)
            {
                Fail(e);
                return 0;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_subscription_drain(nint sub, byte* out_, nuint len)
        {
            try { return SubscriptionOf(sub).Drain(out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_subscription_wait(nint sub, uint timeout_ms)
        {
            try { return SubscriptionOf(sub).Wait((int)Math.Min(timeout_ms, int.MaxValue)) ? 1 : 0; }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_unsubscribe(nint sub)
        {
            try { return Done(SubscriptionOf(sub).Dispose); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_record_start(nint rt, uint session, byte* base_, nuint base_len, uint stride, byte* out_, nuint len)
        {
            try { ManagedSession s = SessionOf(rt, session); return Copy(Encoding.UTF8.GetBytes(s.StartRecording(Required(base_, base_len, "the folder"), stride)), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_record_stop(nint rt, uint session, byte* out_, nuint len)
        {
            try { return Copy(Encoding.UTF8.GetBytes(SessionOf(rt, session).StopRecording()), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint emusen_crystal_session_service(nint rt, uint session, uint flags)
        {
            try
            {
                if ((flags & ~CrystalNumbers.ServiceReadOnly) != 0) throw new CrystalException(SessionAbi.NotSupported, $"service flags 0x{flags:X} are not known to 1.{CrystalNumbers.Minor}");
                ManagedSession s = SessionOf(rt, session);
                return (nint)((flags & CrystalNumbers.ServiceReadOnly) != 0 ? s.Provider.ReadOnlyTable : s.Provider.Table);
            }
            catch (Exception e)
            {
                Fail(e);
                return 0;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_sessions(nint rt, byte* out_, nuint len)
        {
            try { return Copy(RuntimeOf(rt).SessionsJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_log_switch(nint rt, uint on)
        {
            try
            {
                RuntimeOf(rt);
                DebugLogging.MasterEnabled = on != 0;
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_log_status(nint rt)
        {
            try
            {
                RuntimeOf(rt);
                return DebugLogging.MasterEnabled ? 1 : 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }
    }
}
