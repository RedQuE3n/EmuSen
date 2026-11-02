# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopPower-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1146
case HomeCardDesktopPower
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1146 by=panel limit=0,0,1920,1146 role=none
    1 AdaptiveLayout at=0,0,1920,1146 by=panel heard=Home limit=0,0,1920,1146 name=PadCard role=pane
      2 Panel at=0,0,1920,1146 by=panel limit=0,0,1920,1146 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1146 by=panel heard=Power limit=0,0,1920,1146 name=AdaptiveCard role=menu
          4 HintBar at=682,1060,556,71 by=panel heard="A Select, B Back, Menu Close Menu" limit=0,0,1920,1146 role=group
          5 Border at=36,497,768,243 by=panel limit=0,0,1920,1146 name=AdaptiveCardRows role=none
            6 Border at=36,497,768,243 by=panel limit=0,0,1920,1146 name=AdaptiveContent role=none
              7 LunaList at=36,497,768,243 by=panel heard=Menu limit=0,0,1920,1146 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,497,768,243 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,497,768,243
                  9 ListBoxItem at=36,497,768,81 by=fixed-scroll drawn focus heard="Full Screen" in=8 limit=36,497,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Full Screen"
                  10 ListBoxItem at=36,578,768,81 by=fixed-scroll drawn focus heard="Big Picture" in=8 limit=36,497,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Big Picture"
                  11 ListBoxItem at=36,659,768,81 by=fixed-scroll drawn focus heard="Quit EmuSen..." in=8 limit=36,497,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Quit EmuSen..."
          12 Border at=840,36,1044,988 by=panel limit=0,0,1920,1146 name=AdaptiveCardPane role=none
            13 Border at=840,36,1044,988 by=panel limit=0,0,1920,1146 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Big Picture" top=none
press up menu="open at 0: Full Screen" top=none
press back menu="open at 6: Power" top=none
