using System;
using System.Reflection;
using System.Reflection.Emit;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // Straight lines of the signal processor's code compiled to calls of its own handlers, run whole while the CPU idles - see Mars_Rsp.md §11.
    public sealed partial class Rsp
    {
        private const int LongestBlock = 32;
        private const int Variants = 4;
        private const int CompileAfter = 3;

        // Off only to measure what the blocks save, or to show they change nothing - see Mars_Rsp.md §11.
        public static bool UseBlocks = Environment.GetEnvironmentVariable("EMUSEN_MARS_NORSPBLOCKS") != "1";

        // Off in tests, so which steps ran compiled does not depend on a thread - see §11.
        public static bool CompileBlocksInBackground = true;

        private sealed class CodeBlock
        {
            public byte[] Image = Array.Empty<byte>();
            public int Length;
            public bool Event;
            public int Runs;
            public bool Queued;
            public Action<Rsp>? Code;
        }

        // Several per address, because the graphics and the sound microcode take turns at the same addresses - see §11.
        [EmuSen.Common.SkipInState] private readonly CodeBlock?[] _codeBlocks = new CodeBlock?[0x400 * Variants];
        [EmuSen.Common.SkipInState] public long BlockSteps, BlocksCompiled;

        // Whole blocks only, at most the steps given, stopping after a block that ends in a move to or from the control registers or a break; returns the steps run - see §11.
        public long RunBlocks(long budget)
        {
            long ran = 0;

            while (!Halted && NextPc == ((Pc + 4) & PcMask))
            {
                CodeBlock? block = Find(Pc);
                if (block?.Code is not { } code || block.Length > budget - ran) break;

                code(this);
                ran += block.Length;
                if (block.Event) break;
            }

            BlockSteps += ran;
            return ran;
        }

        // The block whose words are the ones in memory now; a new shape takes the slot of the least run - see §11.
        private CodeBlock? Find(uint pc)
        {
            int slot = (int)(pc >> 2) * Variants;
            int coldest = slot;

            for (int i = slot; i < slot + Variants; i++)
            {
                CodeBlock? candidate = _codeBlocks[i];
                if (candidate is null) { coldest = i; break; }

                if (_imem.AsSpan((int)pc, candidate.Image.Length).SequenceEqual(candidate.Image))
                {
                    if (candidate.Code is null && !candidate.Queued && ++candidate.Runs >= CompileAfter) Queue(candidate, pc);
                    return candidate;
                }

                if (candidate.Runs < _codeBlocks[coldest]!.Runs) coldest = i;
            }

            CodeBlock? shaped = Shape(pc);
            if (shaped is not null) _codeBlocks[coldest] = shaped;
            return shaped;
        }

        // Up to a branch and its slot, or a control-register move or a break, which end it as an event; a branch whose slot is one of those is left to the interpreter - see §11.
        private CodeBlock? Shape(uint pc)
        {
            int limit = Math.Min(LongestBlock, (int)((0x1000 - pc) >> 2));
            int length = 0;
            bool ends = false;

            while (length < limit)
            {
                uint word = ReadInstruction(pc + (uint)length * 4);

                if (IsEvent(word)) { length++; ends = true; break; }

                if (IsBranch(word))
                {
                    if (length + 1 >= limit) break;
                    uint slot = ReadInstruction(pc + (uint)length * 4 + 4);
                    if (IsEvent(slot) || IsBranch(slot)) break;
                    length += 2;
                    break;
                }

                length++;
            }

            if (length == 0) return null;
            return new CodeBlock { Image = _imem.AsSpan((int)pc, length * 4).ToArray(), Length = length, Event = ends };
        }

        private static bool IsEvent(uint word) => (word >> 26) == 0x10 || ((word >> 26) == 0 && (word & 0x3F) == 0x0D);

        private static bool IsBranch(uint word)
        {
            uint op = word >> 26;
            return op is 0x01 or 0x02 or 0x03 or 0x04 or 0x05 or 0x06 or 0x07 || (op == 0 && (word & 0x3F) is 0x08 or 0x09);
        }

        // Compiled away from the machine's thread; the block runs interpreted until its code is published - see §11.
        private void Queue(CodeBlock block, uint pc)
        {
            block.Queued = true;
            if (CompileBlocksInBackground) Task.Run(() => Publish(block, pc));
            else Publish(block, pc);
        }

        private void Publish(CodeBlock block, uint pc)
        {
            Action<Rsp> code = Compile(block, pc);
            RuntimeHelpers.PrepareDelegate(code);
            System.Threading.Interlocked.Increment(ref BlocksCompiled);
            block.Code = code;
        }

        private static readonly FieldInfo PcField = typeof(Rsp).GetField(nameof(Pc))!;
        private static readonly FieldInfo NextPcField = typeof(Rsp).GetField(nameof(NextPc))!;

        private static MethodInfo Handler(string name) => typeof(Rsp).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, new[] { typeof(uint) })!;

        private static readonly MethodInfo ExecuteAny = Handler(nameof(Execute)), Special = Handler(nameof(ExecuteSpecial)), RegImm = Handler(nameof(ExecuteRegImm)),
            Cop0 = Handler(nameof(ExecuteCop0)), Cop2 = Handler(nameof(ExecuteCop2)), VectorLoad = Handler(nameof(ExecuteVectorLoad)), VectorStore = Handler(nameof(ExecuteVectorStore));

        // Each instruction as the step runs it: the counters moved past it, then its handler called with its word - see §11.
        private static Action<Rsp> Compile(CodeBlock block, uint pc)
        {
            var method = new DynamicMethod($"mars_rsp_{pc:X3}", typeof(void), new[] { typeof(Rsp) }, typeof(Rsp), skipVisibility: true);
            ILGenerator il = method.GetILGenerator();
            bool slot = false;

            for (int i = 0; i < block.Length; i++)
            {
                uint at = pc + (uint)i * 4;
                uint word = System.Buffers.Binary.BinaryPrimitives.ReadUInt32BigEndian(block.Image.AsSpan(i * 4));

                if (slot)
                {
                    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldfld, NextPcField); il.Emit(OpCodes.Stfld, PcField);
                    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldfld, PcField); il.Emit(OpCodes.Ldc_I4_4); il.Emit(OpCodes.Add); il.Emit(OpCodes.Ldc_I4, (int)PcMask); il.Emit(OpCodes.And); il.Emit(OpCodes.Stfld, NextPcField);
                }
                else
                {
                    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldc_I4, (int)((at + 4) & PcMask)); il.Emit(OpCodes.Stfld, PcField);
                    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldc_I4, (int)((at + 8) & PcMask)); il.Emit(OpCodes.Stfld, NextPcField);
                }

                MethodInfo handler = (word >> 26) switch
                {
                    0x00 => Special,
                    0x01 => RegImm,
                    0x10 => Cop0,
                    0x12 => Cop2,
                    0x32 => VectorLoad,
                    0x3A => VectorStore,
                    _ => ExecuteAny,
                };

                il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldc_I4, unchecked((int)word)); il.Emit(OpCodes.Call, handler);
                slot = IsBranch(word);
            }

            il.Emit(OpCodes.Ret);
            return method.CreateDelegate<Action<Rsp>>();
        }
    }
}
