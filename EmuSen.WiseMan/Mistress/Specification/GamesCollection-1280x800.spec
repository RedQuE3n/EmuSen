# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesCollection-1280x800
size 1280 800
kind menu
root GalleryLayout#GamesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case GamesCollection
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
          4 TextBlock at=1145,50,71,24 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="3 games"
          5 WholeRow at=64,40,1057,44 by=panel heard=Sections limit=0,0,1280,800 name=GallerySections role=none
            6 ToggleButton at=64,40,122,44 by=panel drawn focus heard=Weekend limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=WEEKEND
              7 AccessText at=80,48,90,25 box=90,25 by=clip caps=13.999 drawn face=Barlow layout=89.36,24 limit=64,40,122,44 owner=6 paints=#EEEEF0FF role=none size=20 text=WEEKEND
        8 Panel at=142,108,996,504 by=panel heard=Weekend limit=0,0,1280,800 name=GalleryGrid role=none
          9 CoverCard at=136.6,100.8,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Aurora Drift (Synthetic).sfc" role=none
            10 Border at=138.72,100.8,186.56,254.4 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              11 Image at=138.72,100.8,186.56,254.4 by=clip drawn limit=138.72,100.8,186.56,254.4 nohit role=image
            12 Border at=131.3,93.38,201.4,269.24 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          13 CoverCard at=346,108,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Brass Lantern (Synthetic).sfc" role=none
            14 Border at=348,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              15 Image at=348,108,176,240 by=clip drawn limit=348,108,176,240 nohit role=image
            16 Path at=502,106,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          17 CoverCard at=550,108,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            18 Border at=552,108,176,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              19 Image at=552,108,176,240 by=clip drawn limit=552,108,176,240 nohit role=image
        20 TextBlock at=64,628,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1280,800 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        21 TextBlock at=64,670,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="SNES · 3 h ago" layout=130.75,26.4 limit=0,0,1280,800 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="SNES · 3 h ago"
        22 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          23 HintBar at=64,716,1152,30 by=panel heard="Play, Game page, Favourite, Section, List, Home" limit=0,0,1280,800 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press up focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
