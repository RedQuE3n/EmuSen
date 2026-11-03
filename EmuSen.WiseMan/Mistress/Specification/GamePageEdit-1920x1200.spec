# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageEdit-1920x1200
size 1920 1200
kind menu
root PageLayout#GamePage
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamePageEdit
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 PosterSlot at=0,0,1920,540 by=panel heard=Poster limit=0,0,1920,1200 name=PagePoster nohit role=image
        4 Border at=96,270,294,402 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1920,1200 name=PageCover paints=#26262BFF role=none
          5 Panel at=96,270,294,402 by=clip limit=96,270,294,402 role=none
            6 Image at=96,270,294,402 by=clip drawn limit=96,270,294,402 nohit role=image
        7 Viewbox at=504,339,552,79.5 by=panel limit=0,0,1920,1200 role=none
          8 ViewboxContainer at=504,339,552,79.5 by=clip limit=504,339,552,79.5 role=none
            9 TextBlock at=504,339,552,79.5 box=368,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=504,339,552,79.5 name=PageTitle paints=#FFFFFFFF role=text size=44 text="Cobalt Harbor (Synthetic)"
        10 Viewbox at=504,429,763.5,40.5 by=panel limit=0,0,1920,1200 role=none
          11 ViewboxContainer at=504,429,763.5,40.5 by=clip limit=504,429,763.5,40.5 role=none
            12 TextBlock at=504,429,763.5,40.5 box=509,27 by=clip caps=15.3989 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=508.11,26.4 limit=504,429,763.5,40.5 name=PageFacts paints=#FFFFFFFF role=text size=22 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
        13 SelectionStack at=504,492,1320,66 by=panel heard=Actions limit=0,0,1920,1200 name=PageActions role=list
          14 SelectableCell at=504,492,172.5,66 by=panel focus heard=Play limit=0,0,1920,1200 role=listitem
            15 Border at=504,492,172.5,66 by=panel limit=0,0,1920,1200 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=516,501,130.5,48 by=panel limit=0,0,1920,1200 role=none
                17 PadGlyph at=516,502.5,45,45 by=panel heard=A limit=0,0,1920,1200 role=image
                18 TextBlock at=576,501,70.5,48 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1920,1200 paints=#0E0E10FF role=text size=26 text=Play
          19 SelectableCell at=694.5,492,535.5,66 by=panel focus heard="Resume, Auto · 12 min ago" limit=0,0,1920,1200 role=listitem
            20 Border at=694.5,492,535.5,66 by=panel limit=0,0,1920,1200 name=PageAction.resume paints=#707070FF role=none
              21 StackPanel at=726,501,472.5,48 by=panel limit=0,0,1920,1200 role=none
                22 TextBlock at=726,501,133.5,48 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Resume
                23 Path at=874.5,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
                24 TextBlock at=900,501,273,48 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                25 Path at=1188,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
          26 SelectableCell at=1248,492,307.5,66 by=panel focus heard=Favourite limit=0,0,1920,1200 role=listitem
            27 Border at=1248,492,307.5,66 by=panel limit=0,0,1920,1200 name=PageAction.favourite paints=#707070FF role=none
              28 StackPanel at=1261.5,501,262.5,48 by=panel limit=0,0,1920,1200 role=none
                29 PadGlyph at=1261.5,502.5,45,45 by=panel heard=Y limit=0,0,1920,1200 role=image
                30 Path at=1321.5,508.5,33,33 by=panel limit=0,0,1920,1200 nohit role=none
                31 TextBlock at=1369.5,501,154.5,48 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Favourite
        32 WholeRow at=504,594,1320,66 by=panel heard=Sections limit=0,0,1920,1200 name=PageSections role=tab
          33 SectionTab at=504,594,154.5,66 by=panel drawn focus heard=Details limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=DETAILS
            34 AccessText at=528,606,106.5,37.5 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=504,594,154.5,66 owner=33 paints=#D4D4D4FF role=none size=20 text=DETAILS
          35 SectionTab at=670.5,594,220.5,66 by=panel drawn focus heard="Edit Details" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="EDIT DETAILS"
            36 AccessText at=694.5,606,172.5,37.5 box=115,25 by=clip caps=13.999 drawn face=Barlow layout=114.02,24 limit=670.5,594,220.5,66 owner=35 paints=#EEEEF0FF role=none size=20 text="EDIT DETAILS"
        37 Border at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageContent role=none
          38 SelectionStack at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageRows role=list
            39 MenuRow at=96,696,1728,81 by=panel focus heard="Name, Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name=PageRow.name role=listitem
            40 MenuRow at=96,777,1728,81 by=panel focus heard="Sort name" limit=0,0,1920,1200 name=PageRow.sortname role=listitem
            41 MenuRow at=96,858,1728,81 by=panel focus heard="Custom collections sortname" limit=0,0,1920,1200 name=PageRow.collectionsortname role=listitem
            42 MenuRow at=96,939,1728,81 by=panel focus heard="Description, Cobalt Harbor is a game…" limit=0,0,1920,1200 name=PageRow.description role=listitem
        43 Border at=0,1053,1920,87 by=panel limit=0,0,1920,1200 name=PageHelpBar paints=#1B1B1EFF,#707070FF role=none
          44 HintBar at=96,1069.5,1728,61.5 by=panel heard="Edit, Reset to scraped, Details, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="MenuRow PageRow.sortname" page="Edit Details"
press up focus="MenuRow PageRow.name" page="Edit Details"
press back focus="MenuRow PageRow.edit" page=details
