# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageEdit-1280x800
size 1280 800
kind menu
root PageLayout#GamePage
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case GamePageEdit
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 PosterSlot at=0,0,1280,360 by=panel heard=Poster limit=0,0,1280,800 name=PagePoster nohit role=image
        4 Border at=64,180,201,268 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1280,800 name=PageCover paints=#26262BFF role=none
          5 Panel at=64,180,201,268 by=clip limit=64,180,201,268 role=none
            6 Image at=64,180,201,268 by=clip drawn limit=64,180,201,268 nohit role=image
        7 Viewbox at=336,226,368,53 by=panel limit=0,0,1280,800 role=none
          8 ViewboxContainer at=336,226,368,53 by=clip limit=336,226,368,53 role=none
            9 TextBlock at=336,226,368,53 box=368,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=336,226,368,53 name=PageTitle paints=#FFFFFFFF role=text size=44 text="Cobalt Harbor (Synthetic)"
        10 Viewbox at=336,286,509,27 by=panel limit=0,0,1280,800 role=none
          11 ViewboxContainer at=336,286,509,27 by=clip limit=336,286,509,27 role=none
            12 TextBlock at=336,286,509,27 box=509,27 by=clip caps=15.3989 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=508.11,26.4 limit=336,286,509,27 name=PageFacts paints=#FFFFFFFF role=text size=22 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
        13 SelectionStack at=336,328,880,44 by=panel heard=Actions limit=0,0,1280,800 name=PageActions role=list
          14 SelectableCell at=336,328,115,44 by=panel focus heard=Play limit=0,0,1280,800 role=listitem
            15 Border at=336,328,115,44 by=panel limit=0,0,1280,800 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=344,334,87,32 by=panel limit=0,0,1280,800 role=none
                17 PadGlyph at=344,335,30,30 by=panel heard=A limit=0,0,1280,800 role=image
                18 TextBlock at=384,334,47,32 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1280,800 paints=#0E0E10FF role=text size=26 text=Play
          19 SelectableCell at=463,328,357,44 by=panel focus heard="Resume, Auto · 12 min ago" limit=0,0,1280,800 role=listitem
            20 Border at=463,328,357,44 by=panel limit=0,0,1280,800 name=PageAction.resume paints=#707070FF role=none
              21 StackPanel at=484,334,315,32 by=panel limit=0,0,1280,800 role=none
                22 TextBlock at=484,334,89,32 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Resume
                23 Path at=583,344,7,12 by=panel limit=0,0,1280,800 nohit role=none
                24 TextBlock at=600,334,182,32 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                25 Path at=792,344,7,12 by=panel limit=0,0,1280,800 nohit role=none
          26 SelectableCell at=832,328,205,44 by=panel focus heard=Favourite limit=0,0,1280,800 role=listitem
            27 Border at=832,328,205,44 by=panel limit=0,0,1280,800 name=PageAction.favourite paints=#707070FF role=none
              28 StackPanel at=841,334,175,32 by=panel limit=0,0,1280,800 role=none
                29 PadGlyph at=841,335,30,30 by=panel heard=Y limit=0,0,1280,800 role=image
                30 Path at=881,339,22,22 by=panel limit=0,0,1280,800 nohit role=none
                31 TextBlock at=913,334,103,32 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Favourite
        32 WholeRow at=336,396,880,44 by=panel heard=Sections limit=0,0,1280,800 name=PageSections role=tab
          33 SectionTab at=336,396,103,44 by=panel drawn focus heard=Details limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=DETAILS
            34 AccessText at=352,404,71,25 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=336,396,103,44 owner=33 paints=#D4D4D4FF role=none size=20 text=DETAILS
          35 SectionTab at=447,396,147,44 by=panel drawn focus heard="Edit Details" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="EDIT DETAILS"
            36 AccessText at=463,404,115,25 box=115,25 by=clip caps=13.999 drawn face=Barlow layout=114.02,24 limit=447,396,147,44 owner=35 paints=#EEEEF0FF role=none size=20 text="EDIT DETAILS"
        37 Border at=64,464,1152,238 by=panel limit=0,0,1280,800 name=PageContent role=none
          38 SelectionStack at=64,464,1152,238 by=panel limit=0,0,1280,800 name=PageRows role=list
            39 MenuRow at=64,464,1152,54 by=panel focus heard="Name, Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name=PageRow.name role=listitem
            40 MenuRow at=64,518,1152,54 by=panel focus heard="Sort name" limit=0,0,1280,800 name=PageRow.sortname role=listitem
            41 MenuRow at=64,572,1152,54 by=panel focus heard="Custom collections sortname" limit=0,0,1280,800 name=PageRow.collectionsortname role=listitem
            42 MenuRow at=64,626,1152,54 by=panel focus heard="Description, Cobalt Harbor is a game…" limit=0,0,1280,800 name=PageRow.description role=listitem
        43 Border at=0,702,1280,58 by=panel limit=0,0,1280,800 name=PageHelpBar paints=#1B1B1EFF,#707070FF role=none
          44 HintBar at=64,713,1152,41 by=panel heard="Edit, Reset to scraped, Details, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow PageRow.sortname" page="Edit Details"
press up focus="MenuRow PageRow.name" page="Edit Details"
press back focus="MenuRow PageRow.edit" page=details
