import json
import sys
from collections import Counter

import numpy as np
import torch
from PIL import Image
from torch import nn

COLOR_TOLERANCE = 8
MAXIMUM_COLOR_COUNT = 16
MINIMUM_COMPONENT_AREA = 400
MINIMUM_CANDIDATE_WIDTH = 32
MINIMUM_CANDIDATE_HEIGHT = 24
MINIMUM_CANDIDATE_SHARE_PERCENT = 60
CONTRAST_THRESHOLD = 32
FEATURE_COUNT = 6
TRAINING_SAMPLE_COUNT = 24000
TRAINING_EPOCH_COUNT = 20
TRAINING_BATCH_SIZE = 2048
LEARNING_RATE = 0.01
RANDOM_SEED = 1234
POSITIVE_THRESHOLD = 0.5


def write_response(**payload):
    print(json.dumps(payload, ensure_ascii=False), flush=True)


def generate_training_samples():
    random_generator = np.random.default_rng(RANDOM_SEED)
    positive_count = TRAINING_SAMPLE_COUNT // 2
    negative_count = TRAINING_SAMPLE_COUNT - positive_count
    positive_width = random_generator.uniform(0.05, 0.98, positive_count)
    positive_height = random_generator.uniform(0.04, 0.98, positive_count)
    positive_fill = random_generator.uniform(0.6, 1.0, positive_count)
    positive_contrast = random_generator.uniform(0.01, 0.5, positive_count)
    positive_aspect = random_generator.uniform(0.5, 8.0, positive_count)
    positive_features = np.stack(
        [
            positive_width,
            positive_height,
            positive_aspect / 10.0,
            positive_fill,
            positive_contrast,
            positive_width * positive_height,
        ],
        axis=1,
    )
    negative_width = random_generator.uniform(0.02, 0.98, negative_count)
    negative_height = random_generator.uniform(0.02, 0.98, negative_count)
    negative_fill = random_generator.uniform(0.05, 0.58, negative_count)
    negative_contrast = random_generator.uniform(0.0, 1.0, negative_count)
    negative_aspect = random_generator.uniform(0.1, 20.0, negative_count)
    negative_features = np.stack(
        [
            negative_width,
            negative_height,
            negative_aspect / 10.0,
            negative_fill,
            negative_contrast,
            negative_width * negative_height,
        ],
        axis=1,
    )
    uniform_indices = random_generator.choice(
        negative_count, size=negative_count // 3, replace=False
    )
    negative_features[uniform_indices, 3] = random_generator.uniform(0.9, 1.0, len(uniform_indices))
    negative_features[uniform_indices, 4] = random_generator.uniform(0.0, 0.005, len(uniform_indices))
    thin_indices = random_generator.choice(
        negative_count, size=negative_count // 4, replace=False
    )
    negative_features[thin_indices, 0] = random_generator.uniform(0.01, 0.06, len(thin_indices))
    features = np.concatenate([positive_features, negative_features]).astype(np.float32)
    labels = np.concatenate(
        [np.ones(positive_count, dtype=np.float32), np.zeros(negative_count, dtype=np.float32)]
    )
    permutation = random_generator.permutation(len(features))
    return features[permutation], labels[permutation]


class FrameClassifier(nn.Module):
    def __init__(self):
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(FEATURE_COUNT, 32),
            nn.ReLU(),
            nn.Linear(32, 32),
            nn.ReLU(),
            nn.Linear(32, 1),
        )

    def forward(self, features):
        return self.layers(features)


def train_classifier():
    torch.manual_seed(RANDOM_SEED)
    torch.set_num_threads(4)
    features, labels = generate_training_samples()
    feature_tensor = torch.from_numpy(features)
    label_tensor = torch.from_numpy(labels).unsqueeze(1)
    classifier = FrameClassifier()
    optimizer = torch.optim.Adam(classifier.parameters(), lr=LEARNING_RATE)
    loss_function = nn.BCEWithLogitsLoss()
    classifier.train()
    sample_count = len(feature_tensor)
    for _ in range(TRAINING_EPOCH_COUNT):
        permutation = torch.randperm(sample_count)
        for start in range(0, sample_count, TRAINING_BATCH_SIZE):
            indices = permutation[start : start + TRAINING_BATCH_SIZE]
            optimizer.zero_grad()
            loss = loss_function(classifier(feature_tensor[indices]), label_tensor[indices])
            loss.backward()
            optimizer.step()
    classifier.eval()
    return classifier


