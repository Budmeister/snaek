import numpy as np
from PIL import Image
from noise import pnoise2

def generate_terrain(width, height, scale=100):
    terrain = np.zeros((height, width))
    for y in range(height):
        for x in range(width):
            terrain[y][x] = pnoise2(x / scale, y / scale, octaves=6)
    return terrain

def find_high_points(terrain, num_points=5):
    high_points = []
    y_max, x_max = terrain.shape
    for _ in range(num_points):
        x = np.random.randint(0, x_max)
        y = np.random.randint(0, y_max)
        if terrain[y][x] > 0.2:  # Threshold for high points
            high_points.append((x, y))
    return high_points

def simulate_river(terrain, start_point):
    x, y = start_point
    path = []
    while 0 <= x < terrain.shape[1] and 0 <= y < terrain.shape[0]:
        path.append((x, y))
        neighbors = [(x + dx, y + dy) for dx in [-1, 0, 1] for dy in [-1, 0, 1] if dx != 0 or dy != 0]
        neighbors = [(nx, ny) for nx, ny in neighbors if 0 <= nx < terrain.shape[1] and 0 <= ny < terrain.shape[0]]
        if not neighbors:
            break
        next_point = min(neighbors, key=lambda p: terrain[p[1]][p[0]])
        if terrain[next_point[1]][next_point[0]] >= terrain[y][x]:
            break  # Stop if moving to higher terrain
        x, y = next_point
    return path

def draw_rivers(image, rivers):
    for river in rivers:
        for point in river:
            image.putpixel(point, (0, 0, 255))  # Blue color for rivers

def main():
    width, height = 200, 160
    terrain = generate_terrain(width, height)
    high_points = find_high_points(terrain)
    rivers = [simulate_river(terrain, start) for start in high_points]

    image = Image.new("RGB", (width, height), (255, 255, 255))  # White background
    draw_rivers(image, rivers)
    image.save("rivers.png")
    # image.show()

if __name__ == "__main__":
    main()
