"""Decode only serde_json::Value's Rust Debug grammar; never execute input text."""

from pathlib import Path
import hashlib
import json
import re


class Reader:
    def __init__(self, text):
        self.text = text
        self.position = 0

    def whitespace(self):
        while self.position < len(self.text) and self.text[self.position].isspace():
            self.position += 1

    def take(self, token):
        self.whitespace()
        assert self.text.startswith(token, self.position), (self.position, token, self.text[self.position:self.position + 80])
        self.position += len(token)

    def quoted(self):
        self.take('"')
        result = []
        while True:
            char = self.text[self.position]
            self.position += 1
            if char == '"':
                return "".join(result)
            if char != "\\":
                result.append(char)
                continue
            char = self.text[self.position]
            self.position += 1
            escapes = {'"': '"', "'": "'", "\\": "\\", "n": "\n", "r": "\r", "t": "\t", "0": "\0"}
            if char in escapes:
                result.append(escapes[char])
            elif char == "u":
                self.take("{")
                end = self.text.index("}", self.position)
                result.append(chr(int(self.text[self.position:end], 16)))
                self.position = end + 1
            elif char == "x":
                result.append(chr(int(self.text[self.position:self.position + 2], 16)))
                self.position += 2
            else:
                raise ValueError((self.position, char))

    def value(self):
        self.whitespace()
        if self.text.startswith("Object", self.position):
            self.take("Object")
            self.take("{")
            result = {}
            self.whitespace()
            while self.text[self.position] != "}":
                key = self.quoted()
                self.take(":")
                assert key not in result
                result[key] = self.value()
                self.whitespace()
                if self.text[self.position] != ",":
                    break
                self.take(",")
            self.take("}")
            return result
        if self.text.startswith("Array", self.position):
            self.take("Array")
            self.take("[")
            result = []
            self.whitespace()
            while self.text[self.position] != "]":
                result.append(self.value())
                self.whitespace()
                if self.text[self.position] != ",":
                    break
                self.take(",")
            self.take("]")
            return result
        if self.text.startswith("String", self.position):
            self.take("String")
            self.take("(")
            result = self.quoted()
            self.take(")")
            return result
        if self.text.startswith("Number", self.position):
            self.take("Number")
            self.take("(")
            result, end = json.JSONDecoder().raw_decode(self.text, self.position)
            self.position = end
            self.take(")")
            return result
        if self.text.startswith("Bool", self.position):
            self.take("Bool")
            self.take("(")
            result, end = json.JSONDecoder().raw_decode(self.text, self.position)
            assert type(result) is bool
            self.position = end
            self.take(")")
            return result
        self.take("Null")
        return None


directory = Path(__file__).parent
raw = (directory / "stderr.log").read_text()
operands = {}
for side, text in re.findall(r"^[ \t]+(left|right): ([^\n]+)$", raw, re.M):
    reader = Reader(text)
    operands[side] = reader.value()
    reader.whitespace()
    assert reader.position == len(text)
    path = directory / (side + "-normalized-metadata.json")
    assert not path.exists()
    path.write_text(json.dumps(operands[side], indent=2) + "\n")
assert set(operands) == {"left", "right"}
previous = Path("develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-edges/before-builtin-throw/control-edges/index_update.json")
assert operands["left"] == json.loads(previous.read_text())
changes = []


def compare(before, after, path=""):
    assert type(before) is type(after), path
    if isinstance(before, dict):
        assert set(before) == set(after), path
        for key in sorted(before):
            compare(before[key], after[key], path + "/" + key)
    elif isinstance(before, list):
        assert len(before) == len(after), path
        for number, (left, right) in enumerate(zip(before, after)):
            compare(left, right, path + "/" + str(number))
    elif before != after:
        changes.append({"path": path, "before": before, "after": after})


compare(operands["left"], operands["right"])
path = directory / "exact-original-failure-diff.json"
assert not path.exists()
path.write_text(json.dumps({
    "status": "exact_original_debug_operands_decoded",
    "raw_stderr_sha256": hashlib.sha256((directory / "stderr.log").read_bytes()).hexdigest(),
    "original_left_matches_exact_before_metadata": str(previous),
    "changes": changes,
    "scope": "Exact operands after the unchanged original test's existing metadata transformations. No assertion, semantic field or hash was removed by this decoder.",
}, indent=2) + "\n")
print(json.dumps({"operand_count": len(operands), "changes": changes}))
