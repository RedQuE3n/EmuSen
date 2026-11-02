# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardSave-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PauseCardSave
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Pause limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,1115,45 by=panel heard="A Select, B Resume, D-pad left and right Change slot, X Screenshot, Menu Resume" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,567 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,567
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard=Resume in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard=Rewind in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard="Save State" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard="Load State" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard=Speed in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                    15 ListBoxItem at=96,639,636,81 by=fixed-scroll drawn focus heard="Game Options" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                    16 ListBoxItem at=96,720,636,81 by=fixed-scroll drawn focus heard=Leave in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
            17 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                19 PauseCardPane at=900,270,888,349 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  20 StackPanel at=900,270,888,349 by=panel limit=0,0,1920,1200 role=none
                    21 StackPanel at=900,270,888,291 by=panel limit=0,0,1920,1200 name=PaneSlots role=none
                      22 StackPanel at=900,270,276,291 by=panel limit=0,0,1920,1200 role=none
                        23 Border at=900,270,276,207 by=panel limit=0,0,1920,1200 name=PaneSlot1 paints=#4C9AE8FF role=none
                          24 Panel at=904,274,268,199 by=clip limit=900,270,276,207 role=none
                            25 Rectangle at=904,274,268,199 by=clip limit=900,270,276,207 role=none
                            26 TextBlock at=904,354,268,40 box=268,40 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=90.72,39.6 limit=900,270,276,207 paints=#8D8D91FF role=text size=22 text=Empty
                        27 TextBlock at=900,483,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 1" layout=64.02,36 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text="Slot 1"
                        28 TextBlock at=900,525,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=Empty
                      29 StackPanel at=1206,270,276,291 by=panel limit=0,0,1920,1200 role=none
                        30 Border at=1206,270,276,207 by=panel limit=0,0,1920,1200 name=PaneSlot2 paints=#4C9AE8FF role=none
                          31 Panel at=1206,270,276,207 by=clip limit=1206,270,276,207 role=none
                            32 Rectangle at=1206,270,276,207 by=clip limit=1206,270,276,207 role=none
                            33 TextBlock at=1206,354,276,40 box=276,40 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=90.72,39.6 limit=1206,270,276,207 paints=#8D8D91FF role=text size=22 text=Empty
                        34 TextBlock at=1206,483,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 2" layout=69.78,36 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text="Slot 2"
                        35 TextBlock at=1206,525,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=Empty
                      36 StackPanel at=1512,270,276,291 by=panel limit=0,0,1920,1200 role=none
                        37 Border at=1512,270,276,207 by=panel limit=0,0,1920,1200 name=PaneSlot3 paints=#4C9AE8FF role=none
                          38 Panel at=1512,270,276,207 by=clip limit=1512,270,276,207 role=none
                            39 Rectangle at=1512,270,276,207 by=clip limit=1512,270,276,207 role=none
                            40 TextBlock at=1512,354,276,40 box=276,40 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=90.72,39.6 limit=1512,270,276,207 paints=#8D8D91FF role=text size=22 text=Empty
                        41 TextBlock at=1512,483,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 3" layout=69.45,36 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text="Slot 3"
                        42 TextBlock at=1512,525,276,36 box=276,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=Empty
                    43 TextBlock at=900,579,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="A saves to Slot 1." layout=225.39,39.6 limit=0,0,1920,1200 name=PaneSentence paints=#D4D4D4FF role=text size=22 text="A saves to Slot 1."
focus none
press down menu="open at 3: Load      <  Slot 1 (Empty)  >" top=none
press up menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press back top=none
