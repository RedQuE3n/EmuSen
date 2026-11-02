# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardSave-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PauseCardSave
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 AdaptiveLayout at=0,0,1280,800 by=panel heard=Pause limit=0,0,1280,800 name=PadCard role=pane
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,800 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name=AdaptiveCard role=menu
            5 HintBar at=64,736,744,30 by=panel heard="A Select, B Resume, D-pad left and right Change slot, X Screenshot, Menu Resume" limit=0,0,1280,800 role=group
            6 Border at=64,156,424,378 by=panel limit=0,0,1280,800 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,378 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,378 by=panel heard=Menu limit=0,0,1280,800 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,378
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard=Resume in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard=Rewind in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard="Save State" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                    13 ListBoxItem at=64,318,424,54 by=fixed-scroll drawn focus heard="Load State" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                    14 ListBoxItem at=64,372,424,54 by=fixed-scroll drawn focus heard=Speed in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                    15 ListBoxItem at=64,426,424,54 by=fixed-scroll drawn focus heard="Game Options" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                    16 ListBoxItem at=64,480,424,54 by=fixed-scroll drawn focus heard=Leave in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
            17 Border at=576,156,640,530 by=panel limit=0,0,1280,800 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=600,180,592,482 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                19 PauseCardPane at=600,180,592,233 by=panel limit=0,0,1280,800 name=PauseCardPane role=none
                  20 StackPanel at=600,180,592,233 by=panel limit=0,0,1280,800 role=none
                    21 StackPanel at=600,180,592,194 by=panel limit=0,0,1280,800 name=PaneSlots role=none
                      22 StackPanel at=600,180,184,194 by=panel limit=0,0,1280,800 role=none
                        23 Border at=600,180,184,138 by=panel limit=0,0,1280,800 name=PaneSlot1 paints=#4C9AE8FF role=none
                          24 Panel at=603,183,178,132 by=clip limit=600,180,184,138 role=none
                            25 Rectangle at=603,183,178,132 by=clip limit=600,180,184,138 role=none
                            26 TextBlock at=603,235,178,27 box=178,27 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=60.48,26.4 limit=600,180,184,138 paints=#8D8D91FF role=text size=22 text=Empty
                        27 TextBlock at=600,322,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 1" layout=42.68,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text="Slot 1"
                        28 TextBlock at=600,350,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=Empty
                      29 StackPanel at=804,180,184,194 by=panel limit=0,0,1280,800 role=none
                        30 Border at=804,180,184,138 by=panel limit=0,0,1280,800 name=PaneSlot2 paints=#4C9AE8FF role=none
                          31 Panel at=804,180,184,138 by=clip limit=804,180,184,138 role=none
                            32 Rectangle at=804,180,184,138 by=clip limit=804,180,184,138 role=none
                            33 TextBlock at=804,236,184,27 box=184,27 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=60.48,26.4 limit=804,180,184,138 paints=#8D8D91FF role=text size=22 text=Empty
                        34 TextBlock at=804,322,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 2" layout=46.52,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text="Slot 2"
                        35 TextBlock at=804,350,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=Empty
                      36 StackPanel at=1008,180,184,194 by=panel limit=0,0,1280,800 role=none
                        37 Border at=1008,180,184,138 by=panel limit=0,0,1280,800 name=PaneSlot3 paints=#4C9AE8FF role=none
                          38 Panel at=1008,180,184,138 by=clip limit=1008,180,184,138 role=none
                            39 Rectangle at=1008,180,184,138 by=clip limit=1008,180,184,138 role=none
                            40 TextBlock at=1008,236,184,27 box=184,27 by=clip caps=15.3989 drawn face=Barlow heard=Empty layout=60.48,26.4 limit=1008,180,184,138 paints=#8D8D91FF role=text size=22 text=Empty
                        41 TextBlock at=1008,322,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard="Slot 3" layout=46.3,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text="Slot 3"
                        42 TextBlock at=1008,350,184,24 box=184,24 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=54.98,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=Empty
                    43 TextBlock at=600,386,592,27 box=592,27 by=panel caps=15.3989 drawn face=Barlow heard="A saves to Slot 1." layout=150.26,26.4 limit=0,0,1280,800 name=PaneSentence paints=#D4D4D4FF role=text size=22 text="A saves to Slot 1."
focus none
press down menu="open at 3: Load      <  Slot 1 (Empty)  >" top=none
press up menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press back top=none
