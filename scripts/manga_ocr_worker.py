import os
import sys

from loguru import logger

os.environ.setdefault("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")
os.environ.setdefault("HF_HUB_DISABLE_PROGRESS_BARS", "1")
os.environ.setdefault("TRANSFORMERS_VERBOSITY", "error")
logger.remove()

from manga_ocr import MangaOcr


def write_response(prefix, text):
    print(f"{prefix}\t{text}", flush=True)


def main():
    sys.stdin.reconfigure(encoding="utf-8")
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    try:
        manga_ocr = MangaOcr(force_cpu=True)
    except Exception as error:
        print(f"INIT_ERROR\t{error}", file=sys.stderr, flush=True)
        return 1
    print("READY", flush=True)
    for line in sys.stdin:
        path = line.strip()
        if not path:
            continue
        try:
            text = manga_ocr(path)
            write_response("OK", " ".join(text.splitlines()))
        except Exception as error:
            write_response("ERR", str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
