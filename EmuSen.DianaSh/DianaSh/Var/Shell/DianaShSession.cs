using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Var.Shell
{
    // See `man tmux`.
    public sealed class DianaShSession
    {
        public string Name { get; }
        public DianaShInterpreter Interpreter { get; }

        public DianaShSession(string name, DianaShInterpreter interpreter)
        {
            Name = name;
            Interpreter = interpreter;
        }
    }
}
