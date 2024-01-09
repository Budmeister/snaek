# To Do
1. ~~Add MSPT meter: For draw and logic threads: "{lock} + {processing} / {max}" - in red if lock + processing > max~~
2. ~~Water and Lava flow downhill~~
3. ~~Water and Lava have a "height" value that, along with the elevation, determines where they flow~~
4. ~~When water and lava meet, one "height" of each gets turned into one elevation~~
5. Remove Seasons and add l: Box<dyn LevelState>, which shall be stored on the logic thread stack. Each level will provide its own implementation of LevelState, which will update the board every tick and provide shop information. Unlike GameState and ViewState, LevelState will be a fully fledged object with methods and can mutate itself, but the logic thread only cares that it implements LevelState.
6. The current level, Rivers, will let the user buy seconds of rain, and the rest of the time, it will be raining lava. 
7. Seed "dist" value has a max speed of 1 per tick
8. Food no longer spawns naturally, but Gold does
9. Powerups spawn less frequently
10. Add shop where you can spend Gold for all powerups
11. Add Shovel powerup: lets you pick up one piece of dirt and place it back down later
12. Invincibility is an on-demand powerup
13. Explosion also decreases elevation