### Building
Using 'cargo' to build only requires that you have the SDL2 libraries installed.
I currently use ubuntu based linux so for me that package is libsdl2-dev (and its slight mountain of deps)

    sudo apt install libsdl2-dev

then just run cargo to build/run
    
    cargo build

### Parameters
Currently takes just 1 parameter thats the path to the rom

### Known Issue
Pong and Brick Break have an issue with detecting collision of the paddle.