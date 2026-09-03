"""Independent finite arithmetic oracle for the private W08 .NET probe.

No .NET results are consulted when constructing inputs or expected outputs.
W07 owns decimal arithmetic/codecs; its sealed record is referenced separately.
"""

from __future__ import annotations

import calendar
import datetime
import itertools
import struct
from decimal import Decimal, localcontext, ROUND_HALF_EVEN, ROUND_HALF_UP, ROUND_DOWN, ROUND_FLOOR, ROUND_CEILING

from foundation_model import ModelError

DAY = 864_000_000_000
MIN64 = -(1 << 63)
MAX64 = (1 << 63) - 1
RUNTIME_PATH = "develop/migrations/csharp-03/probes/runtime-foundation-data.json"
SOURCE_PATH = "develop/probes/csharp-03/FoundationDataProbe.cs"
SCHEMA = "mpk.csharp_practical.t01_w08.runtime_foundation.v0"
DECIMAL_MAX = Decimal("79228162514264337593543950335")
ROUNDING = (ROUND_HALF_EVEN, ROUND_HALF_UP, ROUND_DOWN, ROUND_FLOOR, ROUND_CEILING)


def decimal_value(text: str) -> str:
    value = Decimal(text)
    if value == 0:
        return "0"
    rendered = format(value, "f")
    return rendered.rstrip("0").rstrip(".") if "." in rendered else rendered


def money(operation: str, inputs: list[str]) -> list[str]:
    """Fixture-owned closed outcome; decimal value equivalence, not decimal bits."""
    with localcontext() as context:
        context.prec = 150
        if operation == "create":
            amount, currency, scale = Decimal(inputs[0]), inputs[1], int(inputs[2])
            if currency not in {"AAA", "BBB"}: return ["error", "invalid_currency"]
            if not 0 <= scale <= 28: return ["error", "invalid_scale"]
            if amount.scaleb(scale) != amount.scaleb(scale).to_integral_value(): return ["error", "invalid_precision"]
            return ["ok", decimal_value(str(amount)), currency]
        left, currency, right = Decimal(inputs[0]), inputs[1], Decimal(inputs[2])
        if operation in {"add", "subtract", "amount_compare", "equal", "compare"}:
            other_currency = inputs[3]
            if operation == "equal": return [str(currency == other_currency and left == right).lower()]
            if operation == "compare":
                result = (currency > other_currency) - (currency < other_currency)
                return [str(result or (left > right) - (left < right))]
            if currency != other_currency: return ["error", "currency_mismatch"]
            if operation == "amount_compare": return ["ok", str((left > right) - (left < right))]
            exact = left + right if operation == "add" else left - right
        else:
            scale, mode = int(inputs[3]), int(inputs[4])
            if not 0 <= scale <= 28: return ["error", "invalid_scale"]
            if not 0 <= mode < 5: return ["error", "invalid_rounding"]
            if operation == "divide" and right == 0: return ["error", "division_by_zero"]
            exact = left * right if operation == "multiply" else left / right
        # Exact finite-width decimal value selection: nearest-even at the finest
        # scale <=28 whose rounded coefficient fits 96 bits; overflow at scale 0.
        represented = None
        for decimal_scale in range(28, -1, -1):
            rounded = exact.quantize(Decimal(1).scaleb(-decimal_scale), rounding=ROUND_HALF_EVEN)
            if abs(rounded.scaleb(decimal_scale)) <= DECIMAL_MAX:
                represented = rounded
                break
        if represented is None: return ["error", "decimal_overflow"]
        if operation in {"multiply", "divide"}:
            represented = represented.quantize(Decimal(1).scaleb(-scale), rounding=ROUNDING[mode])
        return ["ok", decimal_value(str(represented)), currency]


