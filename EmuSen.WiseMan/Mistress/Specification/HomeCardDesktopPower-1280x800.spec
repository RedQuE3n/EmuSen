# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopPower-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,768
case HomeCardDesktopPower
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 role=none
    1 AdaptiveLayout at=0,0,1280,768 by=panel heard=Home limit=0,0,1280,768 name=PadCard role=pane
      2 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,768 by=panel heard=Power limit=0,0,1280,768 name=AdaptiveCard role=menu
            5 HintBar at=64,684,350,30 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1280,768 role=group
            6 Border at=64,156,424,162 by=panel limit=0,0,1280,768 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,162 by=panel limit=0,0,1280,768 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,162 by=panel heard=Menu limit=0,0,1280,768 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,162 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,162
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard="Full Screen" in=9 limit=64,156,424,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Full Screen"
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard="Big Picture" in=9 limit=64,156,424,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Big Picture"
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard="Quit EmuSen..." in=9 limit=64,156,424,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Quit EmuSen..."
focus none
press down menu="open at 1: Big Picture" top=none
press up menu="open at 0: Full Screen" top=none
press back menu="open at 6: Power" top=none
