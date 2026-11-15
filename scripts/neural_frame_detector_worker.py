import json
import sys

import numpy as np
import torch
from PIL import Image
from torch import nn

BACKGROUND_TOLERANCE = 8
TEXT_TOLERANCE = 32
MINIMUM_FRAME_WIDTH = 16
MINIMUM_FRAME_HEIGHT = 16
MINIMUM_TEXT_PIXEL_COUNT = 16
MINIMUM_BACKGROUND_SHARE_PERCENT = 50
TRAINING_EPOCH_COUNT = 20
TRAINING_BATCH_SIZE = 4096
LEARNING_RATE = 0.02
RANDOM_SEED = 1234
BACKGROUND_CLASS = 0
TEXT_CLASS = 1
OTHER_CLASS = 2


def write_response(**payload):
    print(json.dumps(payload, ensure_ascii=False), flush=True)


def parse_colors(argument):
    return [
        (
            (int(value) >> 16) & 0xFF,
            (int(value) >> 8) & 0xFF,
            int(value) & 0xFF,
        )
        for value in argument.split(",")
        if value
    ]


def label_colors(colors, background_colors, text_colors):
    labels = np.full(len(colors), OTHER_CLASS, dtype=np.int64)
    background_colors = np.array(background_colors, dtype=np.int64)
    text_colors = np.array(text_colors, dtype=np.int64)
    background_distance = np.abs(
        colors[:, np.newaxis, :] - background_colors[np.newaxis, :, :]
    ).max(axis=2)
    text_distance = np.abs(
        colors[:, np.newaxis, :] - text_colors[np.newaxis, :, :]
    ).max(axis=2)
    labels[background_distance.min(axis=1) <= BACKGROUND_TOLERANCE] = BACKGROUND_CLASS
    labels[text_distance.min(axis=1) <= TEXT_TOLERANCE] = TEXT_CLASS
    return labels


def generate_training_colors(background_colors, text_colors):
    random_generator = np.random.default_rng(RANDOM_SEED)
    colors = [random_generator.integers(0, 256, size=(6000, 3))]
    for _ in range(6000):
        base = background_colors[random_generator.integers(0, len(background_colors))]
        colors.append(base + random_generator.integers(-24, 25, size=(1, 3)))
    for _ in range(6000):
        base = text_colors[random_generator.integers(0, len(text_colors))]
        colors.append(base + random_generator.integers(-48, 49, size=(1, 3)))
    background_array = np.array(background_colors, dtype=np.float64)
    text_array = np.array(text_colors, dtype=np.float64)
    for _ in range(6000):
        background = background_array[random_generator.integers(0, len(background_array))]
        text = text_array[random_generator.integers(0, len(text_array))]
        alpha = random_generator.random()
        blend = alpha * text + (1.0 - alpha) * background
        colors.append(blend + random_generator.normal(0.0, 4.0, size=(1, 3)))
    return np.clip(np.concatenate(colors), 0, 255)


class PixelClassifier(nn.Module):
    def __init__(self):
        super().__init__()
        self.layers = nn.Sequential(
            nn.Conv2d(3, 32, 1),
            nn.ReLU(),
            nn.Conv2d(32, 32, 1),
            nn.ReLU(),
            nn.Conv2d(32, 3, 1),
        )

    def forward(self, pixels):
        return self.layers(pixels)


def train_classifier(background_colors, text_colors):
    torch.manual_seed(RANDOM_SEED)
    torch.set_num_threads(4)
    training_colors = generate_training_colors(background_colors, text_colors)
    training_labels = torch.from_numpy(
        label_colors(training_colors, background_colors, text_colors)
    )
    training_pixels = torch.from_numpy(
        (training_colors / 255.0).astype(np.float32)
    ).reshape(-1, 3, 1, 1)
    classifier = PixelClassifier()
    optimizer = torch.optim.Adam(classifier.parameters(), lr=LEARNING_RATE)
    loss_function = nn.CrossEntropyLoss()
    color_count = len(training_pixels)
    classifier.train()
    for _ in range(TRAINING_EPOCH_COUNT):
        permutation = torch.randperm(color_count)
        for start in range(0, color_count, TRAINING_BATCH_SIZE):
            indices = permutation[start : start + TRAINING_BATCH_SIZE]
            optimizer.zero_grad()
            loss = loss_function(
                classifier(training_pixels[indices]).reshape(-1, 3),
                training_labels[indices],
            )
            loss.backward()
            optimizer.step()
    classifier.eval()
    return classifier


