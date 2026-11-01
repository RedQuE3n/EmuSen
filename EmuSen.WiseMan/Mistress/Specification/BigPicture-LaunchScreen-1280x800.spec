# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-LaunchScreen-1280x800
size 1280 800
kind overlay
root LaunchScreen#LaunchScreen
unit 1
slack 1.5
floor 0
frame 0,0,1280,800
case LaunchScreen
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 BlurBackdrop at=0,0,1280,800 by=panel limit=0,0,1280,800 nohit role=none
  1 Placed at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    2 Border at=286,138,709,437 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
    3 FontText at=320,178,641,49 by=panel heard="Launching Game" limit=0,0,1280,800 role=text
    4 FittedImage at=462,253,356,154 by=panel drawn limit=0,0,1280,800 role=image
    5 FontText at=320,427,641,62 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 role=text
    6 FontText at=320,494,641,38 by=panel heard="Super Nintendo" limit=0,0,1280,800 role=text
focus none
