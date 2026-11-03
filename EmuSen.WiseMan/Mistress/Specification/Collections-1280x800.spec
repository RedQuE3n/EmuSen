# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Collections-1280x800
size 1280 800
kind menu
root SheetLayer#Sheets
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Collections
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Border at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
        3 CollectionsPane at=48,28,1184,732 by=panel heard=Collections limit=0,0,1280,800 role=pane
          4 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
            5 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
              6 Border at=48,28,1184,60 by=panel heard=COLLECTIONS limit=0,0,1280,800 name=AdaptiveHeader role=none
                7 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                  8 TextBlock at=1040,52,176,24 box=176,24 by=panel caps=13.999 drawn face=Barlow heard="1 collection · 3 games" layout=175.12,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="1 collection · 3 games"
                  9 StackPanel at=64,44,964,39 by=panel limit=0,0,1280,800 role=none
                    10 TextBlock at=64,44,964,39 box=964,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=COLLECTIONS layout=137.02,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=COLLECTIONS
              11 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
                12 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                  13 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                    14 ScrollViewer at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                      15 TextBlock at=987,110,218,24 box=218,24 by=fixed-scroll caps=13.999 drawn face=Barlow heard=Search in=14 layout=59.32,24 limit=987,107,218,30 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
                16 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                  17 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=none
                    18 ToggleButton at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                      19 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=18 paints=#EEEEF0FF role=text size=20 text=ALL
                    20 ToggleButton at=146,100,44,44 by=panel drawn focus heard=Mine limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Mine
                      21 TextBlock at=146,108,44,24 box=44,24 by=clip caps=13.999 drawn face=Barlow heard=MINE layout=43.6,24 limit=146,100,44,44 owner=20 paints=#D4D4D4FF role=text size=20 text=MINE
                    22 ToggleButton at=222,100,98,44 by=panel drawn focus heard=Automatic limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Automatic
                      23 TextBlock at=222,108,98,24 box=98,24 by=clip caps=13.999 drawn face=Barlow heard=AUTOMATIC layout=97.26,24 limit=222,100,98,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AUTOMATIC
              24 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
                25 Border at=64,160,691,542 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  26 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    27 LunaTable at=64,160,691,542 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      28 ScrollViewer at=64,160,691,542 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,542
                        29 ListBoxItem at=64,160,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Weekend, Games: 3 games" in=28 limit=64,160,691,542 op paints=#333337FF,#EEEEF0FF role=listitem
                          30 DockPanel at=64,160,691,53 by=clip in=28 limit=64,160,691,54 role=none
                            31 Border at=64,160,691,53 by=clip in=28 limit=64,160,691,54 paints=#26262BFF,#4C9AE8FF role=none
                              32 Grid at=88,160,667,53 by=clip heard="Mark: not marked, Collection: Weekend, Games: 3 games" in=28 limit=64,160,691,54 role=none
                                33 TableCell at=88,170,476,32 box=476,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=28 layout=102.99,31.2 limit=64,160,691,54 owner=29 paints=#EEEEF0FF role=edit size=26 text=Weekend
                                34 TableCell at=588,170,143,32 box=143,32 by=clip caps=18.1987 drawn face=Barlow heard=Games in=28 layout=91.62,31.2 limit=64,160,691,54 owner=29 paints=#EEEEF0FF role=edit size=26 text="3 games"
                        35 ListBoxItem at=64,214,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Favourites, Games: Automatic · 2" in=28 limit=64,160,691,542 op paints=#333337FF,#D4D4D4FF role=listitem
                          36 DockPanel at=64,214,691,53 by=clip in=28 limit=64,214,691,54 role=none
                            37 Border at=64,214,691,53 by=clip in=28 limit=64,214,691,54 paints=#0E0E10FF role=none
                              38 Grid at=88,214,667,53 by=clip heard="Mark: not marked, Collection: Favourites, Games: Automatic · 2" in=28 limit=64,214,691,54 role=none
                                39 TableCell at=88,224,476,32 box=476,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=28 layout=114.3,31.2 limit=64,214,691,54 owner=35 paints=#8D8D91FF role=edit size=26 text=Favourites
                                40 TableCell at=588,224,143,32 box=143,32 by=clip caps=18.1987 drawn face=Barlow heard=Games in=28 layout=142.61,31.2 limit=64,214,691,54 owner=35 paints=#8D8D91FF role=edit size=26 text="Automatic · 2"
                        41 ListBoxItem at=64,268,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Recently played, Games: Automatic · 8" in=28 limit=64,160,691,542 op paints=#333337FF,#D4D4D4FF role=listitem
                          42 DockPanel at=64,268,691,53 by=clip in=28 limit=64,268,691,54 role=none
                            43 Border at=64,268,691,53 by=clip in=28 limit=64,268,691,54 paints=#0E0E10FF role=none
                              44 Grid at=88,268,667,53 by=clip heard="Mark: not marked, Collection: Recently played, Games: Automatic · 8" in=28 limit=64,268,691,54 role=none
                                45 TableCell at=88,278,476,32 box=476,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=28 layout=173.37,31.2 limit=64,268,691,54 owner=41 paints=#8D8D91FF role=edit size=26 text="Recently played"
                                46 TableCell at=588,278,143,32 box=143,32 by=clip caps=18.1987 drawn face=Barlow heard=Games in=28 layout=142.84,31.2 limit=64,268,691,54 owner=41 paints=#8D8D91FF role=edit size=26 text="Automatic · 8"
                        47 ListBoxItem at=64,322,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: New collection…, Games: " in=28 limit=64,160,691,542 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel at=64,322,691,53 by=clip in=28 limit=64,322,691,54 role=none
                            49 Border at=64,322,691,53 by=clip in=28 limit=64,322,691,54 paints=#0E0E10FF role=none
                              50 Grid at=88,322,667,53 by=clip heard="Mark: not marked, Collection: New collection…, Games: " in=28 limit=64,322,691,54 role=none
                                51 TableCell at=88,332,476,32 box=476,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=28 layout=181.66,31.2 limit=64,322,691,54 owner=47 paints=#D4D4D4FF role=edit size=26 text="New collection…"
                                52 TableCell at=588,332,143,32 by=clip drawn heard=Games in=28 limit=64,322,691,54 owner=47 paints=#D4D4D4FF role=edit
                53 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                  54 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                    55 StackPanel at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                      56 TextBlock at=795,184,397,39 box=397,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=Weekend layout=94.94,38.4 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=32 text=Weekend
                      57 TextBlock at=795,235,397,27 box=397,27 by=panel caps=15.3989 drawn face=Barlow heard="3 games" layout=77.53,26.4 limit=0,0,1280,800 paints=#8D8D91FF role=text size=22 text="3 games"
                      58 WrapPanel at=795,274,397,110 by=panel limit=0,0,1280,800 role=none
                        59 CoverCard at=795,274,100,110 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 role=none
                          60 Border at=805,274,81,110 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
                            61 Image at=805,274,81,110 by=clip drawn limit=805,274,81,110 nohit role=image
                        62 CoverCard at=907,274,100,110 by=panel heard="Brass Lantern (Synthetic)" limit=0,0,1280,800 role=none
                          63 Border at=917,274,81,110 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
                            64 Image at=917,274,81,110 by=clip drawn limit=917,274,81,110 nohit role=image
                        65 CoverCard at=1019,274,100,110 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 role=none
                          66 Border at=1029,274,81,110 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
                            67 Image at=1029,274,81,110 by=clip drawn limit=1029,274,81,110 nohit role=image
                      68 TextBlock at=795,396,397,53 box=397,53 by=panel caps=15.3989 drawn face=Barlow heard="A opens it as a section of the library; X renames it, adds games or removes it." layout=350.57,52.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="A opens it as a section of the library; X renames it, adds games or removes it."
              69 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
                70 HintBar at=64,716,1152,30 by=panel heard="Open, Back, Collections, Add the selected game, More, List, Apply…" limit=0,0,1280,800 role=group
focus none
press down focus="ListBoxItem  (Mark: not marked, Collection: Weekend, Games: 3 games)" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=CollectionsWindow
press up focus="ToggleButton 'All' (All)" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=CollectionsWindow
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
