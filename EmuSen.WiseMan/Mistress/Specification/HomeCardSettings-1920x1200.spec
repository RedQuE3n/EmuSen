# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardSettings-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case HomeCardSettings
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Home limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard=Settings limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,525,45 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,648 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,648 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,648
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard="Players & Controllers" in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text="Players & Controllers"
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard="Controller Bindings" in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text="Controller Bindings"
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard=Graphics in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text=Graphics
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard=Shaders in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text=Shaders
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard=Cheats in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text=Cheats
                    15 ListBoxItem at=96,639,636,81 by=fixed-scroll drawn focus heard="Memory Cards" in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text="Memory Cards"
                    16 ListBoxItem at=96,720,636,81 by=fixed-scroll drawn focus heard=Firmware in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text=Firmware
                    17 ListBoxItem at=96,801,636,81 by=fixed-scroll drawn focus heard=Preferences in=9 limit=96,234,636,648 op paints=#333337FF,#D4D4D4FF role=listitem text=Preferences
focus none
press down menu="open at 1: Controller Bindings" top=none
press up menu="open at 0: Players & Controllers" top=none
press back menu="open at 5: Settings" top=none
