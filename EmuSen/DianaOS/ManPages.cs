using System.Collections.Generic;

namespace EmuSen.DianaOS
{
    // Long-form manual pages for `man <command>` (DianaOSInterpreter.Dispatch's
    // own special case, alongside `help`/`export`/`unset`/`source` -
    // see that method's own comment for why `man` isn't just another
    // IDianaOSCommand). Deliberately a separate, centralized module
    // rather than a `ManPage` member added to IDianaOSCommand itself:
    // that would force all ~40 existing command classes (most of them
    // one-liners) to carry a paragraph of documentation text alongside
    // their actual logic, for a feature that's purely about the shell's
    // own help system, not about how any individual command works. One
    // file, one place to keep this in sync, no per-command interface
    // churn - the same reasoning DebugCommandHelpers already uses for
    // shared, stateless command-support code that doesn't belong wedged
    // into any one command class.
    //
    // Each page follows the same loose structure a real Unix man page
    // does (NAME/SYNOPSIS/DESCRIPTION, EXAMPLES where an example
    // actually clarifies something the synopsis alone doesn't) without
    // being slavish about it - this is a debug console's help text, not
    // a formal reference manual. `man <command>` with no matching entry
    // here falls back to that command's own one-line `Usage` in
    // DianaOSInterpreter.Dispatch, so a newly-added command still gets
    // *something* useful before anyone gets around to writing its real
    // page - `Lookup` returning null is an expected, handled case, not
    // a bug to fix by adding a page for absolutely everything the
    // instant a command is created.
    public static class ManPages
    {
        public static string? Lookup(string name) =>
            _pages.TryGetValue(name, out string? page) ? page : null;

