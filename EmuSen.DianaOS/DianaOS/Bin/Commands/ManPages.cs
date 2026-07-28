using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands
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
    // into any one command class. Lives directly under Commands/ rather
    // than Commands/Unix or Commands/EmuSen - it documents commands from
    // both, and isn't an IDianaOSCommand itself, so neither subfolder fits.
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
                "    'pause'/'resume' commands in EmuSen.Mistress's console window).\n\n" +
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
                "    EmuSen.DianaOS/DianaOS/Usr/Home/Logs/<CoreName>/<file> - no header or\n" +
                "    metadata, so a hex editor can open the result directly. The write-side\n" +
                "    counterpart is 'load'. Same live-hardware-space refusal as\n" +
                "    'search'/'snapshot'.\n\n" +
                "EXAMPLES\n" +
                "    dump WRAM 0 2000 wram.bin",

            ["load"] =
                "NAME\n" +
                "    load - write a raw file's bytes into memory\n\n" +
                "SYNOPSIS\n" +
                "    load <space> <addr> <file>\n\n" +
                "DESCRIPTION\n" +
                "    Reads EmuSen.DianaOS/DianaOS/Usr/Home/Logs/<CoreName>/<file> (typically\n" +
                "    one 'dump' produced, or hand-edited afterward) and pokes its raw bytes\n" +
                "    into <space> starting at\n" +
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
                "    Arms this core's own live CPU instruction trace for the next <count>\n" +
                "    instructions, counted from whenever this command runs (not from\n" +
                "    power-on) - so a trace can be aimed at a specific moment in a play\n" +
                "    session instead of burning its whole budget during boot. 'trace off'\n" +
                "    cancels an in-progress trace early. Not available for a core with no\n" +
                "    equivalent trace mechanism, or a standalone launch with no core loaded.\n\n" +
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
                "    Silences (or restores) every trace/diagnostic logging flag across the\n" +
                "    project at once, without changing any of their individually-set values -\n" +
                "    'log on' afterward brings back exactly whatever was individually enabled\n" +
                "    before. A global settings toggle, not scoped to a particular core instance.",

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
                "    'rm', 'source', 'wc <path>', ...) resolves against. No argument goes to\n" +
                "    the current account's own home directory (see 'whoami'/'hier') - real\n" +
                "    $HOME semantics, scoped to this shell's own tree. A path starting with\n" +
                "    '/' means THIS shell's own root, not the real OS filesystem root -\n" +
                "    'cd /SourceLogs' works from anywhere, the same way a real chroot makes '/'\n" +
                "    mean the chroot directory (see 'hier'). Walled to the project root no\n" +
                "    matter how it's spelled - no amount of 'cd ..', a leading '/', or a long\n" +
                "    '../../..' chain can leave it; this is a deliberate walled garden against\n" +
                "    accidents, not a security boundary. A directory name with spaces doesn't\n" +
                "    need quoting ('cd My Folder' works) - cd only ever takes one path, so\n" +
                "    everything after it is treated as that path literally; quoting\n" +
                "    ('cd \"My Folder\"') also works if you prefer it.\n\n" +
                "EXAMPLES\n" +
                "    cd EmuSen.DianaOS\n" +
                "    cd /SourceLogs\n" +
                "    cd My Folder\n" +
                "    cd",

            ["hier"] =
                "NAME\n" +
                "    hier - this shell's own filesystem layout\n\n" +
                "SYNOPSIS\n" +
                "    man hier\n\n" +
                "DESCRIPTION\n" +
                "    A real Unix precedent for this exact page: 'man hier' documents a\n" +
                "    filesystem's layout without 'hier' being a runnable command - same here.\n\n" +
                "    /home/root        root's own home directory\n" +
                "    /home/<user>      created by 'useradd <user>'; 'cd' with no argument goes\n" +
                "                      to the current account's own home - see 'whoami'/'su'\n" +
                "    /etc              reserved for future shell-level config - empty for now\n" +
                "    /SourceLogs       was 'var/log' - WiseMan test-run scratch space only;\n" +
                "                      dev/test artifacts, not emulator output\n" +
                "    /tmp              scratch space, nothing here is ever auto-deleted\n\n" +
                "    EmuSen.DianaOS/DianaOS/Usr/Home/Logs   was 'var/log' (before that,\n" +
                "                      'Logs/') - 'dump'/'load'/screenshot/recording output\n" +
                "    EmuSen.DianaOS/DianaOS/Usr/Home/Saves  was 'var/lib' + 'var/games'\n" +
                "                      (before that, 'SaveStates/' + 'Saves/') - 'state\n" +
                "                      save'/'state load' snapshots and battery-backed\n" +
                "                      cartridge SRAM ('.srm')\n\n" +
                "    Unlike the short mnemonic paths above, Logs and Saves live several\n" +
                "    levels deep in the real tree, inside the DianaOS project's own source\n" +
                "    folder - there's no short '/...' alias for them (yet).\n\n" +
                "    A leading '/' in any path means THIS root, not the real OS filesystem\n" +
                "    root - see 'cd'. Candidly: the sandbox's root is the REAL project\n" +
                "    directory (see 'ls'), not a fully separate synthetic tree, so this\n" +
                "    project's own real source folders ('EmuSen.Hotaru', '.git', 'Man pages',\n" +
                "    'EmuSen.sln', ...) are still visible at the top level alongside the\n" +
                "    layout above - this shell was always meant to let you poke around the\n" +
                "    project's own files, not hide them.\n\n" +
                "EXAMPLES\n" +
                "    man hier\n" +
                "    ls /\n" +
                "    cd EmuSen.DianaOS/DianaOS/Usr/Home/Logs && ls",

            ["pwd"] =
                "NAME\n" +
                "    pwd - print the current directory\n\n" +
                "SYNOPSIS\n" +
                "    pwd\n\n" +
                "DESCRIPTION\n" +
                "    Prints the process's current working directory - see 'cd'.",

            ["cat"] =
                "NAME\n" +
                "    cat - print a file's contents\n\n" +
                "SYNOPSIS\n" +
                "    cat <path>...\n\n" +
                "DESCRIPTION\n" +
                "    Prints one or more real files' contents, concatenated in the order\n" +
                "    given, walled to the project's own directory tree (see 'cd'). With no\n" +
                "    <path> at all, passes piped stdin straight through unchanged instead -\n" +
                "    the classic 'cat' idiom for previewing a pipeline mid-stage - rather\n" +
                "    than erroring for a missing argument.\n\n" +
                "EXAMPLES\n" +
                "    cat notes.txt\n" +
                "    watch log 1 | cat",

            ["head"] =
                "NAME\n" +
                "    head - print the first lines of a file or stdin\n\n" +
                "SYNOPSIS\n" +
                "    head [-n N] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the first N lines (default 10) of piped stdin, or a real file if\n" +
                "    <path> is given (walled to the project's own directory tree - see 'cd').\n\n" +
                "EXAMPLES\n" +
                "    history | head -n 5\n" +
                "    head notes.txt",

            ["tail"] =
                "NAME\n" +
                "    tail - print the last lines of a file or stdin\n\n" +
                "SYNOPSIS\n" +
                "    tail [-n N] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the last N lines (default 10) of piped stdin, or a real file if\n" +
                "    <path> is given (walled to the project's own directory tree - see 'cd').\n" +
                "    No '-f' follow mode - this shell has nothing that appends to a file\n" +
                "    live while 'tail' is running to follow.\n\n" +
                "EXAMPLES\n" +
                "    history | tail -n 5\n" +
                "    tail notes.txt",

            ["touch"] =
                "NAME\n" +
                "    touch - create an empty file, or update its modified time\n\n" +
                "SYNOPSIS\n" +
                "    touch <path>\n\n" +
                "DESCRIPTION\n" +
                "    Creates <path> as an empty file if it doesn't exist yet, or just updates\n" +
                "    an existing file's last-modified time otherwise - walled to the\n" +
                "    project's own directory tree (see 'cd'). Refuses a path that's already\n" +
                "    a directory.\n\n" +
                "EXAMPLES\n" +
                "    touch scratch.txt",

            ["find"] =
                "NAME\n" +
                "    find - recursively list files and directories\n\n" +
                "SYNOPSIS\n" +
                "    find [path] [-name pattern]\n\n" +
                "DESCRIPTION\n" +
                "    Walks <path> (default: current directory) and everything under it,\n" +
                "    printing one entry per line, each path shown relative the same way it\n" +
                "    was given (e.g. 'find EmuSen' prints 'EmuSen/...' paths). -name filters\n" +
                "    to entries whose own name matches a glob ('*'/'?' wildcards only, no\n" +
                "    regex) - a directory that doesn't match is still walked into, only\n" +
                "    excluded from the printed list itself, matching real find's own\n" +
                "    behavior. Walled to the project's own directory tree (see 'cd').\n\n" +
                "EXAMPLES\n" +
                "    find\n" +
                "    find EmuSen.DianaOS -name '*Command.cs'",

            ["xxd"] =
                "NAME\n" +
                "    xxd, hexdump - hexdump a real file's raw bytes\n\n" +
                "SYNOPSIS\n" +
                "    xxd <path>\n" +
                "    hexdump <path>\n\n" +
                "DESCRIPTION\n" +
                "    Reads <path>'s REAL bytes off disk (not decoded as text the way\n" +
                "    'cat'/'head'/'tail' do, which would corrupt anything that isn't valid\n" +
                "    UTF8) and prints them in the same address/hex/ASCII layout 'mem' uses\n" +
                "    for a live memory space - the on-disk counterpart to 'mem', purpose-\n" +
                "    built so a 'dump'd capture can be inspected without leaving the shell\n" +
                "    for an external hex editor. Walled to the project's own directory tree\n" +
                "    (see 'cd'). With no <path>, hexdumps piped stdin instead - UTF8-encoded\n" +
                "    first, since a pipeline stage's own output is always already-decoded\n" +
                "    text here, never raw bytes (this shell's pipes have no byte-stream\n" +
                "    concept the way a real Unix pipe does). 'hexdump' is a plain alias, not\n" +
                "    a separate command - and NOT real hexdump's own multi-format output,\n" +
                "    just this same one layout under a second, equally-reached-for name.\n\n" +
                "EXAMPLES\n" +
                "    dump WRAM 0 256 wram.bin\n" +
                "    xxd EmuSen.DianaOS/DianaOS/Usr/Home/Logs/SNES/wram.bin\n" +
                "    echo hi | xxd",

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
                "    test) or when there's no real terminal at all (EmuSen.Mistress's GUI\n" +
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
                "    In EmuSen.Mistress's own console window (a TextBox, not a real\n" +
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
                "    In EmuSen.Mistress's own console window (a TextBox, not a real\n" +
                "    terminal - the above wouldn't work there at all), 'coretop' is replaced\n" +
                "    with a windowed equivalent instead: a real, non-blocking Avalonia window\n" +
                "    showing the same data as actual widgets and live images (the palette and\n" +
                "    VRAM tile sheet render as real pictures there, not ANSI blocks) that\n" +
                "    refreshes on its own timer while gameplay keeps running - Ctrl+C doesn't\n" +
                "    apply there, just close the window.\n\n" +
                "    -w  Open the same dashboard in a separate window instead of taking over\n" +
                "        the terminal, so you can dismiss it and keep playing. In\n" +
                "        EmuSen.Mistress, coretop is already windowed by default, so -w is\n" +
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
                "    mv scratch.txt EmuSen.DianaOS/DianaOS/Usr/Home/Logs/",

            ["cp"] =
                "NAME\n" +
                "    cp - copy a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    cp <src> <dst>\n" +
                "    cp -r <src> <dst>\n\n" +
                "DESCRIPTION\n" +
                "    Copies a real file. If <dst> is an existing directory, <src> lands\n" +
                "    inside it ('cp foo.txt logs/' -> 'logs/foo.txt'), matching real cp; an\n" +
                "    existing destination FILE is silently overwritten (no -i prompt anywhere\n" +
                "    in this shell, same as 'mv'). A directory requires -r, which copies it\n" +
                "    and everything inside it. Both <src> and <dst> must resolve inside the\n" +
                "    project's own directory tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    cp notes.txt notes.bak.txt\n" +
                "    cp -r EmuSen.DianaOS/DianaOS/Usr/Home/Logs/Run1 EmuSen.DianaOS/DianaOS/Usr/Home/Logs/Run1Backup",

            ["rm"] =
                "NAME\n" +
                "    rm - delete a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    rm <path>\n" +
                "    rm -r <path>\n\n" +
                "DESCRIPTION\n" +
                "    Deletes a real file. A directory requires -r (matching real rm's own\n" +
                "    refusal to remove a directory without it); with -r, the directory and\n" +
                "    everything inside it is deleted, no further confirmation (no -i prompt\n" +
                "    anywhere in this shell, by design, same as 'mv'). Must resolve inside\n" +
                "    the project's own directory tree - see 'cd' - and refuses to remove that\n" +
                "    root directory itself even with -r, so 'rm -r .' from the project root\n" +
                "    (or 'rm -r' on the root's own path) can't wipe out the whole project.\n" +
                "    Like 'cd', rm only ever takes one path, so a directory name with spaces\n" +
                "    doesn't need quoting ('rm -r My Folder' works) - everything after the\n" +
                "    optional -r is treated as the literal path.\n\n" +
                "EXAMPLES\n" +
                "    rm scratch.txt\n" +
                "    rm -r EmuSen.DianaOS/DianaOS/Usr/Home/Logs/OldRun\n" +
                "    rm -r My Folder",

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

            ["state"] =
                "NAME\n" +
                "    state - save or load emulator state\n\n" +
                "SYNOPSIS\n" +
                "    state save [path]\n" +
                "    state load [path]\n\n" +
                "DESCRIPTION\n" +
                "    Saves or loads a full snapshot of emulator state (not battery-backed\n" +
                "    cartridge SRAM, which persists separately) - the same operation each\n" +
                "    frontend's own Save State/Load State hotkey or menu item performs.\n" +
                "    'path' defaults to this frontend's own save-state path (one slot per\n" +
                "    ROM) if omitted.\n\n" +
                "EXAMPLES\n" +
                "    state save\n" +
                "    state load\n" +
                "    state save EmuSen.DianaOS/DianaOS/Usr/Home/Saves/before-boss.state",

            ["resume"] =
                "NAME\n" +
                "    resume - resume emulation after an interactive halt\n\n" +
                "SYNOPSIS\n" +
                "    resume\n" +
                "    continue\n" +
                "    c\n\n" +
                "DESCRIPTION\n" +
                "    Resumes emulation after a halt (an interactive debug prompt, a\n" +
                "    breakpoint, a single-step) - 'continue' and 'c' are recognized as plain\n" +
                "    aliases, not separate commands. EmuSen.Mistress registers its own\n" +
                "    pause/resume-aware 'resume' instead (its console window runs on a\n" +
                "    separate thread from emulation, unlike EmuSen.Hotaru's console, which\n" +
                "    shares the emulation thread and needs no pause/resume signal at all).\n\n" +
                "EXAMPLES\n" +
                "    resume\n" +
                "    c",

            ["shutdown"] =
                "NAME\n" +
                "    shutdown - terminate the process\n\n" +
                "SYNOPSIS\n" +
                "    shutdown\n" +
                "    quit\n\n" +
                "DESCRIPTION\n" +
                "    Terminates the whole process - 'quit' is recognized as a plain alias,\n" +
                "    not a separate command. Works the same whether typed at the initial\n" +
                "    no-ROM shell or the in-game debug prompt.\n\n" +
                "EXAMPLES\n" +
                "    shutdown\n" +
                "    quit",

            ["core"] =
                "NAME\n" +
                "    core - load or swap the running ROM\n\n" +
                "SYNOPSIS\n" +
                "    core <corename> <path>\n\n" +
                "DESCRIPTION\n" +
                "    Validates <corename> against the known core registry and <path> against\n" +
                "    that core's supported file extensions, then loads it - reloading\n" +
                "    VenusCore in place if a ROM is already running (a fresh SnesDebugTarget/\n" +
                "    DianaOSInterpreter/FrameRecorder get rebuilt afterward; watches/\n" +
                "    breakpoints/cheats registered against the OLD ROM don't survive a swap,\n" +
                "    matching EmuSen.Mistress's own already-accepted 'shell-level state\n" +
                "    resets on reload' convention). Refuses to swap while a frame recording\n" +
                "    is in progress - stop it first (F6). Only registered where a session\n" +
                "    actually exists to reload - EmuSen.Hotaru's own pre-window launch shell\n" +
                "    has a separate, simpler 'core' handler of its own for resolving the\n" +
                "    FIRST ROM (see EmuSen_Frontend_Driver.md §3), since there's nothing to\n" +
                "    reload yet at that point.\n\n" +
                "EXAMPLES\n" +
                "    core venus game2.smc\n" +
                "    core snes /path/to/other-game.sfc",

            ["tmux"] =
                "NAME\n" +
                "    tmux - multiple independent shell sessions\n\n" +
                "SYNOPSIS\n" +
                "    tmux new [name]\n" +
                "    tmux list\n" +
                "    tmux switch <name>\n" +
                "    tmux kill <name>\n\n" +
                "DESCRIPTION\n" +
                "    Each session is a fully independent DianaOSInterpreter - its own\n" +
                "    variables, command history, and $?/pending-multiline-input state -\n" +
                "    that you can create and switch between. 'new' with no name auto-numbers\n" +
                "    it (session-1, session-2, ...). All sessions observe the same one live\n" +
                "    core/ROM, the same way several real tmux windows attached to one real\n" +
                "    machine each get their own independent shell but see the same one\n" +
                "    machine - this is NOT multiple games running at once, just multiple\n" +
                "    independent shells watching the one that is.\n\n" +
                "    Because a DianaOSInterpreter can't be repointed at a new target once\n" +
                "    built (see 'core'), swapping the ROM rebuilds every session's\n" +
                "    interpreter from scratch, the same 'shell-level state resets on reload'\n" +
                "    convention 'core' already documents - it's not just the current session\n" +
                "    that loses its variables/history on a ROM swap, all of them do.\n\n" +
                "    Not available everywhere: a host needs to opt in by handing\n" +
                "    DianaOSInterpreter.CreateDefault a DianaOSSessionManager. Where it IS\n" +
                "    available, whether switching sessions is itself safe against a\n" +
                "    concurrently-running emulation thread is that host's own concern, not\n" +
                "    this command's - EmuSen.Hotaru/EmuSen.Mistress each keep a separate\n" +
                "    DianaOSInterpreterScheduler per session for exactly that reason; the\n" +
                "    standalone DianaOS shell (no core, no concurrent thread) needs none at\n" +
                "    all.\n\n" +
                "EXAMPLES\n" +
                "    tmux new\n" +
                "    tmux new investigation\n" +
                "    tmux list\n" +
                "    tmux switch investigation\n" +
                "    tmux kill session-1",

            ["ps"] =
                "NAME\n" +
                "    ps, jobs - list active breakpoints, watches, and sessions\n\n" +
                "SYNOPSIS\n" +
                "    ps\n" +
                "    jobs\n\n" +
                "DESCRIPTION\n" +
                "    Lists every active breakpoint (see 'bp'), watch (see 'watch'), and tmux\n" +
                "    session (see 'tmux', if a session manager is registered for this host)\n" +
                "    as one unified table - the read-only counterpart to 'kill'. 'jobs' is a\n" +
                "    plain alias, not a separate command. Each row's id column is exactly\n" +
                "    what 'kill' expects: 'bp<N>' for a breakpoint, 'watch<N>' for a watch, or\n" +
                "    a session's own name - copy a row's id straight into 'kill' unchanged.\n" +
                "    Prints nothing wrong, just an empty-looking table, with no ROM loaded and\n" +
                "    no session manager registered (there's nothing to list, not an error).\n\n" +
                "EXAMPLES\n" +
                "    ps\n" +
                "    jobs",

            ["kill"] =
                "NAME\n" +
                "    kill - remove a breakpoint, watch, or session by its 'ps' id\n\n" +
                "SYNOPSIS\n" +
                "    kill <id>\n\n" +
                "DESCRIPTION\n" +
                "    One verb for tearing down whatever 'ps' just showed, instead of needing\n" +
                "    to know which of 'bp remove'/'watch remove'/'tmux kill' owns a given id -\n" +
                "    all three of those still work exactly as before, this just routes to\n" +
                "    whichever one already owns <id>'s kind of thing, based on its shape:\n" +
                "    'bp<N>' removes breakpoint N, 'watch<N>' removes watch N, and anything\n" +
                "    else is looked up as a session name (same rules as 'tmux kill' - can't\n" +
                "    kill the only remaining session; killing the CURRENT session switches you\n" +
                "    to whichever one is left, the same hand-off 'tmux kill' already does).\n" +
                "    A session literally NAMED like 'bp3' or 'watch1' is only reachable this\n" +
                "    way by accident of that ambiguity - 'kill' always tries the breakpoint/\n" +
                "    watch reading of an id shaped like one first; use 'tmux kill <name>'\n" +
                "    directly for a session whose name collides with that shape.\n\n" +
                "EXAMPLES\n" +
                "    kill bp1\n" +
                "    kill watch3\n" +
                "    kill investigation",

            ["whoami"] =
                "NAME\n" +
                "    whoami - print the account this shell is running as\n\n" +
                "SYNOPSIS\n" +
                "    whoami\n\n" +
                "DESCRIPTION\n" +
                "    Prints THIS shell's own current account (see 'su') - every shell starts\n" +
                "    as 'root'. Unix flavor, not real access control - see 'su's own man page\n" +
                "    for what that means today and what's still future work.\n\n" +
                "EXAMPLES\n" +
                "    whoami",

            ["who"] =
                "NAME\n" +
                "    who - list every session and which account it's running as\n\n" +
                "SYNOPSIS\n" +
                "    who\n\n" +
                "DESCRIPTION\n" +
                "    A different axis from 'ps': 'ps' lists killable THINGS (breakpoints/\n" +
                "    watches/sessions); this lists WHO is running each session, if a session\n" +
                "    manager is registered for this host (see 'tmux'). With none registered,\n" +
                "    just reports this one shell's own account instead of failing.\n\n" +
                "EXAMPLES\n" +
                "    who",

            ["su"] =
                "NAME\n" +
                "    su - switch this shell's active account\n\n" +
                "SYNOPSIS\n" +
                "    su [name]\n\n" +
                "DESCRIPTION\n" +
                "    Switches THIS shell (this tmux session, if any - see 'tmux') to <name>,\n" +
                "    or back to 'root' with no argument, matching real su's own default\n" +
                "    target. <name> must already exist (see 'useradd'). Does NOT check\n" +
                "    'passwd's stored password hash - nothing in this shell enforces\n" +
                "    anything based on which account is active yet, so su succeeds\n" +
                "    unconditionally for any existing account. That enforcement (e.g. a\n" +
                "    restricted account that can't run mutating commands) is real future\n" +
                "    work, not yet built - this pass is the identity/account plumbing it\n" +
                "    will eventually sit on top of, nothing more.\n\n" +
                "    A new session created with 'tmux new' inherits whoever created it\n" +
                "    (not always 'root') and a ROM swap preserves each session's own current\n" +
                "    account across the rebuild - see 'tmux'/'core'.\n\n" +
                "EXAMPLES\n" +
                "    su\n" +
                "    su parent",

            ["useradd"] =
                "NAME\n" +
                "    useradd - add a new account\n\n" +
                "SYNOPSIS\n" +
                "    useradd <name>\n\n" +
                "DESCRIPTION\n" +
                "    Adds <name> to the process-wide account directory shared by every tmux\n" +
                "    session (like a real /etc/passwd) - root only. In-memory only, like\n" +
                "    every other piece of DianaOS state; accounts don't survive a process\n" +
                "    restart yet. See 'su' to switch to the new account.\n\n" +
                "EXAMPLES\n" +
                "    useradd kid",

            ["userdel"] =
                "NAME\n" +
                "    userdel - remove an account\n\n" +
                "SYNOPSIS\n" +
                "    userdel <name>\n\n" +
                "DESCRIPTION\n" +
                "    Removes <name> from the account directory - root only. Refuses to\n" +
                "    remove 'root' itself (the one account that always exists), and refuses\n" +
                "    to remove an account that's currently active in any live session (or\n" +
                "    this shell itself, with no session manager registered) - 'su' that\n" +
                "    session to another account first.\n\n" +
                "EXAMPLES\n" +
                "    userdel kid",

            ["passwd"] =
                "NAME\n" +
                "    passwd - set an account's password\n\n" +
                "SYNOPSIS\n" +
                "    passwd <newpassword>\n" +
                "    passwd <name> <newpassword>\n\n" +
                "DESCRIPTION\n" +
                "    Stores a hash of <newpassword> for your OWN account (no other arguments)\n" +
                "    or, root only, for <name>'s account. NOT checked by 'su' or anything\n" +
                "    else today - see that command's own man page for why. No masked/\n" +
                "    interactive prompt either: there's no reliable blocking-input read\n" +
                "    across every host this shell runs under (a real terminal, a GUI\n" +
                "    TextBox-driven console, a headless test), so the new password is just a\n" +
                "    plain trailing argument, the same way 'bp add'/'watch add' take theirs -\n" +
                "    it will show up in this shell's own 'history' in plain text.\n\n" +
                "EXAMPLES\n" +
                "    passwd hunter2\n" +
                "    passwd kid hunter2",

            ["step"] =
                "NAME\n" +
                "    step - single-step one CPU instruction\n\n" +
                "SYNOPSIS\n" +
                "    step\n" +
                "    s\n\n" +
                "DESCRIPTION\n" +
                "    Arms a one-shot halt-before-next-instruction and lets emulation run just\n" +
                "    long enough for exactly one CPU instruction to execute, then halts again\n" +
                "    at the same interactive prompt - 's' is recognized as a plain alias, not\n" +
                "    a separate command. Needs a ROM loaded (an active debug target).\n\n" +
                "EXAMPLES\n" +
                "    step\n" +
                "    s",
        };
    }
}
