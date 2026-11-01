# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Resume-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,96.5,812.7,581
title "Continue Playing"
case Resume
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,196,813,461 by=panel limit=234,196,812.35,461 paints=#D4D4D4FF role=none seen=234,196,812.35,461
    1 StackPanel at=274,224,733,417 by=panel limit=234,196,812.35,461 role=none
      2 Image at=503,224,274.29,240 by=panel drawn limit=234,196,812.35,461 role=image
      3 TextBlock at=274,476,733,29 box=733,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Would you like to continue your last game?" layout=332.38,28.8 limit=234,196,812.35,461 paints=#D4D4D4FF role=text size=24 text="Would you like to continue your last game?"
      4 HintText at=274,517,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again." layout=701.7,24 limit=234,196,812.35,461 paints=#8D8D91FF role=text size=20 text="Cobalt Harbor (Synthetic) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again."
      5 LunaSwitch at=274,553,187,32 by=panel drawn focus heard="Do not ask again" limit=234,196,812.35,461 name=RememberResumeSwitch op paints=#8D8D91FF,#D4D4D4FF role=button
        6 TextBlock at=326,555,129,29 box=129,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Do not ask again" layout=128.76,28.8 limit=274,553,187,32 owner=5 paints=#D4D4D4FF role=text size=24 text="Do not ask again"
      7 ButtonBar at=792,597,215,44 by=panel limit=234,196,812.35,461 paints=#D4D4D4FF role=toolbar
        8 Button at=792,597,101,44 by=clip drawn focus heard=RESUME limit=792,597,215,44 name=ResumeButton op paints=#707070FF,#D4D4D4FF role=button text=Resume
          9 TextBlock at=809,603,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME layout=66.64,31.2 limit=792,597,101,44 owner=8 paints=#EEEEF0FF role=text size=26 text=RESUME
        10 Button at=901,597,106,44 by=clip drawn focus heard=RESTART limit=792,597,215,44 name=RestartButton op paints=#707070FF,#D4D4D4FF role=button text=Restart
          11 TextBlock at=918,603,72,32 box=72,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESTART layout=71.34,31.2 limit=901,597,106,44 owner=10 paints=#D4D4D4FF role=text size=26 text=RESTART
focus 8
edge 5 up same
edge 5 down 8
edge 5 left same
edge 5 right 8
edge 8 up 5
edge 8 down same
edge 8 left 5
edge 8 right 10
edge 10 up 5
edge 10 down same
edge 10 left 8
edge 10 right same
press pageup focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press pagedown focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press first focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press last focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press back top=none
