# Generate Git 3D Trophy

## Generate JSON Heightmap

```
cargo run --release <local_repo>
```

> generates `heightmap.json`

## Parse JSON Heightmap and generate `trophy.stl`

```
python model/main.py heightmap.json
```
> generates `trophy.stl`

## Example

```
cargo run --release ~/clones/cargo --year 2021
python model/main.py heightmap.json
```

> generates `trophy.stl`

![stl file preview](./trophy.png)

> preview generated using [stl-thumb (https://github.com/unlimitedbacon/stl-thumb)](https://github.com/unlimitedbacon/stl-thumb)
