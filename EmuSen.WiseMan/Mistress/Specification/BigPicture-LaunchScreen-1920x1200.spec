# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-LaunchScreen-1920x1200
size 1920 1200
kind overlay
root LaunchScreen#LaunchScreen
unit 1.5
slack 2
floor 0
frame 0,0,1920,1200
case LaunchScreen
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 BlurBackdrop at=0,0,1920,1200 by=panel limit=0,0,1920,1200 nohit role=none
  1 Placed at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    2 Border at=428,206,1064,656 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
    3 FontText at=479,267,962,73 by=panel heard="Launching Game" limit=0,0,1920,1200 role=text
    4 FittedImage at=693,380,534,231 by=panel drawn limit=0,0,1920,1200 role=image
    5 FontText at=479,641,962,93 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 role=text
    6 FontText at=479,741,962,56 by=panel heard="Super Nintendo" limit=0,0,1920,1200 role=text
focus none
