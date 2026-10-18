using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.Crystal.Session
{
    // The C entry points of emusen_session_v1: each catches everything, so nothing unwinds into a caller - see EmuSen_DianaSh.md §13.6.
    internal static unsafe class SessionExports
    {
        [ThreadStatic] private static string? t_lastError;

        private static readonly UTF8Encoding StrictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);

        private static SessionService.Binding B(void* ctx)
            => ctx == null ? throw new SessionException(SessionAbi.Null, "a null ctx") : (SessionService.Binding)GCHandle.FromIntPtr((nint)ctx).Target!;

        private static long Fail(Exception e)
        {
            t_lastError = e.Message;
            return e is SessionException s ? s.Status : SessionAbi.Fault;
        }

        private static long Copy(byte[] bytes, byte* o, nuint n)
        {
            if (o != null && n >= (nuint)bytes.Length) bytes.CopyTo(new Span<byte>(o, bytes.Length));
            return bytes.Length;
        }

        private static string? Text(byte* p, nuint n)
        {
            if (p == null) return n == 0 ? null : throw new SessionException(SessionAbi.Null, "a null string with a length");
            try { return StrictUtf8.GetString(p, checked((int)n)); }
            catch (DecoderFallbackException) { throw new SessionException(SessionAbi.BadString, "text that is not UTF-8"); }
        }

        private static ReadOnlySpan<byte> Bytes(byte* p, nuint n)
            => p == null ? (n == 0 ? default : throw new SessionException(SessionAbi.Null, "a null buffer with a length")) : new ReadOnlySpan<byte>(p, checked((int)n));

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long LastError(void* ctx, byte* o, nuint n)
        {
            try
            {
                return Copy(SessionJson.Utf8(t_lastError ?? ""), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Sessions(void* ctx, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Sessions(), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Info(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Info(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Summary(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Summary(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long RunControl(void* ctx, uint s, uint op, uint p, long argument)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().RunControl(s, op, p, argument);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long HaltState(void* ctx, uint s, uint p, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.HaltState(s, p), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Spaces(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Spaces(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long SpaceRead(void* ctx, uint s, byte* space, nuint space_len, uint address, byte* o, nuint n)
        {
            try
            {
                if (o == null) return checked((long)n);
                B(ctx).Service.SpaceRead(s, Text(space, space_len), address, new Span<byte>(o, checked((int)n)));
                return checked((long)n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int SpaceWrite(void* ctx, uint s, byte* space, nuint space_len, uint address, byte* data, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().SpaceWrite(s, Text(space, space_len), address, Bytes(data, n));
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long ResolvePhysical(void* ctx, uint s, uint address, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.ResolvePhysical(s, address), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Processors(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Processors(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Registers(void* ctx, uint s, uint p, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Registers(s, p), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int SetRegister(void* ctx, uint s, uint p, byte* name, nuint name_len, ulong value)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().SetRegister(s, p, Text(name, name_len), value);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Disassemble(void* ctx, uint s, uint p, byte* space, nuint space_len, uint address, uint count, byte* hints, nuint hints_len, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Disassemble(s, p, Text(space, space_len), address, count, Text(hints, hints_len)), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpAdd(void* ctx, uint s, uint p, byte* spec, nuint spec_len)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().BpAdd(s, p, SessionJson.Parse(Bytes(spec, spec_len)));
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpAddData(void* ctx, uint s, uint p, byte* spec, nuint spec_len)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().BpAddData(s, p, SessionJson.Parse(Bytes(spec, spec_len)));
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpSetEnabled(void* ctx, uint s, uint p, uint id, uint enabled)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpSetEnabled(s, p, id, enabled != 0);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpRemove(void* ctx, uint s, uint p, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpRemove(s, p, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpList(void* ctx, uint s, uint p, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.BpList(s, p), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpLog(void* ctx, uint s, uint p, uint count, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.BpLog(s, p, count), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpLogClear(void* ctx, uint s, uint p)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpLogClear(s, p);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpArmCondition(void* ctx, uint s, uint p, byte* name, nuint name_len, uint flags)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpArmCondition(s, p, Text(name, name_len), flags);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpConditions(void* ctx, uint s, uint p, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.BpConditions(s, p), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpDepthGuard(void* ctx, uint s, uint p, int depth)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpDepthGuard(s, p, depth);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long BpForbidRange(void* ctx, uint s, uint p, uint address, uint end)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().BpForbidRange(s, p, address, end);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int BpUninitialized(void* ctx, uint s, uint p, byte* space, nuint space_len, uint size)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().BpUninitialized(s, p, Text(space, space_len), size);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long WatchAdd(void* ctx, uint s, byte* space, nuint space_len, uint address, uint length, uint kind)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().WatchAdd(s, Text(space, space_len), address, length, kind);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int WatchRemove(void* ctx, uint s, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().WatchRemove(s, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long WatchList(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.WatchList(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long WatchEvents(void* ctx, uint s, uint id, uint count, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.WatchEvents(s, id, count), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long WatchSummary(void* ctx, uint s, uint id, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.WatchSummary(s, id), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int WatchClear(void* ctx, uint s, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().WatchClear(s, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long FramelogAdd(void* ctx, uint s, byte* space, nuint space_len, uint address, uint width)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().FramelogAdd(s, Text(space, space_len), address, width);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int FramelogRemove(void* ctx, uint s, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().FramelogRemove(s, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long FramelogList(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.FramelogList(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long FramelogSamples(void* ctx, uint s, uint id, uint count, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.FramelogSamples(s, id, count), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int FramelogClear(void* ctx, uint s, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().FramelogClear(s, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CoverageOp(void* ctx, uint s, uint p, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Service.CoverageOp(s, p, op, Bytes(args, args_len), B(ctx).ReadOnly), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long ProfileOp(void* ctx, uint s, uint p, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Service.ProfileOp(s, p, op, Bytes(args, args_len), B(ctx).ReadOnly), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CountersOp(void* ctx, uint s, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Service.CountersOp(s, op, Bytes(args, args_len), B(ctx).ReadOnly), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CallStack(void* ctx, uint s, uint p, uint count, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.CallStack(s, p, count), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long FreezeAdd(void* ctx, uint s, byte* space, nuint space_len, uint address, uint value)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return B(ctx).Change().FreezeAdd(s, Text(space, space_len), address, value);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int FreezeRemove(void* ctx, uint s, uint id)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().FreezeRemove(s, id);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long FreezeList(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.FreezeList(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int FreezeClear(void* ctx, uint s)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().FreezeClear(s);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LabelAdd(void* ctx, uint s, uint address, byte* name, nuint name_len, byte* comment, nuint comment_len)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().LabelAdd(s, address, Text(name, name_len), Text(comment, comment_len));
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LabelRemove(void* ctx, uint s, byte* name, nuint name_len, uint address)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().LabelRemove(s, Text(name, name_len), address);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long LabelList(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.LabelList(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long LabelAt(void* ctx, uint s, uint address, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.LabelAt(s, address), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long LabelLoad(void* ctx, uint s, byte* text, nuint text_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Change().LabelLoad(s, Text(text, text_len), o == null ? -1 : (long)Math.Min((ulong)n, (ulong)long.MaxValue)), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long LabelSave(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.LabelSave(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LabelClear(void* ctx, uint s)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().LabelClear(s);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long DmaChannels(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.DmaChannels(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long DmaLogOp(void* ctx, uint s, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Service.DmaLogOp(s, op, Bytes(args, args_len), B(ctx).ReadOnly), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long RegisterFlowOp(void* ctx, uint s, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                return Copy(B(ctx).Service.RegisterFlowOp(s, op, Bytes(args, args_len), B(ctx).ReadOnly), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CoprocessorHistory(void* ctx, uint s, uint count, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.CoprocessorHistory(s, count), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Vectors(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Vectors(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long TilemapEntry(void* ctx, uint s, byte* space, nuint space_len, uint address, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.TilemapEntry(s, Text(space, space_len), address), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long TilePixels(void* ctx, uint s, byte* space, nuint space_len, uint address, uint bpp, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.TilePixels(s, Text(space, space_len), address, bpp), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Sprites(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Sprites(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Palettes(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Palettes(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long AudioChannels(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.AudioChannels(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int SetChannelMuted(void* ctx, uint s, uint channel, uint muted)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().SetChannelMuted(s, channel, muted != 0);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Evaluate(void* ctx, uint s, uint p, byte* text, nuint text_len, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Evaluate(s, p, Text(text, text_len)), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Symbols(void* ctx, uint s, uint p, byte* filter, nuint filter_len, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Symbols(s, p, Text(filter, filter_len)), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CheatOp(void* ctx, uint s, uint op, byte* args, nuint args_len, byte* o, nuint n)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                var (record, value) = B(ctx).Service.CheatOp(s, op, Bytes(args, args_len), B(ctx).ReadOnly);
                if (record is null) return value;
                if (o != null && n >= (nuint)record.Length) B(ctx).Service.TakePending(op);
                return Copy(record, o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long CheatCodecs(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.CheatCodecs(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int Trace(void* ctx, uint s, uint p, uint count)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().TraceOn(s, p, count);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int TraceOff(void* ctx, uint s, uint p)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().TraceOff(s, p);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Telemetry(void* ctx, uint s, uint kind, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Telemetry(s, kind), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long TileSheet(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.TileSheet(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long PaletteSwatch(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.PaletteSwatch(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long AudioPeek(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.AudioPeek(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int StateSave(void* ctx, uint s, byte* path, nuint path_len)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().StateSave(s, Text(path, path_len));
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int StateLoad(void* ctx, uint s, byte* path, nuint path_len)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change().StateLoad(s, Text(path, path_len));
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long StateDefaultPath(void* ctx, uint s, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.StateDefaultPath(s), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LogSwitch(void* ctx, uint on)
        {
            try
            {
                using var gate = new GateScope(B(ctx).Service.Options.Gate);
                B(ctx).Change();
                SessionService.LogSwitch(on != 0);
                return 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int LogStatus(void* ctx)
        {
            try
            {
                B(ctx);
                return SessionService.LogStatus() ? 1 : 0;
            }
            catch (Exception e) { return (int)Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long EventsDrain(void* ctx, uint s, uint kinds, byte* o, nuint n)
        {
            try
            {
                var service = B(ctx).Service;
                byte[] bytes = service.EventsPeek(s, kinds, out int taken);
                if (o == null || n < (nuint)bytes.Length) return bytes.Length;
                bytes.CopyTo(new Span<byte>(o, bytes.Length));
                service.EventsTake(kinds, taken);
                return bytes.Length;
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long Games(void* ctx, byte* o, nuint n)
        {
            try
            {
                return Copy(B(ctx).Service.Games(), o, n);
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long OpenGame(void* ctx, uint s, byte* path, nuint path_len, byte* engine, nuint engine_len)
        {
            try
            {
                return B(ctx).Change().Open(s, Text(path, path_len) ?? throw new SessionException(SessionAbi.BadArgument, "open_game needs the game's path"), null, Text(engine, engine_len));
            }
            catch (Exception e) { return Fail(e); }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static long OpenConsole(void* ctx, uint s, byte* console, nuint console_len, byte* engine, nuint engine_len)
        {
            try
            {
                return B(ctx).Change().Open(s, null, Text(console, console_len) ?? throw new SessionException(SessionAbi.BadArgument, "open_console needs the console's name"), Text(engine, engine_len));
            }
            catch (Exception e) { return Fail(e); }
        }

        // The tables NewTable made and Forget has not freed, so a table handed back is known for one.
        private static readonly System.Collections.Concurrent.ConcurrentDictionary<nint, byte> Tables = new();

        public static bool Made(EmuSenSessionV1* table) => Tables.ContainsKey((nint)table);

        public static void Forget(EmuSenSessionV1* table) => Tables.TryRemove((nint)table, out _);

        // A table over one binding, in native memory a C caller can keep for the provider's life.
        public static EmuSenSessionV1* NewTable(GCHandle binding)
        {
            var t = (EmuSenSessionV1*)NativeMemory.AllocZeroed((nuint)sizeof(EmuSenSessionV1));
            t->size = (uint)sizeof(EmuSenSessionV1);
            t->version = SessionAbi.Version;
            t->ctx = (void*)GCHandle.ToIntPtr(binding);
            t->last_error = &LastError;
            t->sessions = &Sessions;
            t->info = &Info;
            t->summary = &Summary;
            t->run_control = &RunControl;
            t->halt_state = &HaltState;
            t->spaces = &Spaces;
            t->space_read = &SpaceRead;
            t->space_write = &SpaceWrite;
            t->resolve_physical = &ResolvePhysical;
            t->processors = &Processors;
            t->registers = &Registers;
            t->set_register = &SetRegister;
            t->disassemble = &Disassemble;
            t->bp_add = &BpAdd;
            t->bp_add_data = &BpAddData;
            t->bp_set_enabled = &BpSetEnabled;
            t->bp_remove = &BpRemove;
            t->bp_list = &BpList;
            t->bp_log = &BpLog;
            t->bp_log_clear = &BpLogClear;
            t->bp_arm_condition = &BpArmCondition;
            t->bp_conditions = &BpConditions;
            t->bp_depth_guard = &BpDepthGuard;
            t->bp_forbid_range = &BpForbidRange;
            t->bp_uninitialized = &BpUninitialized;
            t->watch_add = &WatchAdd;
            t->watch_remove = &WatchRemove;
            t->watch_list = &WatchList;
            t->watch_events = &WatchEvents;
            t->watch_summary = &WatchSummary;
            t->watch_clear = &WatchClear;
            t->framelog_add = &FramelogAdd;
            t->framelog_remove = &FramelogRemove;
            t->framelog_list = &FramelogList;
            t->framelog_samples = &FramelogSamples;
            t->framelog_clear = &FramelogClear;
            t->coverage_op = &CoverageOp;
            t->profile_op = &ProfileOp;
            t->counters_op = &CountersOp;
            t->call_stack = &CallStack;
            t->freeze_add = &FreezeAdd;
            t->freeze_remove = &FreezeRemove;
            t->freeze_list = &FreezeList;
            t->freeze_clear = &FreezeClear;
            t->label_add = &LabelAdd;
            t->label_remove = &LabelRemove;
            t->label_list = &LabelList;
            t->label_at = &LabelAt;
            t->label_load = &LabelLoad;
            t->label_save = &LabelSave;
            t->label_clear = &LabelClear;
            t->dma_channels = &DmaChannels;
            t->dma_log_op = &DmaLogOp;
            t->register_flow_op = &RegisterFlowOp;
            t->coprocessor_history = &CoprocessorHistory;
            t->vectors = &Vectors;
            t->tilemap_entry = &TilemapEntry;
            t->tile_pixels = &TilePixels;
            t->sprites = &Sprites;
            t->palettes = &Palettes;
            t->audio_channels = &AudioChannels;
            t->set_channel_muted = &SetChannelMuted;
            t->evaluate = &Evaluate;
            t->symbols = &Symbols;
            t->cheat_op = &CheatOp;
            t->cheat_codecs = &CheatCodecs;
            t->trace = &Trace;
            t->trace_off = &TraceOff;
            t->telemetry = &Telemetry;
            t->tile_sheet = &TileSheet;
            t->palette_swatch = &PaletteSwatch;
            t->audio_peek = &AudioPeek;
            t->state_save = &StateSave;
            t->state_load = &StateLoad;
            t->state_default_path = &StateDefaultPath;
            t->log_switch = &LogSwitch;
            t->log_status = &LogStatus;
            t->events_drain = &EventsDrain;
            t->games = &Games;
            t->open_game = &OpenGame;
            t->open_console = &OpenConsole;
            Tables[(nint)t] = 0;
            return t;
        }
    }
}
