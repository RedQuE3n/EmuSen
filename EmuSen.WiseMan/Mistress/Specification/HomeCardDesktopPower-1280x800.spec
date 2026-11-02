# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopPower-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,746
case HomeCardDesktopPower
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,746 by=panel limit=0,0,1280,746 role=none
    1 AdaptiveLayout at=0,0,1280,746 by=panel heard=Home limit=0,0,1280,746 name=PadCard role=pane
      2 Panel at=0,0,1280,746 by=panel limit=0,0,1280,746 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,746 by=panel limit=0,0,1280,746 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,746 by=panel heard=Power limit=0,0,1280,746 name=AdaptiveCard role=menu
            5 HintBar at=454,688,371,48 by=panel heard="A Select, B Back, Menu Close Menu" limit=0,0,1280,746 role=group
            6 Border at=24,322,512,162 by=panel limit=0,0,1280,746 name=AdaptiveCardRows role=none
              7 Border at=24,322,512,162 by=panel limit=0,0,1280,746 name=AdaptiveContent role=none
                8 LunaList at=24,322,512,162 by=panel heard=Menu limit=0,0,1280,746 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=24,322,512,162 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,322,512,162
                    10 ListBoxItem at=24,322,512,54 by=fixed-scroll drawn focus heard="Full Screen" in=9 limit=24,322,512,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Full Screen"
                    11 ListBoxItem at=24,376,512,54 by=fixed-scroll drawn focus heard="Big Picture" in=9 limit=24,322,512,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Big Picture"
                    12 ListBoxItem at=24,430,512,54 by=fixed-scroll drawn focus heard="Quit EmuSen..." in=9 limit=24,322,512,162 op paints=#333337FF,#D4D4D4FF role=listitem text="Quit EmuSen..."
            13 Border at=560,24,696,640 by=panel limit=0,0,1280,746 name=AdaptiveCardPane role=none
              14 Border at=560,24,696,640 by=panel limit=0,0,1280,746 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Big Picture" top=none
press up menu="open at 0: Full Screen" top=none
press back menu="open at 6: Power" top=none
