# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Collections-1920x1200
size 1920 1200
kind menu
root SheetLayer#Sheets
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Collections
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
        3 CollectionsPane at=72,42,1776,1098 by=panel heard=Collections limit=0,0,1920,1200 role=pane
          4 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
            5 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
              6 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard=COLLECTIONS limit=0,0,1920,1200 name=AdaptiveHeader role=none
                7 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                  8 StackPanel at=1558.5,78,265.5,36 by=panel heard="1 collection · 3 games" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                    9 TextBlock at=1558.5,78,139.5,36 box=93,24 by=panel caps=13.999 drawn face=Barlow heard="1 collection" layout=92.44,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="1 collection"
                    10 TextBlock at=1698,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                    11 TextBlock at=1717.5,78,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 games"
                  12 StackPanel at=96,66,1444.5,58.5 by=panel limit=0,0,1920,1200 role=none
                    13 TextBlock at=96,66,1444.5,58.5 box=963,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=COLLECTIONS layout=137.02,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=COLLECTIONS
              14 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
                15 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                  16 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                    17 ScrollViewer at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                      18 TextBlock at=1480.5,165,327,36 box=218,24 by=fixed-scroll caps=13.999 drawn face=Barlow heard=Search in=17 layout=59.32,24 limit=1480.5,160.5,327,45 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
                19 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                  20 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                    21 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                      22 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=21 paints=#EEEEF0FF role=text size=20 text=ALL
                    23 SectionTab at=219,150,66,66 by=panel drawn focus heard=Mine limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Mine
                      24 TextBlock at=219,162,66,36 box=44,24 by=clip caps=13.999 drawn face=Barlow heard=MINE layout=43.6,24 limit=219,150,66,66 owner=23 paints=#D4D4D4FF role=text size=20 text=MINE
                    25 SectionTab at=333,150,147,66 by=panel drawn focus heard=Automatic limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Automatic
                      26 TextBlock at=333,162,147,36 box=98,24 by=clip caps=13.999 drawn face=Barlow heard=AUTOMATIC layout=97.26,24 limit=333,150,147,66 owner=25 paints=#D4D4D4FF role=text size=20 text=AUTOMATIC
              27 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
                28 Border at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  29 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    30 LunaTable at=96,240,1036.5,813 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      31 ScrollViewer at=96,240,1036.5,813 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,813
                        32 ListBoxItem at=96,240,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Weekend, Games: 3 games  ▸" in=31 limit=96,240,1036.5,813 op paints=#333337FF,#EEEEF0FF role=listitem
                          33 DockPanel at=96,240,1036.5,79.5 by=clip in=31 limit=96,240,1036.5,81 role=none
                            34 Border at=96,240,1036.5,79.5 by=clip in=31 limit=96,240,1036.5,81 paints=#26262BFF,#4C9AE8FF role=none
                              35 Grid at=132,240,1000.5,79.5 by=clip heard="Mark: not marked, Collection: Weekend, Games: 3 games  ▸" in=31 limit=96,240,1036.5,81 role=none
                                36 TableCell at=132,255,675,48 box=450,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=31 layout=102.99,31.2 limit=96,240,1036.5,81 owner=32 paints=#EEEEF0FF role=edit size=26 text=Weekend
                                37 TableCell at=843,255,253.5,51 box=169,34 by=clip caps=18.1987 drawn face=Barlow heard=Games in=31 layout=117.62,33.41 limit=96,240,1036.5,81 owner=32 paints=#EEEEF0FF role=edit size=26 text="3 games  ▸"
                        38 ListBoxItem at=96,321,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Favourites, Games: Automatic · 2  ▸" in=31 limit=96,240,1036.5,813 op paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel at=96,321,1036.5,79.5 by=clip in=31 limit=96,321,1036.5,81 role=none
                            40 Border at=96,321,1036.5,79.5 by=clip in=31 limit=96,321,1036.5,81 paints=#0E0E10FF role=none
                              41 Grid at=132,321,1000.5,79.5 by=clip heard="Mark: not marked, Collection: Favourites, Games: Automatic · 2  ▸" in=31 limit=96,321,1036.5,81 role=none
                                42 TableCell at=132,336,675,48 box=450,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=31 layout=114.3,31.2 limit=96,321,1036.5,81 owner=38 paints=#8D8D91FF role=edit size=26 text=Favourites
                                43 TableCell at=843,336,253.5,51 box=169,34 by=clip caps=18.1987 drawn face=Barlow heard=Games in=31 layout=168.61,33.41 limit=96,321,1036.5,81 owner=38 paints=#8D8D91FF role=edit size=26 text="Automatic · 2  ▸"
                        44 ListBoxItem at=96,402,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: Recently played, Games: Automatic · 8  ▸" in=31 limit=96,240,1036.5,813 op paints=#333337FF,#D4D4D4FF role=listitem
                          45 DockPanel at=96,402,1036.5,79.5 by=clip in=31 limit=96,402,1036.5,81 role=none
                            46 Border at=96,402,1036.5,79.5 by=clip in=31 limit=96,402,1036.5,81 paints=#0E0E10FF role=none
                              47 Grid at=132,402,1000.5,79.5 by=clip heard="Mark: not marked, Collection: Recently played, Games: Automatic · 8  ▸" in=31 limit=96,402,1036.5,81 role=none
                                48 TableCell at=132,417,675,48 box=450,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=31 layout=173.37,31.2 limit=96,402,1036.5,81 owner=44 paints=#8D8D91FF role=edit size=26 text="Recently played"
                                49 TableCell at=843,417,253.5,51 box=169,34 by=clip caps=18.1987 drawn face=Barlow heard=Games in=31 layout=168.84,33.41 limit=96,402,1036.5,81 owner=44 paints=#8D8D91FF role=edit size=26 text="Automatic · 8  ▸"
                        50 ListBoxItem at=96,483,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Collection: New collection…, Games: " in=31 limit=96,240,1036.5,813 op paints=#333337FF,#D4D4D4FF role=listitem
                          51 DockPanel at=96,483,1036.5,79.5 by=clip in=31 limit=96,483,1036.5,81 role=none
                            52 Border at=96,483,1036.5,79.5 by=clip in=31 limit=96,483,1036.5,81 paints=#0E0E10FF role=none
                              53 Grid at=132,483,1000.5,79.5 by=clip heard="Mark: not marked, Collection: New collection…, Games: " in=31 limit=96,483,1036.5,81 role=none
                                54 TableCell at=132,498,675,48 box=450,32 by=clip caps=18.1987 drawn face=Barlow heard=Collection in=31 layout=181.66,31.2 limit=96,483,1036.5,81 owner=50 paints=#D4D4D4FF role=edit size=26 text="New collection…"
                                55 TableCell at=843,498,253.5,48 by=clip drawn heard=Games in=31 limit=96,483,1036.5,81 owner=50 paints=#D4D4D4FF role=edit
                56 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                  57 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                    58 StackPanel at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                      59 TextBlock at=1192.5,276,595.5,58.5 box=397,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=Weekend layout=94.94,38.4 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=32 text=Weekend
                      60 TextBlock at=1192.5,352.5,595.5,40.5 box=397,27 by=panel caps=15.3989 drawn face=Barlow heard="3 games" layout=77.53,26.4 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=22 text="3 games"
                      61 WrapPanel at=1192.5,411,595.5,165 by=panel limit=0,0,1920,1200 role=none
                        62 CoverCard at=1192.5,411,150,165 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 role=none
                          63 Border at=1192.5,469.5,150,108 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
                            64 Image at=1192.5,469.5,150,107 by=clip drawn limit=1192.5,469.5,150,108 nohit role=image
                        65 CoverCard at=1360.5,411,150,165 by=panel heard="Brass Lantern (Synthetic)" limit=0,0,1920,1200 role=none
                          66 Border at=1360.5,469.5,150,108 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
                            67 Image at=1360.5,469.5,150,107 by=clip drawn limit=1360.5,469.5,150,108 nohit role=image
                        68 CoverCard at=1528.5,411,150,165 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 role=none
                          69 Border at=1528.5,469.5,150,108 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
                            70 Image at=1528.5,469.5,150,107 by=clip drawn limit=1528.5,469.5,150,108 nohit role=image
                      71 TextBlock at=1192.5,594,595.5,79.5 box=397,53 by=panel caps=15.3989 drawn face=Barlow heard="A opens it as a section of the library; X renames it, adds games or removes it." layout=350.57,52.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="A opens it as a section of the library; X renames it, adds games or removes it."
              72 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
                73 HintBar at=96,1074,1728,45 by=panel heard="Open, Back, Collections, Add the selected game, More, List" limit=0,0,1920,1200 role=group
focus none
press down focus="ListBoxItem  (Mark: not marked, Collection: Weekend, Games: 3 games  ▸)" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=CollectionsWindow
press up focus="SectionTab 'All' (All)" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=CollectionsWindow
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
