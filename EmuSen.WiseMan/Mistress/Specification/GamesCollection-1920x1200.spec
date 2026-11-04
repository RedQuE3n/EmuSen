# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesCollection-1920x1200
size 1920 1200
kind menu
root GalleryLayout#GamesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamesCollection
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 DockPanel at=96,60,1728,66 by=panel limit=0,0,1920,1200 role=none
          4 TextBlock at=1717.5,75,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1920,1200 name=GalleryCount paints=#8D8D91FF role=text size=20 text="3 games"
          5 WholeRow at=96,60,1585.5,66 by=panel heard=Sections limit=0,0,1920,1200 name=GallerySections role=tab
            6 SectionTab at=96,60,183,66 by=panel drawn focus heard=Weekend limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=WEEKEND
              7 AccessText at=120,72,135,37.5 box=90,25 by=clip caps=13.999 drawn face=Barlow layout=89.36,24 limit=96,60,183,66 owner=6 paints=#EEEEF0FF role=none size=20 text=WEEKEND
        8 Panel at=213,162,1494,756 by=panel heard=Weekend limit=0,0,1920,1200 name=GalleryGrid role=none
          9 CoverCard at=204.9,151.2,286.2,381.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            10 Border at=208.08,151.2,279.84,381.6 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              11 Image at=208.08,151.2,279.84,381.6 by=clip drawn limit=208.08,151.2,279.84,381.6 nohit role=image
            12 Border at=196.95,140.07,302.1,403.86 by=panel limit=0,0,1920,1200 name=CoverRing nohit paints=#4C9AE8FF role=none
          13 CoverCard at=519,162,270,360 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            14 Border at=522,162,264,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              15 Image at=522,162,264,360 by=clip drawn limit=522,162,264,360 nohit role=image
            16 Path at=738,174,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          17 CoverCard at=825,162,270,360 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            18 Border at=828,162,264,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              19 Image at=828,162,264,360 by=clip drawn limit=828,162,264,360 nohit role=image
        20 TextBlock at=96,942,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1920,1200 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        21 TextBlock at=96,1005,1728,40.5 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1920,1200 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        22 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          23 HintBar at=96,1074,1728,45 by=panel heard="Play, Game page, Favourite, Section, List, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press up focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