def component_bounds(mask):
    height, width = mask.shape
    visited = np.zeros((height, width), dtype=bool)
    components = []
    for start_y in range(height):
        for start_x in range(width):
            if not mask[start_y, start_x] or visited[start_y, start_x]:
                continue
            stack = [(start_x, start_y)]
            visited[start_y, start_x] = True
            minimum_x = maximum_x = start_x
            minimum_y = maximum_y = start_y
            area = 0
            while stack:
                x, y = stack.pop()
                area += 1
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
                        and mask[neighbour_y, neighbour_x]
                        and not visited[neighbour_y, neighbour_x]
                    ):
                        visited[neighbour_y, neighbour_x] = True
                        stack.append((neighbour_x, neighbour_y))
            components.append(
                (
                    area,
                    (
                        minimum_x,
                        minimum_y,
                        maximum_x - minimum_x + 1,
                        maximum_y - minimum_y + 1,
                    ),
                    np.array([start_x, start_y], dtype=np.int64),
                )
            )
    return components


def box_iou(first, second):
    first_x, first_y, first_width, first_height = first
    second_x, second_y, second_width, second_height = second
    intersection_x = max(first_x, second_x)
    intersection_y = max(first_y, second_y)
    intersection_width = min(first_x + first_width, second_x + second_width) - intersection_x
    intersection_height = min(first_y + first_height, second_y + second_height) - intersection_y
    if intersection_width <= 0 or intersection_height <= 0:
        return 0.0
    intersection_area = intersection_width * intersection_height
    union_area = first_width * first_height + second_width * second_height - intersection_area
    return intersection_area / union_area


def extract_features(image, bounds, color):
    x, y, width, height = bounds
    region = image[y : y + height, x : x + width]
    color_difference = np.max(np.abs(region - color), axis=2)
    background_count = int((color_difference <= COLOR_TOLERANCE).sum())
    contrast_count = int((color_difference > CONTRAST_THRESHOLD).sum())
    area = width * height
    image_height, image_width, _ = image.shape
    return np.array(
        [
            width / image_width,
            height / image_height,
            (width / height) / 10.0,
            background_count / area,
            contrast_count / area,
            area / (image_width * image_height),
        ],
        dtype=np.float32,
    )


def detect_bounds(classifier, image_path):
    image = np.asarray(Image.open(image_path).convert("RGB"), dtype=np.int64)
    image_height, image_width, _ = image.shape
    color_counter = Counter(map(tuple, image.reshape(-1, 3).astype(np.uint8)))
    accepted_bounds = []
    candidates = []
    for color, _ in color_counter.most_common(MAXIMUM_COLOR_COUNT):
        color_array = np.array(color, dtype=np.int64)
        mask = np.max(np.abs(image - color_array), axis=2) <= COLOR_TOLERANCE
        for area, bounds, seed in component_bounds(mask):
            x, y, width, height = bounds
            if area < MINIMUM_COMPONENT_AREA:
                continue
            if width < MINIMUM_CANDIDATE_WIDTH or height < MINIMUM_CANDIDATE_HEIGHT:
                continue
            if area * 100 < width * height * MINIMUM_CANDIDATE_SHARE_PERCENT:
                continue
            candidates.append((bounds, color_array, seed))
    with torch.no_grad():
        for bounds, color_array, _ in candidates:
            features = extract_features(image, bounds, color_array)
            score = torch.sigmoid(classifier(torch.from_numpy(features).unsqueeze(0)))[0, 0].item()
            if score < POSITIVE_THRESHOLD:
                continue
            if any(box_iou(bounds, accepted) > 0.9 for accepted in accepted_bounds):
                continue
            accepted_bounds.append(bounds)
    accepted_bounds.sort(key=lambda bounds: (bounds[1], bounds[0]))
    return accepted_bounds


def main():
    if len(sys.argv) != 1:
        print("usage: neural_frame_detector_worker.py", file=sys.stderr)
        return 2
    sys.stdin.reconfigure(encoding="utf-8")
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    classifier = train_classifier()
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
