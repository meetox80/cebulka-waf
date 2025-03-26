# waf/dataset/generate.py
import os
import shutil
import zipfile
import hashlib
from PIL import Image
from tqdm import tqdm

def _ConfirmAction(Message: str) -> bool:
    _Response = input(f"{Message} [Y/n] ").strip().lower()
    return _Response in {"y", ""}

def _UnzipArchive() -> None:
    if not _ConfirmAction("Found archive.zip - Extract contents?"):
        print("Aborting extraction")
        exit(0)
        
    with zipfile.ZipFile("archive.zip", "r") as Zip:
        Zip.extractall(".")

def _ProcessImages() -> None:
    _ImagePaths: list[str] = []
    for _Root, _, _Files in os.walk("."):
        for _File in _Files:
            if _File.lower().endswith(".jpg"):
                _ImagePaths.append(os.path.join(_Root, _File))

    if not _ImagePaths:
        print("No JPG images found")
        return

    print(f"Found {len(_ImagePaths):,} captcha images")
    if not _ConfirmAction("Process images into split tiles?"):
        print("Skipping processing")
        return

    for _ImgPath in tqdm(_ImagePaths, desc="Splitting", unit="img"):
        with open(_ImgPath, "rb") as _F:
            _FileData = _F.read()
        
        _ShaPrefix = hashlib.sha256(_FileData).hexdigest()[:7]
        _OutputDir = os.path.join(".", _ShaPrefix)
        os.makedirs(_OutputDir, exist_ok=True)

        try:
            with Image.open(_ImgPath) as _Img:
                _Img = _Img.convert("RGB")
                _W, _H = _Img.size
                _TileW, _TileH = _W //3 , _H//3
                
                for _Idx in range(9):
                    _Col = _Idx %3 
                    _Row = _Idx //3 
                    _Tile = _Img.crop((
                        _Col*_TileW,
                        _Row*_TileH,
                        (_Col+1)*_TileW,
                        (_Row+1)*_TileH
                    ))
                    _Tile.save(os.path.join(_OutputDir, f"{_Idx}.png"), "PNG")
        except Exception as _E:
            print(f"Failed {_ImgPath}: {str(_E)}")

def _CleanFiles() -> None:
    _DeletePaths: list[str] = []
    for _Root, _, _Files in os.walk(".", topdown=False):
        for _File in _Files:
            if not _File.lower().endswith((".txt", ".zip", ".py")):
                _DeletePaths.append(os.path.join(_Root, _File))

    if not _DeletePaths:
        print("No cleanup needed")
        return

    print(f"Found {len(_DeletePaths):,} files to clean")
    if not _ConfirmAction("Remove original files and empty directories?"):
        print("Skipping cleanup")
        return

    for _Path in _DeletePaths:
        try:
            os.remove(_Path)
        except Exception as _E:
            print(f"Error removing {_Path}: {str(_E)}")

    _RemovedDirs =0 
    for _Root, _, _Files in os.walk(".", topdown=False):
        if _Root == ".":
            continue
        try:
            os.rmdir(_Root)
            _RemovedDirs +=1 
        except OSError:
            pass

    print(f"Cleaned {len(_DeletePaths)} files, removed {_RemovedDirs} directories")

if __name__ == "__main__":
    if not os.path.exists("archive.zip"):
        print("Error: Missing archive.zip")
        exit(1)
        
    _UnzipArchive()
    _ProcessImages()
    _CleanFiles()
