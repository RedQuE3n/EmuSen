using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Var
{
    // See `man tmux`.
    public sealed class DianaOSSession
    {
        public string Name { get; }
        public DianaOSInterpreter Interpreter { get; }

        public DianaOSSession(string name, DianaOSInterpreter interpreter)
        {
            Name = name;
            Interpreter = interpreter;
        }
    }
}
