import json
import argparse
from dataclasses import dataclass
from typing import List

import numpy as np

#  from madcad import *
from madcad import vec3, brick, union, io, show, normalize
from tqdm import tqdm


def parse_heightmap(heightmap_json_path: str) -> List[int]:
    with open(heightmap_json_path) as json_file:
        data = json.load(json_file)
        return data["commits"]


@dataclass
class Conf:
    base_length: float
    base_width: float
    base_height: float
    max_height: float
    bar_size: float


def main(heightmap: List[int], show_result: bool = False):
    weeks = 52
    weekdays = 7
    conf = Conf(float(weeks), float(weekdays), 2.0, 10.0, 0.99)
    base = brick(
        center=vec3(
            conf.base_length * 0.5, conf.base_width * 0.5, conf.base_height * 0.5
        ),
        width=vec3(conf.base_length, conf.base_width, conf.base_height),
    )

    # extend bottom to create a pyramid base
    for i in range(len(base.points)):
        point = base.points[i]
        if point.z == 0:  # bottom
            direction = point - vec3(
                conf.base_length * 0.5, conf.base_width * 0.5, conf.base_height * 0.5
            )
            direction = normalize(direction)
            translation = vec3(np.sign(direction.x), np.sign(direction.y), 0)
            base.points[i] += translation * conf.base_height

    part = base

    xs = (
        np.arange(0.0, conf.base_length, conf.base_length / weeks)
        + (conf.base_length / weeks) * 0.5
    )
    ys = (
        np.arange(0.0, conf.base_width, conf.base_width / weekdays)
        + (conf.base_width / weekdays) * 0.5
    )

    commits = []
    for xi, x in enumerate(xs):
        for yi, y in enumerate(ys):
            day = yi + weekdays * xi
            commit_height = heightmap[day]
            if commit_height == 0:
                continue
            commit_height = conf.max_height * (commit_height / max(max(heightmap), 1))
            # print(f"{day}: {commit_height}")
            commit = brick(
                center=vec3(x, y, commit_height * 0.5 + conf.base_height + 0.01),
                width=vec3(conf.bar_size, conf.bar_size, commit_height),
            )
            commits.append(commit)

    for commit in tqdm(commits):
        part = union(part, commit)

    io.write(part, "trophy.stl")

    # display in a 3D scene
    if show_result:
        show([part])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate `trophy.stl` from `heightmap.json`.",
    )
    parser.add_argument("heightmap", help="heightmap json file path")
    parser.add_argument("--show", action="store_true", help="show result")
    args = parser.parse_args()
    heightmap = parse_heightmap(args.heightmap)
    main(heightmap, show_result=args.show)
