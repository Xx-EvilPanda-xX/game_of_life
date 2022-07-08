# game_of_life

## Usage:
arg 1: field width in characters
arg 2: field height in characters
arg 3: dead cell character (with a as accent, h as hash, and underscore as space)
arg 4: alive cell character (with a as accent, h as hash, and underscore as space)
arg 5: delay in miliseconds after tick has finished (note: that will not necassarily mean that each tick will actually be this long)

When in inital board selection:
- space to toggle selected cell
- arrow keys to move around the field

Also note: performance is heavily affected by waiting for stdout on single thread, therefore limiting the tick speed to the speed at which things can be printed.
