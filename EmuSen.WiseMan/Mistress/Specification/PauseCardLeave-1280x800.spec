# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardLeave-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case PauseCardLeave
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 AdaptiveLayout at=0,0,1280,774 by=panel heard=Pause limit=0,0,1280,774 name=PadCard role=pane
      2 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1280,774 by=panel heard=Leave limit=0,0,1280,774 name=AdaptiveCard role=menu
          4 HintBar at=376,716,528,48 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot" limit=0,0,1280,774 role=group
          5 Border at=24,338,512,216 by=panel limit=0,0,1280,774 name=AdaptiveCardRows role=none
            6 Border at=24,338,512,216 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
              7 LunaList at=24,338,512,216 by=panel heard=Menu limit=0,0,1280,774 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=24,338,512,216 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,338,512,216
                  9 ListBoxItem at=24,338,512,54 by=fixed-scroll drawn focus heard="Back to Library" in=8 limit=24,338,512,216 op paints=#333337FF,#D4D4D4FF role=listitem text="Back to Library"
                  10 ListBoxItem at=24,392,512,54 by=fixed-scroll drawn focus heard="Restart Game..." in=8 limit=24,338,512,216 op paints=#333337FF,#D4D4D4FF role=listitem text="Restart Game..."
                  11 ListBoxItem at=24,446,512,54 by=fixed-scroll drawn focus heard="Quit Game..." in=8 limit=24,338,512,216 op paints=#333337FF,#D4D4D4FF role=listitem text="Quit Game..."
                  12 ListBoxItem at=24,500,512,54 by=fixed-scroll drawn focus heard="Quit EmuSen..." in=8 limit=24,338,512,216 op paints=#333337FF,#D4D4D4FF role=listitem text="Quit EmuSen..."
          13 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveCardPane role=none
            14 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Restart Game..." top=none
press up menu="open at 0: Back to Library" top=none
press back menu="open at 6: Leave" top=none
