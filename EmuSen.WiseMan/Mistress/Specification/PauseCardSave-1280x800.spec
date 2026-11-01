# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardSave-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case PauseCardSave
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 AdaptiveLayout at=0,0,1280,774 by=panel heard=Pause limit=0,0,1280,774 name=PadCard role=pane
      2 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1280,774 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,774 name=AdaptiveCard role=menu
          4 HintBar at=318,716,645,48 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot, D-pad left and right Change" limit=0,0,1280,774 role=group
          5 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveCardRows role=none
            6 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
              7 LunaList at=24,238,512,378 by=panel heard=Menu limit=0,0,1280,774 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=24,238,512,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,238,512,378
                  9 ListBoxItem at=24,238,512,54 by=fixed-scroll drawn focus heard=Resume in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                  10 ListBoxItem at=24,292,512,54 by=fixed-scroll drawn focus heard=Rewind in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                  11 ListBoxItem at=24,346,512,54 by=fixed-scroll drawn focus heard=Save in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                  12 ListBoxItem at=24,400,512,54 by=fixed-scroll drawn focus heard=Load in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                  13 ListBoxItem at=24,454,512,54 by=fixed-scroll drawn focus heard=Speed in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                  14 ListBoxItem at=24,508,512,54 by=fixed-scroll drawn focus heard="Game Options" in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                  15 ListBoxItem at=24,562,512,54 by=fixed-scroll drawn focus heard=Leave in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
          16 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveCardPane role=none
            17 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
              18 PauseCardPane at=560,166,696,384 by=panel limit=0,0,1280,774 name=PauseCardPane role=none
                19 UniformGrid at=560,166,696,384 by=panel limit=0,0,1280,774 name=PaneSlots role=none
                  20 StackPanel at=568,174,158,176 by=panel limit=0,0,1280,774 role=none
                    21 Border at=567,174,160,120 by=panel limit=0,0,1280,774 name=PaneSlot1 paints=#0E0E10FF,#4C9AE8FF role=none
                    22 TextBlock at=568,298,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 1" layout=42.68,24 limit=0,0,1280,774 paints=#EEEEF0FF role=text size=20 text="Slot 1"
                    23 TextBlock at=568,326,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  24 StackPanel at=742,174,158,176 by=panel limit=0,0,1280,774 role=none
                    25 Border at=741,174,160,120 by=panel limit=0,0,1280,774 name=PaneSlot2 paints=#0E0E10FF,#707070FF role=none
                    26 TextBlock at=742,298,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 2" layout=46.52,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 2"
                    27 TextBlock at=742,326,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  28 StackPanel at=916,174,158,176 by=panel limit=0,0,1280,774 role=none
                    29 Border at=915,174,160,120 by=panel limit=0,0,1280,774 name=PaneSlot3 paints=#0E0E10FF,#707070FF role=none
                    30 TextBlock at=916,298,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 3" layout=46.3,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 3"
                    31 TextBlock at=916,326,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  32 StackPanel at=1090,174,158,176 by=panel limit=0,0,1280,774 role=none
                    33 Border at=1089,174,160,120 by=panel limit=0,0,1280,774 name=PaneSlot4 paints=#0E0E10FF,#707070FF role=none
                    34 TextBlock at=1090,298,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 4" layout=46.48,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 4"
                    35 TextBlock at=1090,326,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  36 StackPanel at=568,366,158,176 by=panel limit=0,0,1280,774 role=none
                    37 Border at=567,366,160,120 by=panel limit=0,0,1280,774 name=PaneSlot5 paints=#0E0E10FF,#707070FF role=none
                    38 TextBlock at=568,490,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 5" layout=46.54,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 5"
                    39 TextBlock at=568,518,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  40 StackPanel at=742,366,158,176 by=panel limit=0,0,1280,774 role=none
                    41 Border at=741,366,160,120 by=panel limit=0,0,1280,774 name=PaneSlot6 paints=#0E0E10FF,#707070FF role=none
                    42 TextBlock at=742,490,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 6" layout=46.34,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 6"
                    43 TextBlock at=742,518,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  44 StackPanel at=916,366,158,176 by=panel limit=0,0,1280,774 role=none
                    45 Border at=915,366,160,120 by=panel limit=0,0,1280,774 name=PaneSlot7 paints=#0E0E10FF,#707070FF role=none
                    46 TextBlock at=916,490,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 7" layout=45.76,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 7"
                    47 TextBlock at=916,518,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
                  48 StackPanel at=1090,366,158,176 by=panel limit=0,0,1280,774 role=none
                    49 Border at=1089,366,160,120 by=panel limit=0,0,1280,774 name=PaneSlot8 paints=#0E0E10FF,#707070FF role=none
                    50 TextBlock at=1090,490,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 8" layout=46.7,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="Slot 8"
                    51 TextBlock at=1090,518,158,24 box=158,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text=Empty
focus none
press down menu="open at 3: Load      <  Slot 1 (Empty)  >" top=none
press up menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press back top=none
