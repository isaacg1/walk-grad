import random
import math
from PIL import Image

# Concept for the artwork:
# Start with a blank grid of pixels
# Pick a random starting point
# Walk around the grid randomly, in two random paths
# When a random path reaches an existing colored-in pixel, halt
# If a walk's been going for a while and hasn't reached a colored-in pixel,
# Halt and generate a new random color in that location
# Fill in the random paths with a smooth gradient between
# the colors of the starting and ending colors

# I've left comments to help you understand the artwork.
# Wherever you see "CHANGE?", that's somewhere I think might be interesting to
# change the program, and see what happens to the resulting artwork.


def run(size, length_alpha, flip_first, flip_second, flip_diff, seed):
    # CHANGE? Each random seed gives a different artwork with the same texture or feel
    # Try a bunch!
    rng = random.Random(seed)
    # Set up a grid of pixels - colors not filled in yet
    grid = [[None for _ in range(size)] for _ in range(size)]
    # This set keeps track of which pixels are still blank
    blank = set()
    for i in range(size):
        for j in range(size):
            blank.add((i, j))
    # This is the longest a random walk is alowed to go,
    # before we end it and pick a random color
    # CHANGE? Larger alpha means bigger regions of similar color.
    walk_length_cap = size**length_alpha
    # We'll keep going until we don't have any blank pixels left
    while blank:
        # Iterating over blank is basically like picking a random blank pixel
        # to start our random walk
        start = next(blank.__iter__())
        blank.remove(start)
        # Double check that this pixel was actually blank
        if grid[start[0]][start[1]]:
            continue
        # Set up the walks - track pixels visited, and whether we've finished yet
        walks = [[[start], False], [[start], False]]
        # A quicker-to-access set of pixels seen
        seen = set()
        seen.add(start)
        # If we get stuck, we'll abort
        abort = False
        # Keep going until both walks are done
        while (
            any(len(walk[0]) < walk_length_cap and not walk[1] for walk in walks)
            and not abort
        ):
            for walk in walks:
                # If the walk isn't done,
                if len(walk[0]) < walk_length_cap and not walk[1]:
                    last = walk[0][-1]
                    # The options for the next pixel to visit - wraps around sides
                    # CHANGE? What if it didn't wrap around?
                    neighbors = [
                        ((last[0] + 1) % size, last[1]),
                        (last[0], (last[1] + 1) % size),
                        ((last[0] - 1) % size, last[1]),
                        (last[0], (last[1] - 1) % size),
                    ]
                    remaining_indexes = [0, 1, 2, 3]
                    inserted = False
                    while remaining_indexes:
                        # Pick a random pixel
                        # CHANGE? What happens if the remaining indices
                        # aren't all equally likely?
                        index = rng.choice(remaining_indexes)
                        remaining_indexes.remove(index)
                        neighbor = neighbors[index]
                        # Check whether the neighbor is somewhere new
                        if neighbor not in seen:
                            # Add it to the walk, add it to seen
                            walk[0].append(neighbor)
                            seen.add(neighbor)
                            # Check if the walk is done
                            if grid[neighbor[0]][neighbor[1]]:
                                walk[1] = True
                            # We successfully added to the walk
                            inserted = True
                            break
                    # Otherwise back up, to try another path.
                    # CHANGE? What if we didn't back up?
                    # If we can't back up any further, give up
                    if not inserted:
                        walk[0].pop()
                        if not walk[0]:
                            abort = True
        # If we aborted, go to another random starting point.
        if abort:
            continue
        for walk in walks:
            # If we gave up without hitting an existing color, fill in a new random color,
            # represented as a red, green, and blue level from 0.0 to 1.0
            if not walk[1]:
                last = walk[0][-1]
                assert not grid[last[0]][last[1]]
                # CHANGE? What if the color distribution wasn't uniformly distributed
                # throughout the RGB color cube?
                grid[last[0]][last[1]] = [rng.random(), rng.random(), rng.random()]
                blank.discard(last)
        walks = [walk[0] for walk in walks]
        lasts = [walk[-1] for walk in walks]
        # Colors of the two endpoints, that we will interpolate between
        ends = [grid[last[0]][last[1]] for last in lasts]
        length = sum(len(walk) - 1 for walk in walks)
        # These three quantities - first_end, second_end, and diff_mult,
        # correspond to three bugs I made while originally writing the program
        # CHANGE? If you change any of flip_first, flip_second, or flip_diff,
        # it'll change the texture of the resulting artwork. Try a bunch!
        # 0,0,0 and 1,1,1 look the smoothest.
        # CHANGE? Are there other alterations you could try, besides these 3?
        first_end = ends[0] if flip_first == 0 else ends[1]
        second_end = ends[0] if flip_second == 0 else ends[1]
        diff_mult = 1 if flip_diff == 0 else -1
        # This calculates the offset in color between consecutive steps
        # along the random walk
        diff = [diff_mult * (e2 - e1) / length for (e1, e2) in zip(ends[0], ends[1])]
        # For the first half of the random walk, we go backwards,
        # from the end that's already colored in back to the start.
        # We skip the very last pixel, because that's already colored in.
        for i, location in enumerate(walks[0][-2::-1]):
            # Calculate the color in this position
            color = [
                b1_item + diff_item * (i + 1)
                for (b1_item, diff_item) in zip(first_end, diff)
            ]
            # This should be a previously blank pixel
            assert not grid[location[0]][location[1]]
            # Fill in the color
            grid[location[0]][location[1]] = color
            # Remove this pixel from our set of blank pixels
            blank.discard(location)
        # For the second half, we go forwards from the starting point
        # to the end that's already colored in
        # We skip the first, repeated, start pixel and the last already colored in pixel.
        for i, location in enumerate(walks[1][1:-1]):
            position = len(walks[0]) + i
            # Calculate the color
            color = [
                b2_item + diff_item * position
                for (b2_item, diff_item) in zip(second_end, diff)
            ]
            # Check blank
            assert not grid[location[0]][location[1]]
            # Add color
            grid[location[0]][location[1]] = color
            # Remove from blank set
            blank.discard(location)
    # Now, we're done! Let's turn this into an image we can output.
    img = Image.new("RGB", (size, size))
    for i, row in enumerate(grid):
        for j, cell in enumerate(row):
            # Cell is usually a list of 3 floating point values between 0 and 1,
            # which we need to convert into RGB integer values between 0 and 255.
            # However, it might be negative or above 1 if we activated
            # any of the bug options.
            # Also, it might be None if we aborted earlier.
            # This is a simple formula to convert the floating point values to integers.
            # CHANGE? Try a different conversion formula - emphasize certain colors,
            # or handle negative values or values above 1 in a smoother way?
            color = (
                [min(max(int(f * 256), 0), 255) for f in cell]
                if cell
                else [128, 128, 128]
            )
            # Here's another option that's better outside of [0, 1]
            # color = (
            #     [
            #         int(256 * (math.atan(math.pi * (f - 0.5)) / math.pi + 0.5))
            #         for f in cell
            #     ]
            #     if cell
            #     else [128, 128, 128]
            # )
            img.putpixel((i, j), tuple(color))
    return img


if __name__ == "__main__":
    # These are the explicit configuration parameters.
    # CHANGE? You can mess around with any of these
    # The size of the artwork - bigger takes longer
    size = 300
    # The length exponent of the random walks. Bigger gives larger regions of similar color
    length_alpha = 1.1
    # The three bugs, to mix stuff up.
    # I think that 000, 111, and 011 look nice. What do you think?
    flip_first = 0
    flip_second = 1
    flip_diff = 1
    # The random seed. Changing this gives another artwork in the same style
    seed = 1
    # The filename of the resulting arwork is based on these parameters.
    # If you change how your program works but keep the parameters the same,
    # move your artwork to a different filename so it isn't overwritten.
    filename = "img-{}-{}-{}-{}-{}-{}.png".format(
        size, length_alpha, flip_first, flip_second, flip_diff, seed
    )
    print(filename)
    # Make the image
    image = run(size, length_alpha, flip_first, flip_second, flip_diff, seed)
    # Save the image to a file
    image.save(filename, "PNG")
