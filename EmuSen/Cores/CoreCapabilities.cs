using System.Collections.Generic;
using System.IO;

namespace EmuSen.Cores
{
    // A core that can attribute its own frame time by subsystem - see EmuSen_Multicore.md §5.
    public interface IFrameProfiler
    {
        // Ordered phases of the last frame; a "parent/child" name is inside that parent, not beside it.
        IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases { get; }
    }

    // A core with more than one separately-haltable processor - see EmuSen_Multicore.md §5.
    public interface ICoprocessorHalt
    {
        bool IsHaltedOnCoprocessor { get; }

        // What to call the processor a halt landed on, for a break message.
        string HaltedProcessorName { get; }
    }

    // Cumulative since power-on; <PerFrameBudget> is what a full-rate frame would allow.
    public readonly record struct CoprocessorClocks(string Name, long Executed, long Offered, long PerFrameBudget);

    // A core whose extra processors run on a clock budget worth reporting - see EmuSen_Multicore.md §5.
    public interface ICoprocessorLoad
    {
        // Empty when the loaded cartridge has no such chip.
        IReadOnlyList<CoprocessorClocks> CoprocessorClocks { get; }
    }

    // A core buffering verbose instruction traces that must reach the log before it closes.
    public interface ITraceFlushable
    {
        void FlushVerboseTrace();
    }

    // What kind of value a setting takes: on or off, a count within a range, or one name of several - see EmuSen_Multicore.md §13.
    public enum CoreSettingKind { Switch, Count, Choice }

    // One setting a console offers its frontend, every value in its text form; the default is the frontend's, not the core's constructed state - see EmuSen_Multicore.md §13.
    public sealed record CoreSetting(string Key, string Label, string Hint, CoreSettingKind Kind, string Default, int Min = 0, int Max = 0, IReadOnlyList<string>? Choices = null);

    // A core whose video settings a frontend reads and writes by key, on the emulation thread between frames - see EmuSen_Multicore.md §13.
    public interface ICoreSettings
    {
        IReadOnlyList<CoreSetting> Settings { get; }

        string Get(string key);

        void Set(string key, string value);
    }

    // The number a core writes at the head of its save states, for a frontend to record beside them - see EmuSen_Save_States.md §6.
    public interface IStateFormat
    {
        // The version SaveState writes now; what LoadState accepts is the core's own business.
        int StateVersion { get; }
    }

    // A core that knows when its picture is the one it gave last, so a frontend need not present it again - see EmuSen_Multicore.md §14.
    public interface IFrameSerial
    {
        // Changes whenever GetFrameBufferRgba may return a different picture; an unchanged value is a promise the pixels are too.
        long FrameSerial { get; }
    }

    // A core whose GetFrameBufferRgba lends arrays a frontend may hand back, so a picture need not be a new array - see EmuSen_Multicore.md §16.
    public interface IFrameBufferPool
    {
        // Once, when the caller will never read the array again; a buffer this core did not lend, or lent at another size, is dropped. Any thread.
        void ReturnFrameBuffer(byte[] buffer);
    }

    // A core whose picture repeats rows, which can hand each over once and say how many times it is shown - see EmuSen_Multicore.md §15.
    public interface IRepeatedRows
    {
        // True, the default, repeats them in the frame; false leaves the repeating to the frontend.
        bool RepeatRows { get; set; }

        // How many times each row of the frame on show is shown: one whenever the frame already repeats it.
        int RowRepeat { get; }
    }

    // A core that can write a state without waiting for work it has on other threads; LoadState reads it - see EmuSen_Rewind_And_FastForward.md §1.8.
    public interface ISnapshotCore
    {
        void SaveSnapshot(Stream stream);
    }
}
