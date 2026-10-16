using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Bin.Commands.Unix;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Bin.Native
{
    // What CreateDefault would give one session, built again for each session the Rust shell runs them in - see EmuSen_DianaSh.md §13.7.
    internal sealed class HostCommandFactory
    {
        private readonly Func<IReadOnlyList<IDianaShCommand>> _build;
        private IReadOnlyList<IDianaShCommand>? _first;

        // Every set handed out, kept alive here while the Rust side holds only weak handles to them.
        private readonly List<Dictionary<string, IDianaShCommand>> _sets = new();

        public IDebugTarget? Target { get; }

        public HostCommandFactory(IDebugTarget? target, Func<IReadOnlyList<IDianaShCommand>> build, IReadOnlyList<IDianaShCommand> prototype)
        {
            Target = target;
            _build = build;
            _first = prototype;
        }

        // The first session takes the prototype the manifest was read from; each later one gets its own, as tmux's builder gave it.
        public Dictionary<string, IDianaShCommand> NewSet()
        {
            IReadOnlyList<IDianaShCommand> commands = Interlocked.Exchange(ref _first, null) ?? _build();
            var set = commands.ToDictionary(c => c.Name, StringComparer.OrdinalIgnoreCase);
            lock (_sets) _sets.Add(set);
            return set;
        }
    }

    // The host actions C# commands return, held by a number the Rust shell carries back as a request - see EmuSen_DianaSh.md §13.7.
    internal static class HostActions
    {
        private static readonly ConcurrentDictionary<ulong, HostAction> Pending = new();
        private static long _next;

        public static ulong Stash(HostAction action)
        {
            ulong token = (ulong)Interlocked.Increment(ref _next);
            Pending[token] = action;
            // A subshell's request is never drained; the oldest are dropped long after any line could ask for them.
            if (Pending.Count > 1024) foreach (ulong old in Pending.Keys.Where(k => k + 512 < token).ToList()) Pending.TryRemove(old, out _);
            return token;
        }

        public static HostAction? Take(ulong token) => Pending.TryRemove(token, out HostAction? action) ? action : null;
    }

    // The C# commands as a Rust shell's host commands: run, jobs and kill of dianash.h - see EmuSen_DianaSh.md §13.7.
    internal static unsafe class HostBridge
    {
        private const int UnknownCommand = -265, Absent = -268, Fault = -320;

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        public static int Run(void* ctx, DianashCall* call, DianashHostResult* result)
        {
            try
            {
                HostCommandFactory factory = Factory(ctx);
                Dictionary<string, IDianaShCommand> set = SetFor(factory, call);
                if (!set.TryGetValue(Text(call->Command), out IDianaShCommand? command)) return UnknownCommand;
                string[] argv = Args(call);
                string? stdin = call->StdinPresent != 0 ? Text(call->StdinData) : null;
                DianaShResult outcome;
                // As the C# dispatcher catches it: a malformed command reads as failure, never takes the console down.
                try { outcome = command.Execute(factory.Target, argv, stdin); }
                catch (Exception ex) { outcome = DianaShResult.Fail($"Error: {ex.Message}"); }
                Finish(call, result, outcome);
                return 0;
            }
            catch (Exception ex)
            {
                Write(call, ex.Message, err: true);
                return Fault;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        public static int Jobs(void* ctx, DianashCall* call)
        {
            try
            {
                foreach (string line in PsCommand.JobLines(Factory(ctx).Target)) Write(call, line + "\n", err: false);
                return 0;
            }
            catch (Exception ex)
            {
                Write(call, ex.Message, err: true);
                return Fault;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        public static int Kill(void* ctx, DianashCall* call, byte* id, nuint idLength, DianashHostResult* result)
        {
            try
            {
                string text = Encoding.UTF8.GetString(id, (int)idLength);
                DianaShResult outcome;
                try
                {
                    if (!KillCommand.TryJob(Factory(ctx).Target, text, out outcome)) return Absent;
                }
                catch (Exception ex) { outcome = DianaShResult.Fail($"Error: {ex.Message}"); }
                Finish(call, result, outcome);
                return 0;
            }
            catch (Exception ex)
            {
                Write(call, ex.Message, err: true);
                return Fault;
            }
        }

        // A session's set, released by the shell when the session ends.
        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        public static void Release(void* state)
        {
            try { GCHandle.FromIntPtr((nint)state).Free(); }
            catch (InvalidOperationException) { }
        }

        private static HostCommandFactory Factory(void* ctx) =>
            GCHandle.FromIntPtr((nint)ctx).Target as HostCommandFactory ?? throw new InvalidOperationException("the shell's host commands outlived their host");

        // The set kept in the io table's slot for this session, made on its first call.
        private static Dictionary<string, IDianaShCommand> SetFor(HostCommandFactory factory, DianashCall* call)
        {
            void* state = null;
            nint release = 0;
            call->Io->Slot(call->Io->Ctx, &state, &release);
            if (state != null && GCHandle.FromIntPtr((nint)state).Target is Dictionary<string, IDianaShCommand> kept) return kept;
            Dictionary<string, IDianaShCommand> set = factory.NewSet();
            state = (void*)GCHandle.ToIntPtr(GCHandle.Alloc(set, GCHandleType.Weak));
            release = (nint)(delegate* unmanaged[Cdecl]<void*, void>)&Release;
            call->Io->Slot(call->Io->Ctx, &state, &release);
            return set;
        }

        private static void Finish(DianashCall* call, DianashHostResult* result, DianaShResult outcome)
        {
            Write(call, outcome.Output, err: false);
            result->ExitStatus = outcome.ExitCode;
            if (outcome.Action is { } action && result->Size >= (uint)sizeof(DianashHostResult)) result->Request = HostActions.Stash(action);
        }

        private static void Write(DianashCall* call, string text, bool err)
        {
            if (text.Length == 0) return;
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            fixed (byte* p = bytes)
            {
                if (err) call->Io->Err(call->Io->Ctx, p, (nuint)bytes.Length);
                else call->Io->Out(call->Io->Ctx, p, (nuint)bytes.Length);
            }
        }

        private static string Text(DianashBytes bytes) => bytes.Data == null ? "" : Encoding.UTF8.GetString(bytes.Data, (int)bytes.Len);

        private static string[] Args(DianashCall* call)
        {
            var args = new string[(int)call->Argc];
            for (int i = 0; i < args.Length; i++) args[i] = Text(call->Argv[i]);
            return args;
        }
    }
}
