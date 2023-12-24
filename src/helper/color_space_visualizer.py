from PIL import Image

def read_colors_from_file(filename):
    with open(filename, 'r') as file:
        return [line.strip() for line in file]

def create_color_stripes_image(colors, filename):
    num_colors = len(colors)
    width = max(400, 10 * num_colors)
    height = 100  # You can adjust the height as needed

    image = Image.new('RGB', (width, height))
    stripe_width = width // num_colors

    for i, color in enumerate(colors):
        rgb_color = tuple(int(color[j:j+2], 16) for j in (1, 3, 5))  # Convert hex to RGB
        for x in range(i * stripe_width, (i + 1) * stripe_width):
            for y in range(height):
                image.putpixel((x, y), rgb_color)

    image.save(filename)

# Example usage
colors = read_colors_from_file('spaced.txt')
create_color_stripes_image(colors, 'color_stripes.png')
