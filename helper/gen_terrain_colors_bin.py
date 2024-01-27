

def gen_terrain_colors_bin():
    bytes_ = []

    MAX = 255
    MAX_FERTILITY = 15
    g = 0
    for r in range(256):
        for b in range(MAX_FERTILITY + 1):
            b_ = MAX * b // MAX_FERTILITY
            bytes_ += [r, r, b_]
    
    print(any([x < 0 or x >= 256 for x in bytes_]))
    bytes_ = bytes(bytes_)

    with open("terrain_colors.bin", "wb") as file:
        print(file.write(bytes_))

def gen_seed_colors_bin():
    bytes_ = []

    MAX = 255
    MAX_FERTILITY = 15
    MAX_SEED_HEIGHT = 30
    MAX_SATURATION = 15
    for r in range(MAX + 1):
        for g in range(MAX_FERTILITY + 1):
            for b in range(MAX_SEED_HEIGHT + 1):
                g_ = MAX * g // MAX_FERTILITY
                b_ = MAX * b // MAX_SEED_HEIGHT
                bytes_ += [r, g_, b_]
    
    print(any([x < 0 or x >= 256 for x in bytes_]))
    bytes_ = bytes(bytes_)

    with open("seed_colors.bin", "wb") as file:
        print(file.write(bytes_))

if __name__ == "__main__":
    gen_terrain_colors_bin()
    gen_seed_colors_bin()
