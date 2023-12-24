def lerp(a, b, t):
    return int(a + (b - a) * t)

def color_space(colors, steps):
    result = []

    for i in range(steps):
        progress = i / (steps - 1)
        prev_color = colors[0]

        for current_color in colors[1:]:
            if progress <= current_color[1]:
                t = (progress - prev_color[1]) / (current_color[1] - prev_color[1])
                r = lerp(prev_color[0][0], current_color[0][0], t)
                g = lerp(prev_color[0][1], current_color[0][1], t)
                b = lerp(prev_color[0][2], current_color[0][2], t)
                result.append((r, g, b))
                break
            prev_color = current_color

    return result

def save_colors_to_file(colors, filename):
    with open(filename, 'w') as file:
        for color in colors:
            file.write('#{:02x}{:02x}{:02x}\n'.format(*color))

def read_colors_from_file(file_path):
    """Read colors from a file and return them as a list of (R, G, B) tuples."""
    colors = []
    try:
        with open(file_path, 'r') as file:
            for line in file:
                hex_color = line.strip()
                # Convert hex to RGB tuple
                rgb_color = tuple(int(hex_color[i:i+2], 16) for i in (1, 3, 5))
                colors.append(rgb_color)
    except FileNotFoundError:
        print(f"File {file_path} not found.")
    except Exception as e:
        print(f"An error occurred: {e}")

    return colors

def read_spaces_from_file(file_path):
    spaces = []
    try:
        with open(file_path, 'r') as file:
            for line in file:
                space = float(line.strip())
                spaces.append(space)
    except FileNotFoundError:
        print(f"File {file_path} not found.")
    except Exception as e:
        print(f"An error occurred: {e}")

    return spaces

colors = read_colors_from_file("colors.txt")
spaces = read_spaces_from_file("spaces.txt")

colors = list(zip(colors, spaces))

interpolated_colors = color_space(colors, 256)
save_colors_to_file(interpolated_colors, 'spaced.txt')
