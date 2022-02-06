# slidetown-cli

A toolkit for Drift City / Skid Rush files

## Supported operations

### World files
| File Type | Unpack             | Pack               | Convert to/from GLTF   | Convert to/from OBJ    |
| --------- | ------------------ | ------------------ | ---------------------- | ---------------------- |
| LBF       | -                  | -                  | :white_check_mark:/:x: | :white_check_mark:/:x: |
| LF        | :white_check_mark: | :white_check_mark: | :white_check_mark:/:x: | :white_check_mark:/:x: |
| LOF       | :white_check_mark: | :white_check_mark: | :white_check_mark:/:x: | -                      |
| LOI       | :white_check_mark: | :white_check_mark: | :white_check_mark:/:x: | -                      |

### Archive files
| File Type | Unpack | Pack |
| --------- | ------ | ---- |
| AGT       | -      | -    |
| NTX       | -      | -    |

### Other files
| File Type           | Unpack | Pack | Convert to/from JSON                  |
| ------------------- | ------ | ---- | ------------------------------------- |
| LevelModifier (DAT) | -      | -    | :white_check_mark:/:white_check_mark: |