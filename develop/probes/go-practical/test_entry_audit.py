"""Adversarial checks for baseline identity/corpus audit failures."""

import copy
import json
from unittest import TestCase, main
from unittest.mock import patch

import entry_audit


class EntryAuditTests(TestCase):
    def reject_json_mutation(self, relative, mutate, message):
        original = entry_audit.read_json

        def changed(root, path):
            value = original(root, path)
            if path == relative:
                value = copy.deepcopy(value)
                mutate(value)
            return value

        with patch.object(entry_audit, "read_json", side_effect=changed):
            with self.assertRaisesRegex(ValueError, message):
                entry_audit.audit(entry_audit.ROOT)

    def test_baseline_is_an_inventory_not_a_practical_proof(self):
        report = entry_audit.audit(entry_audit.ROOT)
        self.assertIn("not practical-Go proof", report["evidence_scope"])
        self.assertEqual(report["fixed_go_regression"]["positive_sources"], 13)
        self.assertEqual(report["fixed_go_regression"]["negative_sources"], 8)
        self.assertEqual(report["fixed_go_regression"]["derived_cases"], 11)
        self.assertEqual(report["fixed_go_regression"]["indexed_artifacts_checked"], 83)
        self.assertEqual(report["fixed_go_regression"]["profile_vectors"], 130)

    def test_pending_historical_receipt_cannot_establish_a_release(self):
        self.reject_json_mutation(
            "develop/migrations/archive/java-03-t10-native-receipt.json",
            lambda value: value.update(status="pending"),
            "historical release receipt is not accepted",
        )

    def test_registry_bytes_must_match_the_receipt(self):
        self.reject_json_mutation(
            "develop/migrations/archive/java-03-t10-native-receipt.json",
            lambda value: value["semantic_registry"].update(raw_sha256="0" * 64),
            "semantic_registry bytes differ",
        )

    def test_target_width_cannot_be_replaced_by_host_width(self):
        self.reject_json_mutation(
            entry_audit.SELECTION,
            lambda value: value["target"].update(pointer_width=32),
            "target proposal drift",
        )

    def test_an_unprobed_language_version_is_not_admitted(self):
        self.reject_json_mutation(
            entry_audit.SELECTION,
            lambda value: value["effective_language_versions"].append("go1.26"),
            "language proposal drift",
        )

    def test_an_indexed_artifact_cannot_hide_changed_bytes(self):
        self.reject_json_mutation(
            "fixtures/vir-go/frontend-index.json",
            lambda value: value["cases"][0]["artifacts"][0].update(sha256="0" * 64),
            "corpus drift",
        )

    def test_dropped_negative_sources_are_detected(self):
        self.reject_json_mutation(
            "fixtures/vir-go/frontend-index.json",
            lambda value: value["negative_cases"].pop(),
            "negative corpus count",
        )

    def test_duplicate_json_identity_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            json.loads('{"id":"one","id":"two"}', object_pairs_hook=entry_audit.strict_object)


if __name__ == "__main__":
    main()
