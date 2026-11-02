# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardDisplay-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case PauseCardDisplay
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Pause limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1161 by=panel heard=Display limit=0,0,1920,1161 name=AdaptiveCard role=menu
            5 HintBar at=564,1075,792,71 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot" limit=0,0,1920,1161 role=group
            6 Border at=36,548,768,243 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
              7 Border at=36,548,768,243 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                8 LunaList at=36,548,768,243 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=36,548,768,243 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,548,768,243
                    10 ListBoxItem at=36,548,768,81 by=fixed-scroll drawn focus heard=Graphics in=9 limit=36,548,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text=Graphics
                    11 ListBoxItem at=36,629,768,81 by=fixed-scroll drawn focus heard=Shaders in=9 limit=36,548,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text=Shaders
                    12 ListBoxItem at=36,710,768,81 by=fixed-scroll drawn focus heard="Crop This Game..." in=9 limit=36,548,768,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Crop This Game..."
            13 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
              14 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Shaders" top=none
press up menu="open at 0: Graphics" top=none
press back menu="open at 2: Display" top=none
