using System.Runtime.CompilerServices;

// BuiltInShaders stays internal (it's an implementation detail of
// GameFrameControl, not a public API surface for other consumers) but
// EmuSen.WiseMan still needs to assert its SkSL source content directly -
// see Serenity/BuiltInShadersTests.cs.
[assembly: InternalsVisibleTo("EmuSen.WiseMan")]
