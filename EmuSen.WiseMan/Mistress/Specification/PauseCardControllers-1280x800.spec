# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardControllers-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PauseCardControllers
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 AdaptiveLayout at=0,0,1280,800 by=panel heard=Pause limit=0,0,1280,800 name=PadCard role=pane
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,800 by=panel heard=Controllers limit=0,0,1280,800 name=AdaptiveCard role=menu
            5 HintBar at=64,736,527,30 by=panel heard="A Choose, D-pad left and right Value, B Back, X Screenshot" limit=0,0,1280,800 role=group
            6 Border at=64,156,424,270 by=panel limit=0,0,1280,800 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,270 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,270 by=panel heard=Menu limit=0,0,1280,800 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,270 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,270
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard="Player 1" in=9 limit=64,156,424,270 op paints=#333337FF,#D4D4D4FF role=listitem text="Player 1"
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard="Player 2" in=9 limit=64,156,424,270 op paints=#333337FF,#D4D4D4FF role=listitem text="Player 2"
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard=Profile in=9 limit=64,156,424,270 op paints=#333337FF,#D4D4D4FF role=listitem text=Profile
                    13 ListBoxItem at=64,318,424,54 by=fixed-scroll drawn focus heard=Bindings in=9 limit=64,156,424,270 op paints=#333337FF,#D4D4D4FF role=listitem text=Bindings
                    14 ListBoxItem at=64,372,424,54 by=fixed-scroll drawn focus heard="Swap Players" in=9 limit=64,156,424,270 op paints=#333337FF,#D4D4D4FF role=listitem text="Swap Players"
            15 Border at=576,156,640,530 by=panel limit=0,0,1280,800 name=AdaptiveCardPane paints=#26262BFF role=none
              16 Border at=600,180,592,482 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                17 PauseCardPane at=600,180,592,86 by=panel limit=0,0,1280,800 name=PauseCardPane role=none
                  18 StackPanel at=600,180,592,86 by=panel limit=0,0,1280,800 name=PaneScreen role=none
                    19 TextBlock at=600,180,592,27 box=592,27 by=panel caps=15.3989 drawn face=Barlow heard="Player 1 plays on the Simulated pad, with the Default profile." layout=548.22,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Player 1 plays on the Simulated pad, with the Default profile."
                    20 TextBlock at=600,213,592,53 box=592,53 by=panel caps=15.3989 drawn face=Barlow heard="Left and Right pick another connected controller; Bindings opens the drawing for this console." layout=539.35,52.8 limit=0,0,1280,800 paints=#8D8D91FF role=text size=22 text="Left and Right pick another connected controller; Bindings opens the drawing for this console."
focus none
press down menu="open at 1: Player 2" top=none
press up menu="open at 0: Player 1" top=none
press back menu="open at 1: Controllers" top=none
