using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using EmuSen.Crystal.Session;

namespace EmuSen.Crystal.Managed
{
    // The functions 1.1 added: the shelves, the device's lend, the raw picture, the states' records, a game's settings with no session, and cheat_op with no game - see EmuSen_Backend.md §14.12.
    public static unsafe partial class ManagedCrystal
    {
        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_shelves(nint rt, byte* out_, nuint len)
        {
            try { RuntimeOf(rt); return Copy(ManagedRuntime.ShelvesJson(), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint emusen_crystal_picture_lend(nint rt, uint session)
        {
            try { return SessionOf(rt, session).PictureLend(); }
            catch (Exception e)
            {
                Fail(e);
                return 0;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_raw_select(nint rt, uint session, byte* encoding, nuint len)
        {
            try { ManagedSession s = SessionOf(rt, session); string? chosen = Text(encoding, len); return Done(() => s.RawSelect(chosen)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_raw_copy(nint rt, uint session, RawInfoV1* info, byte* out_, nuint len)
        {
            try { return SessionOf(rt, session).RawCopy(info, out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_state_record(nint rt, byte* path, nuint path_len, byte* out_, nuint len)
        {
            try { return Copy(RuntimeOf(rt).StateRecordJson(Required(path, path_len, "the path")), out_, len); }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_state_forget(nint rt, byte* path, nuint path_len)
        {
            try { ManagedRuntime r = RuntimeOf(rt); string p = Required(path, path_len, "the path"); return Done(() => r.Records.Forget(p)); }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_game_setting_get(nint rt, byte* path, nuint path_len, byte* key, nuint key_len, byte* out_, nuint len)
        {
            try
            {
                RuntimeOf(rt);
                string value = ManagedRuntime.GameSetting(Required(path, path_len, "the path"), Required(key, key_len, "the key"))
                    ?? throw new CrystalException(SessionAbi.Absent, "the game holds no value of its own for that key");
                return Copy(System.Text.Encoding.UTF8.GetBytes(value), out_, len);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_game_setting_set(nint rt, byte* path, nuint path_len, byte* key, nuint key_len, byte* value, nuint value_len)
        {
            try
            {
                ManagedRuntime r = RuntimeOf(rt);
                string p = Required(path, path_len, "the path"), k = Required(key, key_len, "the key");
                string? v = Text(value, value_len);
                return Done(() => r.SetGameSetting(p, k, v));
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        // The session's table answers a game's cheat ops; with session 0, the runtime's provider with no target answers those that need no game.
        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_cheat_op(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedRuntime r = RuntimeOf(rt);
                if (session == 0)
                {
                    if (!ManagedRuntime.IsGameFreeCheatOp(op)) throw new CrystalException(SessionAbi.NoSession, $"cheat op {op} is a game's: name its session");
                    EmuSenSessionV1* table = r.NoGame.Table;
                    return TwinOf(table->cheat_op(table->ctx, 0, op, args, args_len, out_, len), table);
                }
                ManagedSession s = r.Session(session);
                return Twin(s.Table->cheat_op(s.Table->ctx, s.Id, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return Fail(e); }
        }
    }
}
