$dir = split-path $pwd -leaf
C:\work\hack\target\debug\jcomp.exe -i .\main.jack
C:\work\hack\target\debug\linker.exe -i . --lib C:\work\hack\os\ -v
C:\work\hack\target\debug\vcomp.exe -i .\$dir.vm -a
C:\work\hack\target\debug\assembler.exe -i .\$dir.asm -f hx -l .\$dir.lst