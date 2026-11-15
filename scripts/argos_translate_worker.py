import json
import sys

from argostranslate import translate

SHORT_LABEL_MAX_LENGTH = 4
SENTENCE_PUNCTUATION = ("。", "！", "？", ".", "!", "?")


def write_response(**payload):
    print(json.dumps(payload, ensure_ascii=False), flush=True)


def translate_text(text, from_code, to_code):
    translated_text = translate.translate(text, from_code, to_code)
    if from_code != "ja" or text.endswith(SENTENCE_PUNCTUATION):
        return translated_text
    if translated_text and len(text) > SHORT_LABEL_MAX_LENGTH:
        return translated_text
    punctuated_translation = translate.translate(f"{text}。", from_code, to_code)
    return punctuated_translation or translated_text


def main():
    if len(sys.argv) != 3:
        print("usage: argos_translate_worker.py FROM TO", file=sys.stderr)
        return 2
    from_code, to_code = sys.argv[1], sys.argv[2]
    sys.stdin.reconfigure(encoding="utf-8")
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    print(json.dumps({"ready": True}), flush=True)
    for line in sys.stdin:
        try:
            request = json.loads(line)
            translated_text = translate_text(request["text"], from_code, to_code)
            write_response(ok=True, text=translated_text)
        except Exception as error:
            write_response(ok=False, error=str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
