"""Add an untrusted ordinary exporter to the unchanged private checker build."""
import shutil
from pathlib import Path
from carrier_build import prepare as prior,TARGET
from run_proof_pilot import HERE


def prepare(destination):
    destination=Path(destination);receipt=prior(destination)
    target=destination/'probe/src/bin';target.mkdir()
    shutil.copyfile(HERE/'word_arithmetic_export.rs',target/'go-practical-ordinary-word-seed.rs')
    return receipt