def classify_image(classifier, image):
    pixels = torch.from_numpy((image / 255.0).astype(np.float32)).permute(2, 0, 1)
    with torch.no_grad():
        logits = classifier(pixels.unsqueeze(0))
    return logits.argmax(dim=1)[0].cpu().numpy()


def trace_background_components(background_mask, text_mask):
    height, width = background_mask.shape
    visited = np.zeros((height, width), dtype=bool)
    detected_bounds = []
    for start_y in range(height):
        for start_x in range(width):
            if not background_mask[start_y, start_x] or visited[start_y, start_x]:
                continue
            stack = [(start_x, start_y)]
            visited[start_y, start_x] = True
            minimum_x = maximum_x = start_x
            minimum_y = maximum_y = start_y
            background_pixel_count = 0
            while stack:
                x, y = stack.pop()
                background_pixel_count += 1
                minimum_x = min(minimum_x, x)
                maximum_x = max(maximum_x, x)
                minimum_y = min(minimum_y, y)
                maximum_y = max(maximum_y, y)
                for neighbour_x, neighbour_y in (
                    (x - 1, y),
                    (x + 1, y),
                    (x, y - 1),
                    (x, y + 1),
                ):
                    if (
                        0 <= neighbour_x < width
                        and 0 <= neighbour_y < height
                        and background_mask[neighbour_y, neighbour_x]
                        and not visited[neighbour_y, neighbour_x]
                    ):
                        visited[neighbour_y, neighbour_x] = True
                        stack.append((neighbour_x, neighbour_y))
            bounds = (
                minimum_x,
                minimum_y,
                maximum_x - minimum_x + 1,
                maximum_y - minimum_y + 1,
            )
            if is_text_frame(background_pixel_count, bounds, text_mask):
                detected_bounds.append(bounds)
    detected_bounds.sort(key=lambda bounds: (bounds[1], bounds[0]))
    return detected_bounds


def is_text_frame(background_pixel_count, bounds, text_mask):
    x, y, width, height = bounds
    if width < MINIMUM_FRAME_WIDTH or height < MINIMUM_FRAME_HEIGHT:
        return False
    if text_mask[y : y + height, x : x + width].sum() < MINIMUM_TEXT_PIXEL_COUNT:
        return False
    return (
        background_pixel_count * 100
        >= width * height * MINIMUM_BACKGROUND_SHARE_PERCENT
    )


def detect_bounds(classifier, image_path):
    image = np.asarray(Image.open(image_path).convert("RGB"), dtype=np.float64)
    classes = classify_image(classifier, image)
    return trace_background_components(classes == BACKGROUND_CLASS, classes == TEXT_CLASS)


def main():
    if len(sys.argv) != 3:
        print(
            "usage: neural_frame_detector_worker.py BACKGROUND_COLORS TEXT_COLORS",
            file=sys.stderr,
        )
        return 2
    background_colors = parse_colors(sys.argv[1])
    text_colors = parse_colors(sys.argv[2])
    if not background_colors or not text_colors:
        print("at least one background and text color are required", file=sys.stderr)
        return 2
    sys.stdin.reconfigure(encoding="utf-8")
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    classifier = train_classifier(background_colors, text_colors)
    print(json.dumps({"ready": True}), flush=True)
    for line in sys.stdin:
        image_path = line.strip()
        if not image_path:
            continue
        try:
            write_response(ok=True, bounds=detect_bounds(classifier, image_path))
        except Exception as error:
            write_response(ok=False, error=str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
