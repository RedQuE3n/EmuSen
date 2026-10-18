namespace EmuSen.Crystal.Session
{
    // What a runtime holds a session by while a change is applied: entered and left once each, on the caller's thread - see EmuSen_Backend.md §14.3.
    public interface IChangeGate
    {
        void Enter();

        void Exit();
    }

    // The gate held for one call of the provider, left when the call returns.
    internal readonly struct GateScope : IDisposable
    {
        private readonly IChangeGate? _gate;

        public GateScope(IChangeGate? gate)
        {
            _gate = gate;
            gate?.Enter();
        }

        public void Dispose() => _gate?.Exit();
    }
}
