# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Games-1920x1200
size 1920 1200
kind menu
root GalleryLayout#GamesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Games
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 DockPanel at=96,60,1728,66 by=panel limit=0,0,1920,1200 role=none
          4 TextBlock at=1717.5,75,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 limit=0,0,1920,1200 name=GalleryCount paints=#8D8D91FF role=text size=20 text="8 games"
          5 WholeRow at=96,60,1585.5,66 by=panel heard=Sections limit=0,0,1920,1200 name=GallerySections role=tab
            6 SectionTab at=96,60,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=ALL
              7 AccessText at=120,72,51,37.5 box=34,25 by=clip caps=13.999 drawn face=Barlow layout=33.56,24 limit=96,60,99,66 owner=6 paints=#EEEEF0FF role=none size=20 text=ALL
            8 SectionTab at=207,60,207,66 by=panel drawn focus heard=Favourites limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=FAVOURITES
              9 AccessText at=231,72,159,37.5 box=106,25 by=clip caps=13.999 drawn face=Barlow layout=105.54,24 limit=207,60,207,66 owner=8 paints=#D4D4D4FF role=none size=20 text=FAVOURITES
            10 SectionTab at=426,60,154.5,66 by=panel drawn focus heard=Recent limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=RECENT
              11 AccessText at=450,72,106.5,37.5 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.14,24 limit=426,60,154.5,66 owner=10 paints=#D4D4D4FF role=none size=20 text=RECENT
            12 SectionTab at=592.5,60,102,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=NES
              13 AccessText at=616.5,72,54,37.5 box=36,25 by=clip caps=13.999 drawn face=Barlow layout=35.76,24 limit=592.5,60,102,66 owner=12 paints=#D4D4D4FF role=none size=20 text=NES
            14 SectionTab at=706.5,60,178.5,66 by=panel drawn focus heard="Game Boy" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="GAME BOY"
              15 AccessText at=730.5,72,130.5,37.5 box=87,25 by=clip caps=13.999 drawn face=Barlow layout=86.74,24 limit=706.5,60,178.5,66 owner=14 paints=#D4D4D4FF role=none size=20 text="GAME BOY"
            16 SectionTab at=897,60,279,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="SUPER NINTENDO"
              17 AccessText at=921,72,231,37.5 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=897,60,279,66 owner=16 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
        18 Panel at=213,162,1494,756 by=panel heard=All limit=0,0,1920,1200 name=GalleryGrid role=none
          19 CoverCard at=204.9,146.18,286.2,381.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            20 Border at=204.9,324.26,286.2,205.11 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              21 Image at=204.9,324.26,286.2,204.16 by=clip drawn limit=204.9,324.26,286.2,205.11 nohit role=image
            22 Border at=193.77,313.13,308.46,227.37 by=panel limit=0,0,1920,1200 name=CoverRing nohit paints=#4C9AE8FF role=none
          23 CoverCard at=519,162,270,360 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            24 Border at=519,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              25 Image at=519,330,270,192.6 by=clip drawn limit=519,330,270,193.5 nohit role=image
            26 Path at=741,342,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          27 CoverCard at=825,162,270,360 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            28 Border at=825,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              29 Image at=825,330,270,192.6 by=clip drawn limit=825,330,270,193.5 nohit role=image
          30 CoverCard at=1131,162,270,360 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            31 Border at=1131,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              32 Image at=1131,330,270,192.6 by=clip drawn limit=1131,330,270,193.5 nohit role=image
          33 CoverCard at=1437,162,270,360 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            34 Border at=1437,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              35 Image at=1437,330,270,192.6 by=clip drawn limit=1437,330,270,193.5 nohit role=image
          36 CoverCard at=213,558,270,360 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            37 Border at=222,558,253.5,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              38 Image at=222,558,252.34,360 by=clip drawn limit=222,558,253.5,360 nohit role=image
            39 Path at=426,570,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          40 CoverCard at=519,558,270,360 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            41 Border at=528,558,253.5,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              42 Image at=528,558,252.34,360 by=clip drawn limit=528,558,253.5,360 nohit role=image
          43 CoverCard at=825,558,270,360 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            44 Border at=825,618,270,300 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              45 Image at=825,618,270,299.7 by=clip drawn limit=825,618,270,300 nohit role=image
        46 TextBlock at=96,942,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1920,1200 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        47 TextBlock at=96,1005,1728,40.5 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1920,1200 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        48 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          49 HintBar at=96,1074,1728,45 by=panel heard="Play, Game page, Favourite, Console, List, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Fable of Tiles (Synthetic).nes" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
