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
- q to switch to toggle mode
- w to switch to set alive mode
- e to switch to set dead mode
- s to save current initial state to file
- space to toggle selected cell when in toggle mode
- arrow keys to move around the field
- esc to quit
- enter to start the simulation
- 0-9 to place prefab (then arrow keys for orientation)

During simulation:
- r to stop simulation and reset to previous initial state
- up arrow to increase simulation speed
- down arrow to decrease simulation speed
- esc to quit

When saving a board out to a file, the name given will have the suffix ".life" appended to it
and then be saved to "{WORKING_DIR}/saves/". if "{WORKING_DIR}/saves/" does not exsist, it will be created.

Prefabs are saved in the same format as any other board save but are just stored in
"{WORKING_DIR}/prefabs/". Upon start up, all valid prefabs in the prefab directory are loaded
and assigned to the 0-9 keys in order according to their last modified times. Since prefabs are 
an optional feature, the prefab directory will not be created automatically.

Note: performance is heavily affected by waiting for stdout on a single thread, therefore limiting the tick speed to the speed at which things can be printed. As such, the tick delay that you provide will actually be slightly less than the actual time it takes for a tick, depending on the size of the board.