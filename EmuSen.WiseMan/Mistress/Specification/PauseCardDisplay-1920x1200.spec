# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardDisplay-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PauseCardDisplay
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Pause limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard=Display limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,792,45 by=panel heard="A Change, D-pad left and right Value, B Back, X Screenshot" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,405 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,405 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,405 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,405 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,405
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard="Picture shape" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Picture shape"
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard="CRT shader" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="CRT shader"
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard="Crop overscan" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Crop overscan"
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard="This game's crop" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="This game's crop"
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard="Full screen" in=9 limit=96,234,636,405 op paints=#333337FF,#D4D4D4FF role=listitem text="Full screen"
            15 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              16 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                17 PauseCardPane at=900,270,888,593 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  18 StackPanel at=900,270,888,593 by=panel limit=0,0,1920,1200 name=PaneScreen role=none
                    19 Border at=900,270,888,495 by=panel limit=0,0,1920,1200 name=PaneFrame paints=#000000FF role=none
                      20 Image at=1061,270,565.71,495 by=clip drawn limit=900,270,888,495 role=image
                    21 TextBlock at=900,783,888,80 box=888,80 by=panel caps=15.3989 drawn face=Barlow heard="The Super Nintendo's screen shape, for every Super Nintendo game." layout=847.28,79.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="The Super Nintendo's screen shape, for every Super Nintendo game."
focus none
press down menu="open at 1: CRT shader" top=none
press up menu="open at 0: Picture shape" top=none
press back menu="open at 2: Display" top=none
