using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using EmuSen.Cores.Nintendo.Mars.Native;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The signal processor's Rust twin: pure instructions run there, and every event runs here, in the C# step - see Mars_Native.md §3.
    public sealed unsafe partial class Rsp
    {
        public const string NativeVariable = "EMUSEN_MARS_NATIVERSP";

        // Opt-in until exactness is proven across the corpus and the games - see Mars_Native.md §3.3.
        public static bool UseNative = Environment.GetEnvironmentVariable(NativeVariable) is "1" or "step" && MarsNative.Available;

        // Measurement: the idle loop keeps the C# compiled blocks, and only the stepped paths go native - see Mars_Native.md §3.4.
        public static bool NativeBlocks = Environment.GetEnvironmentVariable(NativeVariable) != "step";

        [StructLayout(LayoutKind.Sequential)]
        private struct Scalars
        {
            public uint Pc, NextPc;
            public ushort Vco, Vcc, DivideInput, DivideOutput;
            public byte Vce, Halted, Broke, DivideLoaded;
        }

        private static readonly delegate* unmanaged[SuppressGCTransition]<nint, nint, nint, nint, nint, nint> New =
            (delegate* unmanaged[SuppressGCTransition]<nint, nint, nint, nint, nint, nint>)MarsNative.Export("mars_rsp_new");
        private static readonly delegate* unmanaged[SuppressGCTransition]<nint, Scalars*> ScalarsOf =
            (delegate* unmanaged[SuppressGCTransition]<nint, Scalars*>)MarsNative.Export("mars_rsp_scalars");
        private static readonly delegate* unmanaged[SuppressGCTransition]<nint, uint> NativeStep =
            (delegate* unmanaged[SuppressGCTransition]<nint, uint>)MarsNative.Export("mars_rsp_step");
        private static readonly delegate* unmanaged[SuppressGCTransition]<nint, ulong, ulong> NativeRunFor =
            (delegate* unmanaged[SuppressGCTransition]<nint, ulong, ulong>)MarsNative.Export("mars_rsp_run");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_rsp_free");

        // Freed by its finalizer, since a machine has no dispose of its own.
        private sealed class Handle(nint value) : System.Runtime.ConstrainedExecution.CriticalFinalizerObject
        {
            public readonly nint Value = value;
            ~Handle() { if (Free != null) Free(Value); }
        }

        // Measurement: lock-step calls, and those that met an event and ran it in C#.
        public static long NativeStepCalls, NativeStepEvents;

        [EmuSen.Common.SkipInState] private Handle? _native;
        [EmuSen.Common.SkipInState] private Scalars* _scalars;

        private static nint Address<T>(T[] pinned) where T : unmanaged => (nint)Unsafe.AsPointer(ref MemoryMarshal.GetArrayDataReference(pinned));

        // Zero when the library lacks the processor or the debugger's coverage is armed, so the C# step runs instead.
        private nint NativeHandle()
        {
            if (Coverage is { IsArmed: true }) return 0;
            if (_native is { } handle) return handle.Value;
            if (New == null || ScalarsOf == null || NativeStep == null || NativeRunFor == null) return 0;

            nint created = New(Address(Gpr), Address(Vector), Address(Accumulator), Address(_imem), Address(_bus.SpDmem));
            _native = new Handle(created);
            _scalars = ScalarsOf(created);
            return created;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void Push()
        {
            WidenAccumulator();
            Scalars* s = _scalars;
            s->Pc = Pc; s->NextPc = NextPc; s->Vco = Vco; s->Vcc = Vcc; s->Vce = Vce;
            s->DivideInput = _divideInput; s->DivideOutput = _divideOutput; s->DivideLoaded = _divideInputLoaded ? (byte)1 : (byte)0;
            s->Halted = Halted ? (byte)1 : (byte)0; s->Broke = Broke ? (byte)1 : (byte)0;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private void Pull()
        {
            Scalars* s = _scalars;
            Pc = s->Pc; NextPc = s->NextPc; Vco = s->Vco; Vcc = s->Vcc; Vce = s->Vce;
            _divideInput = s->DivideInput; _divideOutput = s->DivideOutput; _divideInputLoaded = s->DivideLoaded != 0;
            Halted = s->Halted != 0; Broke = s->Broke != 0;
        }

        // One step; false when the native side is not in use and the caller must step in C#.
        private bool NativeStepOne()
        {
            nint handle = NativeHandle();
            if (handle == 0) return false;

            Push();
            uint ran = NativeStep(handle);
            Pull();
            NativeStepCalls++;
            if (ran == 0) { NativeStepEvents++; StepManaged(); }
            return true;
        }

        // SpInterface.Step's loop: at least one step, then on while cycles remain and nothing halted it - see Mars_Native.md §3.2.
        internal bool NativeRun(long cycles)
        {
            nint handle = NativeHandle();
            if (handle == 0) return false;

            long left = Math.Max(cycles, 1);
            while (left > 0 && !Halted)
            {
                Push();
                long ran = (long)NativeRunFor(handle, (ulong)left);
                Pull();
                left -= ran;
                if (left > 0 && !Halted)
                {
                    StepManaged();
                    left--;
                }
            }
            return true;
        }

        // RunBlocks's contract: at most the budget, stopping after an event; -1 when the native side is not in use.
        private long NativeRunToEvent(long budget)
        {
            if (Halted) return 0;
            nint handle = NativeHandle();
            if (handle == 0 || budget <= 0) return handle == 0 ? -1 : 0;

            Push();
            long ran = (long)NativeRunFor(handle, (ulong)budget);
            Pull();
            if (ran < budget && !Halted)
            {
                StepManaged();
                ran++;
            }
            return ran;
        }
    }
}
