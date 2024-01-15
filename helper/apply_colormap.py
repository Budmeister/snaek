from PIL import ImageOps, Image

IMG_FILENAME = "humble_beginnings_elev_greyscale.png"

with open("spaced.txt", "r") as file:
    colors = file.readlines()
# Convert the hex color codes to RGB
colors_rgb = [tuple(int(color[i:i+2], 16) for i in (1, 3, 5)) for color in colors]

img = Image.open(IMG_FILENAME)

# Convert the pixelated image to grayscale
gray_img = ImageOps.grayscale(img)

# Map the grayscale values to the colors
colormapped_img = gray_img.point(lambda p: colors_rgb[p])

# Save the colormapped image
colormapped_image_path = 'colormapped.png'
colormapped_img.save(colormapped_image_path)
