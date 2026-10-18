using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Managed
{
    // The session service's twins, each answered by the session's own table, so a twin and a module's call are one function - see EmuSen_Backend.md §14.
    public static unsafe partial class ManagedCrystal
    {
        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_info(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->info(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_summary(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->summary(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_run_control(nint rt, uint session, uint op, uint processor, long argument)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->run_control(s.Table->ctx, s.Id, op, processor, argument), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_halt_state(nint rt, uint session, uint processor, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->halt_state(s.Table->ctx, s.Id, processor, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_spaces(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->spaces(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_space_read(nint rt, uint session, byte* space, nuint space_len, uint address, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->space_read(s.Table->ctx, s.Id, space, space_len, address, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_space_write(nint rt, uint session, byte* space, nuint space_len, uint address, byte* data, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->space_write(s.Table->ctx, s.Id, space, space_len, address, data, len), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_resolve_physical(nint rt, uint session, uint address, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->resolve_physical(s.Table->ctx, s.Id, address, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_processors(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->processors(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_registers(nint rt, uint session, uint processor, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->registers(s.Table->ctx, s.Id, processor, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_set_register(nint rt, uint session, uint processor, byte* name, nuint name_len, ulong value)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->set_register(s.Table->ctx, s.Id, processor, name, name_len, value), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_disassemble(nint rt, uint session, uint processor, byte* space, nuint space_len, uint address, uint count, byte* hints, nuint hints_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->disassemble(s.Table->ctx, s.Id, processor, space, space_len, address, count, hints, hints_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_add(nint rt, uint session, uint processor, byte* spec, nuint spec_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_add(s.Table->ctx, s.Id, processor, spec, spec_len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_add_data(nint rt, uint session, uint processor, byte* spec, nuint spec_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_add_data(s.Table->ctx, s.Id, processor, spec, spec_len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_set_enabled(nint rt, uint session, uint processor, uint id, uint enabled)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_set_enabled(s.Table->ctx, s.Id, processor, id, enabled), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_remove(nint rt, uint session, uint processor, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_remove(s.Table->ctx, s.Id, processor, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_list(nint rt, uint session, uint processor, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_list(s.Table->ctx, s.Id, processor, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_log(nint rt, uint session, uint processor, uint count, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_log(s.Table->ctx, s.Id, processor, count, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_log_clear(nint rt, uint session, uint processor)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_log_clear(s.Table->ctx, s.Id, processor), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_arm_condition(nint rt, uint session, uint processor, byte* name, nuint name_len, uint flags)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_arm_condition(s.Table->ctx, s.Id, processor, name, name_len, flags), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_conditions(nint rt, uint session, uint processor, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_conditions(s.Table->ctx, s.Id, processor, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_depth_guard(nint rt, uint session, uint processor, int depth)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_depth_guard(s.Table->ctx, s.Id, processor, depth), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_bp_forbid_range(nint rt, uint session, uint processor, uint address, uint end)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->bp_forbid_range(s.Table->ctx, s.Id, processor, address, end), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_bp_uninitialized(nint rt, uint session, uint processor, byte* space, nuint space_len, uint size)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->bp_uninitialized(s.Table->ctx, s.Id, processor, space, space_len, size), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_watch_add(nint rt, uint session, byte* space, nuint space_len, uint address, uint length, uint kind)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->watch_add(s.Table->ctx, s.Id, space, space_len, address, length, kind), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_watch_remove(nint rt, uint session, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->watch_remove(s.Table->ctx, s.Id, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_watch_list(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->watch_list(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_watch_events(nint rt, uint session, uint id, uint count, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->watch_events(s.Table->ctx, s.Id, id, count, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_watch_summary(nint rt, uint session, uint id, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->watch_summary(s.Table->ctx, s.Id, id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_watch_clear(nint rt, uint session, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->watch_clear(s.Table->ctx, s.Id, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_framelog_add(nint rt, uint session, byte* space, nuint space_len, uint address, uint width)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->framelog_add(s.Table->ctx, s.Id, space, space_len, address, width), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_framelog_remove(nint rt, uint session, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->framelog_remove(s.Table->ctx, s.Id, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_framelog_list(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->framelog_list(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_framelog_samples(nint rt, uint session, uint id, uint count, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->framelog_samples(s.Table->ctx, s.Id, id, count, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_framelog_clear(nint rt, uint session, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->framelog_clear(s.Table->ctx, s.Id, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_coverage_op(nint rt, uint session, uint processor, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->coverage_op(s.Table->ctx, s.Id, processor, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_profile_op(nint rt, uint session, uint processor, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->profile_op(s.Table->ctx, s.Id, processor, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_counters_op(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->counters_op(s.Table->ctx, s.Id, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_call_stack(nint rt, uint session, uint processor, uint count, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->call_stack(s.Table->ctx, s.Id, processor, count, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_freeze_add(nint rt, uint session, byte* space, nuint space_len, uint address, uint value)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->freeze_add(s.Table->ctx, s.Id, space, space_len, address, value), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_freeze_remove(nint rt, uint session, uint id)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->freeze_remove(s.Table->ctx, s.Id, id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_freeze_list(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->freeze_list(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_freeze_clear(nint rt, uint session)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->freeze_clear(s.Table->ctx, s.Id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_label_add(nint rt, uint session, uint address, byte* name, nuint name_len, byte* comment, nuint comment_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->label_add(s.Table->ctx, s.Id, address, name, name_len, comment, comment_len), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_label_remove(nint rt, uint session, byte* name, nuint name_len, uint address)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->label_remove(s.Table->ctx, s.Id, name, name_len, address), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_label_list(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->label_list(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_label_at(nint rt, uint session, uint address, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->label_at(s.Table->ctx, s.Id, address, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_label_load(nint rt, uint session, byte* text, nuint text_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->label_load(s.Table->ctx, s.Id, text, text_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_label_save(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->label_save(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_label_clear(nint rt, uint session)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->label_clear(s.Table->ctx, s.Id), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_dma_channels(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->dma_channels(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_dma_log_op(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->dma_log_op(s.Table->ctx, s.Id, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_register_flow_op(nint rt, uint session, uint op, byte* args, nuint args_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->register_flow_op(s.Table->ctx, s.Id, op, args, args_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_coprocessor_history(nint rt, uint session, uint count, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->coprocessor_history(s.Table->ctx, s.Id, count, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_vectors(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->vectors(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_tilemap_entry(nint rt, uint session, byte* space, nuint space_len, uint address, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->tilemap_entry(s.Table->ctx, s.Id, space, space_len, address, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_tile_pixels(nint rt, uint session, byte* space, nuint space_len, uint address, uint bpp, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->tile_pixels(s.Table->ctx, s.Id, space, space_len, address, bpp, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_sprites(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->sprites(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_palettes(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->palettes(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_audio_channels(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->audio_channels(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_set_channel_muted(nint rt, uint session, uint channel, uint muted)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->set_channel_muted(s.Table->ctx, s.Id, channel, muted), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_evaluate(nint rt, uint session, uint processor, byte* text, nuint text_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->evaluate(s.Table->ctx, s.Id, processor, text, text_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_symbols(nint rt, uint session, uint processor, byte* filter, nuint filter_len, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->symbols(s.Table->ctx, s.Id, processor, filter, filter_len, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_cheat_codecs(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->cheat_codecs(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_trace(nint rt, uint session, uint processor, uint count)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->trace(s.Table->ctx, s.Id, processor, count), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_trace_off(nint rt, uint session, uint processor)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->trace_off(s.Table->ctx, s.Id, processor), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_telemetry(nint rt, uint session, uint kind, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->telemetry(s.Table->ctx, s.Id, kind, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_tile_sheet(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->tile_sheet(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_palette_swatch(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->palette_swatch(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_audio_peek(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->audio_peek(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_state_save(nint rt, uint session, byte* path, nuint path_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->state_save(s.Table->ctx, s.Id, path, path_len), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_state_load(nint rt, uint session, byte* path, nuint path_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (int)Twin(s.Table->state_load(s.Table->ctx, s.Id, path, path_len), s);
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_state_default_path(nint rt, uint session, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->state_default_path(s.Table->ctx, s.Id, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_events_drain(nint rt, uint session, uint kinds, byte* out_, nuint len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->events_drain(s.Table->ctx, s.Id, kinds, out_, len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_open_game(nint rt, uint session, byte* path, nuint path_len, byte* engine, nuint engine_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->open_game(s.Table->ctx, s.Id, path, path_len, engine, engine_len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long emusen_crystal_open_console(nint rt, uint session, byte* console, nuint console_len, byte* engine, nuint engine_len)
        {
            try
            {
                ManagedSession s = SessionOf(rt, session);
                return (long)Twin(s.Table->open_console(s.Table->ctx, s.Id, console, console_len, engine, engine_len), s);
            }
            catch (Exception e) { return (long)Fail(e); }
        }
    }
}
