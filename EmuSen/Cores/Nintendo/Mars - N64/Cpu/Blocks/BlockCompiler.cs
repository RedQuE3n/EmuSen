using System;
using System.Diagnostics;
using System.Reflection;
using System.Reflection.Emit;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Channels;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Blocks
{
    // Emits a block as one method: the interpreter's own opcodes called with constant words, the simple ones inlined - see Mars_Recompiler.md §3.
    internal static class BlockCompiler
    {
#if DEBUG
        public static readonly bool Verify = true;
#else
        public static readonly bool Verify = false;
#endif

        private const BindingFlags Any = BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic;

        private static readonly Type CpuType = typeof(Core.Cpu);

        private static readonly FieldInfo Bus = CpuField("_bus");
        private static readonly FieldInfo Gpr = CpuField("Gpr");
        private static readonly FieldInfo Pc = CpuField("Pc");
        private static readonly FieldInfo NextPc = CpuField("NextPc");
        private static readonly FieldInfo CurrentPc = CpuField("CurrentPc");
        private static readonly FieldInfo InDelaySlot = CpuField("InDelaySlot");
        private static readonly FieldInfo BranchPending = CpuField("_branchPending");
        private static readonly FieldInfo Instructions = CpuField("Instructions");
        private static readonly FieldInfo TimerDue = CpuField("_timerDue");
        private static readonly FieldInfo CapAt = CpuField("_capAt");
        private static readonly FieldInfo ReturnAfterSlot = CpuField("_returnAfterSlot");
        private static readonly FieldInfo ExtraCycles = CpuField("_extraCycles");
        private static readonly FieldInfo Hi = CpuField("Hi");
        private static readonly FieldInfo Lo = CpuField("Lo");
        private static readonly FieldInfo CallObserver = CpuField("CallObserver");
        private static readonly FieldInfo ReturnObserver = CpuField("ReturnObserver");
        private static readonly FieldInfo DpWriteMarks = CpuField("_dpWriteMarks");
        private static readonly FieldInfo Cop0 = CpuField("Cop0");
        private static readonly MethodInfo WriteFpuWord = CpuMethod("WriteFpuWord", typeof(int), typeof(uint));
        private static readonly MethodInfo WriteFpuWide = CpuMethod("WriteFpuWide", typeof(int), typeof(ulong));
        private static readonly MethodInfo ReadFpuWord = CpuMethod("ReadFpuWord", typeof(int));
        private static readonly FieldInfo BusRdram = Field(typeof(MemoryBus), "Rdram");
        private static readonly MethodInfo Swap16 = Method(typeof(System.Buffers.Binary.BinaryPrimitives), "ReverseEndianness", typeof(short));
        private static readonly MethodInfo SwapU16 = Method(typeof(System.Buffers.Binary.BinaryPrimitives), "ReverseEndianness", typeof(ushort));
        private static readonly MethodInfo Swap32 = Method(typeof(System.Buffers.Binary.BinaryPrimitives), "ReverseEndianness", typeof(int));
        private static readonly MethodInfo SwapU32 = Method(typeof(System.Buffers.Binary.BinaryPrimitives), "ReverseEndianness", typeof(uint));
        private static readonly MethodInfo Swap64 = Method(typeof(System.Buffers.Binary.BinaryPrimitives), "ReverseEndianness", typeof(long));

        // Off only to measure what the inlined jumps and likely branches save, or to show they change nothing - see Mars_Recompiler.md §18.
        public static bool InlineJumps = Environment.GetEnvironmentVariable("EMUSEN_MARS_NOINLINEJUMPS") != "1";

        // Off only to measure what unchecked access to the registers and memory saves in bytes and time - see Mars_Recompiler.md §19.
        public static bool Unchecked = Environment.GetEnvironmentVariable("EMUSEN_MARS_CHECKEDBLOCKS") != "1";

        // Off only to measure what the inlined loads save, or to show they change nothing - see Mars_Recompiler.md §16.
        public static bool InlineLoads = Environment.GetEnvironmentVariable("EMUSEN_MARS_NOINLINELOADS") != "1";

        private static readonly FieldInfo BusCycles = Field(typeof(MemoryBus), "Cycles");
        private static readonly FieldInfo BusSp = Field(typeof(MemoryBus), "Sp");
        private static readonly FieldInfo BusWritten = Field(typeof(MemoryBus), "Written");
        private static readonly MethodInfo BusNextEvent = Getter(typeof(MemoryBus), "NextEvent");
        private static readonly FieldInfo SpProcessor = Field(typeof(SpInterface), "Processor");
        private static readonly FieldInfo RspHalted = Field(typeof(Rsp.Rsp), "Halted");
        private static readonly MethodInfo MathMin = Method(typeof(Math), "Min", typeof(long), typeof(long));

        private static readonly MethodInfo Execute = CpuMethod("Execute", typeof(uint));
        private static readonly MethodInfo Store = CpuMethod("Store", typeof(uint), typeof(int));
        private static readonly MethodInfo StoreConditional = CpuMethod("StoreConditional", typeof(uint), typeof(int));
        private static readonly MethodInfo StoreWordLeft = CpuMethod("StoreWordLeft", typeof(uint));
        private static readonly MethodInfo StoreWordRight = CpuMethod("StoreWordRight", typeof(uint));
        private static readonly MethodInfo StoreDoubleLeft = CpuMethod("StoreDoubleLeft", typeof(uint));
        private static readonly MethodInfo StoreDoubleRight = CpuMethod("StoreDoubleRight", typeof(uint));
        private static readonly MethodInfo StoreCop1 = CpuMethod("StoreCop1", typeof(uint), typeof(bool));
        private static readonly MethodInfo AfterInstruction = CpuMethod("AfterInstruction");
        private static readonly MethodInfo ReturnedAfterSlot = CpuMethod("ReturnedAfterSlot");
        private static readonly MethodInfo RspRan = CpuMethod("RspRan", typeof(long), typeof(long));
        private static readonly MethodInfo VerifyBlockStep = CpuMethod("VerifyBlockStep");

        private static readonly Channel<(Block Block, BlockCache Cache, int RdramLength)> Queue = Channel.CreateUnbounded<(Block, BlockCache, int)>(new UnboundedChannelOptions { SingleReader = true });

        private static readonly Thread Worker = Start();

        // Closed over this, so a call through the delegate needs no argument-shuffling thunk - see Mars_Recompiler.md §20.
        private static readonly object Bound = new();

        // Compiled from the image the block already holds, and the JIT forced here rather than on the first call - see Mars_Recompiler.md §2.4.
        public static void Compile(Block block, BlockCache cache, int rdramLength)
        {
            long began = Stopwatch.GetTimestamp();

            var method = new DynamicMethod("mars_block_" + block.Physical.ToString("X8"), typeof(void), new[] { typeof(object), CpuType }, CpuType.Module, skipVisibility: true);
            new Emitter(method.GetILGenerator(), block, rdramLength).Emit();

            var code = (BlockCode)method.CreateDelegate(typeof(BlockCode), Bound);
            RuntimeHelpers.PrepareDelegate(code);

            cache.NoteCompiled(Stopwatch.GetTimestamp() - began);
            Volatile.Write(ref block.Code, code);
        }

        public static void Enqueue(Block block, BlockCache cache, int rdramLength) => Queue.Writer.TryWrite((block, cache, rdramLength));

        private static Thread Start()
        {
            var thread = new Thread(Drain) { IsBackground = true, Name = "Mars block compiler" };
            thread.Start();
            return thread;
        }

        private static void Drain()
        {
            while (Queue.Reader.WaitToReadAsync().AsTask().GetAwaiter().GetResult())
            {
                while (Queue.Reader.TryRead(out var item)) Compile(item.Block, item.Cache, item.RdramLength);
            }
        }

        private static FieldInfo CpuField(string name) => Field(CpuType, name);

        private static FieldInfo Field(Type type, string name) =>
            type.GetField(name, Any | BindingFlags.Static) ?? throw new MissingFieldException(type.Name, name);

        private static MethodInfo Getter(Type type, string name) =>
            type.GetProperty(name, Any)?.GetMethod ?? throw new MissingMemberException(type.Name, name);

        private static MethodInfo CpuMethod(string name, params Type[] parameters) => Method(CpuType, name, parameters);

        private static MethodInfo Method(Type type, string name, params Type[] parameters) =>
            type.GetMethod(name, Any | BindingFlags.Static, parameters) ?? throw new MissingMethodException(type.Name, name);

        private sealed class Emitter
        {
            private readonly ILGenerator _il;
            private readonly Block _block;
            private readonly int _rdramLength;
            private readonly Decoded[] _decoded;

            private readonly LocalBuilder _bus;
            private readonly LocalBuilder _g;
            private readonly LocalBuilder _sp;
            private readonly LocalBuilder _stop;
            private readonly LocalBuilder _written;
            private readonly LocalBuilder _entry;
            private readonly LocalBuilder _p;
            private readonly LocalBuilder _exitAt;
            private readonly LocalBuilder _rdram;
            private readonly LocalBuilder _registers;
            private readonly LocalBuilder _memory;
            private readonly LocalBuilder _marked;
            private readonly LocalBuilder _marks;
            private readonly LocalBuilder _address;
            private readonly LocalBuilder _physical;
            private readonly Label _head;
            private readonly Label _epilogue;
            private readonly Label[] _stubs;

            public Emitter(ILGenerator il, Block block, int rdramLength)
            {
                _il = il;
                _block = block;
                _rdramLength = rdramLength;
                _decoded = new Decoded[block.Length];

                _bus = il.DeclareLocal(typeof(MemoryBus));
                _g = il.DeclareLocal(typeof(ulong[]));
                _sp = il.DeclareLocal(typeof(Rsp.Rsp));
                _stop = il.DeclareLocal(typeof(long));
                _written = il.DeclareLocal(typeof(long));
                _entry = il.DeclareLocal(typeof(ulong));
                _p = il.DeclareLocal(typeof(uint));
                _exitAt = il.DeclareLocal(typeof(int));
                _rdram = il.DeclareLocal(typeof(byte[]));
                _registers = il.DeclareLocal(typeof(ulong).MakeByRefType());
                _memory = il.DeclareLocal(typeof(byte).MakeByRefType());
                _marked = il.DeclareLocal(typeof(long).MakeByRefType());
                _marks = il.DeclareLocal(typeof(long[]));
                _address = il.DeclareLocal(typeof(long));
                _physical = il.DeclareLocal(typeof(int));
                _head = il.DefineLabel();
                _epilogue = il.DefineLabel();
                _stubs = new Label[block.Length];
                for (int i = 0; i < block.Length; i++) _stubs[i] = il.DefineLabel();
            }

            public void Emit()
            {
                for (int i = 0; i < _block.Length; i++) _decoded[i] = BlockShape.Decode(BlockShape.Word(_block.Image!, 0, i));

                Prologue();
                for (int i = 0; i < _block.Length; i++) Instruction(i);
                Stubs();
                Epilogue();
            }

            private void Prologue()
            {
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Bus); _il.Emit(OpCodes.Stloc, _bus);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Gpr); _il.Emit(OpCodes.Stloc, _g);
                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusSp); _il.Emit(OpCodes.Ldfld, SpProcessor); _il.Emit(OpCodes.Stloc, _sp);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Pc); _il.Emit(OpCodes.Stloc, _entry);
                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusRdram); _il.Emit(OpCodes.Stloc, _rdram);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, DpWriteMarks); _il.Emit(OpCodes.Stloc, _marks);

                // The three arrays by their first elements, so an access whose index is already known to be inside is an address and not a checked one - see Mars_Recompiler.md §19.
                _il.Emit(OpCodes.Ldloc, _g); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Ldelema, typeof(ulong)); _il.Emit(OpCodes.Stloc, _registers);
                _il.Emit(OpCodes.Ldloc, _rdram); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Ldelema, typeof(byte)); _il.Emit(OpCodes.Stloc, _memory);
                _il.Emit(OpCodes.Ldloc, _marks); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Ldelema, typeof(long)); _il.Emit(OpCodes.Stloc, _marked);

                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Call, BusNextEvent);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, TimerDue); _il.Emit(OpCodes.Call, MathMin);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, CapAt); _il.Emit(OpCodes.Call, MathMin);
                _il.Emit(OpCodes.Stloc, _stop);

                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusWritten); _il.Emit(OpCodes.Stloc, _written);
                _il.MarkLabel(_head);
            }

            private void Instruction(int k)
            {
                Decoded d = _decoded[k];
                bool last = k == _block.Length - 1;
                bool slot = k > 0 && _decoded[k - 1].Kind == Kind.Branch;
                Label exit = _stubs[k];

                if (slot)
                {
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, BranchPending); _il.Emit(OpCodes.Stfld, InDelaySlot);
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Stfld, BranchPending);
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, NextPc); _il.Emit(OpCodes.Stfld, Pc);
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Pc); _il.Emit(OpCodes.Ldc_I8, 4L); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Stfld, NextPc);
                }

                if (k > 0 && d.Kind != Kind.Pure) StoreAddress(CurrentPc, k * 4);

                if (d.Kind is Kind.Branch or Kind.Ender)
                {
                    StoreAddress(Pc, k * 4 + 4);
                    StoreAddress(NextPc, k * 4 + 8);
                }

                switch (d.Kind)
                {
                    case Kind.Pure: Pure(d.Word); break;
                    case Kind.Store: StoreOf(d); break;
                    case Kind.Branch when Compares(d.Word): Compare(d.Word, k); break;
                    case Kind.Branch when InlineJumps && Jumps(d.Word): Jump(d.Word, k); break;
                    case Kind.Call when InlineLoads && Loads(d.Word): Load(d.Word); break;
                    case Kind.Call when InlineLoads && (d.Word >> 26) is 0x31 or 0x35: LoadFloat(d.Word); break;
                    case Kind.Call when InlineLoads && MovesFloat(d.Word): MoveFloat(d.Word); break;
                    default:
                        _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)d.Word)); _il.Emit(OpCodes.Call, Execute);
                        break;
                }

                if (d.Kind == Kind.MulDiv)
                {
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Stfld, ExtraCycles);
                }

                Tick(d.Cycles, exit);

                if (slot)
                {
                    Label noReturn = _il.DefineLabel();
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, ReturnAfterSlot); _il.Emit(OpCodes.Brfalse, noReturn);
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, InDelaySlot); _il.Emit(OpCodes.Brfalse, noReturn);
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Call, ReturnedAfterSlot);
                    _il.MarkLabel(noReturn);
                }

                if (d.Kind == Kind.Store) StoreCheck(exit);

                if (d.Kind == Kind.Branch)
                {
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, BranchPending); _il.Emit(OpCodes.Brfalse, exit);
                }

                if (!last)
                {
                    if (Verify) { _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Call, VerifyBlockStep); }
                    _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusCycles); _il.Emit(OpCodes.Ldloc, _stop); _il.Emit(OpCodes.Bge, exit);
                    return;
                }

                if (slot) LoopBack(exit);
                _il.Emit(OpCodes.Br, exit);
            }

            // Every exit of instruction k lands here: the index, and whether the counters still need moving past it - see Mars_Recompiler.md §3.2.
            private void Stubs()
            {
                for (int k = 0; k < _block.Length; k++)
                {
                    bool slot = k > 0 && _decoded[k - 1].Kind == Kind.Branch;
                    bool countersSet = _decoded[k].Kind is Kind.Branch or Kind.Ender || slot;

                    _il.MarkLabel(_stubs[k]);
                    _il.Emit(OpCodes.Ldc_I4, countersSet ? ~k : k); _il.Emit(OpCodes.Stloc, _exitAt);
                    _il.Emit(OpCodes.Br, _epilogue);
                }
            }

            private void Epilogue()
            {
                Label countersSet = _il.DefineLabel();
                Label done = _il.DefineLabel();

                _il.MarkLabel(_epilogue);
                _il.Emit(OpCodes.Ldloc, _exitAt); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Blt, countersSet);

                _il.Emit(OpCodes.Ldarg_1); Address(_exitAt, 4); _il.Emit(OpCodes.Stfld, Pc);
                _il.Emit(OpCodes.Ldarg_1); Address(_exitAt, 8); _il.Emit(OpCodes.Stfld, NextPc);
                _il.Emit(OpCodes.Ldarg_1); Address(_exitAt, 0); _il.Emit(OpCodes.Stfld, CurrentPc);
                _il.Emit(OpCodes.Br, done);

                _il.MarkLabel(countersSet);
                _il.Emit(OpCodes.Ldloc, _exitAt); _il.Emit(OpCodes.Not); _il.Emit(OpCodes.Stloc, _exitAt);
                _il.Emit(OpCodes.Ldarg_1); Address(_exitAt, 0); _il.Emit(OpCodes.Stfld, CurrentPc);

                _il.MarkLabel(done);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Call, AfterInstruction); _il.Emit(OpCodes.Ret);
            }

            // entry + 4 * index + offset, as a 64-bit address.
            private void Address(LocalBuilder index, int offset)
            {
                _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Ldloc, index); _il.Emit(OpCodes.Conv_I8); _il.Emit(OpCodes.Ldc_I4_4); _il.Emit(OpCodes.Conv_I8); _il.Emit(OpCodes.Mul);
                _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Ldc_I8, (long)offset); _il.Emit(OpCodes.Add);
            }

            // A slot that lands on the block's own start goes round inside the method, with what the dispatcher would check at entry - see Mars_Recompiler.md §3.4.
            private void LoopBack(Label exit)
            {
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Pc); _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Bne_Un, exit);
                if (Verify) { _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Call, VerifyBlockStep); }
                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusCycles); _il.Emit(OpCodes.Ldloc, _stop); _il.Emit(OpCodes.Bge, exit);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Stfld, InDelaySlot);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Stfld, CurrentPc);
                _il.Emit(OpCodes.Br, _head);
            }

            // The six plain conditional branches: the compare, the target into NextPc when taken, and the pending flag either way, as the interpreter's BranchIf leaves them - see Mars_Recompiler.md §14.
            private static bool Compares(uint word)
            {
                uint op = word >> 26;
                if (op is 0x04 or 0x05 or 0x06 or 0x07 || (op == 0x01 && ((word >> 16) & 0x1F) is 0 or 1)) return true;
                return InlineJumps && Likely(word);
            }

            private void Compare(uint word, int k)
            {
                // A likely branch compares as its plain twin does: the opcode less its likely bit, and the register-immediate pair two on.
                uint op = (word >> 26) & ~0x10u;
                int rs = (int)((word >> 21) & 0x1F), rt = (int)((word >> 16) & 0x1F);
                if (op == 0x01) rt &= 1;
                Label notTaken = _il.DefineLabel();

                LdReg(rs);
                if (op is 0x04 or 0x05)
                {
                    LdReg(rt);
                    _il.Emit(op == 0x04 ? OpCodes.Bne_Un : OpCodes.Beq, notTaken);
                }
                else
                {
                    _il.Emit(OpCodes.Ldc_I8, 0L);
                    _il.Emit(op switch { 0x06 => OpCodes.Bgt, 0x07 => OpCodes.Ble, _ => rt == 0 ? OpCodes.Bge : OpCodes.Blt }, notTaken);
                }

                StoreAddress(NextPc, k * 4 + 4 + ((short)word << 2));

                // A likely branch not taken annuls its slot: the counters go past it and nothing is pending, as NullifyDelaySlot leaves them - see §18.
                if (Likely(word))
                {
                    Label done = _il.DefineLabel();
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_1); _il.Emit(OpCodes.Stfld, BranchPending);
                    _il.Emit(OpCodes.Br, done);
                    _il.MarkLabel(notTaken);
                    StoreAddress(Pc, k * 4 + 8);
                    StoreAddress(NextPc, k * 4 + 12);
                    _il.MarkLabel(done);
                    return;
                }

                _il.MarkLabel(notTaken);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_1); _il.Emit(OpCodes.Stfld, BranchPending);
            }

            private static bool Likely(uint word) => (word >> 26) is 0x14 or 0x15 or 0x16 or 0x17 || ((word >> 26) == 0x01 && ((word >> 16) & 0x1F) is 2 or 3);

            // The jumps and the register jumps, with and without a link - see §18.
            private static bool Jumps(uint word) => (word >> 26) is 0x02 or 0x03 || ((word >> 26) == 0 && (word & 0x3F) is 0x08 or 0x09);

            // What Branch leaves: the target next and a branch pending; a link first, and the interpreter's call instead while a debugger's call stack is listening - see §18.
            private void Jump(uint word, int k)
            {
                uint op = word >> 26;
                int rs = (int)((word >> 21) & 0x1F), rd = (int)((word >> 11) & 0x1F);
                bool register = op == 0, link = op == 0x03 || (register && (word & 0x3F) == 0x09);
                Label slow = _il.DefineLabel(), done = _il.DefineLabel();

                if (link || (register && rs == 31))
                {
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, link ? CallObserver : ReturnObserver); _il.Emit(OpCodes.Brtrue, slow);
                }

                if (register)
                {
                    // The target is read before the link is written, since they may be one register.
                    LdReg(rs); _il.Emit(OpCodes.Stloc, _address);
                    if (link) StReg(rd, () => { _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Ldc_I8, (long)(k * 4 + 8)); _il.Emit(OpCodes.Add); });
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldloc, _address); _il.Emit(OpCodes.Stfld, NextPc);
                }
                else
                {
                    if (link) StReg(31, () => { _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Ldc_I8, (long)(k * 4 + 8)); _il.Emit(OpCodes.Add); });
                    _il.Emit(OpCodes.Ldarg_1);
                    _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Ldc_I8, (long)(k * 4 + 4)); _il.Emit(OpCodes.Add);
                    _il.Emit(OpCodes.Ldc_I8, unchecked((long)0xFFFF_FFFF_F000_0000)); _il.Emit(OpCodes.And);
                    _il.Emit(OpCodes.Ldc_I8, (long)((word & 0x03FF_FFFF) << 2)); _il.Emit(OpCodes.Or);
                    _il.Emit(OpCodes.Stfld, NextPc);
                }

                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4_1); _il.Emit(OpCodes.Stfld, BranchPending);
                _il.Emit(OpCodes.Br, done);

                _il.MarkLabel(slow);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)word)); _il.Emit(OpCodes.Call, Execute);
                _il.MarkLabel(done);
            }

            private void StoreAddress(FieldInfo field, int offset)
            {
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldloc, _entry); _il.Emit(OpCodes.Ldc_I8, (long)offset); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Stfld, field);
            }

            private void Tick(int cycles, Label exit)
            {
                Label halted = _il.DefineLabel();

                _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldloc, _bus); _il.Emit(OpCodes.Ldfld, BusCycles); _il.Emit(OpCodes.Ldc_I8, (long)cycles); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Stfld, BusCycles);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Instructions); _il.Emit(OpCodes.Ldc_I8, 1L); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Stfld, Instructions);

                _il.Emit(OpCodes.Ldloc, _sp); _il.Emit(OpCodes.Ldfld, RspHalted); _il.Emit(OpCodes.Brtrue, halted);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I8, (long)cycles); _il.Emit(OpCodes.Ldloc, _written); _il.Emit(OpCodes.Call, RspRan); _il.Emit(OpCodes.Brtrue, exit);
                _il.MarkLabel(halted);
            }

            private void StoreOf(Decoded d)
            {
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)d.Word));

                switch (d.Store)
                {
                    case StoreKind.Aligned: _il.Emit(OpCodes.Ldc_I4, d.Size); _il.Emit(OpCodes.Call, Store); break;
                    case StoreKind.Conditional: _il.Emit(OpCodes.Ldc_I4, d.Size); _il.Emit(OpCodes.Call, StoreConditional); break;
                    case StoreKind.WordLeft: _il.Emit(OpCodes.Call, StoreWordLeft); break;
                    case StoreKind.WordRight: _il.Emit(OpCodes.Call, StoreWordRight); break;
                    case StoreKind.DoubleLeft: _il.Emit(OpCodes.Call, StoreDoubleLeft); break;
                    case StoreKind.DoubleRight: _il.Emit(OpCodes.Call, StoreDoubleRight); break;
                    case StoreKind.Cop1Word: _il.Emit(OpCodes.Ldc_I4_0); _il.Emit(OpCodes.Call, StoreCop1); break;
                    default: _il.Emit(OpCodes.Ldc_I4_1); _il.Emit(OpCodes.Call, StoreCop1); break;
                }

                _il.Emit(OpCodes.Stloc, _p);
            }

            // A store beyond memory went through the bus; one within eight bytes of the block may have rewritten it - see Mars_Recompiler.md §4.
            private void StoreCheck(Label exit)
            {
                Label noHit = _il.DefineLabel();
                int start = (int)_block.Physical;
                int end = start + _block.Length * 4;

                _il.Emit(OpCodes.Ldloc, _p); _il.Emit(OpCodes.Ldc_I4, _rdramLength); _il.Emit(OpCodes.Bge_Un, exit);
                _il.Emit(OpCodes.Ldloc, _p); _il.Emit(OpCodes.Ldc_I4_8); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Ldc_I4, start); _il.Emit(OpCodes.Ble_Un, noHit);
                _il.Emit(OpCodes.Ldloc, _p); _il.Emit(OpCodes.Ldc_I4, end); _il.Emit(OpCodes.Bge_Un, noHit);
                _il.Emit(OpCodes.Br, exit);
                _il.MarkLabel(noHit);
            }

            // The aligned integer loads into a register other than zero - see Mars_Recompiler.md §16.
            private static bool Loads(uint word) => (word >> 26) is 0x20 or 0x21 or 0x23 or 0x24 or 0x25 or 0x27 or 0x37 && ((word >> 16) & 0x1F) != 0;

            // The interpreter's own fast case, tested in its order: a direct kernel address, aligned, inside RDRAM, on a page the display processor is not writing; anything else is the interpreter's - see §16.
            private void Load(uint word)
            {
                uint op = word >> 26;
                int rs = (int)((word >> 21) & 0x1F), rt = (int)((word >> 16) & 0x1F);
                int size = op switch { 0x20 or 0x24 => 1, 0x21 or 0x25 => 2, 0x37 => 8, _ => 4 };
                Label slow = _il.DefineLabel(), done = _il.DefineLabel();

                DirectAddress(rs, (short)word, size, slow);

                if (Unchecked) RegisterAddress(rt);
                else { _il.Emit(OpCodes.Ldloc, _g); _il.Emit(OpCodes.Ldc_I4, rt); }

                switch (op)
                {
                    case 0x20: MemoryAddress(); _il.Emit(OpCodes.Ldind_I1); _il.Emit(OpCodes.Conv_I8); break;
                    case 0x24: MemoryAddress(); _il.Emit(OpCodes.Ldind_U1); _il.Emit(OpCodes.Conv_U8); break;
                    case 0x21: Unaligned(OpCodes.Ldind_I2); _il.Emit(OpCodes.Call, Swap16); _il.Emit(OpCodes.Conv_I8); break;
                    case 0x25: Unaligned(OpCodes.Ldind_U2); _il.Emit(OpCodes.Call, SwapU16); _il.Emit(OpCodes.Conv_U8); break;
                    case 0x23: Unaligned(OpCodes.Ldind_I4); _il.Emit(OpCodes.Call, Swap32); _il.Emit(OpCodes.Conv_I8); break;
                    case 0x27: Unaligned(OpCodes.Ldind_U4); _il.Emit(OpCodes.Call, SwapU32); _il.Emit(OpCodes.Conv_U8); break;
                    default: Unaligned(OpCodes.Ldind_I8); _il.Emit(OpCodes.Call, Swap64); break;
                }

                _il.Emit(Unchecked ? OpCodes.Stind_I8 : OpCodes.Stelem_I8);
                _il.Emit(OpCodes.Br, done);

                _il.MarkLabel(slow);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)word)); _il.Emit(OpCodes.Call, Execute);
                _il.MarkLabel(done);
            }

            // Leaves the physical address in its local, or goes to the interpreter's call: not a direct kernel address, not aligned, not RDRAM, or a page being drawn to - see §16.
            private void DirectAddress(int rs, short offset, int size, Label slow)
            {
                LdReg(rs); _il.Emit(OpCodes.Ldc_I8, (long)offset); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Stloc, _address);
                _il.Emit(OpCodes.Ldloc, _address); _il.Emit(OpCodes.Ldc_I8, unchecked((long)0xFFFF_FFFF_8000_0000)); _il.Emit(OpCodes.Sub); _il.Emit(OpCodes.Ldc_I8, 0x4000_0000L); _il.Emit(OpCodes.Bge_Un, slow);

                if (size > 1)
                {
                    _il.Emit(OpCodes.Ldloc, _address); _il.Emit(OpCodes.Conv_I4); _il.Emit(OpCodes.Ldc_I4, size - 1); _il.Emit(OpCodes.And); _il.Emit(OpCodes.Brtrue, slow);
                }

                _il.Emit(OpCodes.Ldloc, _address); _il.Emit(OpCodes.Conv_I4); _il.Emit(OpCodes.Ldc_I4, 0x1FFF_FFFF); _il.Emit(OpCodes.And); _il.Emit(OpCodes.Stloc, _physical);
                _il.Emit(OpCodes.Ldloc, _physical); _il.Emit(OpCodes.Ldc_I4, _rdramLength); _il.Emit(OpCodes.Bge_Un, slow);
                if (Unchecked)
                {
                    // The physical address has just been found below RDRAM's length, which the marks cover page for page.
                    _il.Emit(OpCodes.Ldloc, _marked); _il.Emit(OpCodes.Ldloc, _physical); _il.Emit(OpCodes.Ldc_I4, 12); _il.Emit(OpCodes.Shr_Un); _il.Emit(OpCodes.Ldc_I4_3); _il.Emit(OpCodes.Shl); _il.Emit(OpCodes.Add); _il.Emit(OpCodes.Ldind_I8); _il.Emit(OpCodes.Brtrue, slow);
                }
                else
                {
                    _il.Emit(OpCodes.Ldloc, _marks); _il.Emit(OpCodes.Ldloc, _physical); _il.Emit(OpCodes.Ldc_I4, 12); _il.Emit(OpCodes.Shr_Un); _il.Emit(OpCodes.Ldelem_I8); _il.Emit(OpCodes.Brtrue, slow);
                }
            }

            // A load into the coprocessor's register, usable or the interpreter raises what it raises - see §16.
            private void LoadFloat(uint word)
            {
                bool wide = (word >> 26) == 0x35;
                Label slow = _il.DefineLabel(), done = _il.DefineLabel();

                Usable(slow);
                DirectAddress((int)((word >> 21) & 0x1F), (short)word, wide ? 8 : 4, slow);

                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, (int)((word >> 16) & 0x1F));
                if (wide) { Unaligned(OpCodes.Ldind_I8); _il.Emit(OpCodes.Call, Swap64); _il.Emit(OpCodes.Call, WriteFpuWide); }
                else { Unaligned(OpCodes.Ldind_U4); _il.Emit(OpCodes.Call, SwapU32); _il.Emit(OpCodes.Call, WriteFpuWord); }
                _il.Emit(OpCodes.Br, done);

                _il.MarkLabel(slow);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)word)); _il.Emit(OpCodes.Call, Execute);
                _il.MarkLabel(done);
            }

            // A word moved to the coprocessor, or from it into a register other than zero - see §16.
            private static bool MovesFloat(uint word) =>
                (word >> 26) == 0x11 && (((word >> 21) & 0x1F) == 0x04 || (((word >> 21) & 0x1F) == 0x00 && ((word >> 16) & 0x1F) != 0));

            private void MoveFloat(uint word)
            {
                int rt = (int)((word >> 16) & 0x1F), fs = (int)((word >> 11) & 0x1F);
                Label slow = _il.DefineLabel(), done = _il.DefineLabel();

                Usable(slow);

                if (((word >> 21) & 0x1F) == 0x04)
                {
                    _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, fs); LdReg(rt); _il.Emit(OpCodes.Conv_U4); _il.Emit(OpCodes.Call, WriteFpuWord);
                }
                else
                {
                    StReg(rt, () => { _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, fs); _il.Emit(OpCodes.Call, ReadFpuWord); SignExtend32(); });
                }

                _il.Emit(OpCodes.Br, done);
                _il.MarkLabel(slow);
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldc_I4, unchecked((int)word)); _il.Emit(OpCodes.Call, Execute);
                _il.MarkLabel(done);
            }

            private void Usable(Label slow)
            {
                _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Cop0); _il.Emit(OpCodes.Ldc_I4, Core.Cpu.StatusRegister); _il.Emit(OpCodes.Ldelem_I8);
                _il.Emit(OpCodes.Ldc_I8, (long)Core.Cpu.StatusCop1Usable); _il.Emit(OpCodes.And); _il.Emit(OpCodes.Brfalse, slow);
            }

            // The element's address, read through whatever the host's alignment is.
            private void Unaligned(OpCode read)
            {
                MemoryAddress(); _il.Emit(OpCodes.Unaligned, (byte)1); _il.Emit(read);
            }

            // The byte at the physical address the tests above have just found inside RDRAM - see §19.
            private void MemoryAddress()
            {
                if (Unchecked) { _il.Emit(OpCodes.Ldloc, _memory); _il.Emit(OpCodes.Ldloc, _physical); _il.Emit(OpCodes.Add); }
                else { _il.Emit(OpCodes.Ldloc, _rdram); _il.Emit(OpCodes.Ldloc, _physical); _il.Emit(OpCodes.Ldelema, typeof(byte)); }
            }

            private void LdReg(int r)
            {
                if (r == 0) { _il.Emit(OpCodes.Ldc_I8, 0L); return; }
                if (Unchecked) { RegisterAddress(r); _il.Emit(OpCodes.Ldind_I8); return; }
                _il.Emit(OpCodes.Ldloc, _g); _il.Emit(OpCodes.Ldc_I4, r); _il.Emit(OpCodes.Ldelem_I8);
            }

            private void StReg(int r, Action value)
            {
                if (r == 0) return;
                if (Unchecked) { RegisterAddress(r); value(); _il.Emit(OpCodes.Stind_I8); return; }
                _il.Emit(OpCodes.Ldloc, _g); _il.Emit(OpCodes.Ldc_I4, r); value(); _il.Emit(OpCodes.Stelem_I8);
            }

            // A register is five bits of an instruction, so its address needs no check - see §19.
            private void RegisterAddress(int r)
            {
                _il.Emit(OpCodes.Ldloc, _registers);
                if (r != 0) { _il.Emit(OpCodes.Ldc_I4, r * 8); _il.Emit(OpCodes.Add); }
            }

            private void SignExtend32()
            {
                _il.Emit(OpCodes.Conv_I4); _il.Emit(OpCodes.Conv_I8);
            }

            private void Pure(uint word)
            {
                uint op = word >> 26;
                int rs = (int)((word >> 21) & 0x1F);
                int rt = (int)((word >> 16) & 0x1F);
                int rd = (int)((word >> 11) & 0x1F);
                int sa = (int)((word >> 6) & 0x1F);
                long simm = (short)word;
                long imm = word & 0xFFFF;

                switch (op)
                {
                    case 0x00: Special(word, rs, rt, rd, sa); return;
                    case 0x09: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, simm); _il.Emit(OpCodes.Add); SignExtend32(); }); return;
                    case 0x0A: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, simm); _il.Emit(OpCodes.Clt); _il.Emit(OpCodes.Conv_I8); }); return;
                    case 0x0B: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, simm); _il.Emit(OpCodes.Clt_Un); _il.Emit(OpCodes.Conv_I8); }); return;
                    case 0x0C: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, imm); _il.Emit(OpCodes.And); }); return;
                    case 0x0D: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, imm); _il.Emit(OpCodes.Or); }); return;
                    case 0x0E: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, imm); _il.Emit(OpCodes.Xor); }); return;
                    case 0x0F: StReg(rt, () => _il.Emit(OpCodes.Ldc_I8, (long)(int)(imm << 16))); return;
                    case 0x19: StReg(rt, () => { LdReg(rs); _il.Emit(OpCodes.Ldc_I8, simm); _il.Emit(OpCodes.Add); }); return;
                    default: throw new InvalidOperationException($"{word:X8} is not an instruction the compiler inlines.");
                }
            }

            private void Special(uint word, int rs, int rt, int rd, int sa)
            {
                switch (word & 0x3F)
                {
                    case 0x00: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Conv_U4); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shl); SignExtend32(); }); return;
                    case 0x02: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Conv_U4); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shr_Un); SignExtend32(); }); return;
                    case 0x03: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shr); SignExtend32(); }); return;
                    case 0x04: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Conv_U4); Amount(rs, 0x1F); _il.Emit(OpCodes.Shl); SignExtend32(); }); return;
                    case 0x06: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Conv_U4); Amount(rs, 0x1F); _il.Emit(OpCodes.Shr_Un); SignExtend32(); }); return;
                    case 0x07: StReg(rd, () => { LdReg(rt); Amount(rs, 0x1F); _il.Emit(OpCodes.Shr); SignExtend32(); }); return;
                    case 0x0F: return;
                    case 0x10: StReg(rd, () => { _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Hi); }); return;
                    case 0x11: _il.Emit(OpCodes.Ldarg_1); LdReg(rs); _il.Emit(OpCodes.Stfld, Hi); return;
                    case 0x12: StReg(rd, () => { _il.Emit(OpCodes.Ldarg_1); _il.Emit(OpCodes.Ldfld, Lo); }); return;
                    case 0x13: _il.Emit(OpCodes.Ldarg_1); LdReg(rs); _il.Emit(OpCodes.Stfld, Lo); return;
                    case 0x14: StReg(rd, () => { LdReg(rt); Amount(rs, 0x3F); _il.Emit(OpCodes.Shl); }); return;
                    case 0x16: StReg(rd, () => { LdReg(rt); Amount(rs, 0x3F); _il.Emit(OpCodes.Shr_Un); }); return;
                    case 0x17: StReg(rd, () => { LdReg(rt); Amount(rs, 0x3F); _il.Emit(OpCodes.Shr); }); return;
                    case 0x21: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Add); SignExtend32(); }); return;
                    case 0x23: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Sub); SignExtend32(); }); return;
                    case 0x24: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.And); }); return;
                    case 0x25: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Or); }); return;
                    case 0x26: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Xor); }); return;
                    case 0x27: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Or); _il.Emit(OpCodes.Not); }); return;
                    case 0x2A: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Clt); _il.Emit(OpCodes.Conv_I8); }); return;
                    case 0x2B: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Clt_Un); _il.Emit(OpCodes.Conv_I8); }); return;
                    case 0x2D: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Add); }); return;
                    case 0x2F: StReg(rd, () => { LdReg(rs); LdReg(rt); _il.Emit(OpCodes.Sub); }); return;
                    case 0x38: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shl); }); return;
                    case 0x3A: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shr_Un); }); return;
                    case 0x3B: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa); _il.Emit(OpCodes.Shr); }); return;
                    case 0x3C: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa + 32); _il.Emit(OpCodes.Shl); }); return;
                    case 0x3E: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa + 32); _il.Emit(OpCodes.Shr_Un); }); return;
                    case 0x3F: StReg(rd, () => { LdReg(rt); _il.Emit(OpCodes.Ldc_I4, sa + 32); _il.Emit(OpCodes.Shr); }); return;
                    default: throw new InvalidOperationException($"{word:X8} is not an instruction the compiler inlines.");
                }
            }

            private void Amount(int rs, int mask)
            {
                LdReg(rs); _il.Emit(OpCodes.Conv_I4); _il.Emit(OpCodes.Ldc_I4, mask); _il.Emit(OpCodes.And);
            }
        }
    }
}