def trunc_div(a: int, b: int) -> int:
    return (abs(a) // abs(b)) * (-1 if (a < 0) != (b < 0) else 1)


def checked64(value: int) -> int:
    if not MIN64 <= value <= MAX64:
        raise ModelError("System.OverflowException")
    return value


def expected(operation: str, inputs: list[str]) -> dict:
    try:
        result = evaluate(operation, inputs)
        return {"kind": "value", "value": result}
    except ModelError as error:
        return {"kind": "exception", "value": [error.code]}


def evaluate(operation: str, inputs: list[str]) -> list[str]:
    def sign(a, b):
        return str((a > b) - (a < b))
    def comparisons(a, b):
        return [sign(a, b), *[str(x).lower() for x in (a == b, a != b, a < b, a <= b, a > b, a >= b)]]
    if operation.startswith("money."):
        return money(operation.split(".")[1], inputs)
    if operation.startswith("instant."):
        left, right = map(int, inputs)
        name = operation.split(".")[1]
        if name == "compare": return comparisons(left, right)
        if name == "difference": value = (left - right) * 10000
        else:
            if right % 10000: return ["error", "precision"]
            value = left + (right // 10000) * (1 if name == "add_duration" else -1)
        return ["ok", str(value)] if MIN64 <= value <= MAX64 else ["error", "range"]
    if operation.startswith("lifted."):
        _, kind, name = operation.split(".")
        values = [None if x == "none" else float("nan") if x == "nan" else int(x) for x in inputs]
        if name == "compare":
            a, b = values
            present = a is not None and b is not None
            return [str(x).lower() for x in (a == b, a != b, present and a < b, present and a <= b, present and a > b, present and a >= b)]
        if None in values:
            return ["none"]
        a = values[0]
        b = values[-1]
        if name == "add": result = a + b
        elif name == "subtract": result = a - b
        elif name == "multiply": result = a * b
        elif name == "divide": result = a // b  # fixtures use exact integral quotients
        elif name == "remainder": result = a - trunc_div(a, b) * b
        elif name == "negate": result = -a
        elif name == "plus": result = a
        else: raise AssertionError(operation)
        if kind in {"f32", "f64"}:
            value = float(result)
            # IEEE signs of exact zero remain observable.
            if result == 0 and ((name == "negate" and a == 0) or
                               (name == "remainder" and a < 0) or
                               (name in {"multiply", "divide"} and (a < 0) != (b < 0))):
                value = -0.0
            return [struct.pack(">f" if kind == "f32" else ">d", value).hex()]
        return [str(result)]
    if operation.startswith("date."):
        a = list(map(int, inputs))
        if operation == "date.construct":
            try:
                value = datetime.date(*a)
            except ValueError:
                raise ModelError("System.ArgumentOutOfRangeException") from None
        elif operation == "date.compare":
            return comparisons(*a)
        else:
            value = datetime.date.fromordinal(a[0] + 1)
            offset = a[1]
            try:
                if operation == "date.add_days":
                    value = datetime.date.fromordinal(value.toordinal() + offset)
                elif operation in {"date.add_months", "date.add_years"}:
                    limit = 120_000 if operation == "date.add_months" else 10_000
                    if not -limit <= offset <= limit:
                        raise ValueError()
                    months = offset if operation == "date.add_months" else offset * 12
                    year, month = divmod((value.year - 1) * 12 + value.month - 1 + months, 12)
                    year += 1
                    month += 1
                    if not 1 <= year <= 9999:
                        raise ValueError()
                    value = datetime.date(year, month, min(value.day, calendar.monthrange(year, month)[1]))
                else:
                    raise AssertionError(operation)
            except (ValueError, OverflowError):
                raise ModelError("System.ArgumentOutOfRangeException") from None
        return list(map(str, [value.year, value.month, value.day, value.toordinal() - 1, (value.weekday() + 1) % 7]))
    if operation.startswith("time."):
        a = list(map(int, inputs))
        if operation == "time.construct":
            ticks = a[0]
            if not 0 <= ticks < DAY:
                raise ModelError("System.ArgumentOutOfRangeException")
        elif operation == "time.compare":
            return comparisons(*a)
        elif operation == "time.subtract":
            return [str((a[0] - a[1]) % DAY)]
        elif operation == "time.add_duration":
            ticks = (a[0] + a[1]) % DAY
        else:
            raise AssertionError(operation)
        return list(map(str, [ticks, ticks // 36_000_000_000, (ticks // 600_000_000) % 60, (ticks // 10_000_000) % 60, (ticks // 10_000) % 1000]))
    if operation.startswith("duration."):
        a = list(map(int, inputs))
        if operation == "duration.construct":
            ticks = a[0]
            components = [trunc_div(ticks, DAY)]
            for divisor, modulus in ((36_000_000_000, 24), (600_000_000, 60), (10_000_000, 60), (10_000, 1000)):
                q = trunc_div(ticks, divisor)
                components.append(q - trunc_div(q, modulus) * modulus)
            return list(map(str, [ticks, *components]))
        if operation == "duration.compare":
            return comparisons(*a)
        if operation == "duration.negate":
            return [str(checked64(-a[0]))]
        if operation == "duration.add":
            return [str(checked64(a[0] + a[1]))]
        if operation == "duration.subtract":
            return [str(checked64(a[0] - a[1]))]
    if operation.startswith("guid."):
        if operation == "guid.empty":
            return ["0" * 32]
        if operation == "guid.compare":
            a, b = (int(x, 16) for x in inputs)
            return [sign(a, b), str(a == b).lower(), str(a != b).lower()]
    if operation.startswith("nullable."):
        a = [None if x == "none" else int(x) for x in inputs]
        if operation == "nullable.inspect":
            if a[0] is None:
                return ["false", "0", "7"]
            return ["true", str(a[0]), str(a[0])]
        if operation == "nullable.value":
            if a[0] is None:
                raise ModelError("System.InvalidOperationException")
            return [str(a[0])]
        if operation == "nullable.compare":
            left, right = a
            present = left is not None and right is not None
            return [str(x).lower() for x in (left == right, left != right,
                present and left < right, present and left <= right,
                present and left > right, present and left >= right)]
        if operation == "nullable.boolean":
            left, right = a
            conjunction = 0 if 0 in a else None if None in a else 1
            disjunction = 1 if 1 in a else None if None in a else 0
            return ["none" if x is None else str(x) for x in (conjunction, disjunction, None if left is None else 1 - left)] + [str(left == right).lower(), str(left != right).lower()]
        if operation in {"nullable.add", "nullable.divide"}:
            if None in a:
                return ["none"]
            left, right = a
            if operation == "nullable.divide":
                if right == 0:
                    raise ModelError("System.DivideByZeroException")
                result = trunc_div(left, right)
            else:
                result = left + right
            if not -(1 << 31) <= result < 1 << 31:
                raise ModelError("System.OverflowException")
            return [str(result)]
    if operation == "source.construction_order":
        return ["argument,constructor,left,right,getter,receiver,call_argument,method", "23"]
    if operation == "source.null_short_circuit":
        return ["left,fallback,receiver", "7", "none"]
    if operation == "source.null_call_order":
        return ["argument", "System.NullReferenceException"]
    if operation == "source.struct_default":
        return ["0", "false", "none"]
    if operation == "source.array_default":
        return ["0", "none", "0"]
    if operation == "source.array_negative":
        raise ModelError("System.OverflowException")
    if operation == "source.array_index":
        raise ModelError("System.IndexOutOfRangeException")
    if operation == "source.array_two_pass":
        return ["2", "2", "4", "1,2,3,4,2,4"]
    raise AssertionError(operation)


def cases() -> list[dict]:
    result = []
    counts: dict[str, int] = {}
    def add(operation: str, *inputs: object) -> None:
        index = counts.get(operation, 0)
        counts[operation] = index + 1
        values = [str(x) for x in inputs]
        result.append({"id": f"{operation}.{index:04d}", "operation": operation,
                       "inputs": values, "expected": expected(operation, values)})
    for ymd in ((1, 1, 1), (9999, 12, 31), (2000, 2, 29), (1900, 2, 28), (2024, 12, 31), (0, 1, 1), (10000, 1, 1), (2023, 2, 29), (2000, 13, 1), (2000, 0, 0)):
        add("date.construct", *ymd)
    days = [0, 3652058, datetime.date(2024, 2, 29).toordinal() - 1, datetime.date(2023, 1, 31).toordinal() - 1]
    for operation, offsets in (("date.add_days", [-2147483648, -366, -1, 0, 1, 366, 2147483647]),
                               ("date.add_months", [-120001, -120000, -13, -1, 0, 1, 13, 120000, 120001]),
                               ("date.add_years", [-10001, -10000, -1, 0, 1, 10000, 10001])):
        for day, offset in itertools.product(days, offsets):
            add(operation, day, offset)
    for a, b in itertools.product(days, repeat=2):
        add("date.compare", a, b)
    times = [0, 1, 123456789012, DAY - 1]
    durations = [MIN64, -DAY - 1, -1, 0, 1, DAY + 1, MAX64]
    for ticks in [-1, *times, DAY, MAX64]:
        add("time.construct", ticks)
    for a, b in itertools.product(times, repeat=2):
        add("time.compare", a, b)
        add("time.subtract", a, b)
    for a, b in itertools.product(times, durations):
        add("time.add_duration", a, b)
    for ticks in durations:
        add("duration.construct", ticks)
        add("duration.negate", ticks)
    for a, b in itertools.product(durations, repeat=2):
        for operation in ("duration.compare", "duration.add", "duration.subtract"):
            add(operation, a, b)
    guids = ["0" * 32, "7fffffff000000000000000000000000", "80000000000000000000000000000000", "000000007fff00000000000000000000", "00000000800000000000000000000000", "000000000000ffff0000000000000000", "0000000000000000ff00000000000000", "f" * 32]
    add("guid.empty")
    for a, b in itertools.product(guids, repeat=2):
        add("guid.compare", a, b)
    nullables = ["none", -2147483648, -1, 0, 1, 2147483647]
    for value in nullables:
        add("nullable.inspect", value)
        add("nullable.value", value)
    for a, b in itertools.product(nullables, repeat=2):
        for operation in ("nullable.compare", "nullable.add", "nullable.divide"):
            add(operation, a, b)
    for a, b in itertools.product(["none", 0, 1], repeat=2):
        add("nullable.boolean", a, b)
    for kind in ("i32", "i64", "decimal", "f32", "f64"):
        for name in ("add", "subtract", "multiply", "divide", "remainder", "compare"):
            for a, b in itertools.product(["none", -1, 0, 1], repeat=2):
                if name in {"divide", "remainder"} and b == 0:
                    continue  # the already sealed W07 primitive table owns zero errors/NaN bits
                add(f"lifted.{kind}.{name}", a, b)
        for name in ("negate", "plus"):
            for a in ("none", -1, 0, 1):
                add(f"lifted.{kind}.{name}", a)
        if kind in {"f32", "f64"}:
            for a, b in itertools.product(["none", -1, 0, 1, "nan"], repeat=2):
                if a == "nan" or b == "nan":
                    add(f"lifted.{kind}.compare", a, b)
    for name in ("construction_order", "null_short_circuit", "null_call_order", "struct_default", "array_default", "array_negative", "array_index", "array_two_pass"):
        add("source." + name)
    for a, b in itertools.product([MIN64, -1, 0, 1, MAX64], [MIN64, -10000, -1, 0, 1, 10000, MAX64]):
        for name in ("add_duration", "subtract_duration", "difference", "compare"):
            add("instant." + name, a, b)
    for amount, currency, scale in (("1.001", "unknown", 29), ("1.001", "AAA", 29), ("1.001", "AAA", 2), ("1.00", "AAA", 2), (str(DECIMAL_MAX), "AAA", 28)):
        add("money.create", amount, currency, scale)
    for a, b in itertools.product(["-1.25", "0", "1.25", str(DECIMAL_MAX)], repeat=2):
        for name in ("add", "subtract", "amount_compare", "equal", "compare"):
            for currency in ("AAA", "BBB"):
                add("money." + name, a, "AAA", b, currency)
    for name in ("multiply", "divide"):
        for amount, factor, scale, mode in itertools.product(["-1.25", "0", "1.25"], ["0", "2", "3"], [0, 2, 28], range(5)):
            add("money." + name, amount, "AAA", factor, scale, mode)
        for factor, scale, mode in (("0", 29, 9), ("0", 2, 9), ("2", 2, 0), ("0.1", 2, 0)):
            add("money." + name, str(DECIMAL_MAX), "AAA", factor, scale, mode)
    return sorted(result, key=lambda row: row["id"])