        private static readonly Dictionary<string, string> _pages = new(System.StringComparer.OrdinalIgnoreCase)
        {
            ["man"] =
                "NAME\n" +
                "    man - show a command's manual page\n\n" +
                "SYNOPSIS\n" +
                "    man [command]\n\n" +
                "DESCRIPTION\n" +
                "    The detail lookup: with a command name, prints that command's full\n" +
                "    manual page (this text) instead of just its one-line usage. With no\n" +
                "    argument, there's nothing else useful to show, so it falls back to the\n" +
                "    same overview 'help' prints. 'help' and 'man' are two separate commands,\n" +
                "    not aliases of each other - 'help' always lists everything and never\n" +
                "    shows a single command's full page; 'man' is the other way around.\n\n" +
                "EXAMPLES\n" +
                "    man\n" +
                "    man watch",

            ["help"] =
                "NAME\n" +
                "    help - list every command with a brief explanation\n\n" +
                "SYNOPSIS\n" +
                "    help\n\n" +
                "DESCRIPTION\n" +
                "    Prints every command, one line each, with a brief explanation of what it\n" +
                "    does - the quick \"what's available\" overview. Ignores any arguments;\n" +
                "    for a specific command's full manual page, use 'man <command>' instead.",

            ["summary"] =
                "NAME\n" +
                "    summary - free-text state dump\n\n" +
                "SYNOPSIS\n" +
                "    summary\n\n" +
                "DESCRIPTION\n" +
                "    Prints whatever the current IDebugTarget's GetSummaryText() has to say -\n" +
                "    a catch-all, unstructured dump for state that doesn't have its own\n" +
                "    dedicated command yet. 'No target attached.' if no ROM is loaded.",

            ["export"] =
                "NAME\n" +
                "    export - set a shell variable\n\n" +
                "SYNOPSIS\n" +
                "    export NAME[=value]\n\n" +
                "DESCRIPTION\n" +
                "    Real bash exports a variable into child processes' environment; there is\n" +
                "    no such thing here (this shell never spawns a subprocess), so 'export' is\n" +
                "    just a plain variable assignment - or a no-op declaration if no '=value'\n" +
                "    is given and the name is already set. Kept purely so a script written\n" +
                "    with real bash habits ('export FOO=bar') still does something sane\n" +
                "    instead of erroring as an unknown command. Plain 'NAME=value' (no\n" +
                "    'export' keyword) works exactly the same way.\n\n" +
                "EXAMPLES\n" +
                "    export FOO=bar\n" +
                "    echo $FOO",

            ["unset"] =
                "NAME\n" +
                "    unset - remove a shell variable\n\n" +
                "SYNOPSIS\n" +
                "    unset NAME\n\n" +
                "DESCRIPTION\n" +
                "    Removes NAME from this shell's variables. Referencing an unset variable\n" +
                "    afterward ($NAME) expands to an empty string, same as bash.",

            ["source"] =
                "NAME\n" +
                "    source, . - run a script file in this shell's own scope\n\n" +
                "SYNOPSIS\n" +
                "    source <path>\n" +
                "    . <path>\n\n" +
                "DESCRIPTION\n" +
                "    Runs <path>'s lines one at a time, exactly as if each had been typed at\n" +
                "    the prompt - including multi-line if/for/while blocks spanning several\n" +
                "    physical lines in the file. Runs in THIS shell's own variable scope, not\n" +
                "    an isolated subshell: a variable the script sets is still set once\n" +
                "    'source' returns, matching real bash 'source' (and unlike $(...) command\n" +
                "    substitution, which does run in an isolated subshell - see 'man' on\n" +
                "    command substitution... there isn't one; see the shell reference doc).\n" +
                "    Blank lines and lines starting with '#' are skipped. A script's own\n" +
                "    lines are NOT recorded into 'history'/!N recall, and '!'/'!!' expansion\n" +
                "    is disabled for them - only the 'source' invocation itself shows up in\n" +
                "    history, matching real bash. Walled to the project's own directory tree\n" +
                "    (the same sandbox 'cd'/'ls'/'mv' use) - a path outside it is rejected.\n" +
                "    A script ending mid-block, or a self-referential/mutually-recursive\n" +
                "    'source' chain (capped at 20 nested levels), reports a clean error\n" +
                "    instead of hanging the console or crashing it.\n\n" +
                "EXAMPLES\n" +
                "    source setup.txt\n" +
                "    . setup.txt",

            ["if"] =
                "NAME\n" +
                "    if - conditional execution\n\n" +
                "SYNOPSIS\n" +
                "    if LIST; then LIST; [elif LIST; then LIST;] ... [else LIST;] fi\n\n" +
                "DESCRIPTION\n" +
                "    Runs the 'if' condition; if its last pipeline exits 0 (success), runs the\n" +
                "    'then' body and stops. Otherwise tries each 'elif' condition in turn the\n" +
                "    same way, finally falling through to 'else' if none matched. Works\n" +
                "    across multiple physical lines at the interactive prompt (a secondary\n" +
                "    '> ' prompt appears until 'fi' closes it) and inside a 'source'd script\n" +
                "    the same way.\n\n" +
                "EXAMPLES\n" +
                "    if [ -f save.state ]; then echo found; else echo missing; fi",

            ["for"] =
                "NAME\n" +
                "    for - loop over a list of words\n\n" +
                "SYNOPSIS\n" +
                "    for NAME in word...; do LIST; done\n\n" +
                "DESCRIPTION\n" +
                "    Sets variable NAME to each word in turn (already expanded - quoting,\n" +
                "    $VAR, and $(...) all apply to the word list) and runs the body once per\n" +
                "    word. 'break' exits the loop early; 'continue' skips to the next word.\n" +
                "    Every iteration counts against a shared 10-second wall-clock safety\n" +
                "    timeout (see 'while') that aborts a runaway loop with a clear error.\n\n" +
                "EXAMPLES\n" +
                "    for i in 1 2 3; do echo $i; done",

            ["while"] =
                "NAME\n" +
                "    while, until - loop while/until a condition holds\n\n" +
                "SYNOPSIS\n" +
                "    while LIST; do LIST; done\n" +
                "    until LIST; do LIST; done\n\n" +
                "DESCRIPTION\n" +
                "    'while' repeats its body as long as the condition's last pipeline exits\n" +
                "    0; 'until' repeats as long as it does NOT (i.e. until it succeeds). Same\n" +
                "    'break'/'continue' as 'for'. A shared 10-second wall-clock budget (also\n" +
                "    counted against by any nested $(...) command substitution, so it can't\n" +
                "    be bypassed that way) aborts a typo'd infinite loop with a clear error\n" +
                "    instead of hanging the console - a real 'while true; do echo x; done'\n" +
                "    prints for about 10 seconds, then cleanly reports the timeout.\n\n" +
                "EXAMPLES\n" +
                "    while false; do echo never; done\n" +
                "    until [ -f ready.txt ]; do echo waiting; done",

            ["break"] =
                "NAME\n" +
                "    break - exit the nearest enclosing loop\n\n" +
                "SYNOPSIS\n" +
                "    break\n\n" +
                "DESCRIPTION\n" +
                "    A hardcoded, zero-argument parser keyword (not an ordinary dispatched\n" +
                "    command), same as real bash's own break/continue builtins. Ends the\n" +
                "    nearest enclosing 'for'/'while'/'until' loop immediately. Outside any\n" +
                "    loop it's a silent no-op, matching bash. Note: the breakpoint command is\n" +
                "    named 'bp', not 'break' - a command literally named 'break' would be\n" +
                "    unreachable, since this keyword is recognized before any command lookup\n" +
                "    happens at all.",

            ["continue"] =
                "NAME\n" +
                "    continue - skip to the next iteration of the nearest enclosing loop\n\n" +
                "SYNOPSIS\n" +
                "    continue\n\n" +
                "DESCRIPTION\n" +
                "    A hardcoded, zero-argument parser keyword, same as 'break'. Skips the\n" +
                "    rest of the current loop body and re-evaluates the loop's own condition\n" +
                "    (or advances to the next word, for 'for'). Outside any loop it's a\n" +
                "    silent no-op, matching bash.",

            ["spaces"] =
                "NAME\n" +
                "    spaces - list available memory spaces\n\n" +
                "SYNOPSIS\n" +
                "    spaces\n\n" +
                "DESCRIPTION\n" +
                "    Lists every named memory space the current core's IDebugTarget exposes -\n" +
                "    the <space> argument every mem/write/watch/dump/... command takes - along\n" +
                "    with each one's size and whether it's writable. Run this first if a\n" +
                "    space name isn't already known.",

            ["mem"] =
                "NAME\n" +
                "    mem - hexdump a range of memory\n\n" +
                "SYNOPSIS\n" +
                "    mem <space> <addr> [<len>]\n\n" +
                "DESCRIPTION\n" +
                "    Prints <len> bytes (default 16) from <space> starting at <addr> as a\n" +
                "    classic hex + ASCII dump, 16 bytes per row. <space> is one of the names\n" +
                "    'spaces' lists. Addresses/lengths are hex (an optional 0x or $ prefix is\n" +
                "    fine either way).\n\n" +
                "EXAMPLES\n" +
                "    mem WRAM 0\n" +
                "    mem CpuBus 8000 64",

            ["write"] =
                "NAME\n" +
                "    write - write one byte to memory\n\n" +
                "SYNOPSIS\n" +
                "    write <space> <addr> <value>\n\n" +
                "DESCRIPTION\n" +
                "    Writes a single byte to <space> at <addr>. Refuses outright if <space>\n" +
                "    isn't writable (see 'spaces'). No pause/synchronization with the\n" +
                "    emulation thread beyond whatever the current frontend provides (see the\n" +
                "    'pause'/'resume' commands in EmuSen.Mistress9's console window).\n\n" +
                "EXAMPLES\n" +
                "    write WRAM 10 ff",

            ["regs"] =
                "NAME\n" +
                "    regs - dump CPU/video/APU registers\n\n" +
                "SYNOPSIS\n" +
                "    regs\n\n" +
                "DESCRIPTION\n" +
                "    Prints the current core's CPU registers, video (PPU) registers, and (if\n" +
                "    the core reports any) APU registers, each in their own section. Field\n" +
                "    widths adapt to each register's real bit width.",

            ["sprites"] =
                "NAME\n" +
                "    sprites - dump the active sprite/OBJ table\n\n" +
                "SYNOPSIS\n" +
                "    sprites\n\n" +
                "DESCRIPTION\n" +
                "    Lists every active sprite: index, position, size, tile index, palette,\n" +
                "    priority, and horizontal/vertical flip state.",

            ["pal"] =
                "NAME\n" +
                "    pal - dump a color palette\n\n" +
                "SYNOPSIS\n" +
                "    pal [<index>]\n\n" +
                "DESCRIPTION\n" +
                "    Prints every palette's colors as hex RRGGBB triples, or just <index>'s\n" +
                "    palette if given.\n\n" +
                "EXAMPLES\n" +
                "    pal\n" +
                "    pal 3",

            ["channels"] =
                "NAME\n" +
                "    channels - list audio channels/voices\n\n" +
                "SYNOPSIS\n" +
                "    channels\n\n" +
                "DESCRIPTION\n" +
                "    Lists every audio channel/voice the current core reports: index, active\n" +
                "    state, envelope level (0-100), muted state, and core-specific detail.\n" +
                "    Pair with 'mute' to isolate one voice for listening/measurement.",

            ["mute"] =
                "NAME\n" +
                "    mute - mute or unmute one audio channel\n\n" +
                "SYNOPSIS\n" +
                "    mute <index> <on|off>\n\n" +
                "DESCRIPTION\n" +
                "    Excludes (or restores) one channel from the final audio mix. The\n" +
                "    channel's own playback/envelope state keeps advancing regardless -\n" +
                "    muting only affects the mix, not the underlying voice. Useful with\n" +
                "    'channels' to find a suspect voice, mute every other one, then listen\n" +
                "    to (or dump) just that voice in isolation.\n\n" +
                "EXAMPLES\n" +
                "    mute 2 on\n" +
                "    mute 2 off",

            ["watch"] =
                "NAME\n" +
                "    watch - track reads/writes to a memory range\n\n" +
                "SYNOPSIS\n" +
                "    watch add <space> <addr> <len> [write|read|both]\n" +
                "    watch list\n" +
                "    watch log <id> [<count>]\n" +
                "    watch summary <id>\n" +
                "    watch clear <id>\n" +
                "    watch remove <id>\n\n" +
                "DESCRIPTION\n" +
                "    'add' registers a watch over <addr>..<addr>+<len>-1 in <space> (default\n" +
                "    kind: write) and starts recording matching accesses as they happen -\n" +
                "    this is a LIVE, triggered mechanism (see 'framelog' for the sampled-\n" +
                "    every-frame alternative, and 'callers'/'writers'/'readers' for static\n" +
                "    analysis that doesn't require the access to actually happen first).\n" +
                "    'log' shows a watch's recorded (sequence, kind, address, value, context)\n" +
                "    events, newest last. 'summary' groups those events by access site\n" +
                "    instead of listing every one - the dynamic, addressing-mode-agnostic\n" +
                "    equivalent of 'readers'/'writers'. 'clear' empties a watch's recorded\n" +
                "    events without removing the watch itself; 'remove' deletes it entirely.\n\n" +
                "EXAMPLES\n" +
                "    watch add WRAM 13c6 1\n" +
                "    watch log 1\n" +
                "    watch summary 1",

            ["bp"] =
                "NAME\n" +
                "    bp - manage execution breakpoints\n\n" +
                "SYNOPSIS\n" +
                "    bp add <addr>\n" +
                "    bp list\n" +
                "    bp remove <id>\n\n" +
                "DESCRIPTION\n" +
                "    Registers/lists/removes a breakpoint at a 24-bit CPU address. This\n" +
                "    command only edits the breakpoint list - it doesn't halt or resume\n" +
                "    execution itself, since that requires re-entering a core's own frame\n" +
                "    loop, which only the frontend's own main loop can do. See the F4\n" +
                "    prompt's own 'step'/'s' and 'continue'/'c' shortcuts (EmuSen.Hotaru) for\n" +
                "    the actual halt/resume side. Named 'bp', not 'break' - a command\n" +
                "    literally named 'break' would be unreachable, shadowed by the shell's\n" +
                "    own hardcoded break/continue loop-control keywords.\n\n" +
                "EXAMPLES\n" +
                "    bp add 8000\n" +
                "    bp list",

            ["framelog"] =
                "NAME\n" +
                "    framelog - sample a memory value once per frame\n\n" +
                "SYNOPSIS\n" +
                "    framelog add <space> <addr> [<width>]\n" +
                "    framelog list\n" +
                "    framelog show <id> [<count>]\n" +
                "    framelog clear <id>\n" +
                "    framelog remove <id>\n\n" +
                "DESCRIPTION\n" +
                "    Unlike 'watch' (triggered by an actual read/write), a frame log samples\n" +
                "    its value once every frame regardless of whether anything touched it -\n" +
                "    useful for 'what does this value do over time' questions a write watch\n" +
                "    can't answer, e.g. a counter written once at level start but read every\n" +
                "    frame afterward. <width> is 1, 2, or 4 bytes (default 1).\n\n" +
                "EXAMPLES\n" +
                "    framelog add WRAM 1fb0 2\n" +
                "    framelog show 1",

            ["cheat"] =
                "NAME\n" +
                "    cheat - manage RAM pokes and ROM patches\n\n" +
                "SYNOPSIS\n" +
                "    cheat add <code> [description]\n" +
                "    cheat poke <space> <addr> <value> [description]\n" +
                "    cheat gg <code> [description]\n" +
                "    cheat rompatch <addr> <value> [<compare>|-] [description]\n" +
                "    cheat list\n" +
                "    cheat enable <id>\n" +
                "    cheat disable <id>\n" +
                "    cheat remove <id>\n" +
                "    cheat clear\n\n" +
                "DESCRIPTION\n" +
                "    Unifies two SNES-specific cheat mechanisms under one command: RAM pokes\n" +
                "    (Pro Action Replay/Game Wizard style) and ROM-read patches (Game Genie\n" +
                "    style). 'add' decodes an 8-character code, guessing which format it is\n" +
                "    from its punctuation (best-effort, not a guarantee - use 'gg' or 'poke'\n" +
                "    directly if the guess is wrong). 'poke'/'rompatch' add a cheat directly\n" +
                "    without code decoding, e.g. for an address already found with 'search'.\n" +
                "    'enable'/'disable' toggle a cheat without removing it.\n\n" +
                "EXAMPLES\n" +
                "    cheat poke WRAM 9c 63 infinite lives\n" +
                "    cheat list\n" +
                "    cheat disable 1",

            ["search"] =
                "NAME\n" +
                "    search - classic 'first scan, then narrow' memory search\n\n" +
                "SYNOPSIS\n" +
                "    search <space> <value> [<width>]\n" +
                "    search refine <value>\n" +
                "    search changed|unchanged|increased|decreased\n" +
                "    search list [<count>]\n" +
                "    search reset\n\n" +
                "DESCRIPTION\n" +
                "    Finds where a game stores something without already knowing the address\n" +
                "    (a score, lives, a flag). Start with 'search <space> <value>' to find\n" +
                "    every address currently equal to <value> (width 1/2/4 bytes, default 1,\n" +
                "    little-endian); then narrow with 'refine' (equal to a new value) or\n" +
                "    changed/unchanged/increased/decreased (relative to the last scan/refine)\n" +
                "    until only the real address remains. One search session at a time -\n" +
                "    starting a new 'search <space> ...' replaces whatever was active.\n" +
                "    Refuses to scan a live-hardware-routed space (e.g. CpuBus) outright,\n" +
                "    since bulk-reading it can have real side effects (RDNMI clearing the\n" +
                "    pending-NMI flag, etc.) - use a plain space like WRAM instead.\n\n" +
                "EXAMPLES\n" +
                "    search WRAM 64\n" +
                "    search refine 65\n" +
                "    search list",

            ["snapshot"] =
                "NAME\n" +
                "    snapshot - capture a memory space's full contents\n\n" +
                "SYNOPSIS\n" +
                "    snapshot <space> <name>\n" +
                "    snapshot list\n" +
                "    snapshot remove <name>\n\n" +
                "DESCRIPTION\n" +
                "    Saves <space>'s entire current contents under <name> for later 'diff'\n" +
                "    against a further-along state - the general-purpose counterpart to\n" +
                "    'search' when the interesting addresses aren't already narrowed down.\n" +
                "    Same live-hardware-space refusal as 'search'/'dump'.\n\n" +
                "EXAMPLES\n" +
                "    snapshot WRAM before\n" +
                "    diff before",

            ["diff"] =
                "NAME\n" +
                "    diff - compare a snapshot against current memory\n\n" +
                "SYNOPSIS\n" +
                "    diff <name> [<count>]\n\n" +
                "DESCRIPTION\n" +
                "    Compares a snapshot saved with 'snapshot' against that same space's\n" +
                "    CURRENT contents and lists every byte address that's different now\n" +
                "    (default 20 shown). Doesn't touch or replace the saved snapshot, so the\n" +
                "    same baseline can be diffed again later against a further state.\n\n" +
                "EXAMPLES\n" +
                "    diff before\n" +
                "    diff before 50",

            ["dump"] =
                "NAME\n" +
                "    dump - write a raw memory range to disk\n\n" +
                "SYNOPSIS\n" +
                "    dump <space> <addr> <len> <file>\n\n" +
                "DESCRIPTION\n" +
                "    Writes <len> raw bytes from <space> starting at <addr> to\n" +
                "    Logs/<CoreName>/<file> - no header or metadata, so a hex editor can open\n" +
                "    the result directly. The write-side counterpart is 'load'. Same live-\n" +
                "    hardware-space refusal as 'search'/'snapshot'.\n\n" +
                "EXAMPLES\n" +
                "    dump WRAM 0 2000 wram.bin",

            ["load"] =
                "NAME\n" +
                "    load - write a raw file's bytes into memory\n\n" +
                "SYNOPSIS\n" +
                "    load <space> <addr> <file>\n\n" +
                "DESCRIPTION\n" +
                "    Reads Logs/<CoreName>/<file> (typically one 'dump' produced, or hand-\n" +
                "    edited afterward) and pokes its raw bytes into <space> starting at\n" +
                "    <addr>. Refuses if <space> isn't writable.\n\n" +
                "EXAMPLES\n" +
                "    load WRAM 0 wram.bin",

            ["tile"] =
                "NAME\n" +
                "    tile - ASCII-decode one 8x8 tile\n\n" +
                "SYNOPSIS\n" +
                "    tile <space> <addr> <bpp>\n\n" +
                "DESCRIPTION\n" +
                "    Decodes and prints one 8x8 tile's pixels as an ASCII grid of hex nibbles\n" +
                "    (or '.' for a transparent/zero pixel). <bpp> meaning is core-specific -\n" +
                "    2/4/8 bits-per-pixel on the SNES.\n\n" +
                "EXAMPLES\n" +
                "    tile VRAM 0 4",

            ["tilemap"] =
                "NAME\n" +
                "    tilemap - decode a grid of tilemap entries as text\n\n" +
                "SYNOPSIS\n" +
                "    tilemap <space> <addr> <cols> <rows>\n\n" +
                "DESCRIPTION\n" +
                "    Decodes <cols>x<rows> tilemap/nametable entries starting at <addr> as a\n" +
                "    text grid, one label per cell - complementary to 'tile' (which decodes\n" +
                "    one tile's actual pixel content). The caller supplies the exact base\n" +
                "    address (see BG1SC/etc via 'regs') - this doesn't resolve BG-layer-to-\n" +
                "    tilemap-address mapping itself. Built to confirm a menu cursor's\n" +
                "    position or a HUD change numerically, without eyeballing a screenshot.\n\n" +
                "EXAMPLES\n" +
                "    tilemap VRAM 0 32 32",

            ["disasm"] =
                "NAME\n" +
                "    disasm - disassemble instructions\n\n" +
                "SYNOPSIS\n" +
                "    disasm <space> <addr> [<n>]\n\n" +
                "DESCRIPTION\n" +
                "    Disassembles <n> instructions (default 10) starting at <addr>, printing\n" +
                "    address, raw bytes, mnemonic, and operand for each. A linear\n" +
                "    disassembler has no way to know which bytes are really code vs. data\n" +
                "    mixed into the same range, so a run through embedded data can produce\n" +
                "    garbage until it happens to resync - a known, accepted limitation, not a\n" +
                "    bug.\n\n" +
                "EXAMPLES\n" +
                "    disasm CpuBus 8000 20",

            ["trace"] =
                "NAME\n" +
                "    trace - arm a live CPU instruction trace\n\n" +
                "SYNOPSIS\n" +
                "    trace <count>\n" +
                "    trace off\n\n" +
                "DESCRIPTION\n" +
                "    Arms DebugSettings.CpuVerboseLogging for the next <count> instructions,\n" +
                "    counted from whenever this command runs (not from power-on) - so a\n" +
                "    trace can be aimed at a specific moment in a play session instead of\n" +
                "    burning its whole budget during boot. 'trace off' cancels an in-progress\n" +
                "    trace early. A global settings toggle, not scoped to a particular core\n" +
                "    instance.\n\n" +
                "EXAMPLES\n" +
                "    trace 500\n" +
                "    trace off",

            ["callers"] =
                "NAME\n" +
                "    callers - find static call/jump references to an address\n\n" +
                "SYNOPSIS\n" +
                "    callers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    Scans CpuBus (default: <addr>'s own bank, $8000-$FFFF) for instructions\n" +
                "    statically calling/jumping to <addr> - a static-analysis alternative to\n" +
                "    setting a breakpoint and waiting for it to hit: finds every place that\n" +
                "    COULD jump there, whether or not it was ever actually reached during a\n" +
                "    given play session. See 'writers'/'readers' for the memory-access\n" +
                "    equivalents.\n\n" +
                "EXAMPLES\n" +
                "    callers 8000",

            ["writers"] =
                "NAME\n" +
                "    writers - find static write references to an address\n\n" +
                "SYNOPSIS\n" +
                "    writers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    Scans CpuBus (default: <addr>'s own bank, $8000-$FFFF) for instructions\n" +
                "    statically storing to <addr> - answers 'what code exists that's capable\n" +
                "    of writing here', regardless of whether that write path was ever\n" +
                "    actually reached during a traced run (which is all a live 'watch' can\n" +
                "    tell you). Only finds statically-known addressing modes (absolute/\n" +
                "    absolute-long); indexed/indirect writes aren't visible to static\n" +
                "    analysis and won't show up here - use 'watch' for those.\n\n" +
                "EXAMPLES\n" +
                "    writers 13c6",

            ["readers"] =
                "NAME\n" +
                "    readers - find static read references to an address\n\n" +
                "SYNOPSIS\n" +
                "    readers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    The read-side complement to 'writers': finds every instruction\n" +
                "    statically reading <addr>, e.g. every branch of game logic that checks\n" +
                "    a flag's value - not just the one spot noticed while manually tracing\n" +
                "    execution. Same addressing-mode limitation as 'writers'.\n\n" +
                "EXAMPLES\n" +
                "    readers 13c6",

            ["log"] =
                "NAME\n" +
                "    log - master logging on/off switch\n\n" +
                "SYNOPSIS\n" +
                "    log off\n" +
                "    log on\n" +
                "    log status\n\n" +
                "DESCRIPTION\n" +
                "    Silences (or restores) every DebugSettings *Logging flag at once,\n" +
                "    without changing any of their individually-set values - 'log on'\n" +
                "    afterward brings back exactly whatever was individually enabled before.\n" +
                "    A global settings toggle, not scoped to a particular core instance.",

            ["echo"] =
                "NAME\n" +
                "    echo - print arguments\n\n" +
                "SYNOPSIS\n" +
                "    echo <text...>\n\n" +
                "DESCRIPTION\n" +
                "    Prints its arguments back, joined by single spaces - real argument\n" +
                "    quoting applies ('echo 'a b'' is one argument, not two). Ignores stdin\n" +
                "    entirely, matching real echo. Its real use is scripting: a marker line\n" +
                "    in a source'd script, or the natural pipeline source for a text filter\n" +
                "    ('echo <text> | sed s/.../.../ ').\n\n" +
                "EXAMPLES\n" +
                "    echo hello world\n" +
                "    echo $X",

            ["sed"] =
                "NAME\n" +
                "    sed - stream substitution\n\n" +
                "SYNOPSIS\n" +
                "    sed s/pattern/replacement/[gi]\n" +
                "    sed <expr> <text...>\n\n" +
                "DESCRIPTION\n" +
                "    A basic sed-style filter supporting only the s/// substitution command\n" +
                "    (no line addressing, no d/p/hold-space/etc). The delimiter can be any\n" +
                "    character, matching real sed's own 's#/bin#/usr/bin#' convention, with\n" +
                "    '\\<delim>' escaping a literal delimiter inside the pattern/replacement.\n" +
                "    Pattern is a .NET regex. Without 'g', only the first match on EACH LINE\n" +
                "    is replaced (not just the first match in the whole piped blob) - most\n" +
                "    output this runs against (regs, watch log, ...) is multi-line, so this\n" +
                "    matters. Replacement supports '&' (whole match) and \\1-\\9 (capture\n" +
                "    groups), same as real sed. Real use is as a pipeline stage; standalone\n" +
                "    'sed <expr> <text...>' works too when there's nothing to pipe from.\n\n" +
                "EXAMPLES\n" +
                "    regs | sed s/PC=/pc=/\n" +
                "    echo hello | sed s/hello/hi/",

            ["grep"] =
                "NAME\n" +
                "    grep - filter lines by pattern\n\n" +
                "SYNOPSIS\n" +
                "    grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]\n\n" +
                "DESCRIPTION\n" +
                "    Filters lines matching <pattern> (a .NET regex, same choice sed makes).\n" +
                "    -i case-insensitive, -v invert (show non-matching lines), -n prefix each\n" +
                "    match with its 1-based line number, -c print only the match count, -q\n" +
                "    no output at all (exit code only). Exit code is 0 if anything matched, 1\n" +
                "    otherwise (real grep semantics) - -q exists specifically so grep can\n" +
                "    drive an if/while condition ('if regs | grep -q PC; then ...'), not just\n" +
                "    filter text. Reads piped stdin if present, else operates on trailing\n" +
                "    literal text. No -E/-F/-P mode switches, no -A/-B/-C context lines, no\n" +
                "    multi-file support.\n\n" +
                "EXAMPLES\n" +
                "    watch log 1 | grep -v poll\n" +
                "    if regs | grep -q PC; then echo has-pc; fi",

            ["wc"] =
                "NAME\n" +
                "    wc - count lines, words, bytes\n\n" +
                "SYNOPSIS\n" +
                "    wc [-l] [-w] [-c] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Counts lines/words/bytes of piped stdin, or a real file if <path> is\n" +
                "    given (walled to the project's own directory tree). With no flags,\n" +
                "    prints all three in that fixed order regardless of what order flags are\n" +
                "    given in, matching real wc.\n\n" +
                "EXAMPLES\n" +
                "    history | wc -l\n" +
                "    wc -l notes.txt",

            ["sort"] =
                "NAME\n" +
                "    sort - sort lines\n\n" +
                "SYNOPSIS\n" +
                "    sort [-n] [-r]\n\n" +
                "DESCRIPTION\n" +
                "    Sorts piped stdin (or trailing literal text) by line - ordinal string\n" +
                "    comparison by default, numeric with -n, reversed with -r. No -u (that's\n" +
                "    'uniq's job - pair them, 'sort | uniq', the way real shell usage does\n" +
                "    before reaching for 'sort -u' as a shortcut), no field/key selection, no\n" +
                "    locale-aware collation.\n\n" +
                "EXAMPLES\n" +
                "    watch list | sort -n",

            ["uniq"] =
                "NAME\n" +
                "    uniq - collapse adjacent duplicate lines\n\n" +
                "SYNOPSIS\n" +
                "    uniq [-c]\n\n" +
                "DESCRIPTION\n" +
                "    Collapses ADJACENT duplicate lines only, matching real uniq's own\n" +
                "    semantics - it is NOT a global dedup, so pipe through 'sort' first if\n" +
                "    that's what's actually wanted ('regs | sort | uniq'). -c prefixes each\n" +
                "    remaining line with its consecutive repeat count.\n\n" +
                "EXAMPLES\n" +
                "    watch log 1 | sort | uniq -c",

            ["awk"] =
                "NAME\n" +
                "    awk - a small AWK subset for reshaping text by field\n\n" +
                "SYNOPSIS\n" +
                "    awk [-F sep] 'program' [path]\n\n" +
                "DESCRIPTION\n" +
                "    Real AWK is a whole language; this is deliberately a small subset -\n" +
                "    enough to cover 'filter/reshape another command's output by field',\n" +
                "    without embedding a second full expression language. Supported\n" +
                "    patterns: empty (matches every line), /regex/, NR==N (also !=, <, <=, >,\n" +
                "    >=), BEGIN, END. Supported actions: {print expr[, expr...][; print\n" +
                "    ...]}, where expr is $0, $N, $NF, NR, NF, a \"quoted string\", or a bare\n" +
                "    number - no concatenation, no arithmetic, no user variables. A bare\n" +
                "    pattern with no {...} defaults to {print $0} (real awk's own default\n" +
                "    action); an explicit empty {} does nothing. Field splitting is\n" +
                "    whitespace-runs by default, or a literal separator string via -F (not a\n" +
                "    regex the way real awk's multi-character -F is). Reads piped stdin, or a\n" +
                "    trailing file path (walled to the project's own directory tree).\n\n" +
                "EXAMPLES\n" +
                "    regs | awk '{print $1}'\n" +
                "    watch log 1 | awk -F, 'NR>1{print $2}'\n" +
                "    echo -e \"a\\nb\\nc\" | awk 'NR==2'",

            ["ls"] =
                "NAME\n" +
                "    ls - list directory entries\n\n" +
                "SYNOPSIS\n" +
                "    ls [-a] [-l] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Lists a real directory's entries (default: current directory), or a\n" +
                "    single file listed as itself. One entry per line, no multi-column\n" +
                "    terminal layout. -a includes dotfiles; -l shows a simplified type/size/\n" +
                "    modified-time/name line (no permission bits/owner/group - this project\n" +
                "    has no such concept to show). Walled to the project's own directory\n" +
                "    tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    ls\n" +
                "    ls -la EmuSen",

            ["cd"] =
                "NAME\n" +
                "    cd - change the current directory\n\n" +
                "SYNOPSIS\n" +
                "    cd [dir]\n\n" +
                "DESCRIPTION\n" +
                "    Changes the process-wide current directory - the same one 'pwd' reads\n" +
                "    and every relative path elsewhere in this shell (redirection, 'ls', 'mv',\n" +
                "    'source', 'wc <path>', ...) resolves against. No argument goes to the\n" +
                "    project's own root directory (there's no real $HOME concept here). Walled\n" +
                "    to that same project root - no amount of 'cd ..', an absolute path like\n" +
                "    '/etc', or a long '../../..' chain can leave it; this is a deliberate\n" +
                "    walled garden against accidents, not a security boundary.\n\n" +
                "EXAMPLES\n" +
                "    cd EmuSen\n" +
                "    cd",

            ["pwd"] =
                "NAME\n" +
                "    pwd - print the current directory\n\n" +
                "SYNOPSIS\n" +
                "    pwd\n\n" +
                "DESCRIPTION\n" +
                "    Prints the process's current working directory - see 'cd'.",

            ["nano"] =
                "NAME\n" +
                "    nano - simple full-screen text editor\n\n" +
                "SYNOPSIS\n" +
                "    nano <path>\n\n" +
                "DESCRIPTION\n" +
                "    A small full-screen editor for creating/editing a real file (a 'source'\n" +
                "    script, say) without leaving the shell for an external editor. NOT a\n" +
                "    full GNU nano clone - no search, no cut/paste ring, no syntax\n" +
                "    highlighting - just arrow-key movement, insert/Backspace/Delete, and\n" +
                "    Ctrl+O to save / Ctrl+X to exit (prompting to save first if there are\n" +
                "    unsaved changes). <path> is created if it doesn't exist yet, and is\n" +
                "    walled to the project's own directory tree like every other real-file\n" +
                "    command here (see 'cd'). Needs a REAL interactive terminal - it takes\n" +
                "    over the console with raw key reads and full-screen redraws the same\n" +
                "    way this shell's own line editor does for a single line, just for a\n" +
                "    whole buffer - so it refuses cleanly (rather than failing strangely) when\n" +
                "    stdin/stdout is redirected (a piped script, a 'source'd file, a headless\n" +
                "    test) or when there's no real terminal at all (EmuSen.Mistress9's GUI\n" +
                "    console window).\n\n" +
                "EXAMPLES\n" +
                "    nano setup.txt",

            ["clear"] =
                "NAME\n" +
                "    clear - clear the terminal screen\n\n" +
                "SYNOPSIS\n" +
                "    clear\n\n" +
                "DESCRIPTION\n" +
                "    Wipes whatever's currently on screen (a real Console.Clear(), not just\n" +
                "    printing blank lines) so the next prompt starts at the top instead of at\n" +
                "    the bottom of a long scrollback. Only does anything against a real\n" +
                "    interactive console; a redirected/nonexistent one (a piped script, a\n" +
                "    'source'd file, a headless test) is silently a no-op rather than an\n" +
                "    error, since there was never a real screen to clear in the first place.\n\n" +
                "    In EmuSen.Mistress9's own console window (a TextBox, not a real\n" +
                "    terminal), 'clear' is replaced with a windowed equivalent instead: it\n" +
                "    empties that window's own output text rather than touching a system\n" +
                "    console that doesn't exist there.\n\n" +
                "EXAMPLES\n" +
                "    clear",

            ["coretop"] =
                "NAME\n" +
                "    coretop - live htop-style dashboard of the loaded core's hardware\n\n" +
                "SYNOPSIS\n" +
                "    coretop [-w]\n\n" +
                "DESCRIPTION\n" +
                "    A live, auto-refreshing (4x/second) dashboard, styled after htop: per-\n" +
                "    subsystem hardware load bars, CPU registers, a sprite-capacity gauge,\n" +
                "    per-voice audio channel meters, a live color-RAM palette swatch, and (if\n" +
                "    this core has a tilemap concept) a downsampled live preview of VRAM's\n" +
                "    tile sheet - all in one screen, refreshing on its own clock rather than\n" +
                "    waiting for input. Every number comes from IDebugTarget, so a core with\n" +
                "    nothing to report for a given section (no hardware-load breakdown wired\n" +
                "    up, no fixed sprite capacity, no palettes) just makes that section\n" +
                "    shrink or disappear rather than showing something fake.\n\n" +
                "    Load bars are colored green/yellow/red under 60% / 60-85% / over 85%,\n" +
                "    the same rough 'getting busy' convention real htop's CPU bars use.\n\n" +
                "    Press Ctrl+C to exit - unlike everywhere else in this shell, Ctrl+C is\n" +
                "    read as a key here (via Console.TreatControlCAsInput) rather than\n" +
                "    raising a process-level interrupt, restored to normal the moment\n" +
                "    coretop exits. Needs a REAL interactive terminal, same as 'nano' and\n" +
                "    for the same reason - refuses cleanly rather than trying to draw\n" +
                "    anywhere when stdin/stdout is redirected or there's no real terminal at\n" +
                "    all (a piped script, a 'source'd file, a headless test).\n\n" +
                "    In EmuSen.Mistress9's own console window (a TextBox, not a real\n" +
                "    terminal - the above wouldn't work there at all), 'coretop' is replaced\n" +
                "    with a windowed equivalent instead: a real, non-blocking Avalonia window\n" +
                "    showing the same data as actual widgets and live images (the palette and\n" +
                "    VRAM tile sheet render as real pictures there, not ANSI blocks) that\n" +
                "    refreshes on its own timer while gameplay keeps running - Ctrl+C doesn't\n" +
                "    apply there, just close the window.\n\n" +
                "    -w  Open the same dashboard in a separate window instead of taking over\n" +
                "        the terminal, so you can dismiss it and keep playing. In\n" +
                "        EmuSen.Mistress9, coretop is already windowed by default, so -w is\n" +
                "        accepted and simply has no additional effect there. In EmuSen.Hotaru\n" +
                "        (a Raylib console build with no window of its own to reuse), -w spins\n" +
                "        up a small dedicated Avalonia UI thread the first time it's used and\n" +
                "        opens a real second OS window from it, entirely separate from the\n" +
                "        Raylib game window and its main loop - gameplay keeps running behind\n" +
                "        it. Closing that window is -w's equivalent of Ctrl+C: it's the whole\n" +
                "        'stop watching coretop' action, nothing else to press. If the current\n" +
                "        frontend has no window to open at all, -w fails cleanly with\n" +
                "        \"not supported by this frontend\" rather than falling back to the\n" +
                "        terminal dashboard.\n\n" +
                "EXAMPLES\n" +
                "    coretop\n" +
                "    coretop -w",

            ["mv"] =
                "NAME\n" +
                "    mv - move or rename a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    mv <src> <dst>\n\n" +
                "DESCRIPTION\n" +
                "    Moves/renames a real file or directory. If <dst> is an existing\n" +
                "    directory, <src> lands inside it ('mv foo.txt logs/' -> 'logs/foo.txt'),\n" +
                "    matching real mv. An existing destination FILE is silently overwritten\n" +
                "    (no -i prompt anywhere in this shell, by design). Both <src> and <dst>\n" +
                "    must resolve inside the project's own directory tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    mv scratch.txt Logs/",

            ["true"] =
                "NAME\n" +
                "    true - always succeed\n\n" +
                "SYNOPSIS\n" +
                "    true\n\n" +
                "DESCRIPTION\n" +
                "    Does nothing, always exits 0. Its only real use is in control flow -\n" +
                "    'while true; do ...; done', or as a harmless placeholder branch.",

            ["false"] =
                "NAME\n" +
                "    false - always fail\n\n" +
                "SYNOPSIS\n" +
                "    false\n\n" +
                "DESCRIPTION\n" +
                "    Does nothing, always exits 1. Same real use as 'true', just the other\n" +
                "    branch - 'until false; do ...; done' is an infinite loop, same as\n" +
                "    'while true'.",

            ["test"] =
                "NAME\n" +
                "    test, [ - evaluate a condition\n\n" +
                "SYNOPSIS\n" +
                "    test EXPR\n" +
                "    [ EXPR ]\n\n" +
                "DESCRIPTION\n" +
                "    The main way an if/while condition does anything besides check another\n" +
                "    command's own exit code. A deliberately small subset of real test(1):\n" +
                "    a single VALUE (true if non-empty); -z/-n VALUE (string emptiness); -f/-d\n" +
                "    PATH (real file/directory existence - meaningful here since redirection\n" +
                "    already touches real host files); VALUE = / != VALUE (string comparison);\n" +
                "    VALUE -eq/-ne/-lt/-le/-gt/-ge VALUE (numeric comparison); a leading '!'\n" +
                "    negates any of the above. No -a/-o/parenthesized compound expressions -\n" +
                "    combine conditions with the shell's own && / || instead ('[ -f a ] && [\n" +
                "    -f b ]'). '[' is the same evaluator under a different name - the\n" +
                "    trailing ']' is required, matching real bash.\n\n" +
                "EXAMPLES\n" +
                "    if [ -f save.state ]; then echo found; fi\n" +
                "    if test $X -eq 5; then echo five; fi",

            ["["] =
                "NAME\n" +
                "    [ - alias for 'test'\n\n" +
                "SYNOPSIS\n" +
                "    [ EXPR ]\n\n" +
                "DESCRIPTION\n" +
                "    See 'man test' - '[' is the same evaluator under a different name. The\n" +
                "    trailing ']' is required, matching real bash's own '[' builtin (which\n" +
                "    really is just a differently-named 'test').",

            ["history"] =
                "NAME\n" +
                "    history - list previously executed commands\n\n" +
                "SYNOPSIS\n" +
                "    history\n" +
                "    history clear\n" +
                "    !N\n" +
                "    !!\n\n" +
                "DESCRIPTION\n" +
                "    Lists every completed (non-buffered) command line typed at the prompt,\n" +
                "    numbered from 1 like bash's own history builtin. 'history clear' forgets\n" +
                "    all of it. '!N' re-runs history entry N; '!!' re-runs the last command -\n" +
                "    both are expanded before lexing even happens, the same point a real\n" +
                "    shell expands '!'-references, and only at the start of a brand-new\n" +
                "    (not mid-block) line. A script run via 'source' does NOT add its own\n" +
                "    lines to history - only the 'source' invocation itself does.\n\n" +
                "EXAMPLES\n" +
                "    history\n" +
                "    !!\n" +
                "    !12",
        };
    }
}
