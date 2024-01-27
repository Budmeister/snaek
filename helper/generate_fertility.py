from PIL import Image
import numpy as np
from noise import pnoise2

def generate_perlin_noise(width, height, scale=10, octaves=6, persistence=0.5, lacunarity=2.0):
    # Create an empty array to store the Perlin noise values
    image = np.zeros((height, width))

    # Populate the array with Perlin noise
    for i in range(height):
        for j in range(width):
            image[i][j] = pnoise2(i / scale, 
                                  j / scale, 
                                  octaves=octaves, 
                                  persistence=persistence, 
                                  lacunarity=lacunarity, 
                                  repeatx=width, 
                                  repeaty=height, 
                                  base=0)
    
    # Normalize to 0-255 and convert to uint8
    image = np.interp(image, (image.min(), image.max()), (0, 30))
    image = image.astype(np.uint8)
    image -= 15

    return image

# Generate a 200x160 Perlin noise image
width, height = 200, 160
perlin_noise_img = generate_perlin_noise(width, height)

# Convert the array to an image
img = Image.fromarray(perlin_noise_img, 'L')  # 'L' for grayscale mode

# Save the image
img_path = 'fertility.png'
img.save(img_path)

img_path

