@echo off

:: Settings
set COMPILER_PATH="glslc.exe"
set OUTPUT_PATH=shaders\spv

echo Started compiling the shaders...

:: Vertex shaders
for /r %%i in (*.vert) do (
    if not exist spv/%%~ni.vert.spv %COMPILER_PATH% -O %%i -o spv/%%~ni.vert.spv
)

:: Fragment shaders
for /r %%i in (*.frag) do (
    if not exist spv/%%~ni.frag.spv %COMPILER_PATH% -O %%i -o spv/%%~ni.frag.spv
)

echo Compile succeed.

pause
