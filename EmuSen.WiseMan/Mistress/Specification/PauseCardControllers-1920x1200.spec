# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardControllers-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PauseCardControllers
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Pause limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard=Controllers limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,790,45 by=panel heard="A Choose, D-pad left and right Value, B Back, X Screenshot" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,405 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,405 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,405 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,405 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,405
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard="Player 1" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Player 1"
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard="Player 2" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Player 2"
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard=Profile in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text=Profile
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard=Bindings in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text=Bindings
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard="Swap Players" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Swap Players"
            15 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              16 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                17 PauseCardPane at=900,270,888,129 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  18 StackPanel at=900,270,888,129 by=panel limit=0,0,1920,1200 name=PaneScreen role=none
                    19 TextBlock at=900,270,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="Player 1 plays on the Simulated pad, with the Default profile." layout=822.33,39.6 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Player 1 plays on the Simulated pad, with the Default profile."
                    20 TextBlock at=900,319,888,80 box=888,80 by=panel caps=15.3989 drawn face=Barlow heard="Left and Right pick another connected controller; Bindings opens the drawing for this console." layout=809.03,79.2 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=22 text="Left and Right pick another connected controller; Bindings opens the drawing for this console."
focus none
press down menu="open at 1: Player 2" top=none
press up menu="open at 0: Player 1" top=none
press back menu="open at 1: Controllers" top=none
