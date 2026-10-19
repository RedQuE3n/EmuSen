using System;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // A shell refused over a session's table it cannot serve: the C# shell over a table the platform library made - see EmuSen_Settings_Reference.md §4.117.
    public sealed class DianaShTableRefusedException(string message) : InvalidOperationException(message)
    {
    }
}
