import tkinter as tk
from tkinter import colorchooser

def create_color_picker(frame, initial_color):
    def choose_color():
        color = colorchooser.askcolor(initial_color)[1]
        if color:
            color_button.config(bg=color)
            color_button.color = color

    color_button = tk.Button(frame, text='Choose Color', command=choose_color, bg=initial_color)
    color_button.color = initial_color  # Store the current color
    color_button.pack(side=tk.LEFT, padx=5)
    return color_button

def save_colors():
    with open('colors.txt', 'w') as file:
        for button in color_buttons:
            file.write(button.color + '\n')

root = tk.Tk()
root.title("Color Pickers")

frame = tk.Frame(root)
frame.pack(pady=20)

# Read colors from file
color_buttons = []
try:
    with open('colors.txt', 'r') as file:
        colors = [line.strip() for line in file.readlines()]
        for color in colors:
            button = create_color_picker(frame, color)
            color_buttons.append(button)
except FileNotFoundError:
    print("colors.txt not found. Starting with default colors.")
    for _ in range(9):
        button = create_color_picker(frame, "#ffffff")
        color_buttons.append(button)

# Save colors when closing the window
root.protocol("WM_DELETE_WINDOW", lambda: [save_colors(), root.destroy()])

root.mainloop()
