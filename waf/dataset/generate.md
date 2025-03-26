/waf/dataset/generate.md

# Documentation for Dataset Preparation Script (generate.py)

## Overview
This document provides detailed instructions and information on how to use the `/waf/dataset/generate.py` script to prepare the dataset for API integration. The script automates the process of extracting, organizing, and cleaning files to ensure a structured dataset ready for use.

## Folder Naming Convention
- **Format**: Folder names are numerical, ranging from `000000` to the total dataset length (`xxxxxx`). 
- **Hashing**: Each folder name is derived from the SHA-256 hash of its corresponding number, truncated to the first 7 characters.
  - Example:
    - Number: `000001`
    - SHA-256 Hash: `4bf3e1b6...`
    - Folder Name: `4bf3e1b`

---

## Image Organization Within Folders
  - Each folder contains images sorted sequentially:
  - **Order**: Images are arranged from 0 to 8.
  - **Structure**: Images are placed left-to-right and top-to-bottom within a virtual grid.
    - Example:
      ```
      [Image 0] [Image 1] [Image 2]
      [Image 3] [Image 4] [Image 5]
      [Image 6] [Image 7] [Image 8]
      ```
    - If the line ends, placement continues at the beginning of a new row.

---

## Warning
- Running the script will permanently delete:
  - All files that are not `*.txt`, `*.zip`, `*.py`, and their directories

---

## Notes
- This script is designed for streamlined automation, please run it once on the server and assume all input files are correctly formatted.