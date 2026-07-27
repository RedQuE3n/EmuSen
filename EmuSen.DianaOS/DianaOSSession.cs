namespace EmuSen.DianaOS
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
