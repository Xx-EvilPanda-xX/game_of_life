# game_of_life

## Usage:
arg 1: field width in characters

arg 2: field height in characters

arg 3: dead cell character (with a as accent, h as hash, and underscore as space)

arg 4: alive cell character (with a as accent, h as hash, and underscore as space)

arg 5: wether or not to spawn a random board

arg 6: delay in miliseconds after tick has finished (note: that will not necassarily mean that each tick will actually be this long)

arg 7 (optional): name of saved board to load 

When in inital board selection:
- 1 to switch to toggle mode
- 2 to switch to set alive mode
- 3 to switch to set dead mode
- s to save current initial state to file
- space to toggle selected cell when in toggle mode
- arrow keys to move around the field
- esc to quit
- enter to start the simulation

During simulation:
- r to stop simulation and reset to previous initial state
- up arrow to increase simulation speed
- down arrow to decrease simulation speed
- esc to quit

When saving a board out to a file, the name given will have the suffix ".life" appended to it
and then be saved to "{WORKING_DIR}/saves/". if "{WORKING_DIR}/saves/" does not exsist, it will be created.

Note: performance is heavily affected by waiting for stdout on a single thread, therefore limiting the tick speed to the speed at which things can be printed. As such, the tick delay that you provide will actually be slightly less than the actual time it takes for a tick, depending on the size of the board.