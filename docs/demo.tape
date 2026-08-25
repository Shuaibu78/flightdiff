# Run from this directory: cd docs && vhs demo.tape
Output demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 900
Set Height 520
Set Padding 20
Set Theme "Github Dark"
Set TypingSpeed 45ms

Hide
Type "cd .. && clear" Enter
Show

Type "vane info testdata/crash.ulg" Sleep 400ms Enter
Sleep 2s

Type "vane diff testdata/good.ulg testdata/crash.ulg" Sleep 400ms Enter
Sleep 3s

Type "vane info testdata/truncated.ulg" Sleep 400ms Enter
Sleep 3s
