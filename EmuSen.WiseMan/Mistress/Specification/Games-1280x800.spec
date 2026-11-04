# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Games-1280x800
size 1280 800
kind menu
root GalleryLayout#GamesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Games
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
          4 TextBlock at=1145,50,71,24 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="8 games"
          5 WholeRow at=64,40,1057,44 by=panel heard=Sections limit=0,0,1280,800 name=GallerySections role=tab
            6 SectionTab at=64,40,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=ALL
              7 AccessText at=80,48,34,25 box=34,25 by=clip caps=13.999 drawn face=Barlow layout=33.56,24 limit=64,40,66,44 owner=6 paints=#EEEEF0FF role=none size=20 text=ALL
            8 SectionTab at=138,40,138,44 by=panel drawn focus heard=Favourites limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=FAVOURITES
              9 AccessText at=154,48,106,25 box=106,25 by=clip caps=13.999 drawn face=Barlow layout=105.54,24 limit=138,40,138,44 owner=8 paints=#D4D4D4FF role=none size=20 text=FAVOURITES
            10 SectionTab at=284,40,103,44 by=panel drawn focus heard=Recent limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=RECENT
              11 AccessText at=300,48,71,25 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.14,24 limit=284,40,103,44 owner=10 paints=#D4D4D4FF role=none size=20 text=RECENT
            12 SectionTab at=395,40,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=NES
              13 AccessText at=411,48,36,25 box=36,25 by=clip caps=13.999 drawn face=Barlow layout=35.76,24 limit=395,40,68,44 owner=12 paints=#D4D4D4FF role=none size=20 text=NES
            14 SectionTab at=471,40,119,44 by=panel drawn focus heard="Game Boy" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="GAME BOY"
              15 AccessText at=487,48,87,25 box=87,25 by=clip caps=13.999 drawn face=Barlow layout=86.74,24 limit=471,40,119,44 owner=14 paints=#D4D4D4FF role=none size=20 text="GAME BOY"
            16 SectionTab at=598,40,186,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="SUPER NINTENDO"
              17 AccessText at=614,48,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=598,40,186,44 owner=16 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
        18 Panel at=142,108,996,504 by=panel heard=All limit=0,0,1280,800 name=GalleryGrid role=none
          19 CoverCard at=136.6,97.45,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            20 Border at=136.6,216.17,190.8,136.74 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              21 Image at=136.6,216.17,190.8,136.1 by=clip drawn limit=136.6,216.17,190.8,136.74 nohit role=image
            22 Border at=129.18,208.75,205.64,151.58 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          23 CoverCard at=346,108,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            24 Border at=346,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              25 Image at=346,220,180,128.4 by=clip drawn limit=346,220,180,129 nohit role=image
            26 Path at=494,228,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          27 CoverCard at=550,108,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            28 Border at=550,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              29 Image at=550,220,180,128.4 by=clip drawn limit=550,220,180,129 nohit role=image
          30 CoverCard at=754,108,180,240 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            31 Border at=754,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              32 Image at=754,220,180,128.4 by=clip drawn limit=754,220,180,129 nohit role=image
          33 CoverCard at=958,108,180,240 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            34 Border at=958,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              35 Image at=958,220,180,128.4 by=clip drawn limit=958,220,180,129 nohit role=image
          36 CoverCard at=142,372,180,240 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            37 Border at=148,372,169,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              38 Image at=148,372,168.22,240 by=clip drawn limit=148,372,169,240 nohit role=image
            39 Path at=284,380,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          40 CoverCard at=346,372,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            41 Border at=352,372,169,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              42 Image at=352,372,168.22,240 by=clip drawn limit=352,372,169,240 nohit role=image
          43 CoverCard at=550,372,180,240 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            44 Border at=550,412,180,200 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              45 Image at=550,412,180,199.8 by=clip drawn limit=550,412,180,200 nohit role=image
        46 TextBlock at=64,628,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1280,800 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        47 TextBlock at=64,670,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1280,800 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        48 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          49 HintBar at=64,716,1152,30 by=panel heard="Play, Game page, Favourite, Console, List, Home" limit=0,0,1280,800 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Fable of Tiles (Synthetic).nes" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
