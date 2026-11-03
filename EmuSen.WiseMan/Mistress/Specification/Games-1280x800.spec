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
          5 WholeRow at=64,40,1057,44 by=panel heard=Sections limit=0,0,1280,800 name=GallerySections role=none
            6 ToggleButton at=64,40,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=ALL
              7 AccessText at=80,48,34,25 box=34,25 by=clip caps=13.999 drawn face=Barlow layout=33.56,24 limit=64,40,66,44 owner=6 paints=#EEEEF0FF role=none size=20 text=ALL
            8 ToggleButton at=138,40,138,44 by=panel drawn focus heard=Favourites limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=FAVOURITES
              9 AccessText at=154,48,106,25 box=106,25 by=clip caps=13.999 drawn face=Barlow layout=105.54,24 limit=138,40,138,44 owner=8 paints=#D4D4D4FF role=none size=20 text=FAVOURITES
            10 ToggleButton at=284,40,103,44 by=panel drawn focus heard=Recent limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=RECENT
              11 AccessText at=300,48,71,25 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.14,24 limit=284,40,103,44 owner=10 paints=#D4D4D4FF role=none size=20 text=RECENT
            12 ToggleButton at=395,40,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
              13 AccessText at=411,48,36,25 box=36,25 by=clip caps=13.999 drawn face=Barlow layout=35.76,24 limit=395,40,68,44 owner=12 paints=#D4D4D4FF role=none size=20 text=NES
            14 ToggleButton at=471,40,56,44 by=panel drawn focus heard=GB limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=GB
              15 AccessText at=487,48,24,25 box=24,25 by=clip caps=13.999 drawn face=Barlow layout=23.52,24 limit=471,40,56,44 owner=14 paints=#D4D4D4FF role=none size=20 text=GB
            16 ToggleButton at=535,40,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
              17 AccessText at=551,48,47,25 box=47,25 by=clip caps=13.999 drawn face=Barlow layout=46.76,24 limit=535,40,79,44 owner=16 paints=#D4D4D4FF role=none size=20 text=SNES
        18 Panel at=142,108,996,504 by=panel heard=All limit=0,0,1280,800 name=GalleryGrid role=none
          19 CoverCard at=136.6,100.8,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Aurora Drift (Synthetic).sfc" role=none
            20 Border at=138.72,100.8,186.56,254.4 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              21 Image at=138.72,100.8,186.56,254.4 by=clip drawn limit=138.72,100.8,186.56,254.4 nohit role=image
            22 Border at=131.3,93.38,201.4,269.24 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          23 CoverCard at=346,108,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Brass Lantern (Synthetic).sfc" role=none
            24 Border at=348,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              25 Image at=348,108,176,240 by=clip drawn limit=348,108,176,240 nohit role=image
            26 Path at=502,106,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          27 CoverCard at=550,108,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            28 Border at=552,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              29 Image at=552,108,176,240 by=clip drawn limit=552,108,176,240 nohit role=image
          30 CoverCard at=754,108,180,240 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Dune Relay (Synthetic).sfc" role=none
            31 Border at=756,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              32 Image at=756,108,176,240 by=clip drawn limit=756,108,176,240 nohit role=image
          33 CoverCard at=958,108,180,240 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Ember Circuit (Synthetic).sfc" role=none
            34 Border at=960,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              35 Image at=960,108,176,240 by=clip drawn limit=960,108,176,240 nohit role=image
          36 CoverCard at=142,372,180,240 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Fable of Tiles (Synthetic).nes" role=none
            37 Border at=142,486,180,126 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              38 Image at=142,486,180,126 by=clip drawn limit=142,486,180,126 nohit role=image
            39 Path at=298,370,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          40 CoverCard at=346,372,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Granite Choir (Synthetic).nes" role=none
            41 Border at=346,486,180,126 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              42 Image at=346,486,180,126 by=clip drawn limit=346,486,180,126 nohit role=image
          43 CoverCard at=550,372,180,240 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Hollow Comet (Synthetic).gb" role=none
            44 Border at=550,432,180,180 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              45 Image at=550,432,180,180 by=clip drawn limit=550,432,180,180 nohit role=image
        46 TextBlock at=64,628,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1280,800 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        47 TextBlock at=64,670,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="SNES · 3 h ago" layout=130.75,26.4 limit=0,0,1280,800 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="SNES · 3 h ago"
        48 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          49 HintBar at=64,716,1152,30 by=panel heard="Play, Game page, Favourite, Console, List, Home" limit=0,0,1280,800 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Fable of Tiles (Synthetic).nes" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
