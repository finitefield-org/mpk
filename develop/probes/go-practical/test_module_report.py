"""Reject stale, incomplete, or overstated retained compiler observations."""

import copy
from unittest import TestCase, main

import check_module_report
from entry_audit import read_json
from run_module_probes import ROOT, REPORT


class ModuleReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = read_json(ROOT, REPORT.relative_to(ROOT).as_posix())

    def test_retained_report(self):
        self.assertEqual(check_module_report.check(self.report), 39)

    def reject(self, change, message):
        value = copy.deepcopy(self.report)
        change(value)
        with self.assertRaisesRegex(ValueError, message):
            check_module_report.check(value)

    def test_missing_case(self):
        self.reject(lambda value: value["cases"].pop(), "case membership/order")

    def test_invented_proof_claim(self):
        self.reject(lambda value: value.update(scope="verified application theorem"), "scope overstated")

    def test_host_target_confusion(self):
        self.reject(lambda value: value.update(target="darwin/arm64"), "host/target confusion")

    def test_stale_probe_source(self):
        self.reject(lambda value: value.update(probe_source_sha256="0" * 64), "stale source pin")

    def test_rewritten_loaded_input(self):
        self.reject(
            lambda value: value["cases"][0]["observations"]["observed_loaded_inputs"][0].update(sha256="0" * 64),
            "loaded source identity differs",
        )

    def test_omitted_selected_body(self):
        def remove(value):
            functions = value["cases"][0]["observations"]["functions"]
            next(x for x in functions if x["disposition"] == "selected_root")["disposition"] = "type_check_only"
        self.reject(remove, "closure size")


if __name__ == "__main__":
    main()
