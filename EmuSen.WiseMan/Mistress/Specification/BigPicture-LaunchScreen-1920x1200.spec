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
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
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
