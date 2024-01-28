import numpy as np
import sys

MAX = 255
MAX_FERTILITY = 15
MAX_SEED_HEIGHT = 30
MAX_SATURATION = 15

def lerp(a, b, t):
    return a * (1 - t) + b * t

def gen_terrain_colors():
    colors = np.zeros(shape=(MAX+1, MAX_FERTILITY+1, 3), dtype=np.uint8)
    max_distance = 0.2
    fertile_color = np.array([138, 75, 50], dtype=np.uint8)

    def set_terrain_color(elev, fert, color):
        colors[elev, fert] = color

    for elev in range(MAX+1):
        for fert in range(MAX_FERTILITY+1):
            t = fert / MAX_FERTILITY * max_distance
            elev_color = np.array([elev, elev, elev], dtype=np.uint8)
            color = lerp(elev_color, fertile_color, t)
            set_terrain_color(elev, fert, color)

    def get_terrain_color(elev, fert):
        return colors[elev, fert]

    return get_terrain_color

def gen_terrain_colors_bin(get_terrain_color, output_file):
    bytes_ = []

    g = 0
    for elev in range(MAX + 1):
        for fert in range(MAX_FERTILITY + 1):
            bytes_ += list(get_terrain_color(elev=elev, fert=fert))
    
    print(any([x < 0 or x >= 256 for x in bytes_]))
    bytes_ = bytes(bytes_)

    with open(output_file, "wb") as file:
        print(file.write(bytes_))

def gen_seed_colors(get_terrain_color):
    colors = np.zeros(shape=(MAX+1, MAX_FERTILITY+1, MAX_SEED_HEIGHT+1, 3), dtype=np.uint8)
    max_distance = 1.0
    seed_color = np.array([75, 138, 50], dtype=np.uint8)

    def set_seed_color(elev, fert, seed_height, color):
        colors[elev, fert, seed_height] = color

    for elev in range(MAX+1):
        for fert in range(MAX_FERTILITY+1):
            for seed_height in range(MAX_SEED_HEIGHT+1):
                t = seed_height / MAX_SEED_HEIGHT * max_distance
                terrain_color = get_terrain_color(elev=elev, fert=fert)
                color = lerp(terrain_color, seed_color, t)
                set_seed_color(elev, fert, seed_height, color)

    def get_seed_color(elev, fert, seed_height):
        return colors[elev, fert, seed_height]

    return get_seed_color

def gen_seed_colors_bin(get_seed_color, output_file):
    bytes_ = []

    for elev in range(MAX + 1):
        for fert in range(MAX_FERTILITY + 1):
            for seed_height in range(MAX_SEED_HEIGHT + 1):
                bytes_ += list(get_seed_color(elev=elev, fert=fert, seed_height=seed_height))
    
    print(any([x < 0 or x >= 256 for x in bytes_]))
    bytes_ = bytes(bytes_)

    with open(output_file, "wb") as file:
        print(file.write(bytes_))

if __name__ == "__main__":
    get_terrain_color = gen_terrain_colors()
    gen_terrain_colors_bin(get_terrain_color, output_file=sys.argv[1])

    get_seed_color = gen_seed_colors(get_terrain_color)
    gen_seed_colors_bin(get_seed_color, output_file=sys.argv[2])
