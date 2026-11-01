# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Shell-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Shell
case Shell
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 DockPanel at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
      2 ToolBar at=258,136,765,53 by=panel limit=234,124,812.35,548 name=HotaruToolbar paints=#707070FF,#D4D4D4FF role=toolbar
        3 ActionButton at=262,140,78,44 by=clip drawn focus heard=BACK limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Back
          4 TextBlock at=279,146,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=BACK layout=43.89,31.2 limit=262,140,78,44 owner=3 paints=#EEEEF0FF role=text size=26 text=BACK
        5 ActionButton at=344,140,115,44 by=clip drawn focus heard=FORWARD limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Forward
          6 TextBlock at=361,146,81,32 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FORWARD layout=80.99,31.2 limit=344,140,115,44 owner=5 paints=#D4D4D4FF role=text size=26 text=FORWARD
        7 ActionButton at=463,140,121,44 by=clip drawn focus heard=COMPLETE limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Complete
          8 TextBlock at=480,146,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COMPLETE layout=86.61,31.2 limit=463,140,121,44 owner=7 paints=#D4D4D4FF role=text size=26 text=COMPLETE
        9 ActionButton at=588,140,88,44 by=clip drawn focus heard=CLEAR limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Clear
          10 TextBlock at=605,146,54,32 box=54,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR layout=53.17,31.2 limit=588,140,88,44 owner=9 paints=#D4D4D4FF role=text size=26 text=CLEAR
        11 ActionButton at=680,140,78,44 by=clip drawn focus heard=COPY limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Copy
          12 TextBlock at=697,146,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COPY layout=43.03,31.2 limit=680,140,78,44 owner=11 paints=#D4D4D4FF role=text size=26 text=COPY
        13 ActionButton at=762,140,126,44 by=clip drawn focus heard=INTERRUPT limit=258,136,765,53 op paints=#707070FF,#D4D4D4FF role=button text=Interrupt
          14 TextBlock at=779,146,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INTERRUPT layout=91.81,31.2 limit=762,140,126,44 owner=13 paints=#D4D4D4FF role=text size=26 text=INTERRUPT
      15 HintText at=258,648,765,24 box=765,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes" layout=443.3,24 limit=234,124,812.35,548 name=HotaruHint paints=#8D8D91FF role=text size=20 text="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes"
      16 ConsolePane at=266,197,749,443 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=group
        17 TextBlock at=266,598,128,33 box=128,33 by=clip caps=17.1387 drawn face="Noto Sans" heard="DianaSh #: " layout=121.66,32.69 limit=266,197,749,443 name=PART_Prompt paints=#D4D4D4FF part role=text size=24 text="DianaSh #: "
        18 TextBox at=398,590,617,50 drawn focus heard="Console input" name=PART_Input op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
          19 ScrollViewer allow="a text box scrolls its own text" at=409,597,595,36 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
            20 TextBlock allow="a text box scrolls its own text" at=409,597,595,36 drawn in=19 name=PART_Placeholder paints=#8D8D91FF part role=text
        21 ScrollViewer at=266,197,749,385 fades=b name=PART_Scroll overflow=760 paints=#707070FF,#D4D4D4FF,#FFFFFFFF part role=scrollviewer viewport=266,197,749,385
          22 SelectableTextBlock at=266,197,749,1145 box=749,1145 by=scroll caps=17.1387 drawn face="Noto Sans" heard="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each." in=21 layout=736.78,1144.08 limit=266,197,749,385 name=PART_Output paints=#D4D4D4FF part role=text seen=266,197,749,385 size=24 text="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each."
focus 3
edge 3 up same
edge 3 down 18
edge 3 left same
edge 3 right 5
edge 5 up same
edge 5 down 18
edge 5 left 3
edge 5 right 7
edge 7 up same
edge 7 down 18
edge 7 left 5
edge 7 right 9
edge 9 up same
edge 9 down 18
edge 9 left 7
edge 9 right 11
edge 11 up same
edge 11 down 18
edge 11 left 9
edge 11 right 13
edge 13 up same
edge 13 down 18
edge 13 left 11
edge 13 right same
edge 18 up 5
edge 18 down same
edge 18 left 3
edge 18 right same
press pageup focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press pagedown focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press first focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press last focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press back top=none
