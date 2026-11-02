# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardSettings-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case HomeCardSettings
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 AdaptiveLayout at=0,0,1280,774 by=panel heard=Home limit=0,0,1280,774 name=PadCard role=pane
      2 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1280,774 by=panel heard=Settings limit=0,0,1280,774 name=AdaptiveCard role=menu
          4 HintBar at=454,716,371,48 by=panel heard="A Select, B Back, Menu Close Menu" limit=0,0,1280,774 role=group
          5 Border at=24,230,512,432 by=panel limit=0,0,1280,774 name=AdaptiveCardRows role=none
            6 Border at=24,230,512,432 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
              7 LunaList at=24,230,512,432 by=panel heard=Menu limit=0,0,1280,774 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=24,230,512,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,230,512,432
                  9 ListBoxItem at=24,230,512,54 by=fixed-scroll drawn focus heard="Players & Controllers" in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text="Players & Controllers"
                  10 ListBoxItem at=24,284,512,54 by=fixed-scroll drawn focus heard="Controller Bindings" in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text="Controller Bindings"
                  11 ListBoxItem at=24,338,512,54 by=fixed-scroll drawn focus heard=Graphics in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text=Graphics
                  12 ListBoxItem at=24,392,512,54 by=fixed-scroll drawn focus heard=Shaders in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text=Shaders
                  13 ListBoxItem at=24,446,512,54 by=fixed-scroll drawn focus heard=Cheats in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text=Cheats
                  14 ListBoxItem at=24,500,512,54 by=fixed-scroll drawn focus heard="Memory Cards" in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text="Memory Cards"
                  15 ListBoxItem at=24,554,512,54 by=fixed-scroll drawn focus heard=Firmware in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text=Firmware
                  16 ListBoxItem at=24,608,512,54 by=fixed-scroll drawn focus heard=Preferences in=8 limit=24,230,512,432 op paints=#333337FF,#D4D4D4FF role=listitem text=Preferences
          17 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveCardPane role=none
            18 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Controller Bindings" top=none
press up menu="open at 0: Players & Controllers" top=none
press back menu="open at 5: Settings" top=none
