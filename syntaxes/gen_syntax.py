import json


def print_tm_block(name: str, match_str: str):
    print("{")
    print(f'"name": "{name}",')
    print(f'"match": "{match_str}"')
    print("},")


with open('lang_db.json', encoding='utf-8') as f:
    lang_db = json.load(f)
    print_tm_block("storage.type", f"\\\\b({'|'.join(lang_db['types'])})\\\\b")
    print_tm_block("keyword.control", f"\\\\b({'|'.join(lang_db['control'])})\\\\b")
    print_tm_block("constant.language", f"\\\\b({'|'.join(lang_db['constants'])})\\\\b")

    # using `#` breaks the word boundary check
    # keyword.control.directive matches the C++ behavior
    print_tm_block("keyword.control.directive", f"({'|'.join(lang_db['preprocessor'])})\\\\b")
